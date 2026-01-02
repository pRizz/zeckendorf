//! This binary is basically a playground/scratchpad used to test the library and its functions.
//!
//! Example usages:
//! `cargo run --release --bin zeckendorf`

use num_bigint::BigUint;
use num_format::ToFormattedString;
use rand::RngCore;
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
            memoized_slow_fibonacci_recursive(i)
        );
    }
    for i in 0..20 {
        println!(
            "The bigint {i}th Fibonacci number is: {}",
            memoized_slow_fibonacci_biguint_iterative(i)
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
            memoized_zeckendorf_list_descending_for_biguint(&BigUint::from(i as u64))
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
    // Get stack overflow at recursive Fibonacci at 100_000u64;
    // Time taken: ~674.49ms for iterative
    // let limit = 100_000u64;
    // Time taken: ~65.6s for iterative
    // let limit = 1_000_000u64;

    let big_fibonacci = memoized_slow_fibonacci_biguint_iterative(limit);
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

    // _test_slow_fibonacci_bigint_iterative_large(100_000);
    // _test_slow_fibonacci_bigint_iterative_large(200_000);
    // _test_slow_fibonacci_bigint_iterative_large(500_000);
    // _test_slow_fibonacci_bigint_iterative_large(1_000_000);

    test_fast_doubling_fibonacci_bigint();

    all_ones_decompressions();

    test_all_ones_zeckendorf_ratios();

    test_phi_squared_and_all_ones_zeckendorf_ratios();

    test_all_ones_zeckendorf_bits_to_binary_bits_ratios();

    test_decompressing_large_random_data();

    print_aozns_as_binary_bits();

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
    let bigint = all_ones_zeckendorf_to_biguint(100000);
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
        let fibonacci = memoized_slow_fibonacci_biguint_iterative(index);
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
        let fibonacci = slow_fibonacci_biguint_iterative(i);
        println!("The {i}th Fibonacci number is: {}", fibonacci);
    }
}

fn _test_slow_fibonacci_bigint_iterative_large(fi: u64) {
    println!(
        "Testing slow Fibonacci bigint iterative function for index: {}",
        fi.to_formatted_string(&num_format::Locale::en)
    );
    let start_time = Instant::now();
    let fibonacci = slow_fibonacci_biguint_iterative(fi);
    std::hint::black_box(fibonacci);
    // println!("The {fi}th Fibonacci number is: {}", fibonacci);
    let end_time = Instant::now();
    println!(
        "It took {:0.3?} to calculate the {}th Fibonacci number",
        end_time.duration_since(start_time),
        fi.to_formatted_string(&num_format::Locale::en)
    );
}

fn test_fast_doubling_fibonacci_bigint() {
    println!("Testing fast doubling Fibonacci bigint function");
    let fibonacci = memoized_fast_doubling_fibonacci_biguint(100);
    println!("The 100th Fibonacci number is: {}", fibonacci);
    let cache = zeckendorf_rs::FAST_DOUBLING_FIBONACCI_BIGUINT_CACHE
        .read()
        .expect("Failed to read fast doubling Fibonacci cache");
    println!(
        "Querying the 100th Fibonacci number, using the fast doubling algorithm, generated only {} cached Fibonacci numbers",
        cache.len()
    );
    assert_eq!(cache.len(), 10);
    let mut sorted_cache = cache.iter().collect::<Vec<_>>();
    sorted_cache.sort_by_key(|(fi, _)| *fi);
    for (fi, value) in sorted_cache.iter() {
        println!(
            "The {fi}th Fibonacci number, using the fast doubling algorithm, is: {}",
            value
        );
    }
}

