//! Zeckendorf compression and decompression library
//!
//! This library provides functionality for compressing and decompressing data using the Zeckendorf algorithm.
//!
//! The Zeckendorf algorithm is a way to represent numbers as a sum of non-consecutive Fibonacci numbers.
//! If we first interpret the input data as a big integer, we can then represent the integer as a sum of non-consecutive Fibonacci numbers.
//! Sometimes this results in a more compact representation of the data, but it is not guaranteed.
//! Learn more about the Zeckendorf Theorem in the [Zeckendorf's Theorem](https://en.wikipedia.org/wiki/Zeckendorf%27s_theorem) Wikipedia article.
//!
//! # ⚠️ Warning
//!
//! **Compressing or decompressing files larger than 10KB (10,000 bytes) is unstable due to time and memory pressure.**
//! The library may experience performance issues, excessive memory usage, or failures when processing files exceeding this size.
//!
//! This library is also available as a WebAssembly module for use in web browsers. Available functions are marked with the `#[wasm_bindgen]` attribute. The WebAssembly module can be built using the convenience script at `scripts/build_wasm_bundle.sh` that builds the WebAssembly module with the `wasm-pack` tool.
//!
//! You can see a live demo of the WebAssembly module in action at <https://prizz.github.io/zeckendorf-webapp/>. The source code for the demo is available at <https://github.com/pRizz/zeckendorf-webapp>.
//!
//! ## Command-Line Tools
//!
//! This library includes two command-line tools for compressing and decompressing data.
//! They can be installed globally via:
//! - `cargo install zeck` (from crates.io)
//! - `cargo install --git https://github.com/pRizz/zeckendorf zeck` (from GitHub)
//!
//! After installation, `zeck-compress` and `zeck-decompress` will be available in your PATH.
//!
//! The compression tool automatically uses `.zbe` extension for big-endian compression and `.zle`
//! extension for little-endian compression. The decompression tool automatically detects endianness
//! from these file extensions.
//!
//! ### zeck-compress
//!
//! Compresses data using the Zeckendorf representation algorithm. Supports reading from files or stdin,
//! writing to files or stdout, and choosing between big-endian, little-endian, or automatic best compression.
//! Automatically adds the appropriate file extension (`.zbe` or `.zle`) based on the endianness used.
//!
//! When using `--endian best`, if neither compression method produces a smaller output, the tool will
//! exit with an error showing compression statistics. When writing to a file, the output filename is
//! printed to stdout. Verbose statistics are shown by default and include descriptive compression ratio messages.
//!
//! ```bash
//! # Compress a file (output filename automatically created from input with extension)
//! zeck-compress input.bin
//! # Creates input.bin.zbe or input.bin.zle depending on which endianness was used
//!
//! # Compress with best endianness (statistics shown by default)
//! zeck-compress input.bin --endian best
//!
//! # Compress from stdin to stdout
//! cat input.bin | zeck-compress
//! ```
//!
//! ### zeck-decompress
//!
//! Decompresses data that was compressed using the Zeckendorf representation algorithm. Supports reading
//! from files or stdin, writing to files or stdout. Automatically detects endianness from file extension
//! (`.zbe` for big-endian, `.zle` for little-endian), but allows manual override with the `--endian` flag.
//!
//! ```bash
//! # Decompress a file (endianness detected from .zbe extension, output filename automatically created)
//! zeck-decompress input.zbe
//! # Automatically uses big-endian decompression, creates output file "input"
//!
//! # Decompress to a specific output file
//! zeck-decompress input.zbe -o output.bin
//! # Automatically uses big-endian decompression
//!
//! # Override automatic detection
//! zeck-decompress input.zbe --endian little -o output.bin
//!
//! # Decompress from stdin to stdout (--endian is required)
//! cat input.zbe | zeck-decompress --endian big
//! ```

use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use wasm_bindgen::prelude::*;

pub mod zeck_file_format;

pub use zeck_file_format::{
    ZeckFile, ZeckFormatError, compress::compress_zeck_be, compress::compress_zeck_best,
    compress::compress_zeck_le, decompress::decompress_zeck_file, file::deserialize_zeck_file,
};

/// Golden ratio constant.
/// This constant is in the rust standard library as [`std::f64::consts::PHI`], but only available on nightly.
pub const PHI: f64 = 1.618033988749894848204586834365638118_f64;

/// Phi squared constant.
/// This also equals the golden ratio plus one.
pub const PHI_SQUARED: f64 = 2.618033988749894848204586834365638118_f64;

/// Returns the number of bits required to represent the given number. Returns 0 if the number is less than or equal to 0.
///
/// # Examples
///
/// ```
/// # use zeck::bit_count_for_number;
/// assert_eq!(bit_count_for_number(0), 0);
/// assert_eq!(bit_count_for_number(1), 1);  // 0b1
/// assert_eq!(bit_count_for_number(2), 2);  // 0b10
/// assert_eq!(bit_count_for_number(3), 2);  // 0b11
/// assert_eq!(bit_count_for_number(4), 3);  // 0b100
/// ```
#[wasm_bindgen]
pub fn bit_count_for_number(n: i32) -> u32 {
    if n <= 0 {
        return 0;
    }
    32 - n.leading_zeros()
}

// Memoization maps for Fibonacci numbers
static FIBONACCI_CACHE: LazyLock<RwLock<Vec<u64>>> = LazyLock::new(|| RwLock::new(vec![0, 1]));

static FIBONACCI_BIGUINT_CACHE: LazyLock<RwLock<Vec<Arc<BigUint>>>> =
    LazyLock::new(|| RwLock::new(vec![Arc::new(BigUint::zero()), Arc::new(BigUint::one())]));

/// Memoization maps for Zeckendorf representations
static ZECKENDORF_MAP: LazyLock<RwLock<HashMap<u64, Vec<u64>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
/// We will store the Zeckendorf list descending as [`u64`]s because the Fibonacci indices are small enough to fit in a [`u64`].
/// It takes up to 694,241 bits, or ~694kbits, to represent the 1,000,000th Fibonacci number.
/// The max [`u64`] is 18,446,744,073,709,551,615 which is ~18 quintillion.
/// So a [`u64`] can represent Fibonacci indices 18 trillion times larger than the 1,000,000th,
/// so a [`u64`] can represent Fibonacci values up to
/// roughly 18 trillion times 694,241 bits which is 1.249*10^19 bits which or 1.56 exabytes.
/// We will consider larger numbers in the future :-)
static ZECKENDORF_BIGUINT_MAP: LazyLock<RwLock<HashMap<BigUint, Vec<u64>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Sparse cache for fast doubling Fibonacci algorithm
pub static FAST_DOUBLING_FIBONACCI_BIGUINT_CACHE: LazyLock<RwLock<HashMap<u64, Arc<BigUint>>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert(0, Arc::new(BigUint::zero()));
        map.insert(1, Arc::new(BigUint::one()));
        RwLock::new(map)
    });

