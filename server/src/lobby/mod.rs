use crate::conn::{ConnId, Connection};
use crate::game::GameId;
use boombots_core::net::{AcceptChallengeInfo, ChallengeInfo, SendChallengeInfo};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

mod handler;
pub use handler::Lobby;

pub type LobbySender = UnboundedSender<LobbyEvent>;
pub type LobbyReceiver = UnboundedReceiver<LobbyEvent>;

#[derive(Debug)]
pub enum LobbyEvent {
    Join(Connection),
    Disconnect(ConnId),
    SendChallenge(SendChallengeInfo),
    AcceptChallenge(AcceptChallengeInfo),
    GameReady(GameId, ChallengeInfo),
}
