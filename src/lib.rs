//! Zeckendorf compression and decompression library
//!
//! This library provides functionality for compressing and decompressing data using the Zeckendorf algorithm.
//!
//! The Zeckendorf algorithm is a way to represent numbers as a sum of non-consecutive Fibonacci numbers.
//! If we first interpret the input data as a big integer, we can then represent the integer as a sum of non-consecutive Fibonacci numbers.
//! Sometimes this results in a more compact representation of the data, but it is not guaranteed.

use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// Returns the number of bits required to represent the given number. Returns 0 if the number is less than or equal to 0.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::bit_count_for_number;
/// assert_eq!(bit_count_for_number(0), 0);
/// assert_eq!(bit_count_for_number(1), 1);  // 0b1
/// assert_eq!(bit_count_for_number(2), 2);  // 0b10
/// assert_eq!(bit_count_for_number(3), 2);  // 0b11
/// assert_eq!(bit_count_for_number(4), 3);  // 0b100
/// ```
pub fn bit_count_for_number(n: i32) -> u32 {
    if n <= 0 {
        return 0;
    }
    32 - n.leading_zeros()
}

// Memoization maps for Fibonacci numbers
static FIBONACCI_MAP: LazyLock<Mutex<HashMap<u64, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static FIBONACCI_BIGINT_MAP: LazyLock<Mutex<HashMap<u64, BigUint>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Memoization maps for Zeckendorf representations
static ZECKENDORF_MAP: LazyLock<Mutex<HashMap<u64, Vec<u64>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
/// We will store the Zeckendorf list descending as u64s because the Fibonacci indices are small enough to fit in a u64.
/// It takes up to 694,241 bits, or ~694kbits, to represent the 1,000,000th Fibonacci number.
/// The max u64 is 18,446,744,073,709,551,615 which is ~18 quintillion.
/// So a u64 can represent Fibonacci indices 18 trillion times larger than the 1,000,000th,
/// so a u64 can represent Fibonacci values up to
/// roughly 18 trillion times 694,241 bits which is 1.249*10^19 bits which or 1.56 exabytes.
/// We will consider larger numbers in the future :-)
static ZECKENDORF_BIGINT_MAP: LazyLock<Mutex<HashMap<BigUint, Vec<u64>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// fibonacci(x) is equal to 0 if x is 0; 1 if x is 1; else return fibonacci(x - 1) + fibonacci(x - 2)
/// fi stands for Fibonacci Index
/// This function fails for large numbers (e.g. 100_000) with stack overflow.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::memoized_fibonacci_recursive;
/// // Base cases
/// assert_eq!(memoized_fibonacci_recursive(0), 0);
/// assert_eq!(memoized_fibonacci_recursive(1), 1);
///
/// // Small Fibonacci numbers
/// assert_eq!(memoized_fibonacci_recursive(2), 1);
/// assert_eq!(memoized_fibonacci_recursive(3), 2);
/// assert_eq!(memoized_fibonacci_recursive(4), 3);
/// assert_eq!(memoized_fibonacci_recursive(5), 5);
/// assert_eq!(memoized_fibonacci_recursive(6), 8);
/// assert_eq!(memoized_fibonacci_recursive(7), 13);
/// assert_eq!(memoized_fibonacci_recursive(8), 21);
/// assert_eq!(memoized_fibonacci_recursive(9), 34);
/// assert_eq!(memoized_fibonacci_recursive(10), 55);
/// ```
pub fn memoized_fibonacci_recursive(fi: u64) -> u64 {
    let fibonacci_map = FIBONACCI_MAP.lock().expect("Failed to lock Fibonacci map");

    let maybe_cached = fibonacci_map.get(&fi);
    if let Some(&cached) = maybe_cached {
        return cached;
    }
    // Drop the lock to allow other threads to access the map during the recursive calls
    drop(fibonacci_map);

    let result = if fi == 0 {
        0
    } else if fi == 1 {
        1
    } else {
        memoized_fibonacci_recursive(fi - 1) + memoized_fibonacci_recursive(fi - 2)
    };

    // Re-lock the map to insert the result
    let mut fibonacci_map = FIBONACCI_MAP.lock().expect("Failed to lock Fibonacci map");
    fibonacci_map.insert(fi, result);
    result
}

