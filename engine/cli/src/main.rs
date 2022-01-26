use engine::{
    bitboard::Bitboard,
    bitwise::Bitwise,
    constants::{BLACK, LOSS, WHITE, WIN},
    format::Format,
    search::Search,
    transpose::TranspositionTable,
};

fn search(n: u8) {
    let mut tt = TranspositionTable::new(29);
    let mut bb = Bitboard::empty()
        .with(Bitwise::pos(0), 2, BLACK)
        .with(Bitwise::pos(10), 1, WHITE)
        .with(Bitwise::pos(40), 1, WHITE);

    Format::frame(bb.board[WHITE], "white");
    Format::frame(bb.board[BLACK], "black");

    for i in 1..n {
        let result = Search::negamax_move(&mut bb, &mut tt, i, LOSS, WIN);
        println!(
            "[depth={}] {} <= {:?} || t={} n={}",
            result.depth, result.score, result.action, result.trans, result.nodes
        );
    }
}

fn main() {
    search(10);
}
