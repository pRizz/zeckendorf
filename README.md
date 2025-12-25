# Zeckendorf Compression

A Rust library for compressing and decompressing data using the Zeckendorf representation algorithm.

## Overview

The Zeckendorf algorithm represents numbers as a sum of non-consecutive Fibonacci numbers. This library interprets input data as a big integer, converts it to its Zeckendorf representation, and sometimes achieves compression. However, compression is not guaranteed; the algorithm may result in a larger representation depending on the input data.

## Features

- **Compression & Decompression**: Convert data to/from Zeckendorf representation
- **Multiple Fibonacci Algorithms**: 
  - Slow recursive (memoized, for small numbers)
  - Slow iterative (memoized, for large numbers)
  - Fast Doubling (optimized, ~160x faster for large indices)
- **BigInt Support**: Handle arbitrarily large numbers using `num-bigint`
- **Memoization**: Thread-safe caching for improved performance
- **Statistics & Visualization**: Generate compression statistics and plots
- **Benchmarking**: Comprehensive performance benchmarks

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zeckendorf = "0.1.0"
```

For plotting features:

```toml
[dependencies]
zeckendorf = { version = "0.1.0", features = ["plotting"] }
```

## Usage

### Basic Compression/Decompression

```rust
use zeckendorf_rs::{zeckendorf_compress_be, zeckendorf_decompress_be};

// Compress data
let data = vec![12u8];
let compressed = zeckendorf_compress_be(&data);

// Decompress data
let decompressed = zeckendorf_decompress_be(&compressed);
assert_eq!(data, decompressed);
```

### Fibonacci Numbers

```rust
use zeckendorf_rs::memoized_slow_fibonacci_recursive;

// Calculate Fibonacci numbers (for indices up to 93)
let fib_10 = memoized_slow_fibonacci_recursive(10); // Returns 55

// For larger numbers, use BigInt versions
use zeckendorf_rs::fast_doubling_fibonacci_bigint;
let fib_100 = fast_doubling_fibonacci_bigint(100);
```

### Zeckendorf Representation

```rust
use zeckendorf_rs::memoized_zeckendorf_list_descending_for_integer;

// Get Zeckendorf representation as a list of Fibonacci indices
let zld = memoized_zeckendorf_list_descending_for_integer(12);
// Returns [6, 2] meaning F(6) + F(2) = 8 + 1 = 9
// Note: The actual representation may differ based on the algorithm
```

## Binaries

The project includes several utility binaries:

### Main Playground

```bash
cargo run --release --bin zeckendorf
```

A playground/scratchpad for testing library functions.

### Generate Test Data

```bash
cargo run --release --bin generate_data <size_in_bytes> [filename]
```

Generates random test data files in the `generated_data/` directory.

Example:
```bash
cargo run --release --bin generate_data 1024 my_file.bin
```

### Generate Statistics

```bash
cargo run --release --bin generate_statistics --features plotting
```

Generates comprehensive compression statistics and plots:
- Compression ratios across different input sizes
- Chance of compression being favorable
- Average and median compression ratios
- Statistics saved to `statistics_history/` directory
- Plots saved to `plots/` directory

### Plot Compression Ratios

```bash
cargo run --release --bin plot --features plotting
```

Generates visualization plots of:
- Fibonacci numbers
- Compression ratios for various input ranges

## Benchmarks

### Zeckendorf Compression Benchmarks

```bash
cargo bench --bench zeckendorf_bench
```

Benchmarks compression, decompression, and round-trip performance for various data sizes (4 bytes to 16KB).

### Fibonacci Benchmarks

```bash
cargo bench --bench fibonacci_bench
```

Compares performance of different Fibonacci calculation algorithms:
- Slow iterative method
- Fast doubling method (~160x faster for large indices)

### Working with Benchmark Baselines

Save a new baseline:
```bash
cargo bench --bench zeckendorf_bench -- --save-baseline <name>
```

Compare to an existing baseline:
```bash
cargo bench --bench zeckendorf_bench -- --baseline <name>
```

## Performance Characteristics

- **Fast Doubling Fibonacci**: ~160x faster than iterative method for the 100,000th Fibonacci number
- **Memoization**: Thread-safe caching significantly improves repeated calculations
- **Compression Effectiveness**: Varies by input; compression ratios oscillate and become less favorable as input size increases

## Algorithm Details

### Zeckendorf Representation

Every positive integer can be uniquely represented as a sum of non-consecutive Fibonacci numbers. For example:
- 12 = 8 + 3 + 1 = F(6) + F(4) + F(2)

### Compression Process

1. Input data is interpreted as a big-endian integer
2. The integer is converted to its Zeckendorf representation (list of Fibonacci indices)
3. The representation is encoded as bits (use/skip bits)
4. Bits are packed into bytes (little-endian output)

### Effective Fibonacci Indices

The library uses "Effective Fibonacci Indices" (EFI) starting from 0, where:
- EFI 0 = Fibonacci Index 2 (value 1)
- EFI 1 = Fibonacci Index 3 (value 2)
- etc.

This avoids redundant Fibonacci numbers (F(0)=0 and F(1)=F(2)=1).

## Limitations

- Compression is not guaranteed—some inputs may result in larger output
- Compression effectiveness decreases as input size increases
- The current implementation interprets input as big-endian (see TODO in code for potential improvements)

## License

This project is licensed under the MIT License - see the [LICENSE.txt](LICENSE.txt) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## References

- [Fast Fibonacci Algorithms](https://www.nayuki.io/page/fast-fibonacci-algorithms) - Fast doubling algorithm reference
- [Zeckendorf's Theorem](https://en.wikipedia.org/wiki/Zeckendorf%27s_theorem) - Every positive integer has a unique representation as a sum of non-consecutive Fibonacci numbers