/// Tests the decompression of all ones data of varying sizes.
/// This is interesting to see how big an all ones Zeckendorf bits list get expanded to when "decompressed".
/// After testing larger byte sizes, it seems like the decompressed data converges around being ~38.85% larger that the original all ones bits.
/// More testing is needed to verify larger byte sizes.
/// Larger byte sizes (like 100,000 bytes) take an extreme amount of memory to test, on the order of 60 GB, and can cause the process to be killed by the OS (exit code 137).
fn all_ones_decompressions() {
    let mut all_ones_byte_size = 10;
    let size_multipier = 10;
    let max_byte_size = 10_000;
    while all_ones_byte_size <= max_byte_size {
        let start_time = Instant::now();
        println!("Testing all ones byte size: {}", all_ones_byte_size);
        let mock_compressed_all_ones_data = vec![0xFF; all_ones_byte_size];
        // println!("Mock compressed data byte size: {:?}", mock_compressed_data.len());
        // println!("Mock compressed data raw bit size: {:?}", mock_compressed_data.len() * 8);
        let mock_decompressed_data = zeckendorf_decompress_be(&mock_compressed_all_ones_data);
        println!(
            "Mock decompressed data byte size: {:?}",
            mock_decompressed_data.len()
        );
        println!(
            "Mock decompressed data raw bit size: {:?}",
            mock_decompressed_data.len() * 8
        );
        let size_ratio =
            mock_compressed_all_ones_data.len() as f64 / mock_decompressed_data.len() as f64;
        println!("Size ratio: {x:0.3}", x = size_ratio);
        println!(
            "If an input data happens to be Zeckendorf compressed as all ones of size {all_ones_byte_size} bytes, the decompressed data will be {} bytes",
            mock_decompressed_data.len()
        );
        println!(
            "This means the input data was compressed by {x:0.3}%",
            x = (1.0 - size_ratio) * 100.0
        );
        println!(
            "In other words, the compressed data was {x:0.3}% of the original data",
            x = size_ratio * 100.0
        );
        println!(
            "In other other words, the decompressed data was {x:0.3}% of the size of the compressed data",
            x = 1.0 / size_ratio * 100.0
        );
        let end_time = Instant::now();
        println!(
            "Time taken to test all ones decompression for byte size {all_ones_byte_size}: {:?}",
            end_time.duration_since(start_time)
        );
        all_ones_byte_size *= size_multipier;
    }
}

/// Tests the ratios of all ones Zeckendorf numbers to the previous all ones Zeckendorf numbers.
///
/// It turns out that this ratio seems to converge to the golden ratio plus one, which also apparently equals the square of the golden ratio, or phi squared.
fn test_all_ones_zeckendorf_ratios() {
    let start_time = Instant::now();
    let mut prev = all_ones_zeckendorf_to_biguint(1);

    // We stop at 46 because the 47th all ones Zeckendorf number is too large to fit in a u64, which causes the f64 approximation to be inaccurate.
    for i in 2..=46 {
        let curr = all_ones_zeckendorf_to_biguint(i);
        println!("The {i}th all ones Zeckendorf number is: {}", curr);
        let ratio = biguint_to_approximate_f64(&curr) / biguint_to_approximate_f64(&prev);
        println!(
            "The {i}th all ones Zeckendorf number is {ratio} times larger than the {}th all ones Zeckendorf number",
            i - 1
        );
        let delta = ratio - PHI_SQUARED;
        println!("The delta between the ratio and phi squared is {delta}");
        prev = curr;
    }
    let end_time = Instant::now();
    println!(
        "Time taken to test all ones Zeckendorf ratio: {:?}",
        end_time.duration_since(start_time)
    );
}

/// Helper function to convert BigUint to f64 for plotting.
/// For values that don't fit in f64, uses an approximation based on bits, but capped at 1023 bits to avoid overflow.
fn biguint_to_approximate_f64(value: &BigUint) -> f64 {
    // Try to convert to u64 first
    let digits = value.to_u64_digits();
    if digits.len() == 1 {
        digits[0] as f64
    } else if digits.is_empty() {
        0.0
    } else {
        // For very large numbers, approximate using bits
        // We'll use: value ≈ 2^bits, but cap to avoid overflow
        let bits = value.bits() as f64;
        // f64::MAX is around 1.8e308, which corresponds to 2^1024 - 1
        // So we cap bits at 1023 to avoid overflow
        let capped_bits = bits.min(1023.0);
        2_f64.powf(capped_bits)
    }
}

/// Since phi squared to the n seems to be parallel to the all ones Zeckendorf numbers on the plot, I wanted to figure out by how much phi squared is greater than the all ones Zeckendorf numbers. After testing, it seems to converge on the ratio of ~1.3819660112501047.
///
/// This just means that for large n, phi squared to the n is ~1.3819660112501047 times larger than n all-ones Zeckendorf bits.
///
/// This could potentially be used to get a fast approximation of the all ones Zeckendorf number for large n, by using the formula:
/// `all_ones_zeckendorf_number(n) = phi^(2n) / 1.3819660112501047`
///
/// See this plot to get a better intuition about the ratios: `plots/fibonacci_binary_all_ones_power3_phi_squared_0_to_30.png`
fn test_phi_squared_and_all_ones_zeckendorf_ratios() {
    let start_time = Instant::now();

    let mut prev_ratio =
        PHI_SQUARED / biguint_to_approximate_f64(&all_ones_zeckendorf_to_biguint(1));

    // We stop at 46 because the 47th all ones Zeckendorf number is too large to fit in a u64, which causes the f64 approximation to be inaccurate.
    for i in 2..=46 {
        let phi_squared_i = PHI_SQUARED.powi(i as i32);
        println!("The {i}th phi squared is: {phi_squared_i}");
        let curr = all_ones_zeckendorf_to_biguint(i);
        println!("The {i}th all ones Zeckendorf number is: {}", curr);
        let ratio = phi_squared_i / biguint_to_approximate_f64(&curr);
        println!(
            "The {i}th phi squared is {ratio} times larger than {i}th all ones Zeckendorf number"
        );
        let ratio_delta = ratio - prev_ratio;
        println!("The delta between the ratio and the previous ratio is {ratio_delta}");
        let ratio_growth_rate = ratio_delta / prev_ratio;
        println!("The growth rate of the ratio is {ratio_growth_rate}");
        prev_ratio = ratio;
    }

    let end_time = Instant::now();
    println!(
        "Time taken to test phi squared and all ones Zeckendorf ratio: {:?}",
        end_time.duration_since(start_time)
    );
}

