# Zeckendorf Compression

A Rust library for compressing and decompressing data using the Zeckendorf representation algorithm.

## Overview

The Zeckendorf algorithm represents numbers as a sum of non-consecutive Fibonacci numbers. This library interprets input data as a big integer (either big-endian or little-endian), converts it to its Zeckendorf representation, and sometimes achieves compression. However, compression is not guaranteed; the algorithm may result in a larger representation depending on the input data. The library can automatically try both endian interpretations and select the one that produces the best compression.

**⚠️ Warning:** Compressing or decompressing files larger than 10KB (10,000 bytes) is unstable due to time and memory pressure. The library may experience performance issues, excessive memory usage, or failures when processing files exceeding this size.

**Command-line tools** (`zeck-compress` and `zeck-decompress`) are available and can be installed via `cargo install zeck`. See the [Binaries](#binaries) section for usage details.

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

The project includes several utility binaries. The command-line compression tools (`zeck-compress` and `zeck-decompress`) can be installed globally via:

### Install from crates.io

```bash
cargo install zeck
```

### Install from GitHub (development version)

```bash
cargo install --git https://github.com/pRizz/zeckendorf zeck
```

After installation, you can use `zeck-compress` and `zeck-decompress` directly from your command line.

### Compression/Decompression Tools

**⚠️ Warning:** Compressing or decompressing files larger than 10KB (10,000 bytes) is unstable due to time and memory pressure. The library may experience performance issues, excessive memory usage, or failures when processing files exceeding this size.

#### zeck-compress

Compresses data using the Zeckendorf representation algorithm. Automatically adds `.zbe` extension for big-endian compression and `.zle` extension for little-endian compression.

```bash
zeck-compress [INPUT] [-o OUTPUT] [--endian ENDIAN] [-v]
```

**Options:**
- `INPUT`: Input file path (optional, reads from stdin if not specified)
  - Shows a warning if reading from stdin and no data was piped in
- `-o, --output FILE`: Output file path (optional)
  - If not specified and input is a file, uses the input filename with the appropriate extension (`.zbe` or `.zle`) appended
  - If not specified and reading from stdin, writes to stdout
  - The appropriate extension (`.zbe` for big-endian, `.zle` for little-endian) is automatically added unless the file already ends with `.zbe` or `.zle`
- `--endian ENDIAN`: Endianness to use (`big`, `little`, or `best`). Default: `best`
  - `big`: Use big-endian interpretation (output will have `.zbe` extension)
  - `little`: Use little-endian interpretation (output will have `.zle` extension)
  - `best`: Try both and use the best result (default, extension added based on which was used)
  - **Note:** When using `best`, if neither method produces compression (both result in larger or equal output), the tool will exit with an error showing compression statistics
- `-v, --verbose`: Show compression statistics (default: true, use `--no-verbose` to disable)

**Examples:**
```bash
# Compress a file (output filename automatically created from input with extension)
zeck-compress input.bin
# Creates input.bin.zbe or input.bin.zle depending on which endianness was used

# Compress with best endianness (statistics shown by default)
zeck-compress input.bin --endian best

# Compress with specific endianness (creates input.bin.zbe)
zeck-compress input.bin --endian big

# Compress to a specific output file
zeck-compress input.bin -o output
# Creates output.zbe or output.zle depending on which endianness was used

# Compress from stdin to stdout
cat input.bin | zeck-compress
```

**Note:** When writing to a file, the output filename is printed to stdout (e.g., "Compressed to: input.bin.zbe"). Verbose statistics are shown by default and include descriptive messages about compression ratios (e.g., "File was compressed by X.XX% (Y bytes -> Z bytes)"). A warning is shown when reading from stdin if no data was piped in.

#### zeck-decompress

Decompresses data that was compressed using the Zeckendorf representation algorithm. Automatically detects endianness from file extension (`.zbe` for big-endian, `.zle` for little-endian).

```bash
zeck-decompress [INPUT] [-o OUTPUT] [--endian ENDIAN] [-v]
```

**Options:**
- `INPUT`: Input file path (optional, reads from stdin if not specified)
  - When reading from a file, endianness is automatically detected from file extension (`.zbe` for big-endian, `.zle` for little-endian)
  - **If extension is not recognized, `--endian` is REQUIRED** (exits with error if not specified)
  - **When reading from stdin, `--endian` is REQUIRED**
  - Shows a warning if reading from stdin and no data was piped in
- `-o, --output FILE`: Output file path (optional)
  - If not specified and input is a file, uses the input filename with `.zbe` or `.zle` extension removed
  - If not specified and reading from stdin, writes to stdout
- `--endian ENDIAN`: Endianness used during compression (`big` or `little`)
  - `big`: Decompress as big-endian
  - `little`: Decompress as little-endian
  - **REQUIRED when reading from stdin** (no input file specified)
  - When reading from a file, this option overrides automatic detection from file extension
- `-v, --verbose`: Show decompression statistics (default: true, use `--no-verbose` to disable)

**Examples:**
```bash
# Decompress a file (endianness detected from .zbe extension, output filename automatically created)
zeck-decompress input.zbe
# Automatically uses big-endian decompression, creates output file "input"

# Decompress with little-endian file
zeck-decompress input.zle
# Automatically uses little-endian decompression, creates output file "input"

# Decompress to a specific output file
zeck-decompress input.zbe -o output.bin
# Automatically uses big-endian decompression

# Override automatic detection
zeck-decompress input.zbe --endian little -o output.bin
# Overrides the .zbe extension and uses little-endian

# Decompress from stdin to stdout (--endian is required)
cat input.zbe | zeck-decompress --endian big
```

**Note:** The endianness used for decompression must match the endianness used during compression. The file extension (`.zbe` or `.zle`) indicates which endianness was used, so decompression will automatically use the correct endianness when reading from a file. **If the input file doesn't have a recognized extension, `--endian` must be explicitly specified** (the tool will exit with an error if not provided). You can override automatic detection with the `--endian` flag if needed. **When reading from stdin, `--endian` must be explicitly specified** since there's no file extension to detect from.

**Additional features:**
- When writing to a file, the output filename is printed to stdout (e.g., "Compressed to: input.bin.zbe" or "Decompressed to: output.bin")
- Verbose statistics are shown by default (use `--no-verbose` to disable) and include descriptive messages about compression/decompression ratios
- Compression will exit with an error if the data cannot be compressed (when using `--endian best` and neither method produces compression)
- A warning is shown when reading from stdin if no data was piped in

### Main Playground

```bash
cargo run --release --bin zeck
```

A playground/scratchpad for testing library functions.

### Generate Test Data

```bash
cargo run --release --bin zeck-generate-data --features development_tools -- <size_in_bytes> [filename]
```

Generates random test data files in the `generated_data/` directory.

Example:
```bash
cargo run --release --bin zeck-generate-data --features development_tools -- 1024 my_file.bin
```

### Generate Statistics

```bash
cargo run --release --bin zeck-generate-statistics --features plotting,development_tools
```

Generates comprehensive compression statistics and plots:
- Compression ratios across different input sizes
- Chance of compression being favorable
- Average and median compression ratios
- Statistics saved to `statistics_history/` directory
- Plots saved to `plots/` directory

### Plot Compression Ratios

```bash
cargo run --release --bin zeck-plot --features plotting,development_tools
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
- **⚠️ Warning:** Compressing or decompressing files larger than 10KB (10,000 bytes) is unstable due to time and memory pressure. The library may experience performance issues, excessive memory usage, or failures when processing files exceeding this size.

## License

This project is licensed under the MIT License - see the [LICENSE.txt](LICENSE.txt) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## References

- [Fast Fibonacci Algorithms](https://www.nayuki.io/page/fast-fibonacci-algorithms) - Fast doubling algorithm reference
- [Zeckendorf's Theorem](https://en.wikipedia.org/wiki/Zeckendorf%27s_theorem) - Every positive integer has a unique representation as a sum of non-consecutive Fibonacci numbers
