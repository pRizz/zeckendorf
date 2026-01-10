//! Zeckendorf decompression CLI tool
//!
//! Decompresses data that was compressed using the Zeckendorf representation algorithm.
//! Automatically detects endianness from the file header.
//!
//! Building and running the tool:
//! `cargo build --release --bin zeck-decompress --features cli_tools`
//! `cargo run --release --bin zeck-decompress --features cli_tools`
//!
//! # Examples
//!
//! Decompress a file (endianness detected from file header):
//! ```bash
//! zeck-decompress input.zeck -o output.bin
//! # Automatically detects endianness from header
//! ```
//!
//! Decompress from stdin to stdout:
//! ```bash
//! cat input.zeck | zeck-decompress
//! ```

// Include the generated version string from the build.rs script
include!(concat!(env!("OUT_DIR"), "/version_string.rs"));

use clap::Parser;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use zeck::zeck_file_format::decompress::decompress_zeck_file;
use zeck::zeck_file_format::file::deserialize_zeck_file;

#[derive(Parser, Debug)]
#[command(
    name = "zeck-decompress",
    version = VERSION_STRING,
    about = "Decompress data that was compressed using the Zeckendorf representation algorithm",
    long_about = None
)]
struct Args {
    /// Input file path. If not specified, reads from stdin.
    /// The .zeck file format includes header information, so endianness is automatically detected.
    #[arg(value_name = "INPUT")]
    maybe_input: Option<String>,

    /// Output file path. If not specified and input is a file, uses the input filename with `.zeck` extension removed.
    /// If not specified and reading from stdin, writes to stdout.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    maybe_output: Option<String>,

    /// Show decompression statistics (default: true)
    #[arg(short, long, default_value_t = true)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    // Read input data
    let zeck_file_data = if let Some(input_path) = &args.maybe_input {
        match fs::read(input_path) {
            Ok(zeck_file_data) => zeck_file_data,
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
            eprintln!("Hint: Pipe data using: cat file.zeck | zeck-decompress");
        }
        let mut zeck_file_data = Vec::new();
        match io::stdin().read_to_end(&mut zeck_file_data) {
            Ok(_) => zeck_file_data,
            Err(err) => {
                eprintln!("Error: Failed to read from stdin: {}", err);
                std::process::exit(1);
            }
        }
    };

    if zeck_file_data.is_empty() {
        eprintln!("Error: Input data is empty");
        std::process::exit(1);
    }

    // Deserialize and decompress data (endianness is automatically detected from header)
    let zeck_file = match deserialize_zeck_file(&zeck_file_data) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: Failed to deserialize .zeck file: {}", e);
            std::process::exit(1);
        }
    };
    let decompressed_data = match decompress_zeck_file(&zeck_file) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error: Decompression failed: {}", e);
            std::process::exit(1);
        }
    };

    let compressed_size = zeck_file.compressed_data.len();
    let total_size = zeck_file.total_size();
    let decompressed_size = decompressed_data.len();

    // Determine output path
    let final_output_path = if let Some(output_path) = &args.maybe_output {
        // Use explicitly specified output path
        output_path.clone()
    } else if let Some(input_path) = &args.maybe_input {
        // Remove .zeck extension from input filename
        if input_path.ends_with(".zeck") {
            input_path
                .strip_suffix(".zeck")
                .unwrap_or(input_path)
                .to_string()
        } else {
            eprintln!(
                "Error: Input file extension did not end with '.zeck'. An output file path must be specified or the input file must have a '.zeck' extension."
            );
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

        let endianness_used = if zeck_file.is_big_endian() {
            "big endian"
        } else {
            "little endian"
        };
        eprintln!("Endianness used: {}", endianness_used);
        if decompressed_size < compressed_size {
            // File got smaller during decompression
            let shrink_percentage = (1.0 - expansion_ratio) * 100.0;
            eprintln!(
                "File was decompressed but shrunk (original content size: {compressed_size} bytes -> decompressed content size: {decompressed_size} bytes, shrunk by {shrink_percentage:.2}%)\nTotal file size with header: {total_size} bytes",
            );
        } else {
            // File got larger or stayed the same
            eprintln!(
                "File was decompressed (original content size: {compressed_size} bytes -> decompressed content size: {decompressed_size} bytes, expanded by {expansion_percentage:.2}%)\nTotal file size with header: {total_size} bytes",
            );
        }
    }
}
