use crate::bitboard::Bitboard;
use crate::bitwise::Bitwise;
use crate::constants::*;

#[derive(Debug)]
pub enum Outcome {
    Win,
    Loss,
    Draw,
}

impl Evaluate {
    fn surface_area(bitboard: &Bitboard) -> i16 {
        let white = Bitwise::pcnt(Bitwise::adj_any(bitboard.board[WHITE])) as i16;
        let black = Bitwise::pcnt(Bitwise::adj_any(bitboard.board[BLACK])) as i16;
        match bitboard.turn {
            WHITE => white - black,
            BLACK => black - white,
            _ => panic!("Never reaches here"),
        }
    }

    fn robots(bitboard: &Bitboard) -> i16 {
        match bitboard.turn {
            WHITE => 100 * (bitboard.robots_white - bitboard.robots_black),
            BLACK => 100 * (bitboard.robots_black - bitboard.robots_white),
            _ => panic!("Never reaches here"),
        }
    }

    fn constrain(score: i16) -> i16 {
        if score > MAX {
            MAX
        } else if score < MIN {
            MIN
        } else {
            score
        }
    }

    /// Evaluates the material advantage of the bitboard
    pub fn evaluate(bitboard: &Bitboard) -> i16 {
        Evaluate::constrain(Evaluate::robots(&bitboard) + Evaluate::surface_area(&bitboard))
    }

    /// Evaluates the outcome of the bitboard
    /// TODO: Move this into bitboard
    pub fn outcome(bitboard: &Bitboard) -> Option<Outcome> {
        match (bitboard.turn, bitboard.robots_white, bitboard.robots_black) {
            (_, 0, 0) => Some(Outcome::Draw),
            (WHITE, _, 0) => Some(Outcome::Win),
            (WHITE, 0, _) => Some(Outcome::Loss),
            (BLACK, 0, _) => Some(Outcome::Win),
            (BLACK, _, 0) => Some(Outcome::Loss),
            _ => None,
        }
    }

    /// Evaluates the outcome of the bitboard
    /// TODO: Move this into bitboard
    fn attacking(bitboard: &Bitboard) -> bool {
        // Calculates white attacking black and black attacking white
        (Bitwise::adj_any(bitboard.board[WHITE]) & bitboard.board[BLACK])
            | (Bitwise::adj_any(bitboard.board[BLACK]) & bitboard.board[WHITE])
            != 0
    }
}

pub struct Evaluate {}
