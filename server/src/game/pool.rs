use super::*;
use crate::lobby::*;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct GamePool {
    games: HashMap<GameId, GameInfo>,
    conn_games: HashMap<GameId, ConnId>,
    sender: GamePoolSender,
    receiver: GamePoolReceiver,
    lobby: Option<LobbySender>,
}

impl GamePool {
    pub fn new() -> GamePool {
        let (sender, receiver) = mpsc::unbounded_channel::<GamePoolEvent>();
        GamePool {
            games: HashMap::new(),
            conn_games: HashMap::new(),
            sender,
            receiver,
            lobby: None,
        }
    }

    // Makes a clone of the game pools transmitter to allow events to be sent to the game pool from
    // other handlers. This is useful so that the lobby can send new game events.
    pub fn sender(&self) -> GamePoolSender {
        self.sender.clone()
    }

    // Add a lobby sender to the game pool to allow communication back to the lobby. This is useful
    // when games are complete and users should return to the lobby.
    pub fn add_lobby(&mut self, lobby: &Lobby) {
        self.lobby = Some(lobby.sender());
    }

    fn next_id(&mut self) -> u32 {
        (self.games.len() + 1) as u32
    }

    fn create_game(&mut self) -> (Game, GameInfo) {
        let id = self.next_id();
        let game = Game::new(id);
        let sender = game.sender();
        let info = GameInfo::new(id, sender);
        (game, info)
    }

    pub async fn listen(&mut self) {
        while let Some(event) = self.receiver.recv().await {
            println!("{:?}", event);
            match event {
                GamePoolEvent::CreateGame(challenge) => {
                    let (mut game, info) = self.create_game();
                    let game_id = info.id;
                    tokio::spawn(async move {
                        game.listen().await.unwrap();
                    });
                    // Keep track of the game
                    self.games.insert(info.id, info);
                    if let Some(lobby) = &self.lobby {
                        lobby.send(LobbyEvent::GameReady(game_id, challenge)).unwrap();
                    }
                }
                GamePoolEvent::GameAction(id, action) => {
                    let game_id = self.conn_games.get(&id).unwrap();
                    let game = self.games.get(game_id).unwrap();
                    game.sender.send(GameEvent::GameAction(id, action)).unwrap();
                }
                GamePoolEvent::Join(game_id, conn) => {
                    self.conn_games.insert(conn.user.id, game_id);
                    let game = self.games.get(&game_id).expect("couldnt get game by id");
                    game.sender.send(GameEvent::Join(conn)).expect("couldnt send event to game");
                }
            }
        }
    }
}
