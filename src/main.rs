
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;
use num_bigint::BigUint;
use num_traits::{One, Zero};

// Memoization maps for Fibonacci numbers
static FIBONACCI_MAP: LazyLock<Mutex<HashMap<u64, u64>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static FIBONACCI_BIGINT_MAP: LazyLock<Mutex<HashMap<u64, BigUint>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Memoization maps for Zeckendorf representations
static ZECKENDORF_MAP: LazyLock<Mutex<HashMap<u64, Vec<u64>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
/// We will store the Zeckendorf list descending as u64s because the Fibonacci indices are small enough to fit in a u64.
/// It takes up to 694,241 bits, or 694kbits, to represent the 1,000,000th Fibonacci number.
/// The max u64 is 18,446,744,073,709,551,615 which is ~18 quintillion.
/// So a u64 can represent Fibonacci indices 18 trillion times larger than the 1,000,000th,
/// so a u64 can represent Fibonacci values up to
/// roughly 18 trillion times 694,241 bits which is 1.249*10^19 bits which or 1.56 exabytes.
/// We will consider larger numbers in the future :-)
static ZECKENDORF_BIGINT_MAP: LazyLock<Mutex<HashMap<BigUint, Vec<u64>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Returns the number of bits required to represent the given number. Returns 0 if the number is less than or equal to 0.
/// Examples: 
///   0 -> 0 bits
///   1 -> 0b1 -> 1 bit
///   2 -> 0b10 -> 2 bits
///   3 -> 0b11 -> 2 bits
///   4 -> 0b100 -> 3 bits
fn bit_count_for_number(n: i32) -> u32 {
    if n <= 0 {
        return 0
    }
    32 - n.leading_zeros()
}

/// fibonacci(x) is equal to 0 if x is 0; 1 if x is 1; else return fibonacci(x - 1) + fibonacci(x - 2)
/// fi stands for Fibonacci Index
/// This function fails for large numbers (e.g. 100_000) with stack overflow.
fn memoized_fibonacci_recursive(n: u64) -> u64 {
    let fibonacci_map = FIBONACCI_MAP.lock().expect("Failed to lock Fibonacci map");
    
    let maybe_cached = fibonacci_map.get(&n);
    if let Some(&cached) = maybe_cached {
        return cached;
    }
    // Drop the lock to allow other threads to access the map during the recursive calls
    drop(fibonacci_map);
    
    let result = if n == 0 {
        0
    } else if n == 1 {
        1
    } else {
        memoized_fibonacci_recursive(n - 1) + memoized_fibonacci_recursive(n - 2)
    };

    // Re-lock the map to insert the result
    let mut fibonacci_map = FIBONACCI_MAP.lock().expect("Failed to lock Fibonacci map");
    fibonacci_map.insert(n, result);
    result
}

/// fibonacci(x) is equal to 0 if x is 0; 1 if x is 1; else return fibonacci(x - 1) + fibonacci(x - 2)
/// fi stands for Fibonacci Index
/// This function fails for large numbers (e.g. 100_000) with stack overflow.
fn memoized_fibonacci_bigint_recursive(n: u64) -> BigUint {
    let fibonacci_bigint_map = FIBONACCI_BIGINT_MAP.lock().expect("Failed to lock Fibonacci BigInt map");
    
    let maybe_cached = fibonacci_bigint_map.get(&n);
    if let Some(cached) = maybe_cached {
        return cached.clone();
    }
    // Drop the lock to allow other threads to access the map during the recursive calls
    drop(fibonacci_bigint_map);

    let result = if n == 0 {
        BigUint::zero()
    } else if n == 1 {
        BigUint::one()
    } else {
        memoized_fibonacci_bigint_recursive(n - 1)
        + memoized_fibonacci_bigint_recursive(n - 2)
    };
    
    // Re-lock the map to insert the result
    let mut fibonacci_bigint_map = FIBONACCI_BIGINT_MAP.lock().expect("Failed to lock Fibonacci BigInt map");
    fibonacci_bigint_map.insert(n.clone(), result.clone());
    result
}

fn fibonacci_bigint_iterative(fi: &BigUint) -> BigUint {
    let mut f0 = BigUint::zero();
    let mut f1 = BigUint::one();
    let mut n = fi.clone();
    while n > BigUint::zero() {
        let f2 = f0 + &f1;
        f0 = f1;
        f1 = f2;
        n -= BigUint::one();
    }
    f0
}

