//! Zeckendorf compression and decompression library
//!
//! This library provides functionality for compressing and decompressing data using the Zeckendorf algorithm.
//!
//! The Zeckendorf algorithm is a way to represent numbers as a sum of non-consecutive Fibonacci numbers.
//! If we first interpret the input data as a big integer, we can then represent the integer as a sum of non-consecutive Fibonacci numbers.
//! Sometimes this results in a more compact representation of the data, but it is not guaranteed.


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
        return 0
    }
    32 - n.leading_zeros()
}
