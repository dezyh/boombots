// Development
#![allow(dead_code)]
#![allow(unused_imports)]

// Nightly features
#![feature(const_option_ext)]
#![feature(unchecked_math)]

use crate::{
    bitboard::Bitboard,
    constants::{LOSS, WIN},
    search::Search,
    transpose::{Transposition, TranspositionTable},
};
pub mod action;
pub mod bitboard;
pub mod bitwise;
pub mod constants;
pub mod evaluate;
pub mod format;
pub mod search;
pub mod transpose;
