use boombots_core::net::Packet;
use boombots_core::{Action, Pos};
use ws::{connect, CloseCode, Handler, Handshake, Message, Result, Sender};

struct Client {
    out: Sender,
}

impl Handler for Client {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        let action = Action {
            a: Pos::new(0, 1),
            b: Pos::new(0, 1),
            n: 0,
        };
        let packet = Packet::GameAction(action);
        let message = serde_json::to_string(&packet).expect("Serialize error");
        self.out.send(message)
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("Got message: {}", msg);
        Ok(())
    }
}

fn main() {
    connect("ws://0.0.0.0:8008", |out| Client { out }).unwrap();
}