/// fibonacci(x) is equal to 0 if x is 0; 1 if x is 1; else return fibonacci(x - 1) + fibonacci(x - 2)
/// fi stands for Fibonacci Index
/// This function fails for large numbers (e.g. 100_000) with stack overflow.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::memoized_fibonacci_bigint_recursive;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// // Base cases
/// assert_eq!(memoized_fibonacci_bigint_recursive(0), BigUint::zero());
/// assert_eq!(memoized_fibonacci_bigint_recursive(1), BigUint::one());
///
/// // Small Fibonacci numbers
/// assert_eq!(memoized_fibonacci_bigint_recursive(2), BigUint::from(1u64));
/// assert_eq!(memoized_fibonacci_bigint_recursive(3), BigUint::from(2u64));
/// assert_eq!(memoized_fibonacci_bigint_recursive(4), BigUint::from(3u64));
/// assert_eq!(memoized_fibonacci_bigint_recursive(5), BigUint::from(5u64));
/// assert_eq!(memoized_fibonacci_bigint_recursive(6), BigUint::from(8u64));
/// assert_eq!(memoized_fibonacci_bigint_recursive(7), BigUint::from(13u64));
/// assert_eq!(memoized_fibonacci_bigint_recursive(8), BigUint::from(21u64));
/// assert_eq!(memoized_fibonacci_bigint_recursive(9), BigUint::from(34u64));
/// assert_eq!(memoized_fibonacci_bigint_recursive(10), BigUint::from(55u64));
/// ```
pub fn memoized_fibonacci_bigint_recursive(fi: u64) -> BigUint {
    let fibonacci_bigint_map = FIBONACCI_BIGINT_MAP
        .lock()
        .expect("Failed to lock Fibonacci BigInt map");

    let maybe_cached = fibonacci_bigint_map.get(&fi);
    if let Some(cached) = maybe_cached {
        return cached.clone();
    }
    // Drop the lock to allow other threads to access the map during the recursive calls
    drop(fibonacci_bigint_map);

    let result = if fi == 0 {
        BigUint::zero()
    } else if fi == 1 {
        BigUint::one()
    } else {
        memoized_fibonacci_bigint_recursive(fi - 1) + memoized_fibonacci_bigint_recursive(fi - 2)
    };

    // Re-lock the map to insert the result
    let mut fibonacci_bigint_map = FIBONACCI_BIGINT_MAP
        .lock()
        .expect("Failed to lock Fibonacci BigInt map");
    fibonacci_bigint_map.insert(fi.clone(), result.clone());
    result
}

/// fibonacci(x) is equal to 0 if x is 0; 1 if x is 1; else return fibonacci(x - 1) + fibonacci(x - 2)
/// fi stands for Fibonacci Index
/// This function fails for large numbers (e.g. 100_000) with stack overflow.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::memoized_fibonacci_bigint_iterative;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// // Base cases
/// assert_eq!(memoized_fibonacci_bigint_iterative(0u64), BigUint::zero());
/// assert_eq!(memoized_fibonacci_bigint_iterative(1u64), BigUint::one());
///
/// // Small Fibonacci numbers
/// assert_eq!(memoized_fibonacci_bigint_iterative(2u64), BigUint::from(1u64));
/// assert_eq!(memoized_fibonacci_bigint_iterative(3u64), BigUint::from(2u64));
/// assert_eq!(memoized_fibonacci_bigint_iterative(4u64), BigUint::from(3u64));
/// assert_eq!(memoized_fibonacci_bigint_iterative(5u64), BigUint::from(5u64));
/// assert_eq!(memoized_fibonacci_bigint_iterative(6u64), BigUint::from(8u64));
/// assert_eq!(memoized_fibonacci_bigint_iterative(7u64), BigUint::from(13u64));
/// assert_eq!(memoized_fibonacci_bigint_iterative(8u64), BigUint::from(21u64));
/// assert_eq!(memoized_fibonacci_bigint_iterative(9u64), BigUint::from(34u64));
/// assert_eq!(memoized_fibonacci_bigint_iterative(10u64), BigUint::from(55u64));
/// ```
pub fn memoized_fibonacci_bigint_iterative(fi: u64) -> BigUint {
    let fibonacci_bigint_map = FIBONACCI_BIGINT_MAP
        .lock()
        .expect("Failed to lock Fibonacci BigInt map");

    let maybe_cached = fibonacci_bigint_map.get(&fi);
    if let Some(cached) = maybe_cached {
        return cached.clone();
    }
    drop(fibonacci_bigint_map);

    let mut f0 = BigUint::zero();
    let mut f1 = BigUint::one();
    let mut n = BigUint::from(fi);
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
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::memoized_zeckendorf_list_descending_for_integer;
/// // Base cases
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(0), vec![]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(1), vec![2]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(2), vec![3]);
///
/// // Small Zeckendorf numbers
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(3), vec![4]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(4), vec![4, 2]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(5), vec![5]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(6), vec![5, 2]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(7), vec![5, 3]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(8), vec![6]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(9), vec![6, 2]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(10), vec![6, 3]);
/// ```
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

    let zeckendorf_map = ZECKENDORF_MAP
        .lock()
        .expect("Failed to lock Zeckendorf map");
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
        let current_fibonacci_value =
            memoized_fibonacci_recursive(max_fibonacci_index_smaller_than_n);
        if current_fibonacci_value > current_n {
            max_fibonacci_index_smaller_than_n -= 1;
            continue;
        }
        current_n -= current_fibonacci_value;
        zeckendorf_list.push(max_fibonacci_index_smaller_than_n);
        // We can subtract 2 because the next Fibonacci number that fits is at least 2 indices away due to the Zeckendorf principle.
        max_fibonacci_index_smaller_than_n -= 2;
    }

    let mut zeckendorf_map = ZECKENDORF_MAP
        .lock()
        .expect("Failed to lock Zeckendorf map");
    zeckendorf_map.insert(n, zeckendorf_list.clone());
    zeckendorf_list
}

