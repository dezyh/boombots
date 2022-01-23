#![allow(dead_code)]
mod auth;
mod conn;
mod game;
mod lobby;
mod server;
use server::*;

#[tokio::main]
async fn main() {
    Server::start("0.0.0.0:8008").await;
}
