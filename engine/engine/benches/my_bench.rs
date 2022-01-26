use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::{
    bitboard::Bitboard,
    constants::{LOSS, WIN},
    search::Search,
    transpose::TranspositionTable,
};

fn search(n: u8) {
    let mut tt = TranspositionTable::new(20);
    let mut bb = Bitboard::new();

    for i in 1..n {
        Search::negamax_move(&mut bb, &mut tt, i, LOSS, WIN);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("search depth=5", |b| b.iter(|| search(black_box(5))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
