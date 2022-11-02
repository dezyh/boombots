use crate::action::Action;
use crate::bitwise::Bitwise;
use crate::constants::*;

#[derive(Debug)]
pub struct DeltaDirectional {
    robots: u8,
    source_pos: u64,
    target_pos: u64,
    old_source_height: u8,
    new_source_height: u8,
    old_target_height: u8,
    new_target_height: u8,
}

#[derive(Debug)]
pub struct DeltaExplosion {
    board: [u64; 14],
}

#[derive(Debug)]
pub enum Delta {
    Explosion(DeltaExplosion),
    Directional(DeltaDirectional),
}

#[derive(Debug, PartialEq)]
pub struct Bitboard {
    pub hash: u64,
    pub turn: usize,
    pub opponent: usize,
    pub board: [u64; 14],
    pub robots_white: i16,
    pub robots_black: i16,
    pub robots_total: i16,
}

use core::arch::x86_64::__m512i as u64x8;
use std::arch::x86_64::{
    _MM_CMPINT_NE as U64_NE, 
    _mm512_and_epi64 as and_u64x8, 
    _mm512_cmp_epu64_mask as cmp_u64x8, 
    _mm512_loadu_epi64 as load_u64x8
};

const fn from_u64x8(vals: [u64; 8]) -> u64x8 {
    union U64x8 {
        vector: u64x8,
        bytes: [u64; 8],
    }

    unsafe { (U64x8 { bytes: vals }).vector }
}

const ZEROS: u64x8 = from_u64x8([0, 0, 0, 0, 0, 0, 0, 0]);

impl Bitboard {
    /// Finds the height of the bot at position.
    ///
    /// Since a bitboard frame is a u64 and the maximum height is 8, we can fit 
    /// all 8 bitboard frames, representing bots with heights 1-8, into a single u64x8 (__m512i).
    ///
    /// 1. One AVX512 op to identify which bit within all 8 bitboard frames has the position bit set.
    /// 2. One AVX512 op to find the index of the bitboard frame which contains the set position bit. 
    /// 3. One CPU op to convert the frame index from a single set bit in a u8 (n) to an integer (n) 
    /// in the range 1-8.
    #[inline(always)]
    fn height_avx512(&self, pos: u64) -> u8 {
        unsafe {
            let board = load_u64x8(self.board[1..=8].as_ptr() as *const i64);

            // Repeat the position mask to AND it against each of the boards height frames in the
            // same AVX512 operation.
            let mask: [u64; 8] = [pos, pos, pos, pos, pos, pos, pos, pos];
            let mask = load_u64x8(mask.as_ptr() as *const i64);

            // Perform an AND of each bitboard frame to the repeated position mask to isolate the
            // only robot at that position but at some unknown frame.
            let bot = and_u64x8(board, mask);

            // To recover the index of the frame which has the only set bit representing the bot,
            // we can compare each frame to zeros. This will set a bit in the resulting u8 that
            // corrosponds to the robots height (eg: 0b1=>1, 0b10=>2, 0b100=>3, 0b1000=>4, etc).
            let height = cmp_u64x8::<U64_NE>(bot, ZEROS) as u8;

            // Now that we have only the n-th bit set, we can count the number of trailing zeros
            // and add 1 to compute the height of the robot at the specified position. 
            height.trailing_zeros() as u8 + 1
        }
    }

    /// Finds the height of the robot at the given position
    #[inline(always)]
    pub fn height(&self, pos: u64) -> u8 {
        self.height_loop(pos)
    }

    #[inline(always)]
    fn height_loop(&self, pos: u64) -> u8 {
        for i in 1..=8 {
            if self.board[i] & pos != 0 {
                return i as u8;
            }
        }
        0
    }