/// fibonacci(x) is equal to 0 if x is 0; 1 if x is 1; else return fibonacci(x - 1) + fibonacci(x - 2)
///
/// This function is slow and should not be used for large numbers. If you want a [`u64`] result, use the faster [`memoized_slow_fibonacci_biguint_iterative`] function instead. If you want a [`BigUint`] result, use the [`fast_doubling_fibonacci_biguint`] function instead.
///
/// This function fails for large numbers (e.g. 100_000) with a stack overflow error.
///
/// `fi` stands for Fibonacci Index.
///
/// # Examples
///
/// ```
/// # use zeck::memoized_slow_fibonacci_recursive;
/// // Base cases
/// assert_eq!(memoized_slow_fibonacci_recursive(0), 0);
/// assert_eq!(memoized_slow_fibonacci_recursive(1), 1);
///
/// // Small Fibonacci numbers
/// assert_eq!(memoized_slow_fibonacci_recursive(2), 1);
/// assert_eq!(memoized_slow_fibonacci_recursive(3), 2);
/// assert_eq!(memoized_slow_fibonacci_recursive(4), 3);
/// assert_eq!(memoized_slow_fibonacci_recursive(5), 5);
/// assert_eq!(memoized_slow_fibonacci_recursive(6), 8);
/// assert_eq!(memoized_slow_fibonacci_recursive(7), 13);
/// assert_eq!(memoized_slow_fibonacci_recursive(8), 21);
/// assert_eq!(memoized_slow_fibonacci_recursive(9), 34);
/// assert_eq!(memoized_slow_fibonacci_recursive(10), 55);
/// ```
#[wasm_bindgen]
pub fn memoized_slow_fibonacci_recursive(fi: u64) -> u64 {
    let fi = fi as usize;

    // Try to get the value with a read lock first
    {
        let fibonacci_cache = FIBONACCI_CACHE
            .read()
            .expect("Failed to read Fibonacci cache");
        if let Some(&fibonacci_value) = fibonacci_cache.get(fi) {
            return fibonacci_value;
        }
    }

    // If not found, get a write lock to update the cache
    let mut fibonacci_cache = FIBONACCI_CACHE
        .write()
        .expect("Failed to write Fibonacci cache");

    // Re-check in case another thread updated it while we were waiting for the write lock
    while fibonacci_cache.len() <= fi {
        let fibonacci_cache_length = fibonacci_cache.len();
        // Fibonacci numbers above index 93 will overflow u64
        if fibonacci_cache_length > 93 {
            panic!("Fibonacci index {} overflows u64", fibonacci_cache_length);
        }
        let next_fibonacci_value = fibonacci_cache[fibonacci_cache_length - 1]
            + fibonacci_cache[fibonacci_cache_length - 2];
        fibonacci_cache.push(next_fibonacci_value);
    }

    fibonacci_cache[fi]
}

/// fibonacci(x) is equal to 0 if x is 0; 1 if x is 1; else return fibonacci(x - 1) + fibonacci(x - 2)
/// fi stands for Fibonacci Index
///
/// This function is slow and should not be used for large numbers. If you are ok with a [`BigUint`] result, use the [`fast_doubling_fibonacci_biguint`] function instead.
///
/// # Examples
///
/// ```
/// # use zeck::memoized_slow_fibonacci_biguint_iterative;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// // Base cases
/// assert_eq!(*memoized_slow_fibonacci_biguint_iterative(0u64), BigUint::zero());
/// assert_eq!(*memoized_slow_fibonacci_biguint_iterative(1u64), BigUint::one());
///
/// // Small Fibonacci numbers
/// assert_eq!(*memoized_slow_fibonacci_biguint_iterative(2u64), BigUint::from(1u64));
/// assert_eq!(*memoized_slow_fibonacci_biguint_iterative(3u64), BigUint::from(2u64));
/// assert_eq!(*memoized_slow_fibonacci_biguint_iterative(4u64), BigUint::from(3u64));
/// assert_eq!(*memoized_slow_fibonacci_biguint_iterative(5u64), BigUint::from(5u64));
/// assert_eq!(*memoized_slow_fibonacci_biguint_iterative(6u64), BigUint::from(8u64));
/// assert_eq!(*memoized_slow_fibonacci_biguint_iterative(7u64), BigUint::from(13u64));
/// assert_eq!(*memoized_slow_fibonacci_biguint_iterative(8u64), BigUint::from(21u64));
/// assert_eq!(*memoized_slow_fibonacci_biguint_iterative(9u64), BigUint::from(34u64));
/// assert_eq!(*memoized_slow_fibonacci_biguint_iterative(10u64), BigUint::from(55u64));
/// ```
///
/// TODO: consider returning a reference to the cached value to avoid the clone.
pub fn memoized_slow_fibonacci_biguint_iterative(fi: u64) -> Arc<BigUint> {
    let fi = fi as usize;

    // Try to get the value with a read lock first
    {
        let fibonacci_cache = FIBONACCI_BIGUINT_CACHE
            .read()
            .expect("Failed to read Fibonacci BigUint cache");
        if let Some(fibonacci_value) = fibonacci_cache.get(fi) {
            return Arc::clone(fibonacci_value);
        }
    }

    // If not found, get a write lock to update the cache
    let mut fibonacci_cache = FIBONACCI_BIGUINT_CACHE
        .write()
        .expect("Failed to write Fibonacci BigUint cache");

    // Re-check in case another thread updated it while we were waiting for the write lock
    while fibonacci_cache.len() <= fi {
        let fibonacci_cache_length = fibonacci_cache.len();
        // Since we initialize with [0, 1], fibonacci_cache_length is at least 2 here
        let next_fibonacci_value = &*fibonacci_cache[fibonacci_cache_length - 1]
            + &*fibonacci_cache[fibonacci_cache_length - 2];
        fibonacci_cache.push(Arc::new(next_fibonacci_value));
    }

    Arc::clone(&fibonacci_cache[fi])
}

/// fibonacci(x) is equal to 0 if x is 0; 1 if x is 1; else return fibonacci(x - 1) + fibonacci(x - 2)
/// fi stands for Fibonacci Index
///
/// This function is slow and should not be used for large numbers. If you are ok with a [`BigUint`] result, use the [`fast_doubling_fibonacci_biguint`] function instead.
///
/// # Examples
///
/// ```
/// # use zeck::slow_fibonacci_biguint_iterative;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// // Base cases
/// assert_eq!(*slow_fibonacci_biguint_iterative(0u64), BigUint::zero());
/// assert_eq!(*slow_fibonacci_biguint_iterative(1u64), BigUint::one());
///
/// // Small Fibonacci numbers
/// assert_eq!(*slow_fibonacci_biguint_iterative(2u64), BigUint::from(1u64));
/// assert_eq!(*slow_fibonacci_biguint_iterative(3u64), BigUint::from(2u64));
/// assert_eq!(*slow_fibonacci_biguint_iterative(4u64), BigUint::from(3u64));
/// assert_eq!(*slow_fibonacci_biguint_iterative(5u64), BigUint::from(5u64));
/// assert_eq!(*slow_fibonacci_biguint_iterative(6u64), BigUint::from(8u64));
/// assert_eq!(*slow_fibonacci_biguint_iterative(7u64), BigUint::from(13u64));
/// assert_eq!(*slow_fibonacci_biguint_iterative(8u64), BigUint::from(21u64));
/// assert_eq!(*slow_fibonacci_biguint_iterative(9u64), BigUint::from(34u64));
/// assert_eq!(*slow_fibonacci_biguint_iterative(10u64), BigUint::from(55u64));
/// ```
pub fn slow_fibonacci_biguint_iterative(fi: u64) -> Arc<BigUint> {
    let mut f0 = BigUint::zero();
    let mut f1 = BigUint::one();
    for _ in 0..fi {
        let f2 = &f0 + &f1;
        f0 = f1;
        f1 = f2;
    }

    Arc::new(f0)
}