/// A descending Zeckendorf list is a sorted list of unique Fibonacci indices, in descending order, that sum to the given number.
/// A Fibonacci index is the index of the Fibonacci number in the Fibonacci sequence.
/// fibonacci(fibonacci_index) = fibonacci_number
/// Examples:
///   fibonacci(0) = 0
///   fibonacci(1) = 1
///   fibonacci(2) = 1
///   fibonacci(3) = 2
///   fibonacci(4) = 3
///   fibonacci(5) = 5
///   fibonacci(6) = 8
///   fibonacci(7) = 13
///   fibonacci(8) = 21
/// Example Zeckendorf list for the number 10: this is 8 + 2 so the fibonacci indices are 6 and 3.
///   zeckendorf_list(0) = Ok([0])
///   zeckendorf_list(1) = Ok([2])
///   zeckendorf_list(2) = Ok([3])
///   zeckendorf_list(3) = Ok([4])
///   zeckendorf_list(4) = Ok([4, 2])
///   zeckendorf_list(5) = Ok([5])
///   zeckendorf_list(6) = Ok([5, 2])
///   zeckendorf_list(7) = Ok([5, 3])
///   zeckendorf_list(8) = Ok([6])
///   zeckendorf_list(9) = Ok([6, 2])
///   zeckendorf_list(10) = Ok([6, 3])
pub fn memoized_zeckendorf_list_descending_for_integer(n: u64) -> Vec<u64> {
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![2];
    }
    if n == 2 {
        return vec![3];
    }

    let zeckendorf_map = ZECKENDORF_MAP.lock().expect("Failed to lock Zeckendorf map");
    let maybe_memoized_zeckendorf_list = zeckendorf_map.get(&n);
    if let Some(cached) = maybe_memoized_zeckendorf_list {
        return cached.clone();
    }
    drop(zeckendorf_map);

    let mut current_n = n;
    let mut max_fibonacci_index_smaller_than_n = 1u64;
    let mut fibonacci_at_index = memoized_fibonacci_recursive(max_fibonacci_index_smaller_than_n);
    
    while fibonacci_at_index < current_n {
        max_fibonacci_index_smaller_than_n += 1;
        fibonacci_at_index = memoized_fibonacci_recursive(max_fibonacci_index_smaller_than_n);
    }
    
    let mut zeckendorf_list: Vec<u64> = Vec::new();
    while current_n > 0 {
        let current_fibonacci_value = memoized_fibonacci_recursive(max_fibonacci_index_smaller_than_n);
        if current_fibonacci_value > current_n {
            max_fibonacci_index_smaller_than_n -= 1;
            continue;
        }
        current_n -= current_fibonacci_value;
        zeckendorf_list.push(max_fibonacci_index_smaller_than_n);
        // We can subtract 2 because the next Fibonacci number that fits is at least 2 indices away due to the Zeckendorf principle.
        max_fibonacci_index_smaller_than_n -= 2;
    }

    let mut zeckendorf_map = ZECKENDORF_MAP.lock().expect("Failed to lock Zeckendorf map");
    zeckendorf_map.insert(n, zeckendorf_list.clone());
    zeckendorf_list
}

pub fn memoized_zeckendorf_list_descending_for_bigint(n: &BigUint) -> Vec<u64> {
    if n == &BigUint::zero() {
        return vec![];
    }
    if n == &BigUint::one() {
        return vec![2];
    }
    if n == &BigUint::from(2u64) {
        return vec![3];
    }

    let zeckendorf_bigint_map = ZECKENDORF_BIGINT_MAP.lock().expect("Failed to lock Zeckendorf BigInt map");
    let maybe_memoized_zeckendorf_list = zeckendorf_bigint_map.get(n);
    if let Some(cached) = maybe_memoized_zeckendorf_list {
        return cached.clone();
    }
    drop(zeckendorf_bigint_map);

    let original_n = n.clone();
    let mut current_n = n.clone();
    let mut max_fibonacci_index_smaller_than_n = 1u64;
    let mut fibonacci_at_index = memoized_fibonacci_bigint_recursive(max_fibonacci_index_smaller_than_n);
    
    while fibonacci_at_index < current_n {
        max_fibonacci_index_smaller_than_n += 1;
        fibonacci_at_index = memoized_fibonacci_bigint_recursive(max_fibonacci_index_smaller_than_n);
    }
    
    let mut zeckendorf_list: Vec<u64> = Vec::new();
    while current_n > BigUint::zero() {
        let current_fibonacci_value = memoized_fibonacci_bigint_recursive(max_fibonacci_index_smaller_than_n);
        if current_fibonacci_value > current_n {
            max_fibonacci_index_smaller_than_n -= 1;
            continue;
        }
        current_n -= current_fibonacci_value;
        zeckendorf_list.push(max_fibonacci_index_smaller_than_n);
        // We can subtract 2 because the next Fibonacci number that fits is at least 2 indices away due to the Zeckendorf principle.
        max_fibonacci_index_smaller_than_n -= 2;
    }

    let mut zeckendorf_bigint_map = ZECKENDORF_BIGINT_MAP.lock().expect("Failed to lock Zeckendorf BigInt map");
    zeckendorf_bigint_map.insert(original_n, zeckendorf_list.clone());
    zeckendorf_list
}