    // Calculates the bitboard changes required to update the game state
    fn delta_directional(&self, action: Action) -> DeltaDirectional {
        // Convert from indexed positions to board positions
        let source_pos = Bitwise::pos(action.source);
        let target_pos = Bitwise::pos(action.target);

        // Find the height at the old and new source and target positions
        let old_source_height = self.height(source_pos);
        let new_source_height = old_source_height - action.robots;
        let old_target_height = self.height(target_pos);
        let new_target_height = old_target_height + action.robots;

        DeltaDirectional {
            source_pos,
            target_pos,
            old_source_height,
            new_source_height,
            old_target_height,
            new_target_height,
            robots: action.robots,
        }
    }

    /// Applies an already computed directional action delta to the current bitboard
    /// TODO: Look into optimizing this because it could be branchless by abusing the x==0 case?
    fn make_directional(&mut self, delta: &DeltaDirectional) -> u64 {
        // Compute the move hash to update the zorbist key
        let mut hash_delta: u64 = 0;

        // Update the source
        if delta.new_source_height != 0 {
            // Some bot remain at the source, so just update it
            self.board[delta.old_source_height as usize] ^= delta.source_pos;
            self.board[delta.new_source_height as usize] |= delta.source_pos;
            // Update the hash delta
            let pos = Bitwise::idx(delta.source_pos);
            hash_delta ^= ZORBIST_KEY[delta.old_source_height as usize][pos];
            hash_delta ^= ZORBIST_KEY[delta.new_source_height as usize][pos];
        } else {
            // All bots are moving so remove the source
            self.board[delta.old_source_height as usize] &= !delta.source_pos;
            self.board[self.turn] ^= delta.source_pos;
            // Update the hash delta
            let pos = Bitwise::idx(delta.source_pos);
            hash_delta ^= ZORBIST_KEY[delta.old_source_height as usize][pos];
            hash_delta ^= ZORBIST_KEY[self.turn][pos];
        }

        // Update the target
        if delta.old_target_height != 0 {
            // Some bots existed at the target, so just update it
            self.board[delta.old_target_height as usize] ^= delta.target_pos;
            self.board[delta.new_target_height as usize] |= delta.target_pos;
            // Update the hash delta
            let pos = Bitwise::idx(delta.target_pos);
            hash_delta ^= ZORBIST_KEY[delta.old_target_height as usize][pos];
            hash_delta ^= ZORBIST_KEY[delta.new_target_height as usize][pos];
        } else {
            // No bots existed at the target, so create it from scratch
            self.board[delta.new_target_height as usize] |= delta.target_pos;
            self.board[self.turn] |= delta.target_pos;
            // Update the hash delta
            let pos = Bitwise::idx(delta.target_pos);
            hash_delta ^= ZORBIST_KEY[delta.new_target_height as usize][pos];
            hash_delta ^= ZORBIST_KEY[self.turn][pos];
        }

        hash_delta
    }

    /// Takes a directional action delta and undos it from the bitboard
    fn undo_directional(&mut self, delta: &DeltaDirectional) {
        // Undo source action from delta
        if delta.new_source_height != 0 {
            // Undo the source update (when only some of a stack is moved)
            self.board[delta.old_source_height as usize] |= delta.source_pos;
            self.board[delta.new_source_height as usize] ^= delta.source_pos;
        } else {
            // Undo the source removal (when all of a stack is moved)
            self.board[delta.old_source_height as usize] |= delta.source_pos;
            self.board[self.turn] |= delta.source_pos;
        }

        // Undo target action from delta
        if delta.old_target_height != 0 {
            // Undo updating the target (when only some of a stack is moved)
            self.board[delta.old_target_height as usize] |= delta.target_pos;
            self.board[delta.new_target_height as usize] ^= delta.target_pos;
        } else {
            // Undo creating the target (when an entire stack is moved)
            self.board[delta.new_target_height as usize] ^= delta.target_pos;
            self.board[self.turn] ^= delta.target_pos;
        }
    }