/// fibonacci(x) is equal to 0 if x is 0; 1 if x is 1; else return fibonacci(x - 1) + fibonacci(x - 2)
/// fi stands for Fibonacci Index
///
/// This function is faster than [`slow_fibonacci_biguint_iterative`] by using a method called Fast Doubling,
/// an optimization of the Matrix Exponentiation method. See <https://www.nayuki.io/page/fast-fibonacci-algorithms> for more details.
///
/// Running the Fibonacci benchmarks (`cargo bench --bench fibonacci_bench`),
/// this function is ~160x faster than [`slow_fibonacci_biguint_iterative`] at calculating the 100,000th Fibonacci number.
/// On my computer, the fast function took around 330µs while the slow function took around 53ms.
///
/// TODO: use Karatsuba multiplication to speed up the multiplication of [`BigUint`].
///
/// # Examples
///
/// ```
/// # use zeck::fast_doubling_fibonacci_biguint;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// // Base cases
/// assert_eq!(*fast_doubling_fibonacci_biguint(0u64), BigUint::zero());
/// assert_eq!(*fast_doubling_fibonacci_biguint(1u64), BigUint::one());
///
/// // Small Fibonacci numbers
/// assert_eq!(*fast_doubling_fibonacci_biguint(2u64), BigUint::from(1u64));
/// assert_eq!(*fast_doubling_fibonacci_biguint(3u64), BigUint::from(2u64));
/// assert_eq!(*fast_doubling_fibonacci_biguint(4u64), BigUint::from(3u64));
/// assert_eq!(*fast_doubling_fibonacci_biguint(5u64), BigUint::from(5u64));
/// assert_eq!(*fast_doubling_fibonacci_biguint(6u64), BigUint::from(8u64));
/// assert_eq!(*fast_doubling_fibonacci_biguint(7u64), BigUint::from(13u64));
/// assert_eq!(*fast_doubling_fibonacci_biguint(8u64), BigUint::from(21u64));
/// assert_eq!(*fast_doubling_fibonacci_biguint(9u64), BigUint::from(34u64));
/// assert_eq!(*fast_doubling_fibonacci_biguint(10u64), BigUint::from(55u64));
/// ```
pub fn fast_doubling_fibonacci_biguint(fi: u64) -> Arc<BigUint> {
    let mut a = BigUint::zero();
    let mut b = BigUint::one();
    let mut m = BigUint::zero();
    let mut fi_msb = highest_one_bit(fi);
    while fi_msb != 0 {
        let d = a.clone() * ((b.clone() << 1) - &a);
        let e = a.pow(2) + b.pow(2);
        a = d;
        b = e;
        m *= 2u8;

        if fi & fi_msb != 0 {
            let tmp = a + &b;
            a = b;
            b = tmp;
            m += 1u8;
        }

        fi_msb >>= 1;
    }

    Arc::new(a)
}

/// fibonacci(x) is equal to 0 if x is 0; 1 if x is 1; else return fibonacci(x - 1) + fibonacci(x - 2)
/// fi stands for Fibonacci Index
///
/// This function is faster than [`slow_fibonacci_biguint_iterative`] by using a method called Fast Doubling,
/// an optimization of the Matrix Exponentiation method. See <https://www.nayuki.io/page/fast-fibonacci-algorithms> for more details.
///
/// This function includes memoization using a sparse [`HashMap`] cache ([`FAST_DOUBLING_FIBONACCI_BIGUINT_CACHE`])
/// to cache results. The implementation uses a [`HashMap`] instead of a [`Vec`] to allow sparse caching of only
/// the computed values, which is more memory-efficient for large, non-contiguous Fibonacci index ranges.
///
/// The algorithm tracks the Fibonacci index `m` during the fast doubling loop. According to the
/// fast doubling algorithm, we maintain `(a, b)` representing `(F(m), F(m+1))`, and at each step:
/// - When we double: `m` becomes `2m`, and we compute `(F(2m), F(2m+1))`
/// - When we advance (if bit is set): `m` becomes `2m+1`, and we compute `(F(2m+1), F(2m+2))`
///
/// Intermediate values (F(m) at each step) are collected during the loop and then batch-written to
/// the cache at the end to reduce lock contention. This approach allows caching intermediate values
/// on the fly while maintaining good performance.
///
/// TODO: use Karatsuba multiplication to speed up the multiplication of [`BigUint`].
///
/// TODO: if we have a cache miss, we could try intelligently walking backwards from the target index to find the nearest cached values and continue the fast doubling algorithm from there.
///
/// FIXME: for some reason, using this fast Fibonacci function in the Zeckendorf functions slows down the Zeckendorf codec benchmarks.
///
/// # Examples
///
/// ```
/// # use zeck::memoized_fast_doubling_fibonacci_biguint;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// // Base cases
/// assert_eq!(*memoized_fast_doubling_fibonacci_biguint(0u64), BigUint::zero());
/// assert_eq!(*memoized_fast_doubling_fibonacci_biguint(1u64), BigUint::one());
///
/// // Small Fibonacci numbers
/// assert_eq!(*memoized_fast_doubling_fibonacci_biguint(2u64), BigUint::from(1u64));
/// assert_eq!(*memoized_fast_doubling_fibonacci_biguint(3u64), BigUint::from(2u64));
/// assert_eq!(*memoized_fast_doubling_fibonacci_biguint(4u64), BigUint::from(3u64));
/// assert_eq!(*memoized_fast_doubling_fibonacci_biguint(5u64), BigUint::from(5u64));
/// assert_eq!(*memoized_fast_doubling_fibonacci_biguint(6u64), BigUint::from(8u64));
/// assert_eq!(*memoized_fast_doubling_fibonacci_biguint(7u64), BigUint::from(13u64));
/// assert_eq!(*memoized_fast_doubling_fibonacci_biguint(8u64), BigUint::from(21u64));
/// assert_eq!(*memoized_fast_doubling_fibonacci_biguint(9u64), BigUint::from(34u64));
/// assert_eq!(*memoized_fast_doubling_fibonacci_biguint(10u64), BigUint::from(55u64));
/// ```
pub fn memoized_fast_doubling_fibonacci_biguint(fi: u64) -> Arc<BigUint> {
    // Try to get the value with a read lock first
    {
        let cache = FAST_DOUBLING_FIBONACCI_BIGUINT_CACHE
            .read()
            .expect("Failed to read fast doubling Fibonacci cache");
        if let Some(cached_value) = cache.get(&fi) {
            return Arc::clone(cached_value);
        }
    }

    // If not found, calculate using fast doubling and cache intermediate values
    // The algorithm maintains (a, b) representing (F(m), F(m+1)) where m is the current index
    // Based on fast doubling identities from https://www.nayuki.io/page/fast-fibonacci-algorithms:
    // F(2k) = F(k) * [2*F(k+1) - F(k)]
    // F(2k+1) = F(k+1)^2 + F(k)^2
    let mut a = BigUint::zero();
    let mut b = BigUint::one();
    let mut m: u64 = 0;
    let mut fi_msb = highest_one_bit(fi);
    let mut values_to_cache: Vec<(u64, Arc<BigUint>)> = Vec::new();

    while fi_msb != 0 {
        // Double: (F(m), F(m+1)) -> (F(2m), F(2m+1))
        // Using the fast doubling identities:
        // F(2m) = d = F(m) * [2*F(m+1) - F(m)]
        let d = a.clone() * ((b.clone() << 1) - &a);
        // F(2m+1) = e = F(m+1)^2 + F(m)^2
        let e = b.pow(2) + a.pow(2);
        a = d;
        b = e;
        m *= 2;

        // Track F(2m) for caching (we'll check if it's already cached when we write)
        values_to_cache.push((m, Arc::new(a.clone())));

        if fi & fi_msb != 0 {
            // Advance: (F(2m), F(2m+1)) -> (F(2m+1), F(2m+2))
            // F(2m+2) = F(2m+1) + F(2m)
            let tmp = a + &b;
            a = b;
            b = tmp;
            m += 1;

            // Track F(2m+1) for caching
            values_to_cache.push((m, Arc::new(a.clone())));
        }

        fi_msb >>= 1;
    }

    // Cache all intermediate values and the final result in a single batch write
    let result = Arc::new(a);
    values_to_cache.push((fi, Arc::clone(&result)));

    let mut cache = FAST_DOUBLING_FIBONACCI_BIGUINT_CACHE
        .write()
        .expect("Failed to write fast doubling Fibonacci cache");

    // Re-check the final value in case another thread updated it while we were computing
    if let Some(cached_value) = cache.get(&fi) {
        return Arc::clone(cached_value);
    }

    // Insert all values that aren't already cached
    for (fi, value) in values_to_cache {
        cache.entry(fi).or_insert(value);
    }

    result
}

