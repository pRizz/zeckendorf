//! Benchmark for the Zeckendorf compression and decompression functions
//!
//! Run with: cargo bench
//!
//! The benchmarks are run for the following sizes: 1, 4, 16, 64, 256, 1024, 4096
//!
//! The benchmarks are run for the following functions:
//! - compress
//! - decompress
//! - round trip

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use zeckendorf_rs::{zeckendorf_compress_be, zeckendorf_decompress_be};

/// Generates test data of the given size
///
/// The test data is a vector of bytes, where the bytes are the numbers from 0 to size - 1, modulo 256. This is to ensure that the data has a simple variety of values. TODO: Consider different data distributions in the future.
///
/// # Examples
///
/// ```
/// # use zeckendorf_bench::generate_test_data;
/// let data = generate_test_data(10);
/// assert_eq!(data, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
/// ```
fn generate_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

fn bench_compress(c: &mut Criterion) {
    let mut group = c.benchmark_group("compress");

    let sizes = vec![1, 4, 16, 64, 256, 1024, 4096];

    for size in sizes {
        let data = generate_test_data(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| zeckendorf_compress_be(black_box(data)));
        });
    }

    group.finish();
}

fn bench_decompress(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompress");

    let sizes = vec![1, 4, 16, 64, 256, 1024, 4096];

    for size in sizes {
        let data = generate_test_data(size);
        let compressed = zeckendorf_compress_be(&data);
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &compressed,
            |b, compressed| {
                b.iter(|| zeckendorf_decompress_be(black_box(compressed)));
            },
        );
    }

    group.finish();
}

fn bench_round_trip(c: &mut Criterion) {
    let mut group = c.benchmark_group("round_trip");

    let sizes = vec![1, 4, 16, 64, 256, 1024, 4096];

    for size in sizes {
        let data = generate_test_data(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| {
                let compressed = zeckendorf_compress_be(black_box(data));
                let decompressed = zeckendorf_decompress_be(&compressed);
                black_box(decompressed);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_compress, bench_decompress, bench_round_trip);
criterion_main!(benches);
