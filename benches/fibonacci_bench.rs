//! Benchmark for the Fibonacci functions
//!
//! Run with: `cargo bench --bench fibonacci_bench`

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use zeck::{fast_doubling_fibonacci_biguint, slow_fibonacci_biguint_iterative};

/// Fibonacci indices to benchmark.
const FIBONACCI_INDICES: [u64; 4] = [10_000, 20_000, 50_000, 100_000];

fn bench_slow_fibonacci_biguint_iterative(c: &mut Criterion) {
    let mut group = c.benchmark_group("slow_fibonacci_biguint_iterative");

    for &fi in &FIBONACCI_INDICES {
        group.bench_with_input(BenchmarkId::from_parameter(fi), &fi, |b, &fi| {
            b.iter(|| {
                let result = slow_fibonacci_biguint_iterative(black_box(fi));
                black_box(result);
            });
        });
    }

    group.finish();
}

fn bench_fast_doubling_fibonacci_biguint(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_doubling_fibonacci_biguint");

    for &fi in &FIBONACCI_INDICES {
        group.bench_with_input(BenchmarkId::from_parameter(fi), &fi, |b, &fi| {
            b.iter(|| {
                let result = fast_doubling_fibonacci_biguint(black_box(fi));
                black_box(result);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_slow_fibonacci_biguint_iterative,
    bench_fast_doubling_fibonacci_biguint
);
criterion_main!(benches);