/// Returns a [`u64`] value with only the most significant set bit of n preserved.
///
/// # Examples
///
/// ```
/// # use zeck::highest_one_bit;
/// assert_eq!(highest_one_bit(0), 0);
/// assert_eq!(highest_one_bit(1), 1);
/// assert_eq!(highest_one_bit(2), 2);
/// assert_eq!(highest_one_bit(3), 2);
/// assert_eq!(highest_one_bit(4), 4);
/// assert_eq!(highest_one_bit(5), 4);
/// assert_eq!(highest_one_bit(6), 4);
/// assert_eq!(highest_one_bit(7), 4);
/// assert_eq!(highest_one_bit(8), 8);
/// assert_eq!(highest_one_bit(9), 8);
/// assert_eq!(highest_one_bit(10), 8);
/// assert_eq!(highest_one_bit(11), 8);
/// assert_eq!(highest_one_bit(12), 8);
/// assert_eq!(highest_one_bit(13), 8);
/// assert_eq!(highest_one_bit(14), 8);
/// ```
#[wasm_bindgen]
pub fn highest_one_bit(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }

    1u64 << (63 - n.leading_zeros())
}

/// A descending Zeckendorf list is a sorted list of unique Fibonacci indices, in descending order, that sum to the given number.
///
/// A Fibonacci index is the index of the Fibonacci number in the Fibonacci sequence.
/// fibonacci(fibonacci_index) = fibonacci_number
///
/// # Examples
///
/// ```
/// # use zeck::memoized_zeckendorf_list_descending_for_integer;
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
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(11), vec![6, 4]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_integer(12), vec![6, 4, 2]);
/// ```
#[wasm_bindgen]
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

    // Try a read lock first
    {
        let zeckendorf_map = ZECKENDORF_MAP
            .read()
            .expect("Failed to read Zeckendorf map");
        if let Some(cached) = zeckendorf_map.get(&n) {
            return cached.clone();
        }
    }

    let mut current_n = n;
    let mut low = 1u64;
    let mut high = 1u64;

    // Exponential search for upper bound
    while memoized_slow_fibonacci_recursive(high) < current_n {
        low = high;
        high *= 2;
        // Fibonacci numbers above index 93 will overflow u64
        if high > 93 {
            panic!("Fibonacci index {} overflows u64", high);
        }
    }

    // Binary search for the smallest index i such that F[i] >= current_n
    while low <= high {
        let mid = low + (high - low) / 2;
        if mid == 0 {
            low = 1;
            break;
        }
        if memoized_slow_fibonacci_recursive(mid) < current_n {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    let mut max_fibonacci_index_smaller_than_n = low;

    let mut zeckendorf_list: Vec<u64> = Vec::new();
    while current_n > 0 {
        let current_fibonacci_value =
            memoized_slow_fibonacci_recursive(max_fibonacci_index_smaller_than_n);
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
        .write()
        .expect("Failed to write Zeckendorf map");
    zeckendorf_map.insert(n, zeckendorf_list.clone());
    zeckendorf_list
}

/// A descending Zeckendorf list is a sorted list of unique Fibonacci indices, in descending order, that sum to the given number.
///
/// A Fibonacci index is the index of the Fibonacci number in the Fibonacci sequence.
/// fibonacci(fibonacci_index) = fibonacci_number
///
/// # Examples
///
/// ```
/// # use zeck::memoized_zeckendorf_list_descending_for_biguint;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// // Base cases
/// assert_eq!(memoized_zeckendorf_list_descending_for_biguint(&BigUint::zero()), vec![]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_biguint(&BigUint::one()), vec![2]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_biguint(&BigUint::from(2u64)), vec![3]);
///
/// // Small Zeckendorf numbers
/// assert_eq!(memoized_zeckendorf_list_descending_for_biguint(&BigUint::from(3u64)), vec![4]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_biguint(&BigUint::from(4u64)), vec![4, 2]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_biguint(&BigUint::from(5u64)), vec![5]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_biguint(&BigUint::from(6u64)), vec![5, 2]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_biguint(&BigUint::from(7u64)), vec![5, 3]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_biguint(&BigUint::from(8u64)), vec![6]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_biguint(&BigUint::from(9u64)), vec![6, 2]);
/// assert_eq!(memoized_zeckendorf_list_descending_for_biguint(&BigUint::from(10u64)), vec![6, 3]);
/// ```
pub fn memoized_zeckendorf_list_descending_for_biguint(n: &BigUint) -> Vec<u64> {
    if n == &BigUint::zero() {
        return vec![];
    }
    if n == &BigUint::one() {
        return vec![2];
    }
    if n == &BigUint::from(2u64) {
        return vec![3];
    }

    // Try a read lock first
    {
        let zeckendorf_biguint_map = ZECKENDORF_BIGUINT_MAP
            .read()
            .expect("Failed to read Zeckendorf BigUint map");
        if let Some(cached) = zeckendorf_biguint_map.get(n) {
            return cached.clone();
        }
    }

    let original_n = n.clone();
    let mut current_n = n.clone();
    let mut low = 1u64;
    let mut high = 1u64;

    // Exponential search for upper bound
    while *memoized_slow_fibonacci_biguint_iterative(high) < current_n {
        low = high;
        high *= 2;
    }

    // Binary search for the smallest index i such that F[i] >= current_n
    while low <= high {
        let mid = low + (high - low) / 2;
        if mid == 0 {
            low = 1;
            break;
        }
        if *memoized_slow_fibonacci_biguint_iterative(mid) < current_n {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    let mut max_fibonacci_index_smaller_than_n = low;

    let mut zeckendorf_list: Vec<u64> = Vec::new();
    while current_n > BigUint::zero() {
        let current_fibonacci_value =
            memoized_slow_fibonacci_biguint_iterative(max_fibonacci_index_smaller_than_n);
        if *current_fibonacci_value > current_n {
            max_fibonacci_index_smaller_than_n -= 1;
            continue;
        }
        current_n -= &*current_fibonacci_value;
        zeckendorf_list.push(max_fibonacci_index_smaller_than_n);
        // We can subtract 2 because the next Fibonacci number that fits is at least 2 indices away due to the Zeckendorf principle.
        max_fibonacci_index_smaller_than_n -= 2;
    }

    let mut zeckendorf_biguint_map = ZECKENDORF_BIGUINT_MAP
        .write()
        .expect("Failed to write Zeckendorf BigUint map");
    zeckendorf_biguint_map.insert(original_n, zeckendorf_list.clone());
    zeckendorf_list
}

/// Bit flag indicating that an effective Fibonacci index (EFI) should be used in the Zeckendorf representation.
///
/// When this bit is set in an Effective Zeckendorf Bits Ascending (EZBA) sequence, it means the corresponding
/// Fibonacci number should be included in the sum. Due to the Zeckendorf theorem, consecutive Fibonacci numbers
/// cannot be used, so when a [`USE_BIT`] is encountered, the next EFI is skipped (incremented by 2).
pub const USE_BIT: u8 = 1;

/// Bit flag indicating that an effective Fibonacci index (EFI) should be skipped in the Zeckendorf representation.
///
/// When this bit is set in an Effective Zeckendorf Bits Ascending (EZBA) sequence, it means the corresponding
/// Fibonacci number should not be included in the sum. The EFI counter advances by 1 when this [`SKIP_BIT`] is encountered.
pub const SKIP_BIT: u8 = 0;

/// Result of attempting padless compression by interpreting the input data as both big endian and little endian big integers.
///
/// This enum represents which interpretation produced the best padless compression result, or if neither
/// produced padless compression (both were larger than the original).
#[derive(Debug, Clone, PartialEq)]
pub enum PadlessCompressionResult {
    /// Big endian padless compression produced the smallest output.
    /// Contains the compressed data and the size of the little endian attempt for comparison.
    BigEndianBest {
        /// The compressed data using big endian interpretation
        compressed_data: Vec<u8>,
        /// Compressed size using little endian interpretation (for comparison)
        le_size: usize,
    },
    /// Little endian padless compression produced the smallest output.
    /// Contains the compressed data and the size of the big endian attempt for comparison.
    LittleEndianBest {
        /// The compressed data using little endian interpretation
        compressed_data: Vec<u8>,
        /// Compressed size using big endian interpretation (for comparison)
        be_size: usize,
    },
    /// Neither padless compression method produced a smaller output than the original.
    /// Contains sizes for both attempts.
    Neither {
        /// Compressed size using big endian interpretation
        be_size: usize,
        /// Compressed size using little endian interpretation
        le_size: usize,
    },
}

/// Effective Fibonacci Index to Fibonacci Index: FI(efi) === efi + 2, where efi is the Effective Fibonacci Index
///
/// # Examples
///
/// ```
/// # use zeck::efi_to_fi;
/// assert_eq!(efi_to_fi(0), 2);
/// assert_eq!(efi_to_fi(1), 3);
/// assert_eq!(efi_to_fi(2), 4);
/// ```
#[wasm_bindgen]
pub fn efi_to_fi(efi: u64) -> u64 {
    return efi + 2;
}

/// Effective Fibonacci Index to Fibonacci Index: FI(efi) === efi + 2, where efi is the Effective Fibonacci Index
///
/// # Examples
///
/// ```
/// # use zeck::efi_to_fi_ref;
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
/// # use zeck::efi_to_fi_biguint;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// assert_eq!(efi_to_fi_biguint(BigUint::zero()), BigUint::from(2u64));
/// assert_eq!(efi_to_fi_biguint(BigUint::one()), BigUint::from(3u64));
/// assert_eq!(efi_to_fi_biguint(BigUint::from(2u64)), BigUint::from(4u64));
/// ```
pub fn efi_to_fi_biguint(efi: BigUint) -> BigUint {
    return efi + BigUint::from(2u64);
}

/// Fibonacci Index to Effective Fibonacci Index: EFI(fi) === fi - 2, where fi is the Fibonacci Index
///
/// # Examples
///
/// ```
/// # use zeck::fi_to_efi;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// assert_eq!(fi_to_efi(2), 0);
/// assert_eq!(fi_to_efi(3), 1);
/// assert_eq!(fi_to_efi(4), 2);
/// ```
#[wasm_bindgen]
pub fn fi_to_efi(fi: u64) -> u64 {
    return fi - 2;
}

/// Fibonacci Index to Effective Fibonacci Index: EFI(fi) === fi - 2, where fi is the Fibonacci Index
///
/// # Examples
///
/// ```
/// # use zeck::fi_to_efi_ref;
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
/// # use zeck::fi_to_efi_biguint;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// assert_eq!(fi_to_efi_biguint(BigUint::from(2u64)), BigUint::zero());
/// assert_eq!(fi_to_efi_biguint(BigUint::from(3u64)), BigUint::one());
/// assert_eq!(fi_to_efi_biguint(BigUint::from(4u64)), BigUint::from(2u64));
/// ```
pub fn fi_to_efi_biguint(fi: BigUint) -> BigUint {
    return fi - BigUint::from(2u64);
}

/// The memoized Fibonacci function taking an Effective Fibonacci Index as input.
///
/// # Examples
///
/// ```
/// # use zeck::memoized_effective_fibonacci;
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
#[wasm_bindgen]
pub fn memoized_effective_fibonacci(efi: u64) -> u64 {
    return memoized_slow_fibonacci_recursive(efi_to_fi(efi));
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
/// # use zeck::zl_to_ezl;
/// assert_eq!(zl_to_ezl(&[2]), vec![0]);
/// assert_eq!(zl_to_ezl(&[3]), vec![1]);
/// assert_eq!(zl_to_ezl(&[4]), vec![2]);
/// ```
#[wasm_bindgen]
pub fn zl_to_ezl(zl: &[u64]) -> Vec<u64> {
    return zl.into_iter().map(fi_to_efi_ref).collect();
}

/// Converts an Effective Zeckendorf List to a Zeckendorf List.
/// It does not matter if the list is ascending or descending; it retains the directionality of the original list.
///
/// # Examples
///
/// ```
/// # use zeck::ezl_to_zl;
/// assert_eq!(ezl_to_zl(&[0]), vec![2]);
/// assert_eq!(ezl_to_zl(&[1]), vec![3]);
/// assert_eq!(ezl_to_zl(&[2]), vec![4]);
/// ```
#[wasm_bindgen]
pub fn ezl_to_zl(ezl: &[u64]) -> Vec<u64> {
    return ezl.into_iter().map(efi_to_fi_ref).collect();
}

/// ezba is Effective Zeckendorf Bits Ascending ; ezld is Effective Zeckendorf List Descending
///
/// The bits represent whether the corresponding effective Fibonacci index is used. I call these "use bits" and "skip bits" where a use bit is 1 and a skip bit is 0. This is by convention that I, Peter Ryszkiewicz decided, but it is theoretically possible to use skip bits and use bits flipped.
///
/// If we use a bit, we then skip the next bit, because it is impossible to use two consecutive bits, or Fibonacci numbers, due to the Zeckendorf principle.
/// The first bit in the ezba represents whether the first effective Fibonacci index is used.
/// The first effective Fibonacci index is always 0 and represents the Fibonacci index 2 which has a value of 1. We use effective Fibonacci indices because the first Fibonacci number, 0, is not useful for sums, and the second Fibonacci number, 1, is redundant because it is the same as the third Fibonacci number.
///
/// TODO: Optimize the size of the output by using a bit vector instead of a vector of [`u8`]s. I made an initial attempt at this in the `use-bitvec` branch, but the benchmarks were slower.
///
/// # Examples
///
/// ```
/// # use zeck::ezba_from_ezld;
/// assert_eq!(ezba_from_ezld(&[]), vec![0]);
/// assert_eq!(ezba_from_ezld(&[0]), vec![1]); // 0th EFI is 2nd FI, which is 1
/// assert_eq!(ezba_from_ezld(&[1]), vec![0, 1]); // 1st EFI is 3rd FI, which is 2
/// assert_eq!(ezba_from_ezld(&[2]), vec![0, 0, 1]); // 2nd EFI is 4th FI, which is 3
/// assert_eq!(ezba_from_ezld(&[2, 0]), vec![1, 1]); // 2nd EFI is 4th FI, which is 3 and 0th EFI is 2nd FI, which is 1, which sums to 4
/// assert_eq!(ezba_from_ezld(&[3]), vec![0, 0, 0, 1]); // 3rd EFI is 5th FI, which is 5
/// ```
#[wasm_bindgen]
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
/// Every 8 bits become a [`u8`] in the output.
/// The last byte is padded with 0s if the number of bits is not a multiple of 8.
///
/// # Examples
///
/// ```
/// # use zeck::pack_ezba_bits_to_bytes;
/// assert_eq!(pack_ezba_bits_to_bytes(&[0]), vec![0]);
/// assert_eq!(pack_ezba_bits_to_bytes(&[1]), vec![1]);
/// assert_eq!(pack_ezba_bits_to_bytes(&[0, 1]), vec![0b10]);
/// assert_eq!(pack_ezba_bits_to_bytes(&[0, 0, 1]), vec![0b100]);
/// assert_eq!(pack_ezba_bits_to_bytes(&[1, 1]), vec![0b11]);
/// ```
#[wasm_bindgen]
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

/// Compresses a slice of bytes using the Padless Zeckendorf Compression algorithm.
///
/// Assumes the input data is interpreted as a big endian integer. The output data is in little endian order, so the first bit and byte is the least significant bit and byte and the last bit and byte is the most significant bit and byte.
///
/// # ⚠️ Important: Original Size Preservation
///
/// **This function strips leading zero bytes from the input data during compression.**
/// It is the caller's responsibility to retain the original size information (e.g., `data.len()`)
/// before calling this function. When decompressing, the original size must be used to pad the
/// decompressed data with leading zeros to restore the exact original data. Without the original
/// size, information will be lost during decompression.
///
/// For a format that automatically handles size preservation, use [`crate::zeck_file_format::compress::compress_zeck_be`]
/// instead, which includes a header with the original size information.
///
/// # ⚠️ Warning
///
/// **Compressing or decompressing data larger than 10KB (10,000 bytes) is unstable due to time and memory pressure.**
/// The library may experience performance issues, excessive memory usage, or failures when processing data exceeding this size.
///
/// TODO: Technically, the way the input data is interpreted is arbitrary; we could interpret it as little endian which could result in a more compact representation. We could go even further and interpret the data at different byte or word boundaries to see if it results in a more compact representation, and signify to the caller which interpretation was used. We probably need a better understanding of random distributions of data to determine what is the optimal interpretation. More investigation is needed here.
///
/// # Examples
///
/// ```
/// # use zeck::padless_zeckendorf_compress_be_dangerous;
/// assert_eq!(padless_zeckendorf_compress_be_dangerous(&[0]), vec![0]);
/// assert_eq!(padless_zeckendorf_compress_be_dangerous(&[1]), vec![1]);
/// assert_eq!(padless_zeckendorf_compress_be_dangerous(&[12]), vec![0b111]);
/// assert_eq!(padless_zeckendorf_compress_be_dangerous(&[54]), vec![30]);
/// assert_eq!(padless_zeckendorf_compress_be_dangerous(&[55]), vec![0, 1]); // 55 is the 10 indexed Fibonacci number, which is the 8 indexed effective Fibonacci number, and therefore is the first number needing two bytes to contain these 8 bits, because there is 1 "use bit" and 7 "skip bits" in the effective zeckendorf bits ascending.
/// assert_eq!(padless_zeckendorf_compress_be_dangerous(&[255]), vec![33, 2]);
/// assert_eq!(padless_zeckendorf_compress_be_dangerous(&[1, 0]), vec![34, 2]);
/// ```
#[wasm_bindgen]
pub fn padless_zeckendorf_compress_be_dangerous(data: &[u8]) -> Vec<u8> {
    let compressed_data: Vec<u8>;
    // Turn data into a biguint
    let data_as_biguint = BigUint::from_bytes_be(data);
    // println!("Data as biguint: {:?}", data_as_biguint);
    // Get the effective zeckendorf list descending
    let data_as_zld = memoized_zeckendorf_list_descending_for_biguint(&data_as_biguint);
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

/// Compresses a slice of bytes using the Padless Zeckendorf Compression algorithm.
///
/// Assumes the input data is interpreted as a little endian integer. The output data is in little endian order, so the first bit and byte is the least significant bit and byte and the last bit and byte is the most significant bit and byte.
///
/// # ⚠️ Important: Original Size Preservation
///
/// **This function strips leading zero bytes from the input data during compression.**
/// It is the caller's responsibility to retain the original size information (e.g., `data.len()`)
/// before calling this function. When decompressing, the original size must be used to pad the
/// decompressed data with leading zeros to restore the exact original data. Without the original
/// size, information will be lost during decompression.
///
/// For a format that automatically handles size preservation, use [`crate::zeck_file_format::compress::compress_zeck_le`]
/// instead, which includes a header with the original size information.
///
/// # ⚠️ Warning
///
/// **Compressing or decompressing data larger than 10KB (10,000 bytes) is unstable due to time and memory pressure.**
/// The library may experience performance issues, excessive memory usage, or failures when processing data exceeding this size.
///
/// # Examples
///
/// ```
/// # use zeck::padless_zeckendorf_compress_le_dangerous;
/// assert_eq!(padless_zeckendorf_compress_le_dangerous(&[0]), vec![0]);
/// assert_eq!(padless_zeckendorf_compress_le_dangerous(&[1]), vec![1]);
/// assert_eq!(padless_zeckendorf_compress_le_dangerous(&[12]), vec![0b111]);
/// assert_eq!(padless_zeckendorf_compress_le_dangerous(&[54]), vec![30]);
/// assert_eq!(padless_zeckendorf_compress_le_dangerous(&[55]), vec![0, 1]); // 55 is the 10 indexed Fibonacci number, which is the 8 indexed effective Fibonacci number, and therefore is the first number needing two bytes to contain these 8 bits, because there is 1 "use bit" and 7 "skip bits" in the effective zeckendorf bits ascending.
/// assert_eq!(padless_zeckendorf_compress_le_dangerous(&[255]), vec![33, 2]);
/// assert_eq!(padless_zeckendorf_compress_le_dangerous(&[0, 1]), vec![34, 2]);
/// ```
#[wasm_bindgen]
pub fn padless_zeckendorf_compress_le_dangerous(data: &[u8]) -> Vec<u8> {
    let compressed_data: Vec<u8>;
    // Turn data into a biguint
    let data_as_biguint = BigUint::from_bytes_le(data);
    // println!("Data as biguint: {:?}", data_as_biguint);
    // Get the effective zeckendorf list descending
    let data_as_zld = memoized_zeckendorf_list_descending_for_biguint(&data_as_biguint);
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
/// # use zeck::unpack_bytes_to_ezba_bits;
/// assert_eq!(unpack_bytes_to_ezba_bits(&[0]), vec![0, 0, 0, 0, 0, 0, 0, 0]);
/// assert_eq!(unpack_bytes_to_ezba_bits(&[1]), vec![1, 0, 0, 0, 0, 0, 0, 0]);
/// assert_eq!(unpack_bytes_to_ezba_bits(&[0b111]), vec![1, 1, 1, 0, 0, 0, 0, 0]);
/// assert_eq!(unpack_bytes_to_ezba_bits(&[1, 1]), vec![1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
/// ```
#[wasm_bindgen]
pub fn unpack_bytes_to_ezba_bits(bytes: &[u8]) -> Vec<u8> {
    let mut ezba_bits = Vec::with_capacity(bytes.len() * 8);
    for byte in bytes {
        for i in 0..8 {
            ezba_bits.push((byte >> i) & 1);
        }
    }
    return ezba_bits;
}

/// Converts a vector of bits (0s and 1s) from an ezba (Effective Zeckendorf Bits Ascending) into a vector of effective Fibonacci indices,
/// the Effective Zeckendorf List Ascending.
///
/// # Examples
///
/// ```
/// # use zeck::ezba_to_ezla;
/// assert_eq!(ezba_to_ezla(&[0, 0, 0, 0, 0, 0, 0, 0]), vec![]);
/// assert_eq!(ezba_to_ezla(&[1, 0, 0, 0, 0, 0, 0, 0]), vec![0]);
/// assert_eq!(ezba_to_ezla(&[1, 1, 1, 0, 0, 0, 0, 0]), vec![0, 2, 4]);
/// ```
#[wasm_bindgen]
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

/// Converts a Zeckendorf List to a [`BigUint`].
///
/// The Zeckendorf List is a list of Fibonacci indices that sum to the given number.
/// It does not matter if the ZL is ascending or descending. The sum operation is commutative.
///
/// # Examples
///
/// ```
/// # use zeck::zl_to_biguint;
/// # use num_bigint::BigUint;
/// # use num_traits::{One, Zero};
/// assert_eq!(zl_to_biguint(&[]), BigUint::zero());
/// assert_eq!(zl_to_biguint(&[0]), BigUint::zero());
/// assert_eq!(zl_to_biguint(&[1]), BigUint::one());
/// assert_eq!(zl_to_biguint(&[2]), BigUint::one());
/// assert_eq!(zl_to_biguint(&[3]), BigUint::from(2u64));
/// assert_eq!(zl_to_biguint(&[4]), BigUint::from(3u64));
/// assert_eq!(zl_to_biguint(&[5]), BigUint::from(5u64));
/// assert_eq!(zl_to_biguint(&[6]), BigUint::from(8u64));
/// assert_eq!(zl_to_biguint(&[6, 2]), BigUint::from(9u64));
/// assert_eq!(zl_to_biguint(&[6, 3]), BigUint::from(10u64));
/// assert_eq!(zl_to_biguint(&[6, 4]), BigUint::from(11u64));
/// assert_eq!(zl_to_biguint(&[6, 4, 2]), BigUint::from(12u64));
/// ```
pub fn zl_to_biguint(zl: &[u64]) -> BigUint {
    zl.iter().fold(BigUint::zero(), |acc, fi| {
        acc + &*memoized_slow_fibonacci_biguint_iterative(*fi)
        // TODO: investigate ways we can get the lower memory usage of the cached fast doubling Fibonacci algorithm but the speed of the cached slow Fibonacci algorithm. As of now, the cached fast doubling Fibonacci algorithm is slower at decompression than the cached slow Fibonacci algorithm at large data inputs, on the order of > 10kB. See the comments in scripts/poll_rss.sh for more information.
        // acc + &*fast_doubling_fibonacci_biguint(*fi)
    })
}

/// Creates an "all ones Zeckendorf number", or AOZN, by creating an Effective Zeckendorf Bits Ascending (EZBA)
/// with `n` consecutive ones, then converting it to a [`BigUint`].
///
/// An AOZN is created by generating a Zeckendorf representation with `n`
/// consecutive ones (in the Effective Zeckendorf Bits Ascending format), then converting that
/// representation back to the actual number value. This is useful for understanding how Zeckendorf
/// representations behave when they contain many ones.
///
/// # Arguments
///
/// * `n` - The number of consecutive ones in the EZBA representation
///
/// # Returns
///
/// Returns [`BigUint::zero()`] if `n` is 0, otherwise returns the [`BigUint`] value of the all-ones
/// Zeckendorf representation.
///
/// # Examples
///
/// ```
/// # use zeck::all_ones_zeckendorf_to_biguint;
/// # use num_bigint::BigUint;
/// # use num_traits::Zero;
/// assert_eq!(all_ones_zeckendorf_to_biguint(0), BigUint::zero());
/// assert_eq!(all_ones_zeckendorf_to_biguint(1), BigUint::from(1u64)); // 1
/// assert_eq!(all_ones_zeckendorf_to_biguint(2), BigUint::from(4u64)); // 1 + 3
/// assert_eq!(all_ones_zeckendorf_to_biguint(3), BigUint::from(12u64)); // 1 + 3 + 8
/// assert_eq!(all_ones_zeckendorf_to_biguint(4), BigUint::from(33u64)); // 1 + 3 + 8 + 21
/// ```
pub fn all_ones_zeckendorf_to_biguint(n: usize) -> BigUint {
    if n == 0 {
        return BigUint::zero();
    }
    let ezba = vec![1u8; n];
    let ezla = ezba_to_ezla(&ezba);
    let zla = ezl_to_zl(&ezla);
    zl_to_biguint(&zla)
}

/// Decompresses a slice of bytes compressed using the Zeckendorf algorithm, assuming the original data was compressed using the big endian bytes interpretation.
///
/// Assume the original input data was interpreted as a big endian integer, for now. See the TODO in the [`padless_zeckendorf_compress_be_dangerous`] function for more information.
///
/// # ⚠️ Important: Leading Zero Padding
///
/// **This function does not pad leading zero bytes.** If the original data had leading zeros, they will not be restored.
/// The decompressed output will be the minimal representation of the number (without leading zeros).
///
/// For a format that automatically handles size preservation and padding, use [`crate::zeck_file_format::file::deserialize_zeck_file`]
/// and [`crate::zeck_file_format::decompress::decompress_zeck_file`] instead, which includes a header with the original size information and restores leading zeros.
///
/// # ⚠️ Warning
///
/// **Compressing or decompressing data larger than 10KB (10,000 bytes) is unstable due to time and memory pressure.**
/// The library may experience performance issues, excessive memory usage, or failures when processing data exceeding this size.
///
/// # Examples
///
/// ```
/// # use zeck::padless_zeckendorf_decompress_be_dangerous;
/// assert_eq!(padless_zeckendorf_decompress_be_dangerous(&[0]), vec![0]);
/// assert_eq!(padless_zeckendorf_decompress_be_dangerous(&[1]), vec![1]);
/// assert_eq!(padless_zeckendorf_decompress_be_dangerous(&[0b111]), vec![12]);
/// assert_eq!(padless_zeckendorf_decompress_be_dangerous(&[33, 2]), vec![255]);
/// assert_eq!(padless_zeckendorf_decompress_be_dangerous(&[34, 2]), vec![1, 0]);
/// ```
#[wasm_bindgen]
pub fn padless_zeckendorf_decompress_be_dangerous(compressed_data: &[u8]) -> Vec<u8> {
    // Unpack the compressed data into bits
    let compressed_data_as_bits = unpack_bytes_to_ezba_bits(compressed_data);
    // println!("Compressed data as bits: {:?}", compressed_data_as_bits);
    // Unpack the bits into an ezla (Effective Zeckendorf List Ascending)
    let compressed_data_as_ezla = ezba_to_ezla(&compressed_data_as_bits);
    // println!("Compressed data as ezla: {:?}", compressed_data_as_ezla);
    // Convert the ezla to a zla (Zeckendorf List Ascending)
    let compressed_data_as_zla = ezl_to_zl(&compressed_data_as_ezla);
    // println!("Compressed data as zla: {:?}", compressed_data_as_zla);
    // Convert the zla to a biguint
    let compressed_data_as_biguint = zl_to_biguint(&compressed_data_as_zla);
    // println!("Compressed data as biguint: {:?}", compressed_data_as_biguint);
    return compressed_data_as_biguint.to_bytes_be();
}

/// Decompresses a slice of bytes compressed using the Zeckendorf algorithm, assuming the original data was compressed using the little endian bytes interpretation.
///
/// # ⚠️ Important: Leading Zero Padding
///
/// **This function does not pad leading zero bytes.** If the original data had leading zeros, they will not be restored.
/// The decompressed output will be the minimal representation of the number (without leading zeros).
///
/// For a format that automatically handles size preservation and padding, use [`crate::zeck_file_format::file::deserialize_zeck_file`]
/// and [`crate::zeck_file_format::decompress::decompress_zeck_file`] instead, which includes a header with the original size information and restores leading zeros.
///
/// # ⚠️ Warning
///
/// **Compressing or decompressing data larger than 10KB (10,000 bytes) is unstable due to time and memory pressure.**
/// The library may experience performance issues, excessive memory usage, or failures when processing data exceeding this size.
///
/// # Examples
///
/// ```
/// # use zeck::padless_zeckendorf_decompress_le_dangerous;
/// assert_eq!(padless_zeckendorf_decompress_le_dangerous(&[0]), vec![0]);
/// assert_eq!(padless_zeckendorf_decompress_le_dangerous(&[1]), vec![1]);
/// assert_eq!(padless_zeckendorf_decompress_le_dangerous(&[0b111]), vec![12]);
/// assert_eq!(padless_zeckendorf_decompress_le_dangerous(&[33, 2]), vec![255]);
/// assert_eq!(padless_zeckendorf_decompress_le_dangerous(&[34, 2]), vec![0, 1]);
/// ```
#[wasm_bindgen]
pub fn padless_zeckendorf_decompress_le_dangerous(compressed_data: &[u8]) -> Vec<u8> {
    // Unpack the compressed data into bits
    let compressed_data_as_bits = unpack_bytes_to_ezba_bits(compressed_data);
    // println!("Compressed data as bits: {:?}", compressed_data_as_bits);
    // Unpack the bits into an ezla (Effective Zeckendorf List Ascending)
    let compressed_data_as_ezla = ezba_to_ezla(&compressed_data_as_bits);
    // println!("Compressed data as ezla: {:?}", compressed_data_as_ezla);
    // Convert the ezla to a zla (Zeckendorf List Ascending)
    let compressed_data_as_zla = ezl_to_zl(&compressed_data_as_ezla);
    // println!("Compressed data as zla: {:?}", compressed_data_as_zla);
    // Convert the zla to a biguint
    let compressed_data_as_biguint = zl_to_biguint(&compressed_data_as_zla);
    // println!("Compressed data as biguint: {:?}", compressed_data_as_biguint);
    return compressed_data_as_biguint.to_bytes_le();
}

/// Attempts to compress the input data using both big endian and little endian interpretations,
/// and returns the best result.
///
/// This function tries compressing the input data with both endian interpretations and returns
/// a [`CompressionResult`] enum indicating which method produced the smallest output, or if neither produced compression.
///
/// # ⚠️ Important: Original Size Preservation
///
/// **This function strips leading zero bytes from the input data during compression.**
/// It is the caller's responsibility to retain the original size information (e.g., `data.len()`)
/// before calling this function. When decompressing, the original size must be used to pad the
/// decompressed data with leading zeros to restore the exact original data. Without the original
/// size, information will be lost during decompression.
///
/// For a format that automatically handles size preservation, use [`crate::zeck_file_format::compress::compress_zeck_be`]
/// instead, which includes a header with the original size information.
///
/// # ⚠️ Warning
///
/// **Compressing or decompressing data larger than 10KB (10,000 bytes) is unstable due to time and memory pressure.**
/// The library may experience performance issues, excessive memory usage, or failures when processing data exceeding this size.
///
/// # Examples
///
/// ```
/// # use zeck::padless_zeckendorf_compress_best_dangerous;
/// # use zeck::PadlessCompressionResult;
/// let data = vec![1, 0];
/// let result = padless_zeckendorf_compress_best_dangerous(&data);
/// match result {
///     PadlessCompressionResult::BigEndianBest { compressed_data, le_size } => {
///         // Use compressed_data for decompression with [`padless_zeckendorf_decompress_be_dangerous`]
///     }
///     PadlessCompressionResult::LittleEndianBest { compressed_data, be_size } => {
///         // Use compressed_data for decompression with [`padless_zeckendorf_decompress_le_dangerous`]
///     }
///     PadlessCompressionResult::Neither { be_size, le_size } => {
///         // Neither method compressed the data
///     }
/// }
/// ```
pub fn padless_zeckendorf_compress_best_dangerous(data: &[u8]) -> PadlessCompressionResult {
    let input_size = data.len();

    // Try both compression methods
    let be_compressed = padless_zeckendorf_compress_be_dangerous(data);
    let le_compressed = padless_zeckendorf_compress_le_dangerous(data);

    let be_size = be_compressed.len();
    let le_size = le_compressed.len();

    // Determine which compression method is best
    if be_size < input_size && be_size < le_size {
        PadlessCompressionResult::BigEndianBest {
            compressed_data: be_compressed,
            le_size,
        }
    } else if le_size < input_size && le_size <= be_size {
        // Less than or equal to because if they are equal, we prefer LE
        PadlessCompressionResult::LittleEndianBest {
            compressed_data: le_compressed,
            be_size,
        }
    } else {
        PadlessCompressionResult::Neither { be_size, le_size }
    }
}
