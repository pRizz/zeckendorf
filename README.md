# Zeckendorf Compression

A Rust library for compressing and decompressing data using the Zeckendorf representation algorithm.

## Overview

The Zeckendorf algorithm represents numbers as a sum of non-consecutive Fibonacci numbers. This library interprets input data as a big integer (either big-endian or little-endian), converts it to its Zeckendorf representation, and sometimes achieves compression. However, compression is not guaranteed; the algorithm may result in a larger representation depending on the input data. The library can automatically try both endian interpretations and select the one that produces the best compression.

**⚠️ Warning:** Compressing or decompressing files larger than 10KB (10,000 bytes) is unstable due to time and memory pressure. The library may experience performance issues, excessive memory usage, or failures when processing files exceeding this size.

**Command-line tools** (`zeck-compress` and `zeck-decompress`) are available and can be installed via `cargo install zeck`. See the [Binaries](#binaries) section for usage details.

## Features

- **Compression & Decompression**: Convert data to/from Zeckendorf representation
- **File Format with Headers**: `.zeck` file format that automatically preserves original file size and endianness information
- **Multiple Endian Interpretations**: Support for both big-endian and little-endian input interpretations
- **Automatic Best Compression**: Try both endian interpretations and automatically select the best result
- **Multiple Fibonacci Algorithms**: 
  - Slow recursive (memoized, for small numbers)
  - Slow iterative (memoized, for large numbers)
  - Fast Doubling (optimized, ~160x faster for large indices)
  - Memoized Fast Doubling (with sparse HashMap caching for large, non-contiguous indices)
- **BigInt Support**: Handle arbitrarily large numbers using `num-bigint`
- **Memoization**: Thread-safe caching for improved performance
- **Statistics & Visualization**: Generate compression statistics and plots
- **Benchmarking**: Comprehensive performance benchmarks
- **WebAssembly Support**: Available as a WebAssembly module for use in web browsers
- **Error Handling**: Comprehensive error types for file format operations

## WebAssembly

This library is also available as a WebAssembly module for use in web browsers and JavaScript/TypeScript projects. The WebAssembly module can be installed via npm:

```bash
npm install zeck
```

Available functions are marked with the `#[wasm_bindgen]` attribute. The WebAssembly module can also be built manually using the convenience script at `scripts/build_wasm_bundle.sh` that builds the WebAssembly module with the `wasm-pack` tool.

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
zeck = "2.1.0"
```

**Features:**
- `cli_tools`: Enables the `zeck-compress` and `zeck-decompress` command-line binaries. This feature includes the `clap` dependency. Not enabled by default - use `--features cli_tools` when installing binaries.

For CLI tools (when installing binaries):

```toml
[dependencies]
zeck = { version = "2.1.0", features = ["cli_tools"] }
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

For CLI tools:

```toml
[dependencies]
zeck = { git = "https://github.com/pRizz/zeckendorf", features = ["cli_tools"] }
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
    "zeck": "^2.1.0"
  }
}
```

## Usage

### File Format Compression (Recommended)

The `.zeck` file format automatically handles size preservation and endianness information. This is the recommended approach for most use cases.

#### Big-Endian File Format

```rust
use zeck::zeck_file_format::{compress::compress_zeck_be, decompress::decompress_zeck_file};

// Compress data (interpreted as big-endian integer)
let data = vec![12u8];
let zeck_file = compress_zeck_be(&data)?;

// Serialize to bytes for storage
let bytes = zeck_file.to_bytes();

// Later, deserialize and decompress
use zeck::zeck_file_format::file::deserialize_zeck_file;
let zeck_file = deserialize_zeck_file(&bytes)?;
let decompressed = decompress_zeck_file(&zeck_file)?;
assert_eq!(data, decompressed);
```

#### Little-Endian File Format

```rust
use zeck::zeck_file_format::{compress::compress_zeck_le, decompress::decompress_zeck_file};

// Compress data (interpreted as little-endian integer)
let data = vec![12u8];
let zeck_file = compress_zeck_le(&data)?;

// Decompress data
let decompressed = decompress_zeck_file(&zeck_file)?;
assert_eq!(data, decompressed);
```

#### Automatic Best Compression (File Format)

```rust
use zeck::zeck_file_format::{
    compress::{compress_zeck_best, BestCompressionResult},
    decompress::decompress_zeck_file,
};

// Try both endian interpretations and get the best result
let data = vec![1, 0];
match compress_zeck_best(&data)? {
    BestCompressionResult::BigEndianBest { zeck_file, le_size } => {
        // Big-endian produced the best compression
        let decompressed = decompress_zeck_file(&zeck_file)?;
        assert_eq!(data, decompressed);
    }
    BestCompressionResult::LittleEndianBest { zeck_file, be_size } => {
        // Little-endian produced the best compression
        let decompressed = decompress_zeck_file(&zeck_file)?;
        assert_eq!(data, decompressed);
    }
    BestCompressionResult::Neither { be_size, le_size } => {
        // Neither method compressed the data (both were larger than original)
        println!("Neither method compressed: BE size = {}, LE size = {}", be_size, le_size);
    }
}
```

### Padless Compression (Advanced)

The padless compression functions strip leading zero bytes and do not preserve original size information. **You must manually track the original size** if you need to restore leading zeros. These functions are marked as `_dangerous` to indicate they require careful handling.

**⚠️ Important:** The padless functions are lower-level and do not preserve leading zero bytes. Use the file format functions above for most use cases.

#### Big-Endian Padless

```rust
use zeck::{padless_zeckendorf_compress_be_dangerous, padless_zeckendorf_decompress_be_dangerous};

// Compress data (interpreted as big-endian integer)
let data = vec![12u8];
let compressed = padless_zeckendorf_compress_be_dangerous(&data);

// Decompress data (leading zeros may be lost)
let decompressed = padless_zeckendorf_decompress_be_dangerous(&compressed);
// Note: decompressed may not equal data if data had leading zeros
```

#### Little-Endian Padless

```rust
use zeck::{padless_zeckendorf_compress_le_dangerous, padless_zeckendorf_decompress_le_dangerous};

// Compress data (interpreted as little-endian integer)
let data = vec![12u8];
let compressed = padless_zeckendorf_compress_le_dangerous(&data);

// Decompress data (trailing zeros may be lost)
let decompressed = padless_zeckendorf_decompress_le_dangerous(&compressed);
```

#### Automatic Best Padless Compression

```rust
use zeck::{padless_zeckendorf_compress_best_dangerous, PadlessCompressionResult};

let data = vec![1, 0];
match padless_zeckendorf_compress_best_dangerous(&data) {
    PadlessCompressionResult::BigEndianBest { compressed_data, le_size } => {
        // Use padless_zeckendorf_decompress_be_dangerous for decompression
    }
    PadlessCompressionResult::LittleEndianBest { compressed_data, be_size } => {
        // Use padless_zeckendorf_decompress_le_dangerous for decompression
    }
    PadlessCompressionResult::Neither { be_size, le_size } => {
        // Neither method compressed the data
    }
}
```

### Fibonacci Numbers

```rust
use zeck::memoized_slow_fibonacci_recursive;

// Calculate Fibonacci numbers (for indices up to 93)
let fib_10 = memoized_slow_fibonacci_recursive(10); // Returns 55

// For larger numbers, use BigInt versions
use zeck::fast_doubling_fibonacci_biguint;
let fib_100 = fast_doubling_fibonacci_biguint(100);

// For even better performance with caching, use memoized fast doubling
use zeck::memoized_fast_doubling_fibonacci_biguint;
let fib_1000 = memoized_fast_doubling_fibonacci_biguint(1000);
```

### Zeckendorf Representation

```rust
use zeck::memoized_zeckendorf_list_descending_for_integer;

// Get Zeckendorf representation as a list of Fibonacci indices
let zld = memoized_zeckendorf_list_descending_for_integer(12);
// Returns [6, 4, 2] meaning F(6) + F(4) + F(2) = 8 + 3 + 1 = 12

// For BigInt numbers
use zeck::memoized_zeckendorf_list_descending_for_biguint;
use num_bigint::BigUint;
let zld = memoized_zeckendorf_list_descending_for_biguint(&BigUint::from(12u64));
```

### Utility Functions

The library provides various utility functions for working with Fibonacci numbers and Zeckendorf representations:

```rust
use zeck::{
    bit_count_for_number,           // Count bits needed to represent a number
    highest_one_bit,                 // Get the highest set bit
    efi_to_fi, fi_to_efi,            // Convert between Effective Fibonacci Index and Fibonacci Index
    memoized_effective_fibonacci,     // Get Fibonacci number from Effective Fibonacci Index
    zl_to_ezl, ezl_to_zl,            // Convert between Zeckendorf List and Effective Zeckendorf List
    all_ones_zeckendorf_to_biguint,   // Create "all ones" Zeckendorf numbers
    PHI, PHI_SQUARED,                 // Golden ratio constants
};
```

### Error Handling

The file format functions return `Result` types with comprehensive error handling:

```rust
use zeck::zeck_file_format::{compress::compress_zeck_be, error::ZeckFormatError};

match compress_zeck_be(&data) {
    Ok(zeck_file) => {
        // Compression succeeded
    }
    Err(ZeckFormatError::DataSizeTooLarge { size }) => {
        // Data size exceeds u64::MAX
    }
    Err(e) => {
        // Handle other errors
        eprintln!("Compression error: {}", e);
    }
}
```

Common error types include:
- `HeaderTooShort`: Input data is too short to contain a valid header
- `UnsupportedVersion`: File format version is not supported
- `ReservedFlagsSet`: Reserved flags are set (indicating a newer format)
- `CompressionFailed`: Compression did not reduce the data size
- `DecompressedTooLarge`: Decompressed data is larger than expected
- `DataSizeTooLarge`: Data size exceeds the maximum representable size

## Binaries

The project includes several utility binaries. The command-line compression tools (`zeck-compress` and `zeck-decompress`) can be installed globally via:

### Install from crates.io

```bash
cargo install zeck --features cli_tools
```

### Install from GitHub (development version)

```bash
cargo install --git https://github.com/pRizz/zeckendorf --features cli_tools zeck
```

After installation, you can use `zeck-compress` and `zeck-decompress` directly from your command line.

### Compression/Decompression Tools

**⚠️ Warning:** Compressing or decompressing files larger than 10KB (10,000 bytes) is unstable due to time and memory pressure. The library may experience performance issues, excessive memory usage, or failures when processing files exceeding this size.

#### zeck-compress

Compresses data using the Zeckendorf representation algorithm. Automatically adds `.zeck` extension for compressed files.

```bash
zeck-compress [INPUT] [-o OUTPUT] [--endian ENDIAN] [-v]
```

**Options:**
- `INPUT`: Input file path (optional, reads from stdin if not specified)
  - Shows a warning if reading from stdin and no data was piped in
- `-o, --output FILE`: Output file path (optional)
  - If not specified and input is a file, uses the input filename with the `.zeck` extension appended
  - If not specified and reading from stdin, writes to stdout
  - The `.zeck` extension is automatically added unless the file already ends with `.zeck`
- `--endian ENDIAN`: Endianness to use (`big`, `little`, or `best`). Default: `best`
  - `big`: Use big-endian interpretation
  - `little`: Use little-endian interpretation
  - `best`: Try both and use the best result (default)
  - **Note:** When using `best`, if neither method produces compression (both result in larger or equal output), the tool will exit with an error showing compression statistics
- `-v, --verbose`: Show compression statistics (default: true, use `--no-verbose` to disable)

**Examples:**
```bash
# Compress a file (output filename automatically created from input with extension)
zeck-compress input.bin
# Creates input.bin.zeck

# Compress with best endianness (statistics shown by default)
zeck-compress input.bin --endian best

# Compress with specific endianness
zeck-compress input.bin --endian big

# Compress to a specific output file
zeck-compress input.bin -o output
# Creates output.zeck

# Compress from stdin to stdout
cat input.bin | zeck-compress
```

**Note:** When writing to a file, the output filename is printed to stdout (e.g., "Compressed to: input.bin.zeck"). Verbose statistics are shown by default and include descriptive messages about compression ratios (e.g., "File was compressed by X.XX% (Y bytes -> Z bytes)"). A warning is shown when reading from stdin if no data was piped in.

#### zeck-decompress

Decompresses data that was compressed using the Zeckendorf representation algorithm. Automatically detects endianness from the file header.

```bash
zeck-decompress [INPUT] [-o OUTPUT] [-v]
```

**Options:**
- `INPUT`: Input file path (optional, reads from stdin if not specified)
  - When reading from a file, endianness is automatically detected from the file header
  - When reading from stdin, endianness is automatically detected from the file header
  - Shows a warning if reading from stdin and no data was piped in
- `-o, --output FILE`: Output file path (optional)
  - If not specified and input is a file, uses the input filename with `.zeck` extension removed
  - If not specified and reading from stdin, writes to stdout
- `-v, --verbose`: Show decompression statistics (default: true, use `--no-verbose` to disable)

**Examples:**
```bash
# Decompress a file (endianness detected from file header, output filename automatically created)
zeck-decompress input.zeck
# Automatically detects endianness from header, creates output file "input"

# Decompress to a specific output file
zeck-decompress input.zeck -o output.bin
# Automatically detects endianness from header

# Decompress from stdin to stdout
cat input.zeck | zeck-decompress
# Automatically detects endianness from header
```

**Note:** The endianness used for decompression must match the endianness used during compression. The file header stores which endianness was used, so decompression will automatically use the correct endianness when reading from a file or from stdin.

**Additional features:**
- When writing to a file, the output filename is printed to stdout (e.g., "Compressed to: input.bin.zeck" or "Decompressed to: output.bin")
- Verbose statistics are shown by default (use `--no-verbose` to disable) and include descriptive messages about compression/decompression ratios
- Compression will exit with an error if the data cannot be compressed (when using `--endian best` and neither method produces compression)
- A warning is shown when reading from stdin if no data was piped in

### Main Playground

```bash
cargo run --release --example playground
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
cargo run --release --example generate-statistics
```

Generates comprehensive compression statistics and plots:
- Compression ratios across different input sizes
- Chance of compression being favorable
- Average and median compression ratios
- Statistics saved to `statistics_history/` directory
- Plots saved to `plots/` directory

### Plot Compression Ratios

```bash
cargo run --release --example plot
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
- **Memoized Fast Doubling**: Uses sparse HashMap caching for efficient memory usage with large, non-contiguous Fibonacci indices
- **Memoization**: Thread-safe caching significantly improves repeated calculations. The trade-off is that the cache takes up memory.
- **Compression Effectiveness**: Varies by input; compression ratios oscillate and become less favorable as input size increases

## Algorithm Details

### Zeckendorf Representation

Every positive integer can be uniquely represented as a sum of non-consecutive Fibonacci numbers. For example:
- 12 = 8 + 3 + 1 = F(6) + F(4) + F(2)

### Compression Process

1. Input data is interpreted as either a big-endian or little-endian integer (you can choose, or use `compress_zeck_best` to try both)
2. The integer is converted to its Zeckendorf representation (list of Fibonacci indices)
3. The representation is encoded as bits (use/skip bits)
4. Bits are packed into bytes (little-endian output)

The library provides functions to compress with either interpretation, or you can use `compress_zeck_best` to automatically try both and select the one that produces the smallest output.

### File Format

The `.zeck` file format includes a 10-byte header:
- **Version** (1 byte): File format version (currently 1)
- **Original Size** (8 bytes): Original uncompressed file size in bytes (little-endian)
- **Flags** (1 byte): Endianness and reserved flags
  - Bit 0: Big endian flag (1 = big endian, 0 = little endian)
  - Bits 1-7: Reserved for future use

The header is followed by the compressed data. This format automatically preserves the original file size, allowing proper restoration of leading or trailing zero bytes during decompression.

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
- Padless compression functions (`*_dangerous`) do not preserve leading/trailing zero bytes—use the file format functions for automatic size preservation

## NPM Versioning Quirk

For some reason, NPM was showing there were versions of zeck published between `1.0.0` and `1.0.6` from 2024 (we are in 2026), even though I never published them to npm. I don't know how this happened. So I bumped the version to `1.0.7` and was able to successfully publish it to npm. Maybe there was an old package with the same name that was deleted, and NPM is still showing the old versions.

Here is a snippet of the `time` object from the npm registry JSON (https://registry.npmjs.org/zeck):

```json
  "time": {
    "created": "2026-01-02T20:19:14.018Z",
    "modified": "2026-01-03T17:25:15.940Z",
    "1.0.0": "2024-02-21T14:36:36.292Z",
    "1.0.1": "2024-02-21T15:26:38.621Z",
    "1.0.2": "2024-02-21T15:36:30.258Z",
    "1.0.3": "2024-02-21T15:48:07.853Z",
    "1.0.4": "2024-02-21T15:48:38.804Z",
    "1.0.5": "2024-02-21T16:02:36.339Z",
    "1.0.6": "2024-02-21T16:36:36.643Z",
    "0.1.0": "2026-01-02T20:19:14.175Z",
    "0.2.0": "2026-01-03T17:25:15.702Z"
  },
```

## License

This project is licensed under the MIT License - see the [LICENSE.txt](LICENSE.txt) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## References

- [Fast Fibonacci Algorithms](https://www.nayuki.io/page/fast-fibonacci-algorithms) - Fast doubling algorithm reference
- [Zeckendorf's Theorem](https://en.wikipedia.org/wiki/Zeckendorf%27s_theorem) - Every positive integer has a unique representation as a sum of non-consecutive Fibonacci numbers
- [Exploring Fibonacci Based Compression](https://medium.com/@peterryszkiewicz/exploring-fibonacci-based-compression-8713770f5598) - My blog post about the Zeckendorf representation algorithm and this library
