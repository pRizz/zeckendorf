//! Benchmark for the Zeckendorf compression and decompression functions
//!
//! Run with: `cargo bench --bench zeckendorf_bench`
//!
//! The benchmarks are run for a variety of byte sizes, from 4 to 16K.
//!
//! The benchmarks are run for the following functions:
//! - compress
//! - decompress
//! - round trip, which is the compress and decompress functions combined
//!
//! Criterion notes:
//! To save a new named baseline, run:
//!     `cargo bench --bench zeckendorf_bench -- --save-baseline <name_of_new_baseline>`
//! To run a new benchmark and compare to a baseline without saving the results, run:
//!     `cargo bench --bench zeckendorf_bench -- --baseline <name_of_baseline>`
//! To compare two benchmarks that have already been saved:
//!     `cargo bench --bench zeckendorf_bench -- --load-baseline <name_of_newer_baseline> --baseline <name_of_older_baseline>`
//!
//! Any time `cargo bench` is run without any arguments, it will, by default, save the result to a baseline called "new" and compare it to the previous run, called "base".

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

/// The byte sizes to benchmark.
///
/// TODO: test larger sizes. Right now, the 16K benchmark emits a warning about taking too long.
const BYTE_SIZES_TO_BENCH: [usize; 7] = [4, 16, 64, 256, 1024, 4096, 16384];

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

    for size in BYTE_SIZES_TO_BENCH {
        let data = generate_test_data(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| {
                let compressed = zeck::padless_zeckendorf_compress_be_dangerous(black_box(data));
                black_box(compressed);
            });
        });
    }

    group.finish();
}

fn bench_decompress(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompress");

    for size in BYTE_SIZES_TO_BENCH {
        let data = generate_test_data(size);
        let compressed = zeck::padless_zeckendorf_compress_be_dangerous(&data);

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &compressed,
            |b, compressed| {
                b.iter(|| {
                    let decompressed =
                        zeck::padless_zeckendorf_decompress_be_dangerous(black_box(compressed));
                    black_box(decompressed);
                });
            },
        );
    }

    group.finish();
}

fn bench_round_trip(c: &mut Criterion) {
    let mut group = c.benchmark_group("round_trip");

    for size in BYTE_SIZES_TO_BENCH {
        let data = generate_test_data(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| {
                let compressed_data =
                    zeck::padless_zeckendorf_compress_be_dangerous(black_box(data));
                let decompressed =
                    zeck::padless_zeckendorf_decompress_be_dangerous(&compressed_data);
                black_box(decompressed);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_compress, bench_decompress, bench_round_trip);
criterion_main!(benches);
