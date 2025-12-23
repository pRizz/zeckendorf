//! This binary is basically a playground/scratchpad used to test the library and its functions.
//!
//! Example usages:
//! `cargo run --release --bin zeckendorf`

use num_bigint::BigUint;
use num_format::ToFormattedString;
use std::time::Instant;
use zeckendorf_rs::*;

fn main() {
    let start_time = Instant::now();

    for i in 0..20 {
        println!("Bit count for {}: {}", i, bit_count_for_number(i));
    }
    for i in 0..20 {
        println!(
            "The {i}th Fibonacci number is: {}",
            memoized_fibonacci_recursive(i)
        );
    }
    for i in 0..20 {
        println!(
            "The bigint {i}th Fibonacci number is: {}",
            memoized_fibonacci_bigint_iterative(i)
        );
    }
    for i in 0..20 {
        println!(
            "Zeckendorf descending list for {}: {:?}",
            i,
            memoized_zeckendorf_list_descending_for_integer(i)
        );
    }
    for i in 0..20 {
        println!(
            "Zeckendorf descending list for {}: {:?}",
            i,
            memoized_zeckendorf_list_descending_for_bigint(&BigUint::from(i as u64))
        );
    }
    for i in 0..20 {
        let zld = memoized_zeckendorf_list_descending_for_integer(i);
        println!("Zeckendorf descending list for {}: {:?}", i, zld);
        let ezld = zl_to_ezl(&zld);
        println!("Effective Zeckendorf list descending for {}: {:?}", i, ezld);
        let ezba = ezba_from_ezld(&ezld);
        println!("Effective Zeckendorf bits ascending for {}: {:?}", i, ezba);
    }

    // for i in 0..20 {
    //     let fibonacci = memoized_fibonacci_bigint_iterative(BigUint::from(i as u64));
    //     println!("The {i}th Fibonacci number is: {}", fibonacci);
    //     println!("it takes {} bits to represent the {i}th Fibonacci number", fibonacci.bits());
    // }

    let limit = 10u64;
    // Get stack overflow at recursive fibonacci at 100_000u64;
    // Time taken: ~674.49ms for iterative
    // let limit = 100_000u64;
    // Time taken: ~65.6s for iterative
    // let limit = 1_000_000u64;

    let big_fibonacci = memoized_fibonacci_bigint_iterative(limit);
    println!("The {limit}th Fibonacci number is: {}", big_fibonacci);
    // it takes 6 bits to represent the 10th Fibonacci number
    // it takes 69424 bits to represent the 100_000th Fibonacci number
    // it takes 694241 bits to represent the 1_000_000th Fibonacci number
    println!(
        "it takes {} bits to represent the {limit}th Fibonacci number",
        big_fibonacci.bits()
    );

    test_zeckendorf_compress_and_decompress_number(12_u64);
    // 255 is 0b11111111 which is 8 bits
    test_zeckendorf_compress_and_decompress_number(255_u64);
    // Test two byte boundary
    // 256 is 0b100000000 which is 9 bits
    test_zeckendorf_compress_and_decompress_number(256_u64);

    test_zeckendorf_compress_and_decompress_file("generated_data/random_data_1025_bytes.bin");

    flamegraph_zeckendorf_decompress_be();

    test_bit_count_for_all_ones_effective_zeckendorf_bits_ascending();

    test_find_fibonacci_by_bit_count();

    test_slow_fibonacci_bigint_iterative();

    test_slow_fibonacci_bigint_iterative_large(100_000);
    test_slow_fibonacci_bigint_iterative_large(200_000);
    test_slow_fibonacci_bigint_iterative_large(500_000);
    test_slow_fibonacci_bigint_iterative_large(1_000_000);

    let end_time = Instant::now();
    println!("Time taken: {:?}", end_time.duration_since(start_time));
}

fn test_zeckendorf_compress_and_decompress_number(number: u64) {
    println!("Number to compress: {:?}", number);
    let data = BigUint::from(number).to_bytes_be();
    println!("Number as big endian bytes: {:?}", data);
    let compressed_data = zeckendorf_compress_be(&data);
    println!("Compressed data: {:?}", compressed_data);
    let decompressed_data = zeckendorf_decompress_be(&compressed_data);
    println!("Decompressed data: {:?}", decompressed_data);
    let decompressed_number = BigUint::from_bytes_be(&decompressed_data);
    println!("Decompressed number: {:?}", decompressed_number);
}

