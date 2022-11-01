use std::hint::unreachable_unchecked;

use crate::constants::*;

impl Bitwise {
    #[inline(always)]
    pub fn dist(pos: u64, frame: u64) -> u64 {
        Bitwise::dist_unrolled(pos, frame)
    }

    /// Calculates the total straight line distance from the single set bit in a to the nearest set bit in b
    ///
    /// Will block forever if either pos=0 or frame=0 because the source will never reach any
    /// targets in the frame within any number of spreads.
    #[inline(always)]
    pub fn dist_while(pos: u64, frame: u64) -> u64 {
        let mut dist = 1;
        let mut curr = Bitwise::spread(pos);
        while curr & frame == 0 {
            curr |= Bitwise::spread(curr);
            dist += 1;
        }
        dist
    }

    /// Calculates the total straight line distance from the single set bit in a to the nearest set bit in b
    #[inline(always)]
    pub fn dist_const(pos: u64, frame: u64) -> u64 {
        let d1 = Bitwise::spread(pos);
        let d2 = Bitwise::spread(d1);
        let d3 = Bitwise::spread(d2);
        let d4 = Bitwise::spread(d3);
        let d5 = Bitwise::spread(d4);
        let d6 = Bitwise::spread(d5);
        let d7 = Bitwise::spread(d6);

        let d1c: u64 = (d1 & frame == 0).into();
        let d2c: u64 = (d2 & frame == 0).into();
        let d3c: u64 = (d3 & frame == 0).into();
        let d4c: u64 = (d4 & frame == 0).into();
        let d5c: u64 = (d5 & frame == 0).into();
        let d6c: u64 = (d6 & frame == 0).into();
        let d7c: u64 = (d7 & frame == 0).into();

        1 + d1c + d2c + d3c + d4c + d5c + d6c + d7c
    }

    /// Calculates the total straight line distance from the single set bit in a to the nearest set bit in b
    ///
    /// Will result in UB if either pos=0 or frame=0 because the source will not reach any targets in the frame within 7 spreads.
    #[inline(always)]
    pub fn dist_unrolled(pos: u64, frame: u64) -> u64 {
        debug_assert!(pos != 0, "source must have a source (pos != 0) or will cause UB");
        debug_assert!(frame != 0, "frame must have targets (frame != 0) or will cause UB");

        let spread = Bitwise::spread(pos);
        if spread & frame != 0 { return 1 };

        let spread = Bitwise::spread(spread);
        if spread & frame != 0 { return 2 };

        let spread = Bitwise::spread(spread);
        if spread & frame != 0 { return 3 };

        let spread = Bitwise::spread(spread);
        if spread & frame != 0 { return 4 };

        let spread = Bitwise::spread(spread);
        if spread & frame != 0 { return 5 };

        let spread = Bitwise::spread(spread);
        if spread & frame != 0 { return 6 };

        let spread = Bitwise::spread(spread);
        if spread & frame != 0 { return 7 };

        // Max horizontal distance is 7
        unsafe { unreachable_unchecked() }
    }