#[derive(Debug)]
struct AllOnesZeckendorfBitsToBinaryBitsRatio {
    all_ones_zeckendorf_bit_count: usize,
    ratio: f64,
}

impl PartialEq for AllOnesZeckendorfBitsToBinaryBitsRatio {
    fn eq(&self, other: &Self) -> bool {
        self.ratio == other.ratio
    }
}

impl PartialOrd for AllOnesZeckendorfBitsToBinaryBitsRatio {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ratio.partial_cmp(&other.ratio)
    }
}

impl Eq for AllOnesZeckendorfBitsToBinaryBitsRatio {}

impl Ord for AllOnesZeckendorfBitsToBinaryBitsRatio {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.ratio < other.ratio {
            std::cmp::Ordering::Less
        } else if self.ratio > other.ratio {
            std::cmp::Ordering::Greater
        } else {
            // Technically could be wrong if one of the ratios is NaN or +-Infinity, but that "should never happen" in our usage.
            std::cmp::Ordering::Equal
        }
    }
}

/// Compares the compression ratios of all ones Zeckendorf numbers to their representation in binary.
///
/// Example expected compression ratio:
/// - the second all ones Zeckendorf number has 2 bits and is 0b11 and represents the number 4 (1 + 3); in binary the number 4 is represented as 0b100 which is 3 bits, so the compression ratio is 2/3 = 0.6666..
/// - the third all ones Zeckendorf number has 3 bits and is 0b111 and represents the number 12 (1 + 3 + 8); in binary the number 12 is represented as 0b1100 which is 4 bits, so the compression ratio is 3/4 = 0.75
/// - the fourth all ones Zeckendorf number has 4 bits and is 0b1111 and represents the number 33 (1 + 3 + 8 + 21); in binary the number 33 is represented as 0b100001 which is 6 bits, so the compression ratio is 4/6 = 0.6666..
fn test_all_ones_zeckendorf_bits_to_binary_bits_ratios() {
    let start_time = Instant::now();
    let mut all_ratios: Vec<AllOnesZeckendorfBitsToBinaryBitsRatio> = Vec::new();

    // Max of 5,000 should take less than a second; max of 10,000 takes about 3 seconds; max of 50,000 takes about a minute.
    for i in 0..5_000 {
        let all_ones_zeckendorf_bit_count = i;
        let all_ones_zeckendorf = all_ones_zeckendorf_to_biguint(all_ones_zeckendorf_bit_count);
        // println!("The {i}th all ones Zeckendorf number is: {} bits, which represents the number {} which is {} bits in binary", all_ones_zeckendorf_bit_count, all_ones_zeckendorf, all_ones_zeckendorf.bits());
        let compression_ratio =
            all_ones_zeckendorf_bit_count as f64 / all_ones_zeckendorf.bits() as f64;
        // println!("The zeckendorf bits to binary bits ratio for the {i}th all ones Zeckendorf number is: {compression_ratio}");
        all_ratios.push(AllOnesZeckendorfBitsToBinaryBitsRatio {
            all_ones_zeckendorf_bit_count: i,
            ratio: compression_ratio,
        });
    }
    all_ratios.sort();
    // Print the x smallest ratios
    for (index, ratio) in all_ratios.iter().take(20).enumerate() {
        let all_ones_zeckendorf_value =
            all_ones_zeckendorf_to_biguint(ratio.all_ones_zeckendorf_bit_count);
        let binary_value_bit_count = all_ones_zeckendorf_value.bits();
        println!(
            "The {index}th best all ones Zeckendorf to binary bits ratio is {ratio:.10} with AOZN bit count: {all_ones_zeckendorf_bit_count}, compared to binary value: {all_ones_zeckendorf_value} which needs {binary_value_bit_count} bits",
            index = index + 1,
            ratio = ratio.ratio,
            all_ones_zeckendorf_bit_count = ratio.all_ones_zeckendorf_bit_count
        );
    }
    let end_time = Instant::now();
    println!(
        "Time taken to test all ones Zeckendorf bits to binary bits ratios: {:?}",
        end_time.duration_since(start_time)
    );
}

