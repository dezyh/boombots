use crate::action::Action;
use crate::bitboard::Bitboard;
use crate::bitwise::Bitwise;
use crate::constants::*;

#[derive(Clone, Copy, Debug)]
pub enum Bound {
    Lower,
    Upper,
    Exact,
}

#[derive(Clone, Copy, Debug)]
pub struct Transposition {
    pub eval: i16,
    pub hash: u64,
    pub depth: u8,
    pub action: Action,
    pub bound: Bound,
}

pub struct TranspositionTable {
    entries: Vec<Option<Transposition>>,
    overwrites: u16,
    size: u64,
    mask: u64,
}

impl TranspositionTable {
    pub fn new(pow: u32) -> TranspositionTable {
        let size = 1 << pow;
        let mut entries = Vec::new();
        entries.resize_with(size as usize, || None);
        TranspositionTable { size, entries, overwrites: 0, mask: size - 1 }
    }

    pub fn index(&self, hash: u64) -> usize {
        (hash & self.mask) as usize
    }

    pub fn store(&mut self, hash: u64, eval: i16, action: Action, depth: u8, bound: Bound) {
        let i = self.index(hash);

        if self.entries[i].is_some() {
            self.overwrites += 1;
        }

        self.entries[i] = Some(Transposition { eval, hash, depth, action, bound });
    }

    pub fn lookup(&self, hash: u64) -> Option<Transposition> {
        let i = self.index(hash);

        match self.entries[i] {
            Some(entry) => match entry.hash == hash {
                true => Some(entry),
                false => None,
            },
            None => None,
        }
    }
}