    /// Finds the number of set bits in a unsigned 64 bit integer using intrinsics if available.
    /// Otherwise, fall back to using Knuth's multiply method shown below for reference.
    ///
    /// ```rust
    /// fn knuth_popcnt(frame: u64) -> u64 {
    ///     let frame = frame - ((frame >> 1) & KNUTH_K1);
    ///     let frame = (frame & KNUTH_K2) + (frame >> 2 & KNUTH_K2);
    ///     ((frame + (frame >> 4)) & KNUTH_K4) * KNUTH_KF >> 56
    /// }
    /// ```
    #[inline(always)]
    pub fn pcnt(frame: u64) -> u64 {
        frame.count_ones() as u64
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

    /// Calculate the least significant set bit and flip all other set bits. This isolates the 
    /// least significant set bit and is the same as 1 << 0-index of the least significant set bit.
    ///
    /// If there is no set bits (0), then the function will panic.
    #[inline(always)]
    pub fn lsb(frame: u64) -> u64 {
        debug_assert!(frame != 0, "will panic if no set bits exists due to 0u64 - 1");
        frame ^ (frame & (frame - 1))
    }

    /// Finds the 0-indexed position of the only set bit using DeBruijn's multiply and lookup method
    #[inline(always)]
    pub fn idx(frame: u64) -> usize {
        Bitwise::idx_intrinsic(frame)
    }

    /// Finds the 0-indexed position of the only set bit using DeBruijn's multiply and lookup method
    #[inline(always)]
    pub fn idx_debruijn(frame: u64) -> usize {
        DEBRUIJN_LOOKUP[((DEBRUIJN_MULTIPLY.wrapping_mul(frame & ((!frame) + 1))) >> 58) as usize]
    }

    #[inline(always)]
    pub fn idx_intrinsic(frame: u64) -> usize {
        frame.trailing_zeros() as usize
    }

    /// Converts an index 0-63 to one of 64 u64s representing the position on the bitboard of the index
    #[inline(always)]
    pub fn pos(idx: u8) -> u64 {
        1 << idx
    }

    /// Sets all bits that are adjacent to any set bit. Includes diagonally, but never includes
    /// the original set bits. Does not overflow edges, see masks for details.
    #[inline(always)]
    pub fn adj(pos: u64) -> u64 {
        Bitwise::adj_grid(pos)
    }


    /// Shifts the source position left and right then up and down (taking care to not overflow).
    /// This is more optimized: 4 SHIFTs, 2 ANDs, 4 ORs and 1 XOR = 11 ops
    #[inline(always)]
    pub fn adj_grid(pos: u64) -> u64 {
        const LEFT_OVERFLOW: u64 = 0x0101010101010100;
        const RIGHT_OVERFLOW: u64 = 0x8080808080808080;
        // Shift the row left (<< 1) and right (>> 1) and mask off any bits that overflow.
        let row = pos | (!LEFT_OVERFLOW & pos << 1) | (!RIGHT_OVERFLOW & pos >> 1);
        // Shift the row up (<< 8) and down (>> 8)
        let grid = row | (row << 8) | (row >> 8);
        // Don't include the source position as adjacent to itself
        grid ^ pos
    }

    /// Shifts each source position in each 9 cardinal directions. 
    /// This is safe but wasteful: 8 SHIFTs, 9 ANDs, 9 ORs and 1 NEG = 27 ops.
    #[inline(always)]
    pub fn adj_slow(pos: u64) -> u64 {
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

    /// Sets all bits that are adjacent to any set bit. Includes diagonally, and also the original
    /// set bits. Does not overflow edges, see masks for details.
    #[inline(always)]
    pub fn spread(pos: u64) -> u64 {
        const LEFT_OVERFLOW: u64 = 0x0101010101010101;
        const RIGHT_OVERFLOW: u64 = 0x8080808080808080;
        // Shift the row left (<< 1) and right (>> 1) and mask off any bits that overflow.
        let row = pos | (!LEFT_OVERFLOW & pos << 1) | (!RIGHT_OVERFLOW & pos >> 1);
        // Shift the row up (<< 8) and down (>> 8)
        let grid = row | (row << 8) | (row >> 8);
        // Don't include the source position as adjacent to itself
        grid
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

    #[test]
    fn lsb() {
        let cases = [
            // (0b0000, 0b0000),
            (0b0001, 0b0001),
            (0b0010, 0b0010),
            (0b0011, 0b0001),
            (0b0100, 0b0100),
            (0b0101, 0b0001),
            (0b0110, 0b0010),
            (0b0111, 0b0001),
            (0b1000, 0b1000),
            (0b1001, 0b0001),
            (0b1010, 0b0010),
            (0b1011, 0b0001),
            (0b1100, 0b0100),
            (0b1101, 0b0001),
            (0b1110, 0b0010),
            (0b1111, 0b0001),
        ];

        for (case, expected) in cases {
            assert_eq!(Bitwise::lsb(case), expected, "lsb({})={:#04b} != {:#04b}", case, Bitwise::lsb(case), expected);
        }
    }
}
