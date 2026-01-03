//! Zeckendorf decompression CLI tool
//!
//! Decompresses data that was compressed using the Zeckendorf representation algorithm.
//! Automatically detects endianness from file extension (.zbe for big-endian, .zle for little-endian).
//!
//! Building and running the tool:
//! `cargo build --release --bin zeck-decompress`
//! `cargo run --release --bin zeck-decompress`
//!
//! # Examples
//!
//! Decompress a file (endianness detected from .zbe or .zle extension):
//! ```bash
//! zeck-decompress input.zbe -o output.bin
//! # Automatically uses big-endian decompression
//! ```
//!
//! Decompress from stdin to stdout (must specify endianness):
//! ```bash
//! cat input.zbe | zeck-decompress --endian big
//! ```
//!
//! Override automatic endianness detection:
//! ```bash
//! zeck-decompress input.zbe --endian little -o output.bin
//! # Overrides the .zbe extension and uses little-endian
//! ```

use clap::Parser;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use zeck::{zeckendorf_decompress_be, zeckendorf_decompress_le};

#[derive(Parser, Debug)]
#[command(name = "zeck-decompress")]
#[command(about = "Decompress data that was compressed using the Zeckendorf representation algorithm", long_about = None)]
struct Args {
    /// Input file path. If not specified, reads from stdin.
    /// When reading from a file, endianness is automatically detected from file extension (.zbe for big endian, .zle for little endian).
    /// When reading from stdin, --endian must be specified.
    #[arg(value_name = "INPUT")]
    maybe_input: Option<String>,

    /// Output file path. If not specified and input is a file, uses the input filename with .zbe or .zle extension removed.
    /// If not specified and reading from stdin, writes to stdout.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    maybe_output: Option<String>,

    /// Endianness used for compression (must match the compression endianness).
    /// - "big": Decompress as big endian
    /// - "little": Decompress as little endian
    /// If not specified when reading from a file, endianness is automatically detected from the file extension (.zbe or .zle).
    /// This option is REQUIRED when reading from stdin (no input file specified).
    /// This option overrides automatic detection from file extension.
    #[arg(short = 'e', long = "endian", value_name = "ENDIAN")]
    maybe_endian: Option<String>,

    /// Show decompression statistics (default: true)
    #[arg(short, long, default_value_t = true)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    // Determine endianness: use --endian flag if provided, otherwise detect from file extension
    // If reading from stdin, --endian is required
    let endian_to_use = if let Some(endian) = &args.maybe_endian {
        endian.clone()
    } else if let Some(input_path) = &args.maybe_input {
        // Detect from file extension
        if input_path.ends_with(".zbe") {
            "big".to_string()
        } else if input_path.ends_with(".zle") {
            "little".to_string()
        } else {
            // Extension not recognized - require explicit --endian flag
            eprintln!(
                "Error: Input file '{}' does not have a recognized extension (.zbe or .zle)",
                input_path
            );
            eprintln!(
                "Please specify --endian <big|little> to indicate the endianness used during compression."
            );
            eprintln!("Usage: zeck-decompress [INPUT] --endian <big|little> [OPTIONS]");
            std::process::exit(1);
        }
    } else {
        // Reading from stdin, --endian is required
        eprintln!("Error: --endian must be specified when reading from stdin");
        eprintln!("Usage: zeck-decompress --endian <big|little> [OPTIONS]");
        eprintln!("Example: cat input.zbe | zeck-decompress --endian big");
        std::process::exit(1);
    };

    // Read input data
    let compressed_data = if let Some(input_path) = &args.maybe_input {
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
            eprintln!("Hint: Pipe data using: cat file.zbe | zeck-decompress --endian big");
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

    if compressed_data.is_empty() {
        eprintln!("Error: Input data is empty");
        std::process::exit(1);
    }

    let compressed_size = compressed_data.len();

    // Decompress data based on endianness
    let decompressed_data = match endian_to_use.to_lowercase().as_str() {
        "big" => zeckendorf_decompress_be(&compressed_data),
        "little" => zeckendorf_decompress_le(&compressed_data),
        _ => {
            eprintln!(
                "Error: Invalid endianness '{}'. Must be 'big' or 'little'",
                endian_to_use
            );
            std::process::exit(1);
        }
    };

    let decompressed_size = decompressed_data.len();

    // Determine output path
    let final_output_path = if let Some(output_path) = &args.maybe_output {
        // Use explicitly specified output path
        output_path.clone()
    } else if let Some(input_path) = &args.maybe_input {
        // Remove .zbe or .zle extension from input filename
        if input_path.ends_with(".zbe") {
            input_path
                .strip_suffix(".zbe")
                .unwrap_or(input_path)
                .to_string()
        } else if input_path.ends_with(".zle") {
            input_path
                .strip_suffix(".zle")
                .unwrap_or(input_path)
                .to_string()
        } else {
            // Extension not recognized - require explicit --endian flag
            eprintln!(
                "Error: Input file '{}' does not have a recognized extension (.zbe or .zle)",
                input_path
            );
            eprintln!(
                "Please specify --endian <big|little> to indicate the endianness used during compression."
            );
            eprintln!("Usage: zeck-decompress [INPUT] --endian <big|little> [OPTIONS]");
            std::process::exit(1);
        }
    } else {
        // Reading from stdin, no output file - will write to stdout
        String::new()
    };

    // Write output data
    if final_output_path.is_empty() {
        // Write to stdout
        if let Err(err) = io::stdout().write_all(&decompressed_data) {
            eprintln!("Error: Failed to write to stdout: {}", err);
            std::process::exit(1);
        }
    } else {
        // Write to file
        if let Err(err) = fs::write(&final_output_path, &decompressed_data) {
            eprintln!(
                "Error: Failed to write output file '{}': {}",
                final_output_path, err
            );
            std::process::exit(1);
        }
        // Output filename to stdout
        println!("Decompressed to: {}", final_output_path);
    }

    // Print statistics if verbose
    if args.verbose {
        let expansion_ratio = decompressed_size as f64 / compressed_size as f64;
        let expansion_percentage = (expansion_ratio - 1.0) * 100.0;

        eprintln!("Endianness used: {}", endian_to_use);
        if decompressed_size < compressed_size {
            // File got smaller during decompression
            let shrink_percentage = (1.0 - expansion_ratio) * 100.0;
            eprintln!(
                "File was decompressed but shrunk ({} bytes -> {} bytes, shrunk by {:.2}%)",
                compressed_size, decompressed_size, shrink_percentage
            );
        } else {
            // File got larger or stayed the same
            eprintln!(
                "File was decompressed ({} bytes -> {} bytes, expanded by {:.2}%)",
                compressed_size, decompressed_size, expansion_percentage
            );
        }
    }
}