/// The purpose of this is to see if decompressing large random data can produce smaller data than the original data.
/// After testing, it seems that the decompressed data tends to be larger than the original data by around 3-4% on average.
/// This matches earlier data showing the likelihood of compressing data at large sizes (greater than ~1,000 bits) to be extremely unlikely.
/// We are able to find cases pretty easily when the size is less than around 100 bytes.
fn test_decompressing_large_random_data() {
    let start_time = Instant::now();
    let num_bytes = 1_000;
    // 10,000 tests takes about 2 seconds.
    let num_tests = 10_000;
    println!(
        "Searching for a case where the decompressed data is smaller than the original data of size {num_bytes} bytes..."
    );
    for i in 0..num_tests {
        let mut data = vec![0u8; num_bytes];
        let mut rng = rand::rng();
        rng.fill_bytes(&mut data);
        // println!("First 10 bytes of random data: {:X?}", &data[..10]);
        // println!("Data size: {:?}", data.len());
        let decompressed_data = zeckendorf_decompress_be(&data);
        // println!("Decompressed data size: {:?}", decompressed_data.len());
        let decompressed_data_raw_bit_size = decompressed_data.len() * 8;
        // println!("Decompressed data raw bit size: {:?}", decompressed_data_raw_bit_size);
        let original_data_raw_bit_size = data.len() * 8;
        // println!("Original data raw bit size: {:?}", original_data_raw_bit_size);
        let decompressed_data_raw_bit_size_ratio =
            decompressed_data_raw_bit_size as f64 / original_data_raw_bit_size as f64;
        // println!("Decompressed data raw bit size ratio to original data raw bit size: {:?}", decompressed_data_raw_bit_size_ratio);
        // println!("Decompressed data raw bit size ratio to original data raw bit size percentage: {:?}%", decompressed_data_raw_bit_size_ratio * 100.0);
        if decompressed_data_raw_bit_size < original_data_raw_bit_size {
            println!(
                "Found a case where the decompressed data is smaller than the original data, on test {i} of {num_tests}!"
            );
            println!("Original data size: {:?} bytes", data.len());
            println!(
                "Decompressed data size: {:?} bytes",
                decompressed_data.len()
            );
            println!(
                "Original data raw bit size: {:?} bits",
                original_data_raw_bit_size
            );
            println!(
                "Decompressed data raw bit size: {:?} bits",
                decompressed_data_raw_bit_size
            );
            println!(
                "Decompressed data raw bit size ratio to original data raw bit size: {:?}",
                decompressed_data_raw_bit_size_ratio
            );
            println!(
                "Decompressed data raw bit size ratio to original data raw bit size percentage: {:?}%",
                decompressed_data_raw_bit_size_ratio * 100.0
            );
            println!(
                "The difference is {:?} bits",
                original_data_raw_bit_size - decompressed_data_raw_bit_size
            );
            println!(
                "The difference percentage is {:?}%",
                (original_data_raw_bit_size - decompressed_data_raw_bit_size) as f64
                    / original_data_raw_bit_size as f64
                    * 100.0
            );
            println!("Original data: {:X?}", data);
            println!("Decompressed data: {:X?}", decompressed_data);
            println!(
                "Found after {i} tests, and time: {:?}",
                Instant::now().duration_since(start_time)
            );
            return;
        }
    }
    println!(
        "Ran {num_tests} tests, and in none of them did the decompressed data end up being smaller than the original data"
    );
    let end_time = Instant::now();
    println!(
        "Time taken to test decompressing {num_tests} pieces of large random data of size {num_bytes} bytes: {:?}",
        end_time.duration_since(start_time)
    );
}

fn print_aozns_as_binary_bits() {
    let max_bits = 20;
    println!("Printing the first {max_bits} all ones Zeckendorf numbers as binary bits:");
    for i in 0..max_bits {
        let all_ones_zeckendorf = all_ones_zeckendorf_to_biguint(i);
        let binary_bit_count = all_ones_zeckendorf.bits();
        let binary_bits = all_ones_zeckendorf.to_bytes_be();
        let binary_bits_str = binary_bits
            .iter()
            .map(|byte| format!("{:08b}", byte))
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "The all ones Zeckendorf number with {i} bits represents the number {all_ones_zeckendorf} which is {binary_bit_count} bits in binary: {binary_bits_str}"
        );
    }
    println!("Finished printing the first {max_bits} all ones Zeckendorf numbers as binary bits");
}
