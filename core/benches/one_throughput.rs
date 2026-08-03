use ashwa::search_one;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

const KB: usize = 0x400;
const MB: usize = KB * KB;

const SIZES: [usize; 0x0A] = [
    0x200,
    1 * KB,
    0x20 * KB,
    0x40 * KB,
    0x200 * KB,
    1 * MB,
    0x20 * MB,
    0x40 * MB,
    0x80 * MB,
    0x200 * MB,
];

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_one");

    for &size in &SIZES {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("worst_case_not_found", size), &size, |b, &s| {
            let needle = 0x0Au8;
            let haystack = vec![0u8; s];

            b.iter(|| search_one(black_box(&haystack), black_box(needle)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_throughput);
criterion_main!(benches);
