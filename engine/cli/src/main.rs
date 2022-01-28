use engine::{
    bitboard::Bitboard,
    constants::{LOSS, WIN},
    search::Search,
    transpose::TranspositionTable,
};

fn search(n: u8) {
    let mut tt = TranspositionTable::new(28);
    let mut bb = Bitboard::new();

    for i in 1..n {
        let result = Search::negamax_move(&mut bb, &mut tt, i, LOSS, WIN);
        println!(
            "[depth={}] {} <= {:?} || t={} n={}",
            result.depth, result.score, result.action, result.trans, result.nodes
        );
    }
}

fn main() {
    search(20);
}