pub const USE_BIT: u8 = 1;
pub const SKIP_BIT: u8 = 0;

/// Effective Fibonacci Index to Fibonacci Index: FI(efi) === efi + 2, where efi is the Effective Fibonacci Index
fn efi_to_fi(efi: u64) -> u64 {
    return efi + 2
}

/// Effective Fibonacci Index to Fibonacci Index: FI(efi) === efi + 2, where efi is the Effective Fibonacci Index
fn efi_to_fi_bigint(efi: BigUint) -> BigUint {
    return efi + BigUint::from(2u64)
}

/// Fibonacci Index to Effective Fibonacci Index: EFI(fi) === fi - 2, where fi is the Fibonacci Index
fn fi_to_efi(fi: u64) -> u64 {
    return fi - 2
}

/// Fibonacci Index to Effective Fibonacci Index: EFI(fi) === fi - 2, where fi is the Fibonacci Index
fn fi_to_efi_bigint(fi: BigUint) -> BigUint {
    return fi - BigUint::from(2u64)
}

/// Effective Fibonacci function: EF(efi) === fibonacci(efi + 2) === fibonacci(fi), where efi is the Effective Fibonacci Index
fn memoized_effective_fibonacci(efi: u64) -> u64 {
    return memoized_fibonacci_recursive(efi_to_fi(efi))
}

/// An Effective Zeckendorf List (EZL) has a lowest EFI of 0, which is an FI of 2.
/// This is because it doesn't make sense for the lists to contain FIs 0 or 1 because
/// 0 can never be added to a number and will therefore never be in a Zeckendorf List
/// and an FI of 1 is equivalent to an FI of 2 which has a Fibonacci value of 1
/// so let's just use FI starting at 2, which is an EFI of 0.
/// It does not matter if the list is ascending or descending; it retains the directionality of the original list.
fn zl_to_ezl(zl: Vec<u64>) -> Vec<u64> {
    return zl.into_iter().map(fi_to_efi).collect()
}

/// Converts an Effective Zeckendorf List to a Zeckendorf List.
/// It does not matter if the list is ascending or descending; it retains the directionality of the original list.
fn ezl_to_zl(ezl: Vec<u64>) -> Vec<u64> {
    return ezl.into_iter().map(efi_to_fi).collect()
}

/// ezba is Effective Zeckendorf Bits Ascending ; ezld is Effective Zeckendorf List Descending
/// The first bit represents whether the first effective fibonacci index is used. The first
/// effective fibonacci index is always 0 and represents the fibonacci index 2 which has a value of 1.
fn ezba_from_ezld(effective_zeckendorf_list_descending: Vec<u64>) -> Vec<u8> {
    if effective_zeckendorf_list_descending.is_empty() {
        return vec![SKIP_BIT];
    }

    let effective_zeckendorf_list_ascending: Vec<u64> = effective_zeckendorf_list_descending.clone().into_iter().rev().collect();

    let mut effective_zeckendorf_bits_ascending = Vec::new();

    let mut current_ezla_index = 0;

    let mut current_efi = 0;
    // EZLs are guaranteed to be non-empty
    // FIXME: enforce this at the type level
    let max_efi = effective_zeckendorf_list_descending[0];

    while current_efi <= max_efi {
        let current_ezla_value = effective_zeckendorf_list_ascending[current_ezla_index];
        if current_ezla_value == current_efi {
            effective_zeckendorf_bits_ascending.push(USE_BIT);
            current_efi += 2;
            current_ezla_index += 1
        } else {
            effective_zeckendorf_bits_ascending.push(SKIP_BIT);
            current_efi += 1;
        }
    }

    return effective_zeckendorf_bits_ascending
}

/// Packs a vector of bits (0s and 1s) from an ezba (Effective Zeckendorf Bits Ascending) into bytes.
///
/// Bits are in ascending significance: bits[0] = LSB, bits[7] = MSB.
/// Every 8 bits become a u8 in the output.
/// The last byte is padded with 0s if the number of bits is not a multiple of 8.
fn pack_ezba_bits_to_bytes(ezba: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity((ezba.len() + 7) / 8);

    for chunk in ezba.chunks(8) {
        let mut b = 0u8;

        for (i, &bit) in chunk.iter().enumerate() {
            if bit == 1 {
                b |= 1 << i;
            }
        }

        out.push(b);
    }

    out
}