/// A descending Zeckendorf list is a sorted list of unique Fibonacci indices, in descending order, that sum to the given number.
/// A Fibonacci index is the index of the Fibonacci number in the Fibonacci sequence.
/// fibonacci(fibonacci_index) = fibonacci_number
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::memoized_zeckendorf_list_descending_for_bigint;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// // Base cases
/// assert_eq!(memoized_zeckendorf_list_descending_for_bigint(&BigUint::zero()), vec![]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_bigint(&BigUint::one()), vec![2]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_bigint(&BigUint::from(2u64)), vec![3]);
///
/// // Small Zeckendorf numbers
/// assert_eq!(memoized_zeckendorf_list_descending_for_bigint(&BigUint::from(3u64)), vec![4]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_bigint(&BigUint::from(4u64)), vec![4, 2]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_bigint(&BigUint::from(5u64)), vec![5]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_bigint(&BigUint::from(6u64)), vec![5, 2]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_bigint(&BigUint::from(7u64)), vec![5, 3]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_bigint(&BigUint::from(8u64)), vec![6]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_bigint(&BigUint::from(9u64)), vec![6, 2]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_bigint(&BigUint::from(10u64)), vec![6, 3]);
/// ```
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

    let zeckendorf_bigint_map = ZECKENDORF_BIGINT_MAP
        .lock()
        .expect("Failed to lock Zeckendorf BigInt map");
    let maybe_memoized_zeckendorf_list = zeckendorf_bigint_map.get(n);
    if let Some(cached) = maybe_memoized_zeckendorf_list {
        return cached.clone();
    }
    drop(zeckendorf_bigint_map);

    let original_n = n.clone();
    let mut current_n = n.clone();
    let mut max_fibonacci_index_smaller_than_n = 1u64;
    let mut fibonacci_at_index =
        memoized_fibonacci_bigint_recursive(max_fibonacci_index_smaller_than_n);

    while fibonacci_at_index < current_n {
        max_fibonacci_index_smaller_than_n += 1;
        fibonacci_at_index =
            memoized_fibonacci_bigint_recursive(max_fibonacci_index_smaller_than_n);
    }

    let mut zeckendorf_list: Vec<u64> = Vec::new();
    while current_n > BigUint::zero() {
        let current_fibonacci_value =
            memoized_fibonacci_bigint_recursive(max_fibonacci_index_smaller_than_n);
        if current_fibonacci_value > current_n {
            max_fibonacci_index_smaller_than_n -= 1;
            continue;
        }
        current_n -= current_fibonacci_value;
        zeckendorf_list.push(max_fibonacci_index_smaller_than_n);
        // We can subtract 2 because the next Fibonacci number that fits is at least 2 indices away due to the Zeckendorf principle.
        max_fibonacci_index_smaller_than_n -= 2;
    }

    let mut zeckendorf_bigint_map = ZECKENDORF_BIGINT_MAP
        .lock()
        .expect("Failed to lock Zeckendorf BigInt map");
    zeckendorf_bigint_map.insert(original_n, zeckendorf_list.clone());
    zeckendorf_list
}

