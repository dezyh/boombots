use crate::constants::*;

impl Bitwise {
    /// Calculates the total straight line distance from the single set bit in a to the nearest set bit in b
    pub fn dist(pos: u64, frame: u64) -> u64 {
        let mut dist = 1;
        let mut curr = Bitwise::adj_any(pos);
        while curr & frame == 0 {
            curr |= Bitwise::adj_any(curr);
            dist += 1;
        }
        dist
    }

    /// Finds the number of set bits in a uint64 using Knuth's multiply method
    pub fn pcnt(frame: u64) -> u64 {
        let frame = frame - ((frame >> 1) & KNUTH_K1);
        let frame = (frame & KNUTH_K2) + (frame >> 2 & KNUTH_K2);
        ((frame + (frame >> 4)) & KNUTH_K4) * KNUTH_KF >> 56
    }

    /// Finds all set bits that are recursively adjacent to the source using a dfs
    pub fn dfs(frame: u64, source: u8) -> u64 {
        let mut mask: u64 = 0;
        let mut explore: u64 = 1 << source;
        let mut explored: u64 = 0;
        while explore != 0 {
            // Pop a position from the explore queue
            let position = Bitwise::lsb(explore);
            explore ^= position;
            // Update the solution mask
            mask |= position & frame;
            // Find unchecked adjacent positions
            let adjacent = Bitwise::adj(position) & !explored;
            // Add the adjacent positions to the explore queue
            explore |= adjacent & frame;
            // Add the current position to the explored queue
            explored |= position;
        }
        mask
    }

    /// Finds the least significant set bit
    pub fn lsb(frame: u64) -> u64 {
        frame ^ (frame & (frame - 1))
    }

    /// Finds the 0-indexed position of the only set bit using DeBruijn's multiply and lookup method
    pub fn idx(frame: u64) -> usize {
        DEBRUIJN_LOOKUP[((DEBRUIJN_MULTIPLY.wrapping_mul(frame & ((!frame) + 1))) >> 58) as usize]
    }

    /// Converts an index 0-63 to one of 64 u64s representing the position on the bitboard of the index
    pub fn pos(idx: u8) -> u64 {
        1 << idx
    }

    /// Sets all bits that are adjacent to any set bit
    pub fn adj_any(pos: u64) -> u64 {
        let mut adj: u64 = 0;
        adj |= (pos & N_SHIFT_MASK) << N_SHIFT;
        adj |= (pos & W_SHIFT_MASK) << W_SHIFT;
        adj |= (pos & E_SHIFT_MASK) >> E_SHIFT;
        adj |= (pos & S_SHIFT_MASK) >> S_SHIFT;
        adj |= (pos & NE_SHIFT_MASK) << NE_SHIFT;
        adj |= (pos & NW_SHIFT_MASK) << NW_SHIFT;
        adj |= (pos & SE_SHIFT_MASK) >> SE_SHIFT;
        adj |= (pos & SW_SHIFT_MASK) >> SW_SHIFT;
        adj & !pos
    }

    /// Sets all bits that are adjacent to the only set bit
    pub fn adj(pos: u64) -> u64 {
        ADJACENT_LOOKUP[Bitwise::idx(pos)]
    }
}
pub struct Bitwise {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idx() {
        for i in 0..64 {
            assert_eq!(Bitwise::idx(1 << i), i, "Bitwise index of {} should be {}", 2 << i, i);
        }
    }
}
