use crate::auth::Auth;
use crate::conn::Connection;
use crate::game::GamePool;
use crate::lobby::Lobby;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

pub struct Server {}

impl Server {
    pub async fn start(address: &str) {
        let listener = TcpListener::bind(&address)
            .await
            .expect("The TCP connection failed to bind to the address");

        println!("Listening on: {}", address);

        // Create a game pool
        let mut auth = Auth::new();
        let mut lobby = Lobby::new();
        let mut gamepool = GamePool::new();

        let auth_sender = auth.sender();
        let lobby_sender = lobby.sender();
        let gamepool_sender = gamepool.sender();

        // Share senders so that tasks can communicate using events
        lobby.add_gamepool(&gamepool);
        gamepool.add_lobby(&lobby);

        // Starts the tasks
        tokio::spawn(async move {
            auth.listen().await;
        });
        tokio::spawn(async move {
            lobby.listen().await;
        });
        tokio::spawn(async move {
            gamepool.listen().await;
        });

        // Handle TCP connections from peers
        while let Ok((stream, peer)) = listener.accept().await {
            match accept_async(stream).await {
                Ok(stream) => {
                    println!("New connection from: {}", peer);
                    tokio::spawn(Connection::handle(
                        stream,
                        lobby_sender.clone(),
                        gamepool_sender.clone(),
                        auth_sender.clone(),
                    ));
                }
                Err(e) => println!("Websocket connection error: {}", e),
            }
        }
    }
}
