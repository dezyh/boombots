#![no_main]

use libfuzzer_sys::fuzz_target;
use engine::bitwise::Bitwise;

fuzz_target!(|data: [u8; 16]| {
    let d1: [u8; 8] = data[0..8].try_into().expect("can get subslice");
    let d2: [u8; 8] = data[8..16].try_into().expect("can get subslice");

    let source = u64::from_le_bytes(d1);
    let targets = u64::from_le_bytes(d2);

    if source != 0 && targets != 0 {
        let w = Bitwise::dist_while(source, targets);
        let c = Bitwise::dist_const(source, targets);
        let u = Bitwise::dist_unrolled(source, targets);
        assert_eq!(w, c);
        assert_eq!(w, u);
    }
});