    fn delta_explosion(&self, action: Action) -> DeltaExplosion {
        let explosion = Bitwise::dfs(self.board[WHITE] | self.board[BLACK], action.source);
        DeltaExplosion {
            board: [
                explosion & self.board[WHITE],
                explosion & self.board[1],
                explosion & self.board[2],
                explosion & self.board[3],
                explosion & self.board[4],
                explosion & self.board[5],
                explosion & self.board[6],
                explosion & self.board[7],
                explosion & self.board[8],
                explosion & self.board[9],
                explosion & self.board[10],
                explosion & self.board[11],
                explosion & self.board[12],
                explosion & self.board[BLACK],
            ],
        }
    }

    /// Applys a precomputed action delta onto the current bitboard and returns the hash delta
    fn make_explosion(&mut self, delta: &DeltaExplosion) -> u64 {
        self.board[WHITE] &= !delta.board[WHITE];
        self.board[1] &= !delta.board[1];
        self.board[2] &= !delta.board[2];
        self.board[3] &= !delta.board[3];
        self.board[4] &= !delta.board[4];
        self.board[5] &= !delta.board[5];
        self.board[6] &= !delta.board[6];
        self.board[7] &= !delta.board[7];
        self.board[8] &= !delta.board[8];
        self.board[9] &= !delta.board[9];
        self.board[10] &= !delta.board[10];
        self.board[11] &= !delta.board[11];
        self.board[12] &= !delta.board[12];
        self.board[BLACK] &= !delta.board[BLACK];
        self.update_robot_counts();

        // Calculate the hash
        let mut hash_delta: u64 = 0;

        // Calculate for the change in height board frames
        for height in 1..=12 {
            let mut bots = delta.board[height];
            while bots != 0 {
                let bot = Bitwise::lsb(bots);
                bots ^= bot;
                let pos = Bitwise::idx(bot);
                hash_delta ^= ZORBIST_KEY[height][pos];
            }
        }
        // Calculate for the change in colour board frames
        let mut whites = delta.board[WHITE];
        while whites != 0 {
            let bot = Bitwise::lsb(whites);
            whites ^= bot;
            let pos = Bitwise::idx(bot);
            hash_delta ^= ZORBIST_KEY[WHITE][pos];
        }
        let mut blacks = delta.board[BLACK];
        while blacks != 0 {
            let bot = Bitwise::lsb(blacks);
            blacks ^= bot;
            let pos = Bitwise::idx(bot);
            hash_delta ^= ZORBIST_KEY[BLACK][pos];
        }

        hash_delta
    }

    /// Takes a precomputed explosion action delta and undos it from the bitboard
    fn undo_explosion(&mut self, delta: &DeltaExplosion) {
        for i in WHITE..=BLACK {
            self.board[i] |= delta.board[i];
        }
        self.update_robot_counts();
    }

    fn update_robot_counts(&mut self) {}

    // Toggles the turn player
    fn toggle_turn(&mut self) {
        let previous = self.turn;
        self.turn = self.opponent;
        self.opponent = previous;
        self.hash ^= ZORBIST_TURN;
    }

    // Takes a precomputed action delta and applies it onto the bitboard
    pub fn make(&mut self, delta: &Delta) -> u64 {
        // Update the board
        let hash_delta = match delta {
            Delta::Explosion(explosion) => self.make_explosion(explosion),
            Delta::Directional(directional) => self.make_directional(directional),
        };

        // Update the zorbist hash
        self.hash ^= hash_delta;

        // Toggle the turn player
        self.toggle_turn();

        hash_delta
    }

    /// Takes a raw move and computes the changes required in order to update the bitboard and hash
    pub fn delta(&self, action: Action) -> Delta {
        match action.robots {
            0 => Delta::Explosion(self.delta_explosion(action)),
            _ => Delta::Directional(self.delta_directional(action)),
        }
    }

    /// Take a precomputed action delta and undo it from the bitboard
    pub fn undo(&mut self, delta: &Delta, hash_delta: u64) {
        self.toggle_turn();

        // Undo the hash update
        self.hash ^= hash_delta;

        // Undo the move
        match delta {
            Delta::Explosion(explosion) => self.undo_explosion(explosion),
            Delta::Directional(directional) => self.undo_directional(directional),
        };
    }

