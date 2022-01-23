use crate::auth::*;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Auth {
    // Connections
    connections_count: u32,
    connections: HashMap<ConnId, Connection>,
    // Channels
    sender: AuthSender,
    receiver: AuthReceiver,
}

impl Auth {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel::<AuthEvent>();
        Self { connections: HashMap::new(), connections_count: 0, sender, receiver }
    }

    pub fn sender(&self) -> AuthSender {
        self.sender.clone()
    }

    pub fn next_id(&mut self) -> ConnId {
        self.connections_count += 1;
        self.connections_count
    }

    pub async fn listen(&mut self) {
        while let Some(event) = self.receiver.recv().await {
            match event {
                AuthEvent::Handshake(name, sender, handshake_sender) => {
                    let id = self.next_id();
                    let conn = Connection::new(id, name, sender);
                    handshake_sender.send(conn).unwrap();
                }
            }
        }
    }
}
