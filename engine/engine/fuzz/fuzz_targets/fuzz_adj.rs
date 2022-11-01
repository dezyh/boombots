#![no_main]

use libfuzzer_sys::fuzz_target;
use engine::bitwise::Bitwise;

fuzz_target!(|data: [u8; 8]| {
    let frame = u64::from_le_bytes(data);

    let grid = Bitwise::adj_grid(frame);
    let slow = Bitwise::adj_slow(frame);

    assert_eq!(grid, slow);
});
