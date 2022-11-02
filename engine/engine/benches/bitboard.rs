use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::{bitboard::Bitboard, constants};

fn criterion_benchmark(c: &mut Criterion) {
    let bb = Bitboard::new()
        .with(1, 1, constants::WHITE)
        .with(2, 2, constants::WHITE)
        .with(4, 3, constants::WHITE)
        .with(8, 4, constants::WHITE)
        .with(16, 5, constants::WHITE)
        .with(32, 6, constants::WHITE)
        .with(64, 7, constants::WHITE)
        .with(128, 8, constants::WHITE)
        .with(256, 9, constants::WHITE)
        .with(512, 10, constants::WHITE)
        .with(1024, 11, constants::WHITE)
        .with(2048, 12, constants::WHITE);

    c.bench_function("height(1)", |bench| bench.iter(|| bb.height(black_box(1))));
    c.bench_function("height(2)", |bench| bench.iter(|| bb.height(black_box(2))));
    c.bench_function("height(3)", |bench| bench.iter(|| bb.height(black_box(4))));
    c.bench_function("height(4)", |bench| bench.iter(|| bb.height(black_box(8))));
    c.bench_function("height(5)", |bench| bench.iter(|| bb.height(black_box(16))));
    c.bench_function("height(6)", |bench| bench.iter(|| bb.height(black_box(32))));
    c.bench_function("height(7)", |bench| bench.iter(|| bb.height(black_box(64))));
    c.bench_function("height(8)", |bench| bench.iter(|| bb.height(black_box(128))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
