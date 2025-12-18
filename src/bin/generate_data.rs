use rand::RngCore;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

// Example usages:
// Generate a file with default name:
// `cargo run --bin generate_data 1024`
// Generate a file with custom name:
// `cargo run --bin generate_data 1024 my_file.bin`

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <size_in_bytes> [filename]", args[0]);
        eprintln!("  size_in_bytes: The size of the file to generate in bytes");
        eprintln!("  filename: Optional filename (default: random_data_<size>_bytes.bin)");
        std::process::exit(1);
    }

    let size_str = &args[1];
    let size = match size_str.parse::<usize>() {
        Ok(s) => s,
        Err(_) => {
            eprintln!(
                "Error: '{size_str}' is not a valid size. Please provide a positive integer."
            );
            std::process::exit(1);
        }
    };

    if size == 0 {
        eprintln!("Error: Size must be greater than 0");
        std::process::exit(1);
    }

    let maybe_filename = args.get(2);
    let filename = if let Some(name) = maybe_filename {
        name.clone()
    } else {
        format!("random_data_{size}_bytes.bin")
    };

    // Create the generated_data directory if it doesn't exist
    let output_dir = Path::new("generated_data");
    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("Error: Failed to create directory 'generated_data': {e}");
        std::process::exit(1);
    }

    // Generate random data
    let mut rng = rand::rng();
    let mut data = vec![0u8; size];
    rng.fill_bytes(&mut data);

    // Write the file
    let file_path = output_dir.join(&filename);
    let mut file = match fs::File::create(&file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "Error: Failed to create file '{}': {e}",
                file_path.display()
            );
            std::process::exit(1);
        }
    };

    if let Err(e) = file.write_all(&data) {
        eprintln!(
            "Error: Failed to write data to file '{}': {e}",
            file_path.display()
        );
        std::process::exit(1);
    }

    println!(
        "Successfully generated file: {} ({size} bytes)",
        file_path.display()
    );
}
