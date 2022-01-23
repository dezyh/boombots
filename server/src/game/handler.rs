use super::*;
use crate::lobby::*;
use boombots_core::{
    net::{Event, GameInfo},
    GameState,
};
use futures_util::SinkExt;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub struct Game {
    id: GameId,
    sender: GameSender,
    receiver: GameReceiver,
    players: Vec<Connection>,
    lobby: Option<LobbySender>,
    gamestate: GameState,
    started: bool,
}

impl Game {
    pub fn new(id: GameId) -> Game {
        let (sender, receiver) = mpsc::unbounded_channel::<GameEvent>();
        Game {
            id,
            sender,
            receiver,
            players: Vec::new(),
            lobby: None,
            gamestate: GameState::new(),
            started: false,
        }
    }

    // Make a clone of the game sender which can be used to send events to the game
    pub fn sender(&self) -> GameSender {
        self.sender.clone()
    }

    // Add a lobby sender to the game pool to allow communication back to the lobby
    pub fn add_lobby(&mut self, lobby: &Lobby) {
        self.lobby = Some(lobby.sender());
    }

    pub async fn listen(&mut self) -> Result<(), ()> {
        while let Some(event) = self.receiver.recv().await {
            match event {
                GameEvent::Join(conn) => {
                    self.players.push(conn);
                    if self.can_start() {
                        self.start_game().await;
                    }
                }
                GameEvent::GameAction(_id, action) => match self.gamestate.valid(&action) {
                    true => {
                        self.gamestate.make(&action);
                        self.broadcast_gamestate().await;
                    }
                    false => {
                        println!("invalid action");
                    }
                },
            }
        }
        Ok(())
    }

    // Broadcast a message to all players in the room
    async fn broadcast(&mut self, json: String) {
        for player in &mut self.players {
            player.sender.send(Message::Text(json.clone())).await.unwrap();
        }
    }

    async fn broadcast_gamestate(&mut self) {
        let p1 = &self.players.get(0).unwrap();
        let p2 = &self.players.get(1).unwrap();

        let event = Event::GameBroadcast(GameInfo {
            id: self.id,
            white: p1.user.clone(),
            black: p2.user.clone(),
            gamestate: self.gamestate,
        });

        self.broadcast(serde_json::to_string(&event).unwrap()).await;
    }

    fn can_start(&self) -> bool {
        self.players.len() == 2 && !self.started
    }

    async fn start_game(&mut self) {
        self.started = true;
        self.broadcast_gamestate().await;
        println!("Game started");
    }
}
