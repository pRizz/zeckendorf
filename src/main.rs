
use std::time::Instant;
use num_bigint::BigUint;
use zeckendorf_rs::*;

// Example usages:
// cargo run --bin zeckendorf-rs

fn main() {
    let start_time = Instant::now();

    for i in 0..20 {
        println!("Bit count for {}: {}", i, bit_count_for_number(i));
    }
    for i in 0..20 {
        println!("The {i}th Fibonacci number is: {}", memoized_fibonacci_recursive(i));
    }
    for i in 0..20 {
        println!("The bigint {i}th Fibonacci number is: {}", memoized_fibonacci_bigint_recursive(i));
    }
    for i in 0..20 {
        println!("Zeckendorf descending list for {}: {:?}", i, memoized_zeckendorf_list_descending_for_integer(i));
    }
    for i in 0..20 {
        println!("Zeckendorf descending list for {}: {:?}", i, memoized_zeckendorf_list_descending_for_bigint(&BigUint::from(i as u64)));
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

    let big_fibonacci = fibonacci_bigint_iterative(&BigUint::from(limit));
    println!("The {limit}th Fibonacci number is: {}", big_fibonacci);
    // it takes 6 bits to represent the 10th Fibonacci number
    // it takes 69424 bits to represent the 100_000th Fibonacci number
    // it takes 694241 bits to represent the 1_000_000th Fibonacci number
    println!("it takes {} bits to represent the {limit}th Fibonacci number", big_fibonacci.bits());

    test_zeckendorf_compress_and_decompress_number(12_u64);
    // 255 is 0b11111111 which is 8 bits
    test_zeckendorf_compress_and_decompress_number(255_u64);
    // Test two byte boundary
    // 256 is 0b100000000 which is 9 bits
    test_zeckendorf_compress_and_decompress_number(256_u64);
    
    test_zeckendorf_compress_and_decompress_file("generated_data/random_data_1025_bytes.bin");

    let end_time = Instant::now();
    println!("Time taken: {:?}", end_time.duration_since(start_time));

}

fn test_zeckendorf_compress_and_decompress_number(number: u64) {
    println!("Number to compress: {:?}", number);
    let data = BigUint::from(number).to_bytes_be();
    println!("Number as big endian bytes: {:?}", data);
    let compressed_data = zeckendorf_compress(&data);
    println!("Compressed data: {:?}", compressed_data);
    let decompressed_data = zeckendorf_decompress(&compressed_data);
    println!("Decompressed data: {:?}", decompressed_data);
    let decompressed_number = BigUint::from_bytes_be(&decompressed_data);
    println!("Decompressed number: {:?}", decompressed_number);
}

fn test_zeckendorf_compress_and_decompress_file(filename: &str) {
    println!("Testing compression and decompression of file: {:?}", filename);
    let data = std::fs::read(filename).expect("Failed to read file");
    // println!("Data: {:?}", data);
    // Data size
    let data_size = data.len();
    println!("Data bytes size: {:?}", data_size);
    let compressed_data = zeckendorf_compress(&data);
    // println!("Compressed data: {:?}", compressed_data);
    // Compressed data size
    let compressed_data_size = compressed_data.len();
    println!("Compressed data size: {:?}", compressed_data_size);
    let decompressed_data = zeckendorf_decompress(&compressed_data);
    // println!("Decompressed data: {:?}", decompressed_data);
    // Decompressed data size
    let decompressed_data_size = decompressed_data.len();
    println!("Decompressed data size: {:?}", decompressed_data_size);
    assert_eq!(data, decompressed_data);
    // Compression ratio
    let compression_ratio = compressed_data_size as f64 / data_size as f64;
    println!("Compression ratio was {x:0.3}%", x = compression_ratio * 100.0);
    if compression_ratio > 1.0 {
        println!("Compressing this file was {x:0.3}% worse", x = (compression_ratio - 1.0) * 100.0);
    } else {
        println!("Compressing this file was {x:0.3}% better", x = (1.0 - compression_ratio) * 100.0);
    }
}
