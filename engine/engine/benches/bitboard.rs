use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::bitboard::Bitboard;

fn criterion_benchmark(c: &mut Criterion) {
    let bitboard = Bitboard::new();
    c.bench_function("height branching", |b| b.iter(|| bitboard.height(black_box(4))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