pub const USE_BIT: u8 = 1;
pub const SKIP_BIT: u8 = 0;

/// Effective Fibonacci Index to Fibonacci Index: FI(efi) === efi + 2, where efi is the Effective Fibonacci Index
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::efi_to_fi;
/// assert_eq!(efi_to_fi(0), 2);
/// assert_eq!(efi_to_fi(1), 3);
/// assert_eq!(efi_to_fi(2), 4);
/// ```
pub fn efi_to_fi(efi: u64) -> u64 {
    return efi + 2;
}

/// Effective Fibonacci Index to Fibonacci Index: FI(efi) === efi + 2, where efi is the Effective Fibonacci Index
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::efi_to_fi_ref;
/// assert_eq!(efi_to_fi_ref(&0), 2);
/// assert_eq!(efi_to_fi_ref(&1), 3);
/// assert_eq!(efi_to_fi_ref(&2), 4);
/// ```
pub fn efi_to_fi_ref(efi: &u64) -> u64 {
    return *efi + 2;
}

/// Effective Fibonacci Index to Fibonacci Index: FI(efi) === efi + 2, where efi is the Effective Fibonacci Index
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::efi_to_fi_bigint;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// assert_eq!(efi_to_fi_bigint(BigUint::zero()), BigUint::from(2u64));
/// assert_eq!(efi_to_fi_bigint(BigUint::one()), BigUint::from(3u64));
/// assert_eq!(efi_to_fi_bigint(BigUint::from(2u64)), BigUint::from(4u64));
/// ```
pub fn efi_to_fi_bigint(efi: BigUint) -> BigUint {
    return efi + BigUint::from(2u64);
}

/// Fibonacci Index to Effective Fibonacci Index: EFI(fi) === fi - 2, where fi is the Fibonacci Index
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::fi_to_efi;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// assert_eq!(fi_to_efi(2), 0);
/// assert_eq!(fi_to_efi(3), 1);
/// assert_eq!(fi_to_efi(4), 2);
/// ```
pub fn fi_to_efi(fi: u64) -> u64 {
    return fi - 2;
}

/// Fibonacci Index to Effective Fibonacci Index: EFI(fi) === fi - 2, where fi is the Fibonacci Index
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::fi_to_efi_ref;
/// assert_eq!(fi_to_efi_ref(&2), 0);
/// assert_eq!(fi_to_efi_ref(&3), 1);
/// assert_eq!(fi_to_efi_ref(&4), 2);
/// ```
pub fn fi_to_efi_ref(fi: &u64) -> u64 {
    return *fi - 2;
}

/// Fibonacci Index to Effective Fibonacci Index: EFI(fi) === fi - 2, where fi is the Fibonacci Index
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::fi_to_efi_bigint;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// assert_eq!(fi_to_efi_bigint(BigUint::from(2u64)), BigUint::zero());
/// assert_eq!(fi_to_efi_bigint(BigUint::from(3u64)), BigUint::one());
/// assert_eq!(fi_to_efi_bigint(BigUint::from(4u64)), BigUint::from(2u64));
/// ```
pub fn fi_to_efi_bigint(fi: BigUint) -> BigUint {
    return fi - BigUint::from(2u64);
}

/// The memoized Fibonacci function taking an Effective Fibonacci Index as input.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::memoized_effective_fibonacci;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// assert_eq!(memoized_effective_fibonacci(0), 1);
/// assert_eq!(memoized_effective_fibonacci(1), 2);
/// assert_eq!(memoized_effective_fibonacci(2), 3);
/// assert_eq!(memoized_effective_fibonacci(3), 5);
/// assert_eq!(memoized_effective_fibonacci(4), 8);
/// assert_eq!(memoized_effective_fibonacci(5), 13);
/// assert_eq!(memoized_effective_fibonacci(6), 21);
/// assert_eq!(memoized_effective_fibonacci(7), 34);
/// assert_eq!(memoized_effective_fibonacci(8), 55);
/// assert_eq!(memoized_effective_fibonacci(9), 89);
/// assert_eq!(memoized_effective_fibonacci(10), 144);
/// ```
pub fn memoized_effective_fibonacci(efi: u64) -> u64 {
    return memoized_fibonacci_recursive(efi_to_fi(efi));
}

