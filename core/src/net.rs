use crate::{Action, GameState};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
}

impl User {
    pub fn new(id: u32, name: String) -> User {
        User { id, name }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "camelCase")]
pub enum Event {
    Handshake(String),
    SendChallenge(SendChallengeInfo),
    AcceptChallenge(AcceptChallengeInfo),
    ChallengeBroadcast(ChallengeInfo),
    LobbyUserBroadcast(u32, Vec<User>),

    GameBroadcast(GameInfo),
    GameAction(Action),
    Quit,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeInfo {
    pub id: u32,
    pub source: User,
    pub target: User,
    pub accepted: bool,
}
impl ChallengeInfo {
    pub fn new(id: u32, source: User, target: User) -> ChallengeInfo {
        ChallengeInfo {
            id,
            source,
            target,
            accepted: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendChallengeInfo {
    pub source: Option<u32>,
    pub target: u32,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AcceptChallengeInfo {
    pub id: u32,
    pub target: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub id: u32,
    pub white: User,
    pub black: User,
    pub gamestate: GameState,
}
