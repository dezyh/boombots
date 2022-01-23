use crate::conn::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

mod handler;
pub use handler::Auth;

pub type AuthSender = UnboundedSender<AuthEvent>;
pub type AuthReceiver = UnboundedReceiver<AuthEvent>;
pub type HandshakeSender = tokio::sync::oneshot::Sender<Connection>;

#[derive(Debug)]
pub enum AuthEvent {
    Handshake(String, ConnectionSender, HandshakeSender),
}
