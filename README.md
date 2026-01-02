# Zeckendorf Compression

A Rust library for compressing and decompressing data using the Zeckendorf representation algorithm.

## Overview

The Zeckendorf algorithm represents numbers as a sum of non-consecutive Fibonacci numbers. This library interprets input data as a big integer (either big-endian or little-endian), converts it to its Zeckendorf representation, and sometimes achieves compression. However, compression is not guaranteed; the algorithm may result in a larger representation depending on the input data. The library can automatically try both endian interpretations and select the one that produces the best compression.

## Features

- **Compression & Decompression**: Convert data to/from Zeckendorf representation
- **Multiple Endian Interpretations**: Support for both big-endian and little-endian input interpretations
- **Automatic Best Compression**: Try both endian interpretations and automatically select the best result
- **Multiple Fibonacci Algorithms**: 
  - Slow recursive (memoized, for small numbers)
  - Slow iterative (memoized, for large numbers)
  - Fast Doubling (optimized, ~160x faster for large indices)
- **BigInt Support**: Handle arbitrarily large numbers using `num-bigint`
- **Memoization**: Thread-safe caching for improved performance
- **Statistics & Visualization**: Generate compression statistics and plots
- **Benchmarking**: Comprehensive performance benchmarks
- **WebAssembly Support**: Available as a WebAssembly module for use in web browsers

## WebAssembly

This library is also available as a WebAssembly module for use in web browsers. Available functions are marked with the `#[wasm_bindgen]` attribute. The WebAssembly module can be built using the convenience script at `scripts/build_wasm_bundle.sh` that builds the WebAssembly module with the `wasm-pack` tool.

You can see a live demo of the WebAssembly module in action at <https://prizz.github.io/zeckendorf-webapp/>. The source code for the demo is available at <https://github.com/pRizz/zeckendorf-webapp>.

## Installation

### Install from crates.io

Run:
```bash
cargo add zeck
```

Or add this to your `Cargo.toml`:

```toml
[dependencies]
zeck = "0.1.0"
```

For plotting features:

```toml
[dependencies]
zeck = { version = "0.1.0", features = ["plotting"] }
```

### Install from GitHub (development version)

Run:
```bash
cargo add zeck --git https://github.com/pRizz/zeckendorf
```

Or add this to your `Cargo.toml`:

```toml
[dependencies]
zeck = { git = "https://github.com/pRizz/zeckendorf" }
```

For plotting features:

```toml
[dependencies]
zeck = { git = "https://github.com/pRizz/zeckendorf", features = ["plotting"] }
```

### Install from npm

Run:
```bash
npm install zeck
```

Or add this to your `package.json`:

```json
{
  "dependencies": {
    "zeck": "^0.1.0"
  }
}
```

## Usage

### Basic Compression/Decompression

#### Big-Endian Interpretation

```rust
use zeck::{zeckendorf_compress_be, zeckendorf_decompress_be};

// Compress data (interpreted as big-endian integer)
let data = vec![12u8];
let compressed = zeckendorf_compress_be(&data);

// Decompress data
let decompressed = zeckendorf_decompress_be(&compressed);
assert_eq!(data, decompressed);
```

#### Little-Endian Interpretation

```rust
use zeck::{zeckendorf_compress_le, zeckendorf_decompress_le};

// Compress data (interpreted as little-endian integer)
let data = vec![12u8];
let compressed = zeckendorf_compress_le(&data);

// Decompress data
let decompressed = zeckendorf_decompress_le(&compressed);
assert_eq!(data, decompressed);
```

#### Automatic Best Compression

```rust
use zeck::{zeckendorf_compress_best, zeckendorf_decompress_be, zeckendorf_decompress_le, CompressionResult};

// Try both endian interpretations and get the best result
let data = vec![1, 0];
let result = zeckendorf_compress_best(&data);

match result {
    CompressionResult::BigEndianBest { compressed_data, le_size } => {
        // Big-endian produced the best compression
        let decompressed = zeckendorf_decompress_be(&compressed_data);
        assert_eq!(data, decompressed);
    }
    CompressionResult::LittleEndianBest { compressed_data, be_size } => {
        // Little-endian produced the best compression
        let decompressed = zeckendorf_decompress_le(&compressed_data);
        assert_eq!(data, decompressed);
    }
    CompressionResult::Neither { be_size, le_size } => {
        // Neither method compressed the data (both were larger than original)
        println!("Neither method compressed: BE size = {}, LE size = {}", be_size, le_size);
    }
}
```

### Fibonacci Numbers

```rust
use zeck::memoized_slow_fibonacci_recursive;

// Calculate Fibonacci numbers (for indices up to 93)
let fib_10 = memoized_slow_fibonacci_recursive(10); // Returns 55

// For larger numbers, use BigInt versions
use zeck::fast_doubling_fibonacci_bigint;
let fib_100 = fast_doubling_fibonacci_bigint(100);
```

### Zeckendorf Representation

```rust
use zeck::memoized_zeckendorf_list_descending_for_integer;

// Get Zeckendorf representation as a list of Fibonacci indices
let zld = memoized_zeckendorf_list_descending_for_integer(12);
// Returns [6, 4, 2] meaning F(6) + F(4) + F(2) = 8 + 3 + 1 = 12
```

## Binaries

The project includes several utility binaries:

### Main Playground

```bash
cargo run --release --bin zeck
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
- **Memoization**: Thread-safe caching significantly improves repeated calculations. The trade-off is that the cache takes up memory.
- **Compression Effectiveness**: Varies by input; compression ratios oscillate and become less favorable as input size increases

## Algorithm Details

### Zeckendorf Representation

Every positive integer can be uniquely represented as a sum of non-consecutive Fibonacci numbers. For example:
- 12 = 8 + 3 + 1 = F(6) + F(4) + F(2)

### Compression Process

1. Input data is interpreted as either a big-endian or little-endian integer (you can choose, or use `zeckendorf_compress_best` to try both)
2. The integer is converted to its Zeckendorf representation (list of Fibonacci indices)
3. The representation is encoded as bits (use/skip bits)
4. Bits are packed into bytes (little-endian output)

The library provides functions to compress with either interpretation, or you can use `zeckendorf_compress_best` to automatically try both and select the one that produces the smallest output.

### Effective Fibonacci Indices

The library uses "Effective Fibonacci Indices" (EFI) starting from 0, where:
- EFI 0 = Fibonacci Index 2 (value 1)
- EFI 1 = Fibonacci Index 3 (value 2)
- etc.

This avoids redundant Fibonacci numbers (F(0)=0 and F(1)=F(2)=1).

## Limitations

- Compression is not guaranteed—some inputs may result in larger output
- Compression effectiveness decreases as input size increases
- The library supports both big-endian and little-endian interpretations, but other byte orderings or word boundaries are not currently explored
- Compression of large amounts of data causes memory issues. It is currently not recommended to compress files larger than 100,000 bytes.

## License

This project is licensed under the MIT License - see the [LICENSE.txt](LICENSE.txt) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## References

- [Fast Fibonacci Algorithms](https://www.nayuki.io/page/fast-fibonacci-algorithms) - Fast doubling algorithm reference
- [Zeckendorf's Theorem](https://en.wikipedia.org/wiki/Zeckendorf%27s_theorem) - Every positive integer has a unique representation as a sum of non-consecutive Fibonacci numbers
