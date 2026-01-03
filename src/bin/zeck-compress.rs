//! Zeckendorf compression CLI tool
//!
//! Compresses data using the Zeckendorf representation algorithm.
//! Automatically uses `.zbe` extension for big-endian compression and `.zle` for little-endian compression.
//!
//! Building and running the tool:
//! `cargo build --release --bin zeck-compress`
//! `cargo run --release --bin zeck-compress`
//!
//! # Examples
//!
//! Compress a file (output filename automatically created from input with extension):
//! ```bash
//! zeck-compress input.bin
//! # Creates input.bin.zbe or input.bin.zle depending on which endianness was used
//! ```
//!
//! Compress from stdin to stdout:
//! ```bash
//! cat input.bin | zeck-compress
//! ```
//!
//! Compress with specific endianness:
//! ```bash
//! zeck-compress input.bin --endian big
//! # Creates input.bin.zbe
//! ```

// Include the generated version string from the build.rs script
include!(concat!(env!("OUT_DIR"), "/version_string.rs"));

use clap::Parser;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use zeck::{
    CompressionResult, zeckendorf_compress_be, zeckendorf_compress_best, zeckendorf_compress_le,
};

#[derive(Debug, Clone, Copy)]
enum EndianUsed {
    Big,
    Little,
}

impl EndianUsed {
    fn extension(&self) -> &'static str {
        match self {
            EndianUsed::Big => ".zbe",
            EndianUsed::Little => ".zle",
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            EndianUsed::Big => "big endian",
            EndianUsed::Little => "little endian",
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "zeck-compress",
    version = VERSION_STRING,
    about = "Compress data using the Zeckendorf representation algorithm",
    long_about = None
)]
struct Args {
    /// Input file path. If not specified, reads from stdin.
    #[arg(value_name = "INPUT")]
    maybe_input: Option<String>,

    /// Output file path. If not specified and input is a file, uses the input filename with the appropriate extension (.zbe or .zle) appended.
    /// If not specified and reading from stdin, writes to stdout.
    /// The appropriate extension (.zbe for big-endian, .zle for little-endian) is automatically added
    /// unless the file already ends with .zbe or .zle.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    maybe_output: Option<String>,

    /// Endianness to use for compression.
    /// - "big": Use big endian interpretation
    /// - "little": Use little endian interpretation
    /// - "best": Try both and use the best result (default)
    #[arg(
        short = 'e',
        long = "endian",
        value_name = "ENDIAN",
        default_value = "best"
    )]
    endian: String,

    /// Show compression statistics (default: true)
    #[arg(short, long, default_value_t = true)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    // Read input data
    let input_data = if let Some(input_path) = &args.maybe_input {
        match fs::read(input_path) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Error: Failed to read input file '{}': {}", input_path, err);
                std::process::exit(1);
            }
        }
    } else {
        // Check if stdin is a TTY (terminal) - if so, no data was piped in
        if io::stdin().is_terminal() {
            eprintln!(
                "Warning: Reading from stdin, but no data was piped in. Waiting for input..."
            );
            eprintln!("Hint: Pipe data using: cat file.bin | zeck-compress");
        }
        let mut data = Vec::new();
        match io::stdin().read_to_end(&mut data) {
            Ok(_) => data,
            Err(err) => {
                eprintln!("Error: Failed to read from stdin: {}", err);
                std::process::exit(1);
            }
        }
    };

    if input_data.is_empty() {
        eprintln!("Error: Input data is empty");
        std::process::exit(1);
    }

    let original_size = input_data.len();

    // Compress data based on endianness option
    let (compressed_data, endian_used, be_size, le_size) = match args.endian.to_lowercase().as_str()
    {
        "big" => {
            let compressed = zeckendorf_compress_be(&input_data);
            (compressed, EndianUsed::Big, original_size, original_size)
        }
        "little" => {
            let compressed = zeckendorf_compress_le(&input_data);
            (compressed, EndianUsed::Little, original_size, original_size)
        }
        "best" => {
            let result = zeckendorf_compress_best(&input_data);
            match result {
                CompressionResult::BigEndianBest {
                    compressed_data,
                    le_size,
                } => (compressed_data, EndianUsed::Big, original_size, le_size),
                CompressionResult::LittleEndianBest {
                    compressed_data,
                    be_size,
                } => (compressed_data, EndianUsed::Little, be_size, original_size),
                CompressionResult::Neither { be_size, le_size } => {
                    eprintln!(
                        "Error: Data could not be compressed using Zeckendorf representation."
                    );
                    eprintln!("Original size: {} bytes", original_size);
                    eprintln!("Big-endian compressed size: {} bytes", be_size);
                    eprintln!("Little-endian compressed size: {} bytes", le_size);
                    eprintln!("Both compression methods resulted in larger or equal output sizes.");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!(
                "Error: Invalid endianness '{}'. Must be 'big', 'little', or 'best'",
                args.endian
            );
            std::process::exit(1);
        }
    };

    let compressed_size = compressed_data.len();

    // Determine which endianness was actually used (for file extension)
    let endian_extension = endian_used.extension();

    // Determine output path
    let final_output_path = if let Some(output_path) = &args.maybe_output {
        // If output is explicitly specified, use it (add extension if needed)
        if output_path.ends_with(".zbe") || output_path.ends_with(".zle") {
            output_path.clone()
        } else {
            format!("{}{}", output_path, endian_extension)
        }
    } else if let Some(input_path) = &args.maybe_input {
        // If no output specified but input file exists, use input filename + extension
        format!("{}{}", input_path, endian_extension)
    } else {
        // Reading from stdin, no output file - will write to stdout
        String::new()
    };

    // Write output data
    if final_output_path.is_empty() {
        // Write to stdout
        if let Err(err) = io::stdout().write_all(&compressed_data) {
            eprintln!("Error: Failed to write to stdout: {}", err);
            std::process::exit(1);
        }
    } else {
        // Write to file
        if let Err(err) = fs::write(&final_output_path, &compressed_data) {
            eprintln!(
                "Error: Failed to write output file '{}': {}",
                final_output_path, err
            );
            std::process::exit(1);
        }
        // Output filename to stdout
        println!("Compressed to: {}", final_output_path);
    }

    // Print statistics if verbose
    if args.verbose {
        let compression_ratio = compressed_size as f64 / original_size as f64;
        let compression_percentage = if compression_ratio < 1.0 {
            (1.0 - compression_ratio) * 100.0
        } else {
            (compression_ratio - 1.0) * 100.0
        };

        eprintln!("Endianness used: {}", endian_used.display_name());
        if args.endian == "best" {
            eprintln!("Big endian size: {} bytes", be_size);
            eprintln!("Little endian size: {} bytes", le_size);
        }
        if compression_ratio < 1.0 {
            eprintln!(
                "File was compressed by {:.2}% ({} bytes -> {} bytes)",
                compression_percentage, original_size, compressed_size
            );
        } else {
            eprintln!(
                "File was expanded by {:.2}% ({} bytes -> {} bytes)",
                compression_percentage, original_size, compressed_size
            );
        }
    }
}
