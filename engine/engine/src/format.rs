use crate::constants::*;

pub struct Format {}

impl Format {
    pub fn frame(frame: u64, title: &str) {
        println!("{}", title);
        println!("╔════════╗");
        println!("║{:08b}║", (frame & BOARD_ROW7) >> 56);
        println!("║{:08b}║", (frame & BOARD_ROW6) >> 48);
        println!("║{:08b}║", (frame & BOARD_ROW5) >> 40);
        println!("║{:08b}║", (frame & BOARD_ROW4) >> 32);
        println!("║{:08b}║", (frame & BOARD_ROW3) >> 24);
        println!("║{:08b}║", (frame & BOARD_ROW2) >> 16);
        println!("║{:08b}║", (frame & BOARD_ROW1) >> 8);
        println!("║{:08b}║", (frame & BOARD_ROW0) >> 0);
        println!("╚════════╝");
    }
}
