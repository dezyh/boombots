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
    pub board: [u64; 14],
    pub robots_white: i16,
    pub robots_black: i16,
    pub robots_total: i16,
}
impl Bitboard {
    /// Finds the height of the robot at the given position
    fn height(&self, pos: u64) -> u8 {
        // TODO: Rewrite this without branching
        for i in 1..=12 {
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
        self.turn = if self.turn == WHITE { BLACK } else { WHITE };
    }

    // Takes a precomputed action delta and applies it onto the bitboard
    pub fn make(&mut self, delta: &Delta) -> u64 {
        // Update the board
        let hash_delta = match delta {
            Delta::Explosion(explosion) => self.make_explosion(explosion),
            Delta::Directional(directional) => self.make_directional(directional),
        };

        // Toggle the turn player
        self.toggle_turn();

        // Update the zorbist hash
        self.hash ^= hash_delta;
        self.hash ^= ZORBIST_TURN;

        self.hash
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
        self.hash ^= ZORBIST_TURN;
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
            hash: ZORBIST_INITIAL,
            robots_white: 0,
            robots_black: 0,
            robots_total: 0,
        }
    }

    pub fn with(mut self, pos: u64, height: usize, team: usize) -> Self {
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
    use super::*;

    #[test]
    fn undo_after_stacking_height_1_onto_1() {
        let mut board = Bitboard::new();
        let action = Action { source: 0, target: 1, robots: 1 };

        let delta = board.delta(action);
        println!("{:?}", delta);
        let hash = board.make(&delta);
        board.undo(&delta, hash);

        assert_eq!(board, Bitboard::new());
    }
}
