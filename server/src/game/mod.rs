use crate::conn::{ConnId, Connection};
use boombots_core::{net::ChallengeInfo, Action};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

mod handler;
mod pool;
pub use handler::Game;
pub use pool::GamePool;

pub type GameId = u32;
pub type GameSender = UnboundedSender<GameEvent>;
pub type GameReceiver = UnboundedReceiver<GameEvent>;
pub type GamePoolSender = UnboundedSender<GamePoolEvent>;
pub type GamePoolReceiver = UnboundedReceiver<GamePoolEvent>;

#[derive(Debug)]
pub enum GamePoolEvent {
    CreateGame(ChallengeInfo),
    GameAction(ConnId, Action),
    Join(GameId, Connection),
    // Disconnect(ConnId),
}

#[derive(Debug)]
pub enum GameEvent {
    Join(Connection),
    GameAction(ConnId, Action),
    // Disconnect(ConnId),
}

#[derive(Debug)]
pub struct GameInfo {
    pub id: GameId,
    pub sender: GameSender,
}

impl GameInfo {
    pub fn new(id: GameId, sender: GameSender) -> GameInfo {
        GameInfo { id, sender }
    }
}
