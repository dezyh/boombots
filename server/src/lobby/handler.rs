use super::{LobbyEvent, LobbyReceiver, LobbySender};
use crate::conn::ConnId;
use crate::conn::Connection;
use crate::game::GameId;
use crate::game::GamePool;
use crate::game::{GamePoolEvent, GamePoolSender};
use boombots_core::net::AcceptChallengeInfo;
use boombots_core::net::{ChallengeInfo, Event, SendChallengeInfo, User};
use futures_util::sink::SinkExt;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub struct Lobby {
    // Connections
    connections: HashMap<u32, Connection>,
    // Challenges
    challenges: HashMap<u32, ChallengeInfo>,
    // Channels
    sender: LobbySender,
    receiver: LobbyReceiver,
    gamepool: Option<GamePoolSender>,
}

impl Lobby {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel::<LobbyEvent>();
        Self {
            connections: HashMap::new(),
            challenges: HashMap::new(),
            gamepool: None,
            sender,
            receiver,
        }
    }

    pub fn sender(&self) -> LobbySender {
        self.sender.clone()
    }

    pub fn add_gamepool(&mut self, gamepool: &GamePool) {
        self.gamepool = Some(gamepool.sender());
    }

    pub async fn listen(&mut self) {
        while let Some(event) = self.receiver.recv().await {
            println!("{:?}", event);
            match event {
                LobbyEvent::Join(conn) => self.add_conn(conn).await,
                LobbyEvent::Disconnect(id) => self.disconnect(id).await,
                LobbyEvent::SendChallenge(info) => self.send_challenge(info).await,
                LobbyEvent::AcceptChallenge(info) => self.accept_challenge(info).await,
                LobbyEvent::GameReady(game, challenge) => self.move_players(game, challenge).await,
            }
        }
    }

    // Adds a new connection to the lobby and broadcasts the current connectiosn to all users
    async fn add_conn(&mut self, conn: Connection) {
        self.connections.insert(conn.user.id, conn);
        self.broadcast_users().await;
    }

    // Disconnect a user from the lobby and broadcast the current connections to all users
    async fn disconnect(&mut self, id: ConnId) {
        if let Some(_conn) = self.connections.remove(&id) {
            self.broadcast_users().await;
        };
    }

    // Move players specified in a challenge into a new game lobby
    async fn move_players(&mut self, game: GameId, challenge: ChallengeInfo) {
        let p1 = self.connections.remove(&challenge.source.id).unwrap();
        let p2 = self.connections.remove(&challenge.target.id).unwrap();

        if let Some(gamepool) = &self.gamepool {
            gamepool.send(GamePoolEvent::Join(game, p1)).unwrap();
            gamepool.send(GamePoolEvent::Join(game, p2)).unwrap();
        }
    }

    async fn send_challenge(&mut self, info: SendChallengeInfo) {
        let challenge = self.create_challenge(info).unwrap();
        self.broadcast_challenge(&challenge).await;
        self.challenges.insert(challenge.id, challenge);
    }

    async fn accept_challenge(&mut self, challenge: AcceptChallengeInfo) {
        if self.can_accept(&challenge) {
            // Remove the challenge from the lobby and broadcast it to the relevent users
            let mut challenge = self.challenges.remove(&challenge.id).unwrap();
            challenge.accepted = true;
            self.broadcast_challenge(&challenge).await;
            // Create a new game
            if let Some(gamepool) = &self.gamepool {
                gamepool.send(GamePoolEvent::CreateGame(challenge)).unwrap();
            }
        }
    }

    // This is to ensure that another user couldn't send an challenge accept for another player
    fn can_accept(&self, accepted_challenge: &AcceptChallengeInfo) -> bool {
        let challenge = self.challenges.get(&accepted_challenge.id).unwrap();
        challenge.target.id == accepted_challenge.target.unwrap()
    }

    async fn broadcast_challenge(&mut self, challenge: &ChallengeInfo) {
        let event = Event::ChallengeBroadcast(challenge.clone());
        let json = serde_json::to_string(&event).unwrap();

        if let Some(connection) = self.connections.get_mut(&challenge.clone().source.id) {
            connection.sender.send(Message::Text(json.clone())).await.unwrap();
        }

        if let Some(connection) = self.connections.get_mut(&challenge.clone().target.id) {
            connection.sender.send(Message::Text(json.clone())).await.unwrap();
        }
    }

    // Given a challenge source and challenge target, try create a new challenge
    pub fn create_challenge(&mut self, challenge: SendChallengeInfo) -> Result<ChallengeInfo, ()> {
        let id = (self.challenges.len() + 1) as u32;
        let source = self.connections.get(&challenge.source.unwrap()).unwrap();
        let target = self.connections.get(&challenge.target).unwrap();

        Ok(ChallengeInfo::new(id, source.user.clone(), target.user.clone()))
    }

    async fn broadcast_users(&mut self) {
        let users: Vec<User> = self
            .connections
            .iter()
            .map(|(_, user)| User { id: user.user.id, name: user.user.name.clone() })
            .collect();

        for connection in self.connections.values_mut() {
            let msg = Event::LobbyUserBroadcast(connection.user.id, users.clone());
            let txt = serde_json::to_string(&msg).unwrap();
            connection.sender.send(Message::Text(txt)).await.unwrap();
        }
    }
}
