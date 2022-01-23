use crate::{auth::*, game::*, lobby::*};
use anyhow::{anyhow, Result};
use boombots_core::net::{Event, User};
use futures_util::stream::{SplitSink, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub type ConnId = u32;
pub type TcpSocketStream = WebSocketStream<TcpStream>;
pub type ConnectionSender = SplitSink<TcpSocketStream, Message>;

#[derive(Debug)]
pub struct Connection {
    pub user: User,
    pub sender: ConnectionSender,
}

impl Connection {
    pub fn new(id: ConnId, name: String, sender: ConnectionSender) -> Self {
        Self { sender, user: User::new(id, name) }
    }

    fn try_deserialize(
        message: Result<Message, tokio_tungstenite::tungstenite::Error>,
    ) -> Result<Event> {
        match message {
            Err(_) => Err(anyhow!("Failed to read message from socket")),
            Ok(message) => match message.to_text() {
                Err(_) => Err(anyhow!("Failed to deserialize event to text")),
                Ok(text) => match serde_json::from_str(text) {
                    Ok(event) => Ok(event),
                    Err(_) => Err(anyhow!("Failed to deserialize event text into frame: {}", text)),
                },
            },
        }
    }

    // Handles the entire lifetime of a users socket connection. The connection must first perform
    // a handshake which validates any tokens. Subsequent messages will then be tagged with the
    // conenctions id and forwarded to other async tasks for processing.
    pub async fn handle(
        stream: TcpSocketStream,
        lobby: LobbySender,
        games: GamePoolSender,
        auth: AuthSender,
    ) {
        let (sender, mut receiver) = stream.split();
        let mut id: Option<ConnId> = None;

        // The user first needs to authenticate via a handshake in which they are assigned an ID
        if let Some(msg) = receiver.next().await {
            if let Ok(event) = Connection::try_deserialize(msg) {
                if let Event::Handshake(name) = event {
                    // Try perform a handshake to authenticate the user
                    let (handshake_sender, handshake_receiver) = oneshot::channel::<Connection>();
                    auth.send(AuthEvent::Handshake(name, sender, handshake_sender))
                        .expect("Failed to send auth handshake event");
                    // Wait for the handshake response from the auth task
                    match handshake_receiver.await {
                        Ok(handshaked) => {
                            // Remember the connection id and move the connection to the lobby
                            id = Some(handshaked.user.id);
                            lobby
                                .send(LobbyEvent::Join(handshaked))
                                .expect("Failed to send lobby join event");
                        }
                        Err(_) => {
                            // TODO: Return an error
                            println!("Error handshaking");
                            return;
                        }
                    }
                } else {
                    // TODO: Return an error here in the future
                    return;
                }
            }
        }
        // Deserialize the event from the network

        // Ensure that we have an id now
        if id.is_none() {
            return;
        }
        let id = id.expect("id was none");

        // Then we can delegete the users events
        // Wait for the next event from the connections socket
        while let Some(msg) = receiver.next().await {
            if let Ok(event) = Connection::try_deserialize(msg) {
                match event {
                    Event::SendChallenge(mut challenge) => {
                        challenge.source = Some(id);
                        lobby.send(LobbyEvent::SendChallenge(challenge)).unwrap();
                    }
                    Event::AcceptChallenge(mut challenge) => {
                        // Tag the challenge as accepted by the current connection's id
                        challenge.target = Some(id);
                        lobby.send(LobbyEvent::AcceptChallenge(challenge)).unwrap();
                    }
                    Event::GameAction(action) => {
                        games.send(GamePoolEvent::GameAction(id, action)).unwrap();
                    }
                    Event::Handshake(_name) => println!("Already performed a handshake"),
                    _ => println!("Unknown event"),
                }
            }
        }

        // Clean up when they disconnect
        lobby.send(LobbyEvent::Disconnect(id)).unwrap();
    }
}