/// An Effective Zeckendorf List (EZL) has a lowest EFI of 0, which is an FI of 2.
/// This is because it doesn't make sense for the lists to contain FIs 0 or 1 because
/// 0 can never be added to a number and will therefore never be in a Zeckendorf List
/// and an FI of 1 is equivalent to an FI of 2 which has a Fibonacci value of 1
/// so let's just use FI starting at 2, which is an EFI of 0.
/// It does not matter if the list is ascending or descending; it retains the directionality of the original list.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::zl_to_ezl;
/// assert_eq!(zl_to_ezl(&[2]), vec![0]);
/// assert_eq!(zl_to_ezl(&[3]), vec![1]);
/// assert_eq!(zl_to_ezl(&[4]), vec![2]);
/// ```
pub fn zl_to_ezl(zl: &[u64]) -> Vec<u64> {
    return zl.into_iter().map(fi_to_efi_ref).collect();
}

/// Converts an Effective Zeckendorf List to a Zeckendorf List.
/// It does not matter if the list is ascending or descending; it retains the directionality of the original list.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::ezl_to_zl;
/// assert_eq!(ezl_to_zl(&[0]), vec![2]);
/// assert_eq!(ezl_to_zl(&[1]), vec![3]);
/// assert_eq!(ezl_to_zl(&[2]), vec![4]);
/// ```
pub fn ezl_to_zl(ezl: &[u64]) -> Vec<u64> {
    return ezl.into_iter().map(efi_to_fi_ref).collect();
}

/// ezba is Effective Zeckendorf Bits Ascending ; ezld is Effective Zeckendorf List Descending
///
/// The bits represent whether the corresponding effective Fibonacci index is used. I call these "use bits" and "skip bits" where a use bit is 1 and a skip bit is 0. This is by convention that I, Peter Ryszkiewicz decided, but it is theoretically possible to use skip bits and use bits flipped.
///
/// If we use a bit, we then skip the next bit, because it is impossible to use two consecutive bits, or Fibonacci numbers, due to the Zeckendorf principle.
/// The first bit in the ezba represents whether the first effective Fibonacci index is used.
/// The first effective fibonacci index is always 0 and represents the fibonacci index 2 which has a value of 1. We use effective Fibonacci indices because the first Fibonacci number, 0, is not useful for sums, and the second Fibonacci number, 1, is redundant because it is the same as the third Fibonacci number.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::ezba_from_ezld;
/// assert_eq!(ezba_from_ezld(&[]), vec![0]);
/// assert_eq!(ezba_from_ezld(&[0]), vec![1]); // 0th EFI is 2nd FI, which is 1
/// assert_eq!(ezba_from_ezld(&[1]), vec![0, 1]); // 1st EFI is 3rd FI, which is 2
/// assert_eq!(ezba_from_ezld(&[2]), vec![0, 0, 1]); // 2nd EFI is 4th FI, which is 3
/// assert_eq!(ezba_from_ezld(&[2, 0]), vec![1, 1]); // 2nd EFI is 4th FI, which is 3 and 0th EFI is 2nd FI, which is 1, which sums to 4
/// assert_eq!(ezba_from_ezld(&[3]), vec![0, 0, 0, 1]); // 3rd EFI is 5th FI, which is 5
/// ```
pub fn ezba_from_ezld(effective_zeckendorf_list_descending: &[u64]) -> Vec<u8> {
    if effective_zeckendorf_list_descending.is_empty() {
        return vec![SKIP_BIT];
    }

    let effective_zeckendorf_list_ascending: Vec<u64> = effective_zeckendorf_list_descending
        .to_vec()
        .into_iter()
        .rev()
        .collect();

    let mut effective_zeckendorf_bits_ascending = Vec::new();

    let mut current_ezla_index = 0;

    let mut current_efi = 0;
    // This EZLD is guaranteed to be non-empty because of the guard at the beginning of the function.
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

    return effective_zeckendorf_bits_ascending;
}

