//! Shared functions for the examples
//!
//! Every function in this file is marked with #[allow(unused)] to ignore compiler warnings.
//! This happens because this file gets recompiled for every example, and the compiler doesn't know which functions are used.
//! This is a workaround to prevent the compiler from complaining about unused functions.
//!
//! Build with: `cargo build --release --example shared`

use num_bigint::BigUint;
use std::sync::Arc;

// #[allow(unused)]
// pub fn sqrt(x: f64) -> f64 {
//     x.sqrt()
// }

/// Helper function to convert `Arc<BigUint>` to `u64` for plotting.
///
/// **WARNING:** This is a **lossy/unsafe boundary** for large values:
/// - If the value does not fit in `u64`, this function **panics** (it does not clamp or approximate).
/// - If you need a lossy approximation instead of a hard failure, use `biguint_to_approximate_f64`.
#[allow(unused)]
pub fn biguint_to_approximate_u64(value: &Arc<BigUint>) -> u64 {
    let digits = value.to_u64_digits();
    if digits.len() == 1 {
        digits[0]
    } else if digits.is_empty() {
        0
    } else {
        panic!("Fibonacci value too large to fit in u64");
    }
}

/// Helper function to convert BigUint to f64 for plotting.
///
/// For values that don't fit in `f64`, this uses a **lossy** approximation based only on the number of bits.
///
/// **WARNING:** This can **truncate/lose information**:
/// - Many distinct `BigUint` values will map to the same output (it ignores all but the bit-length).
/// - Very large inputs are **clamped** to `2^1023` to avoid `f64` overflow, so magnitude above that is discarded.
///
/// Use this only for coarse plotting/visualization, not for computations requiring numeric fidelity.
#[allow(unused)]
pub fn biguint_to_approximate_f64(value: &BigUint) -> f64 {
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

/// Dummy main function to make the example build.
#[allow(unused)]
fn main() {}