    pub fn new() -> Self {
        let mut board = [0; 14];
        board[WHITE] = 0b0000000000000000000000000000000000000000000000001101101111011011;
        board[BLACK] = 0b1101101111011011000000000000000000000000000000000000000000000000;
        board[1] = board[WHITE] | board[BLACK];
        Self {
            board,
            turn: WHITE,
            opponent: BLACK,
            hash: ZORBIST_INITIAL,
            robots_white: 12,
            robots_black: 12,
            robots_total: 24,
        }
    }

    pub fn empty() -> Self {
        Self {
            board: [0; 14],
            turn: WHITE,
            opponent: BLACK,
            hash: ZORBIST_INITIAL,
            robots_white: 0,
            robots_black: 0,
            robots_total: 0,
        }
    }

    /// Adds a bot at the u64 position, with specified height and team.
    pub fn with(mut self, pos: u64, height: usize, team: usize) -> Self {
        debug_assert!(pos.count_ones() == 1, "pos must by a non-zero power of 2 to be a valid position");

        self.board[team] |= pos;
        self.board[height] |= pos;
        self.hash ^= ZORBIST_KEY[team][Bitwise::idx(pos) as usize];
        self.robots_white += if team == WHITE { 1 } else { 0 };
        self.robots_black += if team == BLACK { 1 } else { 0 };
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::format::Format;

    use super::*;

    #[test]
    fn undo_after_stacking_height_1_onto_1() {
        let mut board = Bitboard::new();
        let ihash = board.hash;
        let action = Action { source: 0, target: 1, robots: 1 };
        let delta = board.delta(action);
        let hash = board.make(&delta);
        board.undo(&delta, hash);
        assert_eq!(board, Bitboard::new());
    }

    #[test]
    fn undo_after_moving_height_1_onto_empty() {
        let mut board = Bitboard::new();
        let ihash = board.hash;
        let action = Action { source: 1, target: 2, robots: 1 };
        let delta = board.delta(action);
        let hash = board.make(&delta);

        board.undo(&delta, hash);
        assert_eq!(board, Bitboard::new());
    }

    #[test]
    fn undo_after_moving_1_from_2_stack_onto_empty() {
        let mut board = Bitboard::new();
        let action1 = Action { source: 0, target: 1, robots: 1 };
        let delta1 = board.delta(action1);
        let hash1 = board.make(&delta1);
        let action2 = Action { source: 1, target: 2, robots: 1 };
        let delta2 = board.delta(action2);
        let hash2 = board.make(&delta2);
        board.undo(&delta2, hash2);
        let mut correct = Bitboard::new();
        correct.make(&delta1);

        assert_eq!(board, correct);
    }

    #[test]
    fn undo_after_moving_1_from_2_stack_onto_1() {
        let mut board = Bitboard::new();
        let mut correct = Bitboard::new();

        let action = Action { source: 0, target: 1, robots: 1 };
        let delta = board.delta(action);

        board.make(&delta);
        correct.make(&delta);

        let action = Action { source: 1, target: 3, robots: 1 };
        let delta = board.delta(action);

        let hash = board.make(&delta);
        board.undo(&delta, hash);

        assert_eq!(board, correct);
    }

    #[test]
    fn height_avx512() {
        let bb = Bitboard::empty()
            .with(1, 1, WHITE)
            .with(2, 2, WHITE)
            .with(4, 3, WHITE)
            .with(8, 4, WHITE)
            .with(16, 5, WHITE)
            .with(32, 6, WHITE)
            .with(64, 7, WHITE)
            .with(128, 8, WHITE);

        assert_eq!(bb.height_avx512(1), 1);
        assert_eq!(bb.height_avx512(2), 2);
        assert_eq!(bb.height_avx512(4), 3);
        assert_eq!(bb.height_avx512(8), 4);
        assert_eq!(bb.height_avx512(16), 5);
        assert_eq!(bb.height_avx512(32), 6);
        assert_eq!(bb.height_avx512(64), 7);
        assert_eq!(bb.height_avx512(128), 8);
    }
}