/// Packs a slice of bits (0s and 1s) from an ezba (Effective Zeckendorf Bits Ascending) into bytes.
/// 
/// The output bytes are in little endian order, so the first byte is the least significant byte and the last byte is the most significant byte.
///
/// Input bits and output bits are in ascending significance: bits\[0\] = LSB, bits\[7\] = MSB.
/// Every 8 bits become a u8 in the output.
/// The last byte is padded with 0s if the number of bits is not a multiple of 8.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::pack_ezba_bits_to_bytes;
/// assert_eq!(pack_ezba_bits_to_bytes(&[0]), vec![0]);
/// assert_eq!(pack_ezba_bits_to_bytes(&[1]), vec![1]);
/// assert_eq!(pack_ezba_bits_to_bytes(&[0, 1]), vec![0b10]);
/// assert_eq!(pack_ezba_bits_to_bytes(&[0, 0, 1]), vec![0b100]);
/// assert_eq!(pack_ezba_bits_to_bytes(&[1, 1]), vec![0b11]);
/// ```
pub fn pack_ezba_bits_to_bytes(ezba: &[u8]) -> Vec<u8> {
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

/// Compresses a slice of bytes using the Zeckendorf algorithm.
/// 
/// Assumes the input data is interpreted as a big endian integer. The output data is in little endian order, so the first bit and byte is the least significant bit and byte and the last bit and byte is the most significant bit and byte.
/// 
/// TODO: Technically, the way the input data is interpreted is arbitrary; we could interpret it as little endian which could result in a more compact representation. We could go even further and interpret the data at different byte or word boundaries to see if it results in a more compact representation, and signify to the caller which interpretation was used. We probably need a better understanding of random distributions of data to determine what is the optimal interpretation. More investigation is needed here.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::zeckendorf_compress_be;
/// assert_eq!(zeckendorf_compress_be(&[0]), vec![0]);
/// assert_eq!(zeckendorf_compress_be(&[1]), vec![1]);
/// assert_eq!(zeckendorf_compress_be(&[12]), vec![0b111]);
/// assert_eq!(zeckendorf_compress_be(&[54]), vec![30]);
/// assert_eq!(zeckendorf_compress_be(&[55]), vec![0, 1]); // 55 is the 10 indexed Fibonacci number, which is the 8 indexed effective Fibonacci number, and therefore is the first number needing two bytes to contain these 8 bits, because there is 1 "use bit" and 7 "skip bits" in the effective zeckendorf bits ascending.
/// assert_eq!(zeckendorf_compress_be(&[255]), vec![33, 2]);
/// assert_eq!(zeckendorf_compress_be(&[1, 0]), vec![34, 2]);
/// ```
pub fn zeckendorf_compress_be(data: &[u8]) -> Vec<u8> {
    let compressed_data: Vec<u8>;
    // Turn data into a bigint
    let data_as_bigint = BigUint::from_bytes_be(data);
    // println!("Data as bigint: {:?}", data_as_bigint);
    // Get the effective zeckendorf list descending
    let data_as_zld = memoized_zeckendorf_list_descending_for_bigint(&data_as_bigint);
    // println!("Data as zld: {:?}", data_as_zld);
    let data_as_ezld = zl_to_ezl(&data_as_zld);
    // println!("Data as ezld: {:?}", data_as_ezld);
    // Get the effective zeckendorf bits ascending
    let data_as_ezba = ezba_from_ezld(&data_as_ezld);
    // println!("Data as ezba: {:?}", data_as_ezba);
    // Compress the data
    compressed_data = pack_ezba_bits_to_bytes(&data_as_ezba);
    // println!("Compressed data: {:?}", compressed_data);
    return compressed_data;
}

/// Unpacks a vector of bytes into a vector of bits (0s and 1s) from an ezba (Effective Zeckendorf Bits Ascending).
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::unpack_bytes_to_ezba_bits;
/// assert_eq!(unpack_bytes_to_ezba_bits(&[0]), vec![0, 0, 0, 0, 0, 0, 0, 0]);
/// assert_eq!(unpack_bytes_to_ezba_bits(&[1]), vec![1, 0, 0, 0, 0, 0, 0, 0]);
/// assert_eq!(unpack_bytes_to_ezba_bits(&[0b111]), vec![1, 1, 1, 0, 0, 0, 0, 0]);
/// assert_eq!(unpack_bytes_to_ezba_bits(&[1, 1]), vec![1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
/// ```
pub fn unpack_bytes_to_ezba_bits(bytes: &[u8]) -> Vec<u8> {
    let mut ezba_bits = Vec::new();
    for byte in bytes {
        for i in 0..8 {
            ezba_bits.push((byte >> i) & 1);
        }
    }
    return ezba_bits;
}

/// Converts a vector of bits (0s and 1s) from an ezba (Effective Zeckendorf Bits Ascending) into a vector of effective fibonacci indices,
/// the Effective Zeckendorf List Ascending.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::ezba_to_ezla;
/// assert_eq!(ezba_to_ezla(&[0, 0, 0, 0, 0, 0, 0, 0]), vec![]);
/// assert_eq!(ezba_to_ezla(&[1, 0, 0, 0, 0, 0, 0, 0]), vec![0]);
/// assert_eq!(ezba_to_ezla(&[1, 1, 1, 0, 0, 0, 0, 0]), vec![0, 2, 4]);
/// ```
pub fn ezba_to_ezla(ezba_bits: &[u8]) -> Vec<u64> {
    let mut ezla = Vec::new();
    let mut current_efi = 0;
    for bit in ezba_bits {
        if *bit == USE_BIT {
            ezla.push(current_efi);
            current_efi += 2;
        } else {
            current_efi += 1;
        }
    }
    return ezla;
}

/// Converts a Zeckendorf List to a BigInt.
/// The Zeckendorf List is a list of Fibonacci indices that sum to the given number.
/// It does not matter if the ZL is ascending or descending. The sum operation is commutative.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::zl_to_bigint;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// assert_eq!(zl_to_bigint(&[]), BigUint::zero());
/// assert_eq!(zl_to_bigint(&[0]), BigUint::zero());
/// assert_eq!(zl_to_bigint(&[1]), BigUint::one());
/// assert_eq!(zl_to_bigint(&[2]), BigUint::one());
/// assert_eq!(zl_to_bigint(&[3]), BigUint::from(2u64));
/// assert_eq!(zl_to_bigint(&[4]), BigUint::from(3u64));
/// assert_eq!(zl_to_bigint(&[5]), BigUint::from(5u64));
/// assert_eq!(zl_to_bigint(&[6]), BigUint::from(8u64));
/// assert_eq!(zl_to_bigint(&[6, 2]), BigUint::from(9u64));
/// assert_eq!(zl_to_bigint(&[6, 3]), BigUint::from(10u64));
/// assert_eq!(zl_to_bigint(&[6, 4]), BigUint::from(11u64));
/// assert_eq!(zl_to_bigint(&[6, 4, 2]), BigUint::from(12u64));
/// ```
pub fn zl_to_bigint(zl: &[u64]) -> BigUint {
    return zl
        .into_iter()
        .map(|fi| memoized_fibonacci_bigint_recursive(*fi))
        .sum();
}

/// Decompresses a slice of bytes compressed using the Zeckendorf algorithm, assuming the original data was compressed using the big endian bytes interpretation.
/// 
/// Assume the original input data was interpreted as a big endian integer, for now. See the TODO in the zeckendorf_compress_be function for more information.
///
/// # Examples
///
/// ```
/// # use zeckendorf_rs::zeckendorf_decompress_be;
/// assert_eq!(zeckendorf_decompress_be(&[0]), vec![0]);
/// assert_eq!(zeckendorf_decompress_be(&[1]), vec![1]);
/// assert_eq!(zeckendorf_decompress_be(&[0b111]), vec![12]);
/// assert_eq!(zeckendorf_decompress_be(&[33, 2]), vec![255]);
/// assert_eq!(zeckendorf_decompress_be(&[34, 2]), vec![1, 0]);
/// ```
pub fn zeckendorf_decompress_be(compressed_data: &[u8]) -> Vec<u8> {
    // Unpack the compressed data into bits
    let compressed_data_as_bits = unpack_bytes_to_ezba_bits(compressed_data);
    // println!("Compressed data as bits: {:?}", compressed_data_as_bits);
    // Unpack the bits into an ezla (Effective Zeckendorf List Ascending)
    let compressed_data_as_ezla = ezba_to_ezla(&compressed_data_as_bits);
    // println!("Compressed data as ezla: {:?}", compressed_data_as_ezla);
    // Convert the ezla to a zla (Zeckendorf List Ascending)
    let compressed_data_as_zla = ezl_to_zl(&compressed_data_as_ezla);
    // println!("Compressed data as zla: {:?}", compressed_data_as_zla);
    // Convert the zla to a bigint
    let compressed_data_as_bigint = zl_to_bigint(&compressed_data_as_zla);
    // println!("Compressed data as bigint: {:?}", compressed_data_as_bigint);
    return compressed_data_as_bigint.to_bytes_be();
}