/// Compresses a vector of bytes using the Zeckendorf algorithm.
/// Assume big endian bytes for now.
fn zeckendorf_compress(data: Vec<u8>) -> Vec<u8> {
    let compressed_data: Vec<u8>;
    // Turn data into a bigint
    let data_as_bigint = BigUint::from_bytes_be(&data);
    println!("Data as bigint: {:?}", data_as_bigint);
    // Get the effective zeckendorf list descending
    let data_as_zld = memoized_zeckendorf_list_descending_for_bigint(&data_as_bigint);
    println!("Data as zld: {:?}", data_as_zld);
    let data_as_ezld = zl_to_ezl(data_as_zld);
    println!("Data as ezld: {:?}", data_as_ezld);
    // Get the effective zeckendorf bits ascending
    let data_as_ezba = ezba_from_ezld(data_as_ezld);
    println!("Data as ezba: {:?}", data_as_ezba);
    // Compress the data
    compressed_data = pack_ezba_bits_to_bytes(data_as_ezba);
    println!("Compressed data: {:?}", compressed_data);
    return compressed_data
}

fn unpack_bytes_to_ezba_bits(bytes: Vec<u8>) -> Vec<u8> {
    let mut ezba_bits = Vec::new();
    for byte in bytes {
        for i in 0..8 {
            ezba_bits.push((byte >> i) & 1);
        }
    }
    return ezba_bits
}

/// Converts a vector of bits (0s and 1s) from an ezba (Effective Zeckendorf Bits Ascending) into a vector of effective fibonacci indices,
/// the Effective Zeckendorf List Ascending.
fn ezba_to_ezla(ezba_bits: Vec<u8>) -> Vec<u64> {
    let mut ezla = Vec::new();
    let mut current_efi = 0;
    for bit in ezba_bits {
        if bit == USE_BIT {
            ezla.push(current_efi);
            current_efi += 2;
        } else {
            current_efi += 1;
        }
    }
    return ezla
}

/// Converts a Zeckendorf List to a BigInt.
/// The Zeckendorf List is a list of Fibonacci indices that sum to the given number.
/// It does not matter if the ZL is ascending or descending. The sum operation is commutative.
fn zl_to_bigint(zl: Vec<u64>) -> BigUint {
    return zl.into_iter().map(memoized_fibonacci_bigint_recursive).sum()
}

/// Decompresses a vector of bytes compressed using the Zeckendorf algorithm.
/// Assume big endian bytes for now.
fn zeckendorf_decompress(compressed_data: Vec<u8>) -> Vec<u8> {
    // Unpack the compressed data into bits
    let compressed_data_as_bits = unpack_bytes_to_ezba_bits(compressed_data);
    println!("Compressed data as bits: {:?}", compressed_data_as_bits);
    // Unpack the bits into an ezla (Effective Zeckendorf List Ascending)
    let compressed_data_as_ezla = ezba_to_ezla(compressed_data_as_bits);
    println!("Compressed data as ezla: {:?}", compressed_data_as_ezla);
    // Convert the ezla to a zla (Zeckendorf List Ascending)
    let compressed_data_as_zla = ezl_to_zl(compressed_data_as_ezla);
    println!("Compressed data as zla: {:?}", compressed_data_as_zla);
    // Convert the zla to a bigint
    let compressed_data_as_bigint = zl_to_bigint(compressed_data_as_zla);
    println!("Compressed data as bigint: {:?}", compressed_data_as_bigint);
    return compressed_data_as_bigint.to_bytes_be()
}

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
        let zl = memoized_zeckendorf_list_descending_for_integer(i);
        println!("Zeckendorf descending list for {}: {:?}", i, zl);
        let ezl = zl_to_ezl(zl);
        println!("Effective Zeckendorf list descending for {}: {:?}", i, ezl);
        let ezba = ezba_from_ezld(ezl);
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


    let data = BigUint::from(12_u64).to_bytes_be();
    println!("Data: {:?}", data);
    let compressed_data = zeckendorf_compress(data);
    // Expect 7 or 0b111 as the compressed data for the number 12.
    println!("Compressed data: {:?}", compressed_data);
    let decompressed_data = zeckendorf_decompress(compressed_data);
    println!("Decompressed data: {:?}", decompressed_data);
    
    // TODO: test larger data

    let end_time = Instant::now();
    println!("Time taken: {:?}", end_time.duration_since(start_time));

}
