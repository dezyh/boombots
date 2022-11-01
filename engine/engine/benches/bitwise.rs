use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::bitwise::Bitwise;

const SINGLE: u64 = 0b0000100000000000000000000000000000000000000000000000000000000000;
const MULTIPLE: u64 = 0b1101101111111111111110111011111111101110111111111111111000111111;

fn dist(c: &mut Criterion) {
    const CLOSE_SOURCE: u64 = 0x40000;
    const CLOSE_TARGETS: u64 = 0x4004000000010;
    const FAR_SOURCE: u64 = 0x1;
    const FAR_TARGETS: u64 = 0x8000000000000000;

    c.bench_function("dist_while(SHORT)=2", |b| b.iter(|| Bitwise::dist_while(black_box(CLOSE_SOURCE), black_box(CLOSE_TARGETS))));
    c.bench_function("dist_const(SHORT)=2", |b| b.iter(|| Bitwise::dist_const(black_box(CLOSE_SOURCE), black_box(CLOSE_TARGETS))));
    c.bench_function("dist_unrolled(SHORT)=2", |b| b.iter(|| Bitwise::dist_unrolled(black_box(CLOSE_SOURCE), black_box(CLOSE_TARGETS))));

    c.bench_function("dist_while(FAR)=7", |b| b.iter(|| Bitwise::dist_while(black_box(FAR_SOURCE), black_box(FAR_TARGETS))));
    c.bench_function("dist_const(FAR)=7", |b| b.iter(|| Bitwise::dist_const(black_box(FAR_SOURCE), black_box(FAR_TARGETS))));
    c.bench_function("dist_unrolled(FAR)=7", |b| b.iter(|| Bitwise::dist_unrolled(black_box(FAR_SOURCE), black_box(FAR_TARGETS))));
}

fn lsb(c: &mut Criterion) {
    c.bench_function("lsb()", |b| b.iter(|| Bitwise::lsb(black_box(MULTIPLE))));
}

fn popcnt(c: &mut Criterion) {
    c.bench_function("popcnt()", |b| b.iter(|| Bitwise::pcnt(black_box(MULTIPLE))));
}

fn idx(c: &mut Criterion) {
    c.bench_function("idx()", |b| b.iter(|| Bitwise::idx(black_box(SINGLE))));
}

fn adj(c: &mut Criterion) {
    c.bench_function("adj()", |b| b.iter(|| Bitwise::adj(black_box(SINGLE))));
}

criterion_group!(benches, dist, lsb, adj, idx, popcnt);
criterion_main!(benches);
