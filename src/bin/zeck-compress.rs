//! Zeckendorf compression CLI tool
//!
//! Compresses data using the Zeckendorf representation algorithm.
//! Automatically uses `.zeck` extension for compressed files.
//!
//! Building and running the tool:
//! `cargo build --release --bin zeck-compress --features cli_tools`
//! `cargo run --release --bin zeck-compress --features cli_tools`
//!
//! # Examples
//!
//! Compress a file (output filename automatically created from input with extension):
//! ```bash
//! zeck-compress input.bin
//! # Creates input.bin.zeck
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
//! # Creates input.bin.zeck
//! ```

// Include the generated version string from the build.rs script
include!(concat!(env!("OUT_DIR"), "/version_string.rs"));

use clap::Parser;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use zeck::zeck_file_format::compress::BestCompressionResult;
use zeck::zeck_file_format::{
    compress::compress_zeck_be, compress::compress_zeck_best, compress::compress_zeck_le,
};

#[derive(Debug, Clone, Copy)]
enum EndianUsed {
    Big,
    Little,
}

impl EndianUsed {
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

    /// Output file path. If not specified and input is a file, uses the input filename with the `.zeck` extension appended.
    /// If not specified and reading from stdin, writes to stdout.
    /// The `.zeck` extension is automatically added unless the file already ends with `.zeck`.
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
    let (zeck_file, maybe_be_size, maybe_le_size) = match args.endian.to_lowercase().as_str() {
        "big" => {
            let zeck_file = match compress_zeck_be(&input_data) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error: Compression failed: {}", e);
                    std::process::exit(1);
                }
            };
            let be_size = zeck_file.compressed_data.len();
            (zeck_file, Some(be_size), None)
        }
        "little" => {
            let zeck_file = match compress_zeck_le(&input_data) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error: Compression failed: {}", e);
                    std::process::exit(1);
                }
            };
            let le_size = zeck_file.compressed_data.len();
            (zeck_file, None, Some(le_size))
        }
        "best" => {
            let best_compression_result = match compress_zeck_best(&input_data) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Error: Best compression failed: {}", e);
                    std::process::exit(1);
                }
            };
            match best_compression_result {
                BestCompressionResult::BigEndianBest { zeck_file, le_size } => {
                    let be_size = zeck_file.compressed_data.len();
                    (zeck_file, Some(be_size), Some(le_size))
                }
                BestCompressionResult::LittleEndianBest { zeck_file, be_size } => {
                    let le_size = zeck_file.compressed_data.len();
                    (zeck_file, Some(be_size), Some(le_size))
                }
                BestCompressionResult::Neither { be_size, le_size } => {
                    eprintln!(
                        "Error: Neither compression method produced a smaller output than the original. Big endian size: {} bytes, Little endian size: {} bytes",
                        be_size, le_size
                    );
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!(
                "Error: Invalid endianness '{}'. Must be 'big', 'little', or 'best'.",
                args.endian
            );
            std::process::exit(1);
        }
    };

    // Determine endianness from the zeck_file
    let endian_used = if zeck_file.is_big_endian() {
        EndianUsed::Big
    } else {
        EndianUsed::Little
    };

    let zeck_file_as_data = zeck_file.to_bytes();
    let compressed_data_size = zeck_file.compressed_data.len();
    let total_size = zeck_file.total_size();

    // Use .zeck extension for the new format
    let file_extension = ".zeck";

    // Determine output path
    let final_output_path = if let Some(output_path) = &args.maybe_output {
        // If output is explicitly specified, use it (add extension if needed)
        if output_path.ends_with(".zeck") {
            output_path.clone()
        } else {
            format!("{output_path}{file_extension}")
        }
    } else if let Some(input_path) = &args.maybe_input {
        // If no output specified but input file exists, use input filename + extension
        format!("{input_path}{file_extension}")
    } else {
        // Reading from stdin, no output file - will write to stdout
        String::new()
    };

    // Write output data
    if final_output_path.is_empty() {
        // Write to stdout
        if let Err(err) = io::stdout().write_all(&zeck_file_as_data) {
            eprintln!("Error: Failed to write to stdout: {}", err);
            std::process::exit(1);
        }
    } else {
        // Write to file
        if let Err(err) = fs::write(&final_output_path, &zeck_file_as_data) {
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
        let compression_ratio = compressed_data_size as f64 / original_size as f64;
        let compression_percentage = if compression_ratio < 1.0 {
            (1.0 - compression_ratio) * 100.0
        } else {
            (compression_ratio - 1.0) * 100.0
        };

        eprintln!("Endianness used: {}", endian_used.display_name());
        if let (Some(be_size), Some(le_size)) = (maybe_be_size, maybe_le_size) {
            eprintln!("Big endian size: {} bytes", be_size);
            eprintln!("Little endian size: {} bytes", le_size);
        }
        eprintln!("Original size: {} bytes", original_size);
        eprintln!("Compressed data size: {} bytes", compressed_data_size);
        eprintln!("Total file size (with header): {} bytes", total_size);
        if compression_ratio < 1.0 {
            eprintln!(
                "File content was compressed by {compression_percentage:.2}% (original content size: {original_size} bytes -> compressed content size: {compressed_data_size} bytes)\nTotal file size with header: {total_size} bytes",
            );
        } else {
            eprintln!(
                "File content was expanded by {compression_percentage:.2}% (original content size: {original_size} bytes -> expanded content size: {compressed_data_size} bytes)\nTotal file size with header: {total_size} bytes",
            );
        }
    }
}