fn test_zeckendorf_compress_and_decompress_file(filename: &str) {
    println!(
        "Testing compression and decompression of file: {:?}",
        filename
    );
    let data = std::fs::read(filename).expect("Failed to read file");
    // println!("Data: {:?}", data);
    // Data size
    let data_size = data.len();
    println!("Data bytes size: {:?}", data_size);
    let compressed_data = zeckendorf_compress_be(&data);
    // println!("Compressed data: {:?}", compressed_data);
    // Compressed data size
    let compressed_data_size = compressed_data.len();
    println!("Compressed data size: {:?}", compressed_data_size);
    let decompressed_data = zeckendorf_decompress_be(&compressed_data);
    // println!("Decompressed data: {:?}", decompressed_data);
    // Decompressed data size
    let decompressed_data_size = decompressed_data.len();
    println!("Decompressed data size: {:?}", decompressed_data_size);
    assert_eq!(data, decompressed_data);
    // Compression ratio
    let compression_ratio = compressed_data_size as f64 / data_size as f64;
    println!(
        "Compression ratio was {x:0.3}%",
        x = compression_ratio * 100.0
    );
    if compression_ratio > 1.0 {
        println!(
            "Compressing this file was {x:0.3}% worse",
            x = (compression_ratio - 1.0) * 100.0
        );
    } else {
        println!(
            "Compressing this file was {x:0.3}% better",
            x = (1.0 - compression_ratio) * 100.0
        );
    }
}

/// Runs the zeckendorf_decompress_be function many times to generate a flamegraph showing the hot spots.
/// See the scripts/gen_flamegraph.sh script for more information.
fn flamegraph_zeckendorf_decompress_be() {
    for i in 0..1000000 {
        let data = BigUint::from(i as u64).to_bytes_be();
        let compressed_data = data;
        let decompressed_data = zeckendorf_decompress_be(&compressed_data);
        std::hint::black_box(decompressed_data);
    }
    return;
}

fn test_bit_count_for_all_ones_effective_zeckendorf_bits_ascending() {
    let one_hundred_thousand_ones = vec![1; 100000];
    // println!("One hundred thousand ones: {:?}", one_hundred_thousand_ones);
    let ezla = ezba_to_ezla(&one_hundred_thousand_ones);
    // println!("Effective Zeckendorf list ascending: {:?}", ezla);
    let zla = ezl_to_zl(&ezla);
    // println!("Zeckendorf list ascending: {:?}", zla);
    let bigint = zl_to_bigint(&zla);
    // println!("Bigint: {:?}", bigint);
    println!("Bit count: {:?}", bigint.bits());
}

/// Finds the first Fibonacci number that has at least the specified number of bits.
/// Returns a tuple containing the Fibonacci index and the Fibonacci number value.
///
/// # Arguments
///
/// * `target_bits` - The minimum number of bits the Fibonacci number should have
///
/// # Returns
///
/// A tuple `(u64, BigUint)` where:
/// - The first element is the Fibonacci index
/// - The second element is the Fibonacci number value
fn find_fibonacci_by_bit_count(target_bits: u64) -> (u64, BigUint) {
    let mut index = 0u64;
    loop {
        let fibonacci = memoized_fibonacci_bigint_iterative(index);
        let bit_count = fibonacci.bits();
        if bit_count >= target_bits {
            return (index, (*fibonacci).clone());
        }
        index += 1;
    }
}

fn test_find_fibonacci_by_bit_count() {
    let start_time = Instant::now();
    for i in (500..=1500).step_by(100) {
        let (index, fibonacci) = find_fibonacci_by_bit_count(i);
        println!(
            "The index of the Fibonacci number that has at least {i} bits is: {:?}, at bit count: {:?}",
            index,
            fibonacci.bits()
        );
        println!(
            "The Fibonacci number that has at least {i} bits is: {:?}",
            fibonacci
        );
    }
    let end_time = Instant::now();
    println!(
        "Time taken to find Fibonacci numbers by bit count: {:?}",
        end_time.duration_since(start_time)
    );
}

fn test_slow_fibonacci_bigint_iterative() {
    println!("Testing slow Fibonacci bigint iterative function");
    for i in 0..20 {
        let fibonacci = slow_fibonacci_bigint_iterative(i);
        println!("The {i}th Fibonacci number is: {}", fibonacci);
    }
}

fn test_slow_fibonacci_bigint_iterative_large(fi: u64) {
    println!(
        "Testing slow Fibonacci bigint iterative function for index: {}",
        fi.to_formatted_string(&num_format::Locale::en)
    );
    let start_time = Instant::now();
    let fibonacci = slow_fibonacci_bigint_iterative(fi);
    std::hint::black_box(fibonacci);
    // println!("The {fi}th Fibonacci number is: {}", fibonacci);
    let end_time = Instant::now();
    println!(
        "It took {:0.3?} to calculate the {}th Fibonacci number",
        end_time.duration_since(start_time),
        fi.to_formatted_string(&num_format::Locale::en)
    );
}
