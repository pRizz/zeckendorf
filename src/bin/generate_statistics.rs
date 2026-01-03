//! Binary for generating statistics about the compression ratio of the Zeckendorf representation
//!
//! The statistics are saved in the statistics_history directory in a file named statistics_up_to_<limit>_inputs.csv and sampled_statistics_up_to_<limit>_bits.csv
//!
//! The purpose of this binary is to determine the average compression ratio, median compression ratio, best compression ratio, and chance of compression being favorable for a given limit. As we compress to higher limits, the statistics should become more stable.
//!
//! The Zeckendorf compression oscillates between being favorable and unfavorable, as the data changes, and the statistics are used to determine the average and median compression ratios, and the chance of compression being favorable. See this crate's `plot` binary for more details about the oscillation and to visualize the compression ratios.
//!
//! The meaning of "compression up to input" in the csv header is such that the statistics are gathered for all inputs up to and including the given limit. For example, "compression up to 100" means that the corresponding statistics in that row in the csv are gathered for all inputs from 1 to 100.
//!
//! Run with: `cargo run --release --bin zeck-generate-statistics --features plotting,development_tools`

use num_format::{Locale, ToFormattedString};
use plotters::prelude::*;

use num_bigint::BigUint;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::{cmp::Ordering, fs, path::Path, time::Instant};
use zeck::zeckendorf_compress_be;

const AXIS_FONT_SIZE: u32 = 100;
const AXIS_TICK_FONT_SIZE: u32 = 64;
const CAPTION_FONT_SIZE: u32 = 160;
const LEGEND_FONT_SIZE: u32 = 70;
const CHART_MARGIN: u32 = 120;
const PLOT_WIDTH: u32 = 3840;
const PLOT_HEIGHT: u32 = 2160;
const LEGEND_MARGIN: u32 = 50;
const SERIES_LINE_STROKE_WIDTH: u32 = 3;
const SERIES_LINE_DOT_SIZE: u32 = 5;
const LEGEND_PATH_LEFT_OFFSET: i32 = 30;
const LEGEND_PATH_RIGHT_OFFSET: i32 = 10;

// Time taken to generate bit limit statistics: 111.330666ms
const INPUT_LIMITS: [u64; 5] = [10, 100, 1_000, 10_000, 100_000];

// Time taken to generate bit limit statistics: 861.613791ms
// const INPUT_LIMITS: [u64; 6] = [10, 100, 1_000, 10_000, 100_000, 1_000_000];

// Time taken to generate bit limit statistics: 110.184340667s; 110 / 100000000 ~ 1.1 microseconds per input on average.
// const INPUT_LIMITS: [u64; 8] = [10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000];

// Sampled statistics configuration
// The odds of compression being favorable starts to decrease significantly at around 1000 bits.
// I added more samplings around 1000 bits to get a more accurate picture of the decrease.
// At 1500 bits, the odds of compression being favorable is around 100 / 250,000 samples or 0.04%.
// At and after 1600 bits, there were no favorable samples out of 250,000 samples.
// Time taken to generate sampled statistics: 95.059394875s
const BIT_SIZE_LIMITS: [u64; 21] = [
    20, 50, 100, 200, 500, 600, 700, 800, 900, 1_000, 1_100, 1_200, 1_300, 1_400, 1_500, 1_600,
    1_700, 1_800, 1_900, 2_000, 5_000,
];
const SAMPLES_PER_BIT_SIZE: u64 = 250_000;

// Wide-scale sampled statistics configuration
// Samples at sparser bit size limits with a much higher ceiling: 1e3, 1e4, 1e5
const WIDE_SCALE_BIT_SIZE_LIMITS: [u64; 3] = [1_000, 10_000, 100_000];
// ⚠️ process gets killed after sampling at 1 Mbit, with out of memory error.
// const WIDE_SCALE_BIT_SIZE_LIMITS: [u64; 5] = [1_000, 10_000, 100_000, 1_000_000, 10_000_000];
const WIDE_SCALE_STARTING_SAMPLES_PER_BIT_SIZE: u64 = 100_000;
const WIDE_SCALE_SAMPLE_REDUCTION_FACTOR: f64 = 0.15;

// Seed for the random number generator to ensure reproducible results
const RNG_SEED: u64 = 42;

#[derive(Debug, Clone)]
struct CompressionStats {
    limit: u64,
    favorable_pct: f64,
    average_pct: f64,
    median_pct: f64,
    maybe_best_compressed_input: Option<u64>,
    maybe_best_compression_amount: Option<f64>,
    maybe_average_favorable_pct: Option<f64>,
    maybe_median_favorable_pct: Option<f64>,
}

fn main() {
    let start_time = Instant::now();

    // debug_2000_bits_case();

    generate_bit_limit_stats();
    generate_sampled_bit_limit_stats();
    generate_wide_scale_sampled_bit_limit_stats();

    let end_time = Instant::now();
    println!(
        "Time taken to generate statistics for input limits {:?} and sampled limits {:?}: {:?}",
        INPUT_LIMITS,
        BIT_SIZE_LIMITS,
        end_time.duration_since(start_time)
    );
}

fn generate_stats_csv(stats: &[CompressionStats], csv_header: &str) -> String {
    let mut output = String::new();
    output.push_str(csv_header);
    for stat in stats {
        let line = format!(
            "{},{:.6},{:.6},{:.6},{:.6},{},{:.6},{:.6}",
            stat.limit,
            stat.favorable_pct,
            stat.average_pct,
            stat.median_pct,
            stat.maybe_best_compression_amount
                .map_or("None".to_string(), |f| f.to_string()),
            stat.maybe_best_compressed_input
                .map(|input| input.to_string())
                .unwrap_or_else(|| "".to_string()),
            stat.maybe_average_favorable_pct
                .map_or("None".to_string(), |f| f.to_string()),
            stat.maybe_median_favorable_pct
                .map_or("None".to_string(), |f| f.to_string())
        );
        println!("{}", line);
        output.push_str(&line);
        output.push_str("\n");
    }
    output
}

fn write_stats_csv(csv_content: &str, file_name_without_extension: &str) {
    let statistics_directory = Path::new("statistics_history");
    if let Err(e) = fs::create_dir_all(statistics_directory) {
        eprintln!("Error: Failed to create directory 'statistics_history': {e}");
        std::process::exit(1);
    }
    let statistics_file_name = format!("{file_name_without_extension}.csv");
    println!(
        "Writing statistics to '{}'",
        statistics_directory.join(&statistics_file_name).display()
    );
    fs::write(
        statistics_directory.join(&statistics_file_name),
        csv_content,
    )
    .expect("Failed to write statistics to file");
}

fn generate_bit_limit_stats() {
    let start_time = Instant::now();
    println!("\n=== Generating bit limit statistics ===");
    let csv_header = "compression up to input,chance of compression being favorable,average compression ratio,median compression ratio,best compression ratio,best compression input,average favorable compression ratio,median favorable compression ratio\n";

    let all_stats = INPUT_LIMITS
        .iter()
        .map(|&limit| gather_stats_for_limit(limit))
        .collect::<Vec<CompressionStats>>();
    let statistics_file_name = format!("statistics_up_to_{}_inputs", INPUT_LIMITS.last().unwrap());
    let csv_content = generate_stats_csv(&all_stats, csv_header);
    write_stats_csv(&csv_content, &statistics_file_name);

    let plot_filename_ratios = format!(
        "plots/compression_ratios_up_to_{}_inputs.png",
        INPUT_LIMITS.last().unwrap()
    );
    if let Err(e) = plot_compression_ratios(&plot_filename_ratios, &all_stats) {
        eprintln!("Error: Failed to plot compression ratios: {e}");
    }

    let plot_filename_favorable = format!(
        "plots/favorable_percentages_up_to_{}_inputs.png",
        INPUT_LIMITS.last().unwrap()
    );
    if let Err(e) = plot_favorable_percentages(&plot_filename_favorable, &all_stats) {
        eprintln!("Error: Failed to plot favorable percentages: {e}");
    }
    let end_time = Instant::now();
    println!(
        "Time taken to generate bit limit statistics: {:?}",
        end_time.duration_since(start_time)
    );
}

fn generate_sampled_bit_limit_stats() {
    let csv_header = "max bit size,chance of compression being favorable,average compression ratio,median compression ratio,best compression ratio,best compression input,average favorable compression ratio,median favorable compression ratio\n";

    println!("\n=== Generating sampled statistics ===");
    let sampled_start_time = Instant::now();
    let sampled_stats = BIT_SIZE_LIMITS
        .iter()
        .map(|&bit_size_limit| gather_sampled_stats(bit_size_limit, SAMPLES_PER_BIT_SIZE))
        .collect::<Vec<CompressionStats>>();
    let csv_content = generate_stats_csv(&sampled_stats, csv_header);
    let sampled_statistics_file_name = format!(
        "sampled_statistics_up_to_{}_bits",
        BIT_SIZE_LIMITS.last().unwrap()
    );
    write_stats_csv(&csv_content, &sampled_statistics_file_name);
    let sampled_end_time = Instant::now();
    println!(
        "Time taken to generate sampled statistics: {:?}",
        sampled_end_time.duration_since(sampled_start_time)
    );

    let plot_filename_ratios = format!(
        "plots/compression_ratios_sampled_up_to_{}_bits.png",
        BIT_SIZE_LIMITS.last().unwrap()
    );
    if let Err(e) = plot_sampled_compression_ratios(&plot_filename_ratios, &sampled_stats) {
        eprintln!("Error: Failed to plot sampled compression ratios: {e}");
    }

    let plot_filename_favorable = format!(
        "plots/favorable_percentages_sampled_up_to_{}_bits.png",
        BIT_SIZE_LIMITS.last().unwrap()
    );
    if let Err(e) = plot_sampled_favorable_percentages(&plot_filename_favorable, &sampled_stats) {
        eprintln!("Error: Failed to plot sampled favorable percentages: {e}");
    }
}

fn generate_wide_scale_sampled_bit_limit_stats() {
    let csv_header = "max bit size,chance of compression being favorable,average compression ratio,median compression ratio,best compression ratio,best compression input,average favorable compression ratio,median favorable compression ratio\n";

    println!("\n=== Generating wide-scale sampled statistics ===");
    let wide_scale_start_time = Instant::now();
    let wide_scale_stats = WIDE_SCALE_BIT_SIZE_LIMITS
        .iter()
        .enumerate()
        .map(|(index, &bit_size_limit)| {
            // Reduce samples by a factor after each iteration because computations gets more costly as the bit size limit increases.
            let sample_count = (WIDE_SCALE_STARTING_SAMPLES_PER_BIT_SIZE as f64
                * (WIDE_SCALE_SAMPLE_REDUCTION_FACTOR.powi(index as i32)))
                as u64;
            gather_sampled_stats(bit_size_limit, sample_count)
        })
        .collect::<Vec<CompressionStats>>();
    let csv_content = generate_stats_csv(&wide_scale_stats, csv_header);
    let wide_scale_statistics_file_name = format!(
        "wide_scale_sampled_statistics_up_to_{}_bits",
        WIDE_SCALE_BIT_SIZE_LIMITS.last().unwrap()
    );
    write_stats_csv(&csv_content, &wide_scale_statistics_file_name);
    let wide_scale_end_time = Instant::now();
    println!(
        "Time taken to generate wide-scale sampled statistics: {:?}",
        wide_scale_end_time.duration_since(wide_scale_start_time)
    );

    let plot_filename_ratios = format!(
        "plots/compression_ratios_wide_scale_sampled_up_to_{}_bits.png",
        WIDE_SCALE_BIT_SIZE_LIMITS.last().unwrap()
    );
    if let Err(e) =
        plot_wide_scale_sampled_compression_ratios(&plot_filename_ratios, &wide_scale_stats)
    {
        eprintln!("Error: Failed to plot wide-scale sampled compression ratios: {e}");
    }

    let plot_filename_favorable = format!(
        "plots/favorable_percentages_wide_scale_sampled_up_to_{}_bits.png",
        WIDE_SCALE_BIT_SIZE_LIMITS.last().unwrap()
    );
    if let Err(e) =
        plot_wide_scale_sampled_favorable_percentages(&plot_filename_favorable, &wide_scale_stats)
    {
        eprintln!("Error: Failed to plot wide-scale sampled favorable percentages: {e}");
    }
}

fn gather_stats_for_limit(limit: u64) -> CompressionStats {
    let start_time = Instant::now();
    let mut compression_amounts = Vec::new();
    let mut maybe_best_value_amount_pair: Option<(u64, f64)> = None;

    for value_to_compress in 1..=limit {
        let Some(compression_amount) = compression_amount_percent(value_to_compress) else {
            continue; // If the compression is not possible, skip this value
        };
        compression_amounts.push(compression_amount);
        maybe_best_value_amount_pair = maybe_best_value_amount_pair.map_or(
            Some((value_to_compress, compression_amount)),
            |(current_best_compressed_value, current_best_compression_amount)| {
                if compression_amount < current_best_compression_amount {
                    Some((value_to_compress, compression_amount))
                } else {
                    Some((
                        current_best_compressed_value,
                        current_best_compression_amount,
                    ))
                }
            },
        );
    }

    if compression_amounts.is_empty() {
        return CompressionStats {
            limit,
            favorable_pct: 0.0,
            average_pct: 0.0,
            median_pct: 0.0,
            maybe_best_compression_amount: None,
            maybe_best_compressed_input: None,
            maybe_average_favorable_pct: None,
            maybe_median_favorable_pct: None,
        };
    }

    let total = compression_amounts.len() as f64;
    let favorable_count = compression_amounts
        .iter()
        .filter(|ratio| **ratio < 1.0)
        .count() as f64;

    let favorable_pct = (favorable_count / total) * 100.0;
    let average_pct = compression_amounts.iter().sum::<f64>() / total;

    let maybe_median = median(&mut compression_amounts);
    let median_pct = if let Some(value) = maybe_median {
        value
    } else {
        0.0
    };

    let mut favorable_amounts: Vec<f64> = compression_amounts
        .iter()
        .copied()
        .filter(|ratio| *ratio < 1.0)
        .collect();

    let maybe_average_favorable_pct = if favorable_amounts.is_empty() {
        None
    } else {
        Some(favorable_amounts.iter().sum::<f64>() / favorable_amounts.len() as f64)
    };

    let maybe_median_favorable_pct = median(&mut favorable_amounts);

    let (maybe_best_compressed_input, maybe_best_compression_amount) =
        if let Some((best_value_input, best_compression_amount)) = maybe_best_value_amount_pair {
            (Some(best_value_input), Some(best_compression_amount))
        } else {
            (None, None)
        };

    let end_time = Instant::now();
    println!(
        "Time taken to gather statistics for limit {:?}: {:?}",
        limit,
        end_time.duration_since(start_time)
    );

    CompressionStats {
        limit,
        favorable_pct,
        average_pct,
        median_pct,
        maybe_best_compressed_input,
        maybe_best_compression_amount,
        maybe_average_favorable_pct,
        maybe_median_favorable_pct,
    }
}

/// Calculates the compression ratio for a given input value, converting the input to a bigint as big endian bytes and then compressing it.
///
/// Returns:
/// - Some(f64) if the compression is possible. The compression ratio as a normalized value where 1.0 = 100% of original size.
///   Values < 1.0 indicate favorable compression (compressed is smaller), values > 1.0 indicate unfavorable compression (compressed is larger).
/// - None if the compression is not possible (e.g. if the input is 0)
fn compression_amount_percent(value: u64) -> Option<f64> {
    let original_number = BigUint::from(value);
    let original_bit_size = original_number.bits();

    if original_bit_size == 0 {
        return None;
    }

    let data_bytes = original_number.to_bytes_be();
    let compressed_as_zeckendorf_data = zeckendorf_compress_be(&data_bytes);
    let compressed_as_bigint = BigUint::from_bytes_le(&compressed_as_zeckendorf_data);
    let compressed_bit_size = compressed_as_bigint.bits();

    let ratio = compressed_bit_size as f64 / original_bit_size as f64;
    Some(ratio)
}

/// Calculates the compression ratio for a given data in bytes.
///
/// Returns:
/// - Some(f64) if the compression is possible. The compression ratio as a normalized value where 1.0 = 100% of original size.
///   Values < 1.0 indicate favorable compression (compressed is smaller), values > 1.0 indicate unfavorable compression (compressed is larger).
/// - None if the compression is not possible (e.g. if the input is an empty bytes array)
fn compression_amount_percent_bytes(data: &[u8]) -> Option<f64> {
    let original_bit_size = data.len() * 8;

    if original_bit_size == 0 {
        return None;
    }

    let compressed_as_zeckendorf_data = zeckendorf_compress_be(data);
    let compressed_as_bigint = BigUint::from_bytes_le(&compressed_as_zeckendorf_data);
    let compressed_bit_size = compressed_as_bigint.bits();

    let ratio = compressed_bit_size as f64 / original_bit_size as f64;
    Some(ratio)
}

/// Generates a random bytes array with roughly the specified number of bits (the number of bits is rounded up to the nearest byte).
fn generate_random_bytes_of_roughly_bit_size(bit_size: u64, rng: &mut StdRng) -> Vec<u8> {
    // Generate random bytes to cover the bit size
    let num_bytes = ((bit_size + 7) / 8) as usize;
    let mut bytes = vec![0u8; num_bytes];
    rng.fill(&mut bytes[..]);

    bytes
}

/// Debug function to investigate issues with the 2000 bits case.
/// This function will:
/// 1. Generate multiple samples at exactly 2000 bits
/// 2. Show detailed information about each sample including:
///    - The generated bytes and their actual bit size
///    - The compression ratio
///    - Any anomalies or edge cases
/// 3. Compare with nearby bit sizes (1999, 2000, 2001) to see if there's a threshold issue
fn _debug_2000_bits_case() {
    println!("\n=== Debugging 2000 bits case ===");

    let test_bit_sizes = [1999, 2000, 2001];
    let num_test_samples = 10;
    let mut rng = StdRng::seed_from_u64(RNG_SEED);

    for &bit_size in &test_bit_sizes {
        println!("\n--- Testing bit size: {} ---", bit_size);

        let mut sample_stats = Vec::new();

        for sample_idx in 0..num_test_samples {
            let random_data = generate_random_bytes_of_roughly_bit_size(bit_size, &mut rng);

            // Calculate actual bit size from bytes
            let bytes_bit_size = random_data.len() * 8;

            // Calculate actual bit size if interpreted as BigUint (leading zeros might be stripped)
            let maybe_bigint = BigUint::from_bytes_be(&random_data);
            let bigint_bit_size = maybe_bigint.bits();

            // Get compression ratio
            let maybe_compression_ratio = compression_amount_percent_bytes(&random_data);

            // Show first few bytes for inspection
            let preview_len = random_data.len().min(8);
            let preview_bytes: Vec<u8> = random_data.iter().take(preview_len).copied().collect();

            println!(
                "Sample {}: bytes_len={}, bytes_bit_size={}, bigint_bit_size={}, preview_bytes={:?}, compression_ratio={:?}",
                sample_idx + 1,
                random_data.len(),
                bytes_bit_size,
                bigint_bit_size,
                preview_bytes,
                maybe_compression_ratio
            );

            if let Some(ratio) = maybe_compression_ratio {
                sample_stats.push(ratio);

                // Check for anomalies
                if ratio > 10.0 || ratio < 0.01 {
                    println!("  ⚠️  ANOMALY: Extreme compression ratio detected!");
                }
                if bytes_bit_size != bigint_bit_size as usize {
                    println!(
                        "  ⚠️  WARNING: Bit size mismatch! bytes_bit_size={}, bigint_bit_size={}",
                        bytes_bit_size, bigint_bit_size
                    );
                }
            } else {
                println!("  ⚠️  WARNING: Compression ratio is None (compression not possible)");
            }
        }

        if !sample_stats.is_empty() {
            let avg_ratio = sample_stats.iter().sum::<f64>() / sample_stats.len() as f64;
            let min_ratio = sample_stats.iter().copied().fold(f64::INFINITY, f64::min);
            let max_ratio = sample_stats
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            let favorable_count = sample_stats.iter().filter(|&&r| r < 1.0).count();

            println!(
                "Summary for {} bits: avg_ratio={:.6}, min_ratio={:.6}, max_ratio={:.6}, favorable_count={}/{}",
                bit_size,
                avg_ratio,
                min_ratio,
                max_ratio,
                favorable_count,
                sample_stats.len()
            );
        }
    }

    // Also test the actual gather_sampled_stats function with a small sample size
    println!("\n--- Testing gather_sampled_stats with 2000 bits (small sample) ---");
    let test_stats = gather_sampled_stats(2000, 100);
    println!("Result: {:?}", test_stats);

    println!("\n=== End of 2000 bits debugging ===\n");
}

fn gather_sampled_stats(bit_size_limit: u64, num_samples: u64) -> CompressionStats {
    let start_time = Instant::now();
    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let mut compression_amounts = Vec::new();
    let mut maybe_best_compression_amount: Option<f64> = None;

    for _ in 0..num_samples {
        let random_data = generate_random_bytes_of_roughly_bit_size(bit_size_limit, &mut rng);
        let Some(compression_amount) = compression_amount_percent_bytes(&random_data) else {
            continue; // If the compression is not possible, skip this sample
        };
        compression_amounts.push(compression_amount);
        maybe_best_compression_amount =
            maybe_best_compression_amount.map_or(Some(compression_amount), |current_best| {
                if compression_amount < current_best {
                    Some(compression_amount)
                } else {
                    Some(current_best)
                }
            });
    }

    if compression_amounts.is_empty() {
        return CompressionStats {
            limit: bit_size_limit,
            favorable_pct: 0.0,
            average_pct: 0.0,
            median_pct: 0.0,
            maybe_best_compression_amount: None,
            maybe_best_compressed_input: None,
            maybe_average_favorable_pct: None,
            maybe_median_favorable_pct: None,
        };
    }

    let total = compression_amounts.len() as f64;
    let favorable_count = compression_amounts
        .iter()
        .filter(|ratio| **ratio < 1.0)
        .count() as f64;

    let favorable_pct = (favorable_count / total) * 100.0;
    let average_pct = compression_amounts.iter().sum::<f64>() / total;

    let maybe_median = median(&mut compression_amounts);
    let median_pct = if let Some(value) = maybe_median {
        value
    } else {
        0.0
    };

    let mut favorable_amounts: Vec<f64> = compression_amounts
        .iter()
        .copied()
        .filter(|ratio| *ratio < 1.0)
        .collect();

    let maybe_average_favorable_pct = if favorable_amounts.is_empty() {
        None
    } else {
        Some(favorable_amounts.iter().sum::<f64>() / favorable_amounts.len() as f64)
    };

    let maybe_median_favorable_pct = median(&mut favorable_amounts);

    let end_time = Instant::now();
    println!(
        "Time taken to gather sampled statistics for bit size {:?} with {} samples: {:?}; time per sample: {:?}",
        bit_size_limit,
        num_samples,
        end_time.duration_since(start_time),
        end_time
            .duration_since(start_time)
            .div_f64(num_samples as f64)
    );

    CompressionStats {
        limit: bit_size_limit,
        favorable_pct,
        average_pct,
        median_pct,
        maybe_best_compressed_input: None,
        maybe_best_compression_amount,
        maybe_average_favorable_pct,
        maybe_median_favorable_pct,
    }
}

fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|a, b| match a.partial_cmp(b) {
        Some(order) => order,
        None => Ordering::Equal,
    });

    let len = values.len();
    let mid = len / 2;

    if len % 2 == 0 {
        let maybe_lower = values.get(mid.saturating_sub(1));
        let maybe_upper = values.get(mid);
        let Some(lower) = maybe_lower else {
            return None;
        };
        let Some(upper) = maybe_upper else {
            return None;
        };
        Some((lower + upper) / 2.0)
    } else {
        let maybe_value = values.get(mid);
        let Some(value) = maybe_value else {
            return None;
        };
        Some(*value)
    }
}

fn plot_compression_ratios(
    filename: &str,
    stats: &[CompressionStats],
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("Plotting compression ratios");

    // Ensure plots directory exists
    std::fs::create_dir_all("plots").expect("Failed to create plots directory");

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    // Find the min and max values for y-axis (only compression ratios)
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for stat in stats {
        min_y = min_y
            .min(stat.average_pct)
            .min(stat.median_pct)
            .min(stat.maybe_average_favorable_pct.unwrap_or(f64::INFINITY))
            .min(stat.maybe_median_favorable_pct.unwrap_or(f64::INFINITY));
        max_y = max_y
            .max(stat.average_pct)
            .max(stat.median_pct)
            .max(
                stat.maybe_average_favorable_pct
                    .unwrap_or(f64::NEG_INFINITY),
            )
            .max(stat.maybe_median_favorable_pct.unwrap_or(f64::NEG_INFINITY));
    }

    // Add some padding
    let y_range = max_y - min_y;
    let y_min = min_y - y_range * 0.1;
    let y_max = max_y + y_range * 0.1;

    // Get x-axis range (logarithmic)
    let x_min = INPUT_LIMITS.first().copied().unwrap_or(1) as f64;
    let x_max = INPUT_LIMITS.last().copied().unwrap_or(1) as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Zeckendorf Compression Ratios",
            ("sans-serif", CAPTION_FONT_SIZE).into_font(),
        )
        .margin(CHART_MARGIN)
        .x_label_area_size(260)
        .y_label_area_size(300)
        .build_cartesian_2d((x_min..x_max).log_scale(), y_min..y_max)?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    // Custom formatter for x-axis labels in scientific notation, because the numbers on the x-axis get too large to be displayed comfortably.
    // Example: 1000000 -> 1e6
    let x_label_formatter = |x: &f64| {
        if *x == 0.0 {
            "0".to_string()
        } else {
            let exponent = x.log10().floor() as i32;
            let mantissa = x / 10_f64.powi(exponent);
            // Round mantissa to 1 decimal place if needed, otherwise show as integer
            let rounded_mantissa = mantissa.round();
            if (mantissa - rounded_mantissa).abs() < 1e-10 {
                format!("{}e{}", rounded_mantissa as i64, exponent)
            } else {
                format!("{:.1}e{}", mantissa, exponent)
            }
        }
    };

    chart
        .configure_mesh()
        .x_desc("Input Limit")
        .y_desc("Compression Ratio")
        .x_label_formatter(&x_label_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Prepare data for each series
    let average_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.average_pct))
        .collect();

    let median_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.median_pct))
        .collect();

    let average_favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map_while(|s| {
            s.maybe_average_favorable_pct
                .map(|average_favorable_pct| (s.limit as f64, average_favorable_pct))
        })
        .collect();

    let median_favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map_while(|s| {
            s.maybe_median_favorable_pct
                .map(|median_favorable_pct| (s.limit as f64, median_favorable_pct))
        })
        .collect();

    // Draw each series with different colors
    chart
        .draw_series(LineSeries::new(
            average_pct_data.iter().copied(),
            BLUE.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Average compression ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                BLUE.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    chart
        .draw_series(LineSeries::new(
            median_pct_data.iter().copied(),
            GREEN.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Median compression ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                GREEN.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    chart
        .draw_series(LineSeries::new(
            average_favorable_pct_data.iter().copied(),
            MAGENTA.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Average favorable compression ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                MAGENTA.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    chart
        .draw_series(LineSeries::new(
            median_favorable_pct_data.iter().copied(),
            CYAN.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Median favorable compression ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                CYAN.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw dots at each point
    chart.draw_series(
        average_pct_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, BLUE.filled())),
    )?;

    chart.draw_series(
        median_pct_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, GREEN.filled())),
    )?;

    chart.draw_series(
        average_favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, MAGENTA.filled())),
    )?;

    chart.draw_series(
        median_favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, CYAN.filled())),
    )?;

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .margin(LEGEND_MARGIN)
        .label_font(("sans-serif", LEGEND_FONT_SIZE).into_font())
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("Compression ratios plot saved to {}", filename);
    let end_time = Instant::now();
    println!(
        "Time taken to plot compression ratios: {:?}",
        end_time.duration_since(start_time)
    );
    Ok(())
}

fn plot_favorable_percentages(
    filename: &str,
    stats: &[CompressionStats],
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("Plotting favorable percentages");

    // Ensure plots directory exists
    std::fs::create_dir_all("plots").expect("Failed to create plots directory");

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    // Find the min and max values for y-axis (only favorable percentages)
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for stat in stats {
        min_y = min_y.min(stat.favorable_pct);
        max_y = max_y.max(stat.favorable_pct);
    }

    // Add some padding
    let y_range = max_y - min_y;
    let y_min = (min_y - y_range * 0.1).max(0.0);
    let y_max = (max_y + y_range * 0.1).min(100.0);

    // Get x-axis range (logarithmic)
    let x_min = INPUT_LIMITS.first().copied().unwrap_or(1) as f64;
    let x_max = INPUT_LIMITS.last().copied().unwrap_or(1) as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Chance of Compression Being Favorable",
            ("sans-serif", CAPTION_FONT_SIZE).into_font(),
        )
        .margin(CHART_MARGIN)
        .x_label_area_size(260)
        .y_label_area_size(300)
        .build_cartesian_2d((x_min..x_max).log_scale(), y_min..y_max)?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    // Custom formatter for x-axis labels in scientific notation, because the numbers on the x-axis get too large to be displayed comfortably.
    // Example: 1000000 -> 1e6
    let x_label_formatter = |x: &f64| {
        if *x == 0.0 {
            "0".to_string()
        } else {
            let exponent = x.log10().floor() as i32;
            let mantissa = x / 10_f64.powi(exponent);
            // Round mantissa to 1 decimal place if needed, otherwise show as integer
            let rounded_mantissa = mantissa.round();
            if (mantissa - rounded_mantissa).abs() < 1e-10 {
                format!("{}e{}", rounded_mantissa as i64, exponent)
            } else {
                format!("{:.1}e{}", mantissa, exponent)
            }
        }
    };

    chart
        .configure_mesh()
        .x_desc("Input Limit")
        .y_desc("Chance of Compression Being Favorable (%)")
        .x_label_formatter(&x_label_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Prepare data for favorable percentage series
    let favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.favorable_pct))
        .collect();

    // Draw the series
    chart
        .draw_series(LineSeries::new(
            favorable_pct_data.iter().copied(),
            RED.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Chance of compression being favorable (%)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                RED.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    const POINT_SIZE: u32 = 5;

    // Draw dots at each point
    chart.draw_series(
        favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, RED.filled())),
    )?;

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .margin(LEGEND_MARGIN)
        .label_font(("sans-serif", LEGEND_FONT_SIZE).into_font())
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("Favorable percentages plot saved to {}", filename);
    let end_time = Instant::now();
    println!(
        "Time taken to plot favorable percentages: {:?}",
        end_time.duration_since(start_time)
    );
    Ok(())
}

fn plot_sampled_compression_ratios(
    filename: &str,
    stats: &[CompressionStats],
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("Plotting sampled compression ratios");

    // Ensure plots directory exists
    std::fs::create_dir_all("plots").expect("Failed to create plots directory");

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    // Find the min and max values for y-axis (only compression ratios)
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for stat in stats {
        min_y = min_y
            .min(stat.average_pct)
            .min(stat.median_pct)
            .min(stat.maybe_average_favorable_pct.unwrap_or(f64::INFINITY))
            .min(stat.maybe_median_favorable_pct.unwrap_or(f64::INFINITY));
        max_y = max_y
            .max(stat.average_pct)
            .max(stat.median_pct)
            .max(
                stat.maybe_average_favorable_pct
                    .unwrap_or(f64::NEG_INFINITY),
            )
            .max(stat.maybe_median_favorable_pct.unwrap_or(f64::NEG_INFINITY));
    }

    // Add some padding
    let y_range = max_y - min_y;
    let y_min = min_y - y_range * 0.1;
    let y_max = max_y + y_range * 0.1;

    // Get x-axis range (logarithmic) - using bit sizes
    let x_min = BIT_SIZE_LIMITS.first().copied().unwrap_or(1) as f64;
    let x_max = BIT_SIZE_LIMITS.last().copied().unwrap_or(1) as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Zeckendorf Compression Ratios"),
            ("sans-serif", CAPTION_FONT_SIZE).into_font(),
        )
        .margin(CHART_MARGIN)
        .x_label_area_size(260)
        .y_label_area_size(300)
        .build_cartesian_2d((x_min..x_max).log_scale(), y_min..y_max)?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    let x_label_bits_formatter = |x: &f64| {
        if *x >= 1000.0 {
            format!("{:.0} kbits", x / 1000.0)
        } else {
            format!("{:.0} bits", x)
        }
    };

    chart
        .configure_mesh()
        .x_desc(format!(
            "Bit Size Limit ({} samples per limit), Log Scale",
            SAMPLES_PER_BIT_SIZE.to_formatted_string(&Locale::en)
        ))
        .y_desc("Compression Ratio")
        .x_label_formatter(&x_label_bits_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Prepare data for each series
    let average_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.average_pct))
        .collect();

    let median_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.median_pct))
        .collect();

    let average_favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map_while(|stat| {
            stat.maybe_average_favorable_pct
                .map(|average_favorable_pct| (stat.limit as f64, average_favorable_pct))
        })
        .collect();

    let median_favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map_while(|stat| {
            stat.maybe_median_favorable_pct
                .map(|median_favorable_pct| (stat.limit as f64, median_favorable_pct))
        })
        .collect();

    const STROKE_WIDTH: u32 = 3;
    const LEGEND_PATH_LEFT_OFFSET: i32 = 30;
    const LEGEND_PATH_RIGHT_OFFSET: i32 = 10;

    // Draw each series with different colors
    chart
        .draw_series(LineSeries::new(
            average_pct_data.iter().copied(),
            BLUE.stroke_width(STROKE_WIDTH),
        ))?
        .label("Average compression ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                BLUE.stroke_width(STROKE_WIDTH),
            )
        });

    chart
        .draw_series(LineSeries::new(
            median_pct_data.iter().copied(),
            GREEN.stroke_width(STROKE_WIDTH),
        ))?
        .label("Median compression ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                GREEN.stroke_width(STROKE_WIDTH),
            )
        });

    chart
        .draw_series(LineSeries::new(
            average_favorable_pct_data.iter().copied(),
            MAGENTA.stroke_width(STROKE_WIDTH),
        ))?
        .label("Average favorable compression ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                MAGENTA.stroke_width(STROKE_WIDTH),
            )
        });

    chart
        .draw_series(LineSeries::new(
            median_favorable_pct_data.iter().copied(),
            CYAN.stroke_width(STROKE_WIDTH),
        ))?
        .label("Median favorable compression ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                CYAN.stroke_width(STROKE_WIDTH),
            )
        });

    // Draw dots at each point
    chart.draw_series(
        average_pct_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, BLUE.filled())),
    )?;

    chart.draw_series(
        median_pct_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, GREEN.filled())),
    )?;

    chart.draw_series(
        average_favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, MAGENTA.filled())),
    )?;

    chart.draw_series(
        median_favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, CYAN.filled())),
    )?;

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .margin(LEGEND_MARGIN)
        .label_font(("sans-serif", LEGEND_FONT_SIZE).into_font())
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("Sampled compression ratios plot saved to {}", filename);
    let end_time = Instant::now();
    println!(
        "Time taken to plot sampled compression ratios: {:?}",
        end_time.duration_since(start_time)
    );
    Ok(())
}

fn plot_sampled_favorable_percentages(
    filename: &str,
    stats: &[CompressionStats],
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("Plotting sampled favorable percentages");

    // Ensure plots directory exists
    std::fs::create_dir_all("plots").expect("Failed to create plots directory");

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    // Find the min and max values for y-axis (only favorable percentages)
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for stat in stats {
        min_y = min_y.min(stat.favorable_pct);
        max_y = max_y.max(stat.favorable_pct);
    }

    // Add some padding
    let y_range = max_y - min_y;
    let y_min = (min_y - y_range * 0.1).max(0.0);
    let y_max = (max_y + y_range * 0.1).min(100.0);

    // Get x-axis range (logarithmic) - using bit sizes
    let x_min = BIT_SIZE_LIMITS.first().copied().unwrap_or(1) as f64;
    let x_max = BIT_SIZE_LIMITS.last().copied().unwrap_or(1) as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Chance of Compression Being Favorable"),
            ("sans-serif", CAPTION_FONT_SIZE).into_font(),
        )
        .margin(CHART_MARGIN)
        .x_label_area_size(260)
        .y_label_area_size(300)
        .build_cartesian_2d((x_min..x_max).log_scale(), (y_min..y_max).log_scale())?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    let x_label_bits_formatter = |x: &f64| {
        if *x >= 1000.0 {
            format!("{:.0} kbits", x / 1000.0)
        } else {
            format!("{:.0} bits", x)
        }
    };

    chart
        .configure_mesh()
        // Format the samples per limit as a string with comma separation for thousands
        .x_desc(format!(
            "Bit Size Limit ({} samples per limit), Log Scale",
            SAMPLES_PER_BIT_SIZE.to_formatted_string(&Locale::en)
        ))
        .y_desc("Chance of Compression Being Favorable (%)")
        .x_label_formatter(&x_label_bits_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Prepare data for favorable percentage series
    let favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.favorable_pct))
        .collect();

    const STROKE_WIDTH: u32 = 3;
    const LEGEND_PATH_LEFT_OFFSET: i32 = 30;
    const LEGEND_PATH_RIGHT_OFFSET: i32 = 10;

    // Draw the series
    chart
        .draw_series(LineSeries::new(
            favorable_pct_data.iter().copied(),
            RED.stroke_width(STROKE_WIDTH),
        ))?
        .label("Chance of compression being favorable (%)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                RED.stroke_width(STROKE_WIDTH),
            )
        });

    const POINT_SIZE: u32 = 5;

    // Draw dots at each point
    chart.draw_series(
        favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, RED.filled())),
    )?;

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .margin(LEGEND_MARGIN)
        .label_font(("sans-serif", LEGEND_FONT_SIZE).into_font())
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("Sampled favorable percentages plot saved to {}", filename);
    let end_time = Instant::now();
    println!(
        "Time taken to plot sampled favorable percentages: {:?}",
        end_time.duration_since(start_time)
    );
    Ok(())
}

fn plot_wide_scale_sampled_compression_ratios(
    filename: &str,
    stats: &[CompressionStats],
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("Plotting wide-scale sampled compression ratios");

    // Ensure plots directory exists
    std::fs::create_dir_all("plots").expect("Failed to create plots directory");

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    // Find the min and max values for y-axis (only compression ratios)
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for stat in stats {
        min_y = min_y
            .min(stat.average_pct)
            .min(stat.median_pct)
            .min(stat.maybe_average_favorable_pct.unwrap_or(f64::INFINITY))
            .min(stat.maybe_median_favorable_pct.unwrap_or(f64::INFINITY));
        max_y = max_y
            .max(stat.average_pct)
            .max(stat.median_pct)
            .max(
                stat.maybe_average_favorable_pct
                    .unwrap_or(f64::NEG_INFINITY),
            )
            .max(stat.maybe_median_favorable_pct.unwrap_or(f64::NEG_INFINITY));
    }

    // Add some padding
    let y_range = max_y - min_y;
    let y_min = min_y - y_range * 0.1;
    let y_max = max_y + y_range * 0.1;

    // Get x-axis range (logarithmic) - using wide-scale bit sizes
    let x_min = WIDE_SCALE_BIT_SIZE_LIMITS.first().copied().unwrap_or(1) as f64;
    let x_max = WIDE_SCALE_BIT_SIZE_LIMITS.last().copied().unwrap_or(1) as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Zeckendorf Compression Ratios (Wide-Scale Sampling)"),
            ("sans-serif", CAPTION_FONT_SIZE).into_font(),
        )
        .margin(CHART_MARGIN)
        .x_label_area_size(260)
        .y_label_area_size(300)
        .build_cartesian_2d((x_min..x_max).log_scale(), y_min..y_max)?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    // Custom formatter for x-axis labels in scientific notation for large bit sizes
    let x_label_formatter = |x: &f64| {
        if *x == 0.0 {
            "0".to_string()
        } else {
            let exponent = x.log10().floor() as i32;
            let mantissa = x / 10_f64.powi(exponent);
            // Round mantissa to 1 decimal place if needed, otherwise show as integer
            let rounded_mantissa = mantissa.round();
            if (mantissa - rounded_mantissa).abs() < 1e-10 {
                format!("{}e{}", rounded_mantissa as i64, exponent)
            } else {
                format!("{:.1}e{}", mantissa, exponent)
            }
        }
    };

    chart
        .configure_mesh()
        .x_desc(format!(
            "Bit Size Limit (starting with {} samples), Log Scale",
            WIDE_SCALE_STARTING_SAMPLES_PER_BIT_SIZE.to_formatted_string(&Locale::en)
        ))
        .y_desc("Compression Ratio")
        .x_label_formatter(&x_label_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Prepare data for each series
    let average_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.average_pct))
        .collect();

    let median_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.median_pct))
        .collect();

    let average_favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map_while(|stat| {
            stat.maybe_average_favorable_pct
                .map(|average_favorable_pct| (stat.limit as f64, average_favorable_pct))
        })
        .collect();

    let median_favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map_while(|stat| {
            stat.maybe_median_favorable_pct
                .map(|median_favorable_pct| (stat.limit as f64, median_favorable_pct))
        })
        .collect();

    // Draw each series with different colors
    chart
        .draw_series(LineSeries::new(
            average_pct_data.iter().copied(),
            BLUE.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Average compression ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                BLUE.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    chart
        .draw_series(LineSeries::new(
            median_pct_data.iter().copied(),
            GREEN.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Median compression ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                GREEN.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    chart
        .draw_series(LineSeries::new(
            average_favorable_pct_data.iter().copied(),
            MAGENTA.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Average favorable compression ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                MAGENTA.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    chart
        .draw_series(LineSeries::new(
            median_favorable_pct_data.iter().copied(),
            CYAN.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Median favorable compression ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                CYAN.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw dots at each point
    chart.draw_series(
        average_pct_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, BLUE.filled())),
    )?;

    chart.draw_series(
        median_pct_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, GREEN.filled())),
    )?;

    chart.draw_series(
        average_favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, MAGENTA.filled())),
    )?;

    chart.draw_series(
        median_favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, CYAN.filled())),
    )?;

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .margin(LEGEND_MARGIN)
        .label_font(("sans-serif", LEGEND_FONT_SIZE).into_font())
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!(
        "Wide-scale sampled compression ratios plot saved to {}",
        filename
    );
    let end_time = Instant::now();
    println!(
        "Time taken to plot wide-scale sampled compression ratios: {:?}",
        end_time.duration_since(start_time)
    );
    Ok(())
}

/// We know the chance of compression being favorable decreases towards 0 as the bit size limit increases, but we have this for completeness.
fn plot_wide_scale_sampled_favorable_percentages(
    filename: &str,
    stats: &[CompressionStats],
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("Plotting wide-scale sampled favorable percentages");

    // Ensure plots directory exists
    std::fs::create_dir_all("plots").expect("Failed to create plots directory");

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    // Find the min and max values for y-axis (only favorable percentages)
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for stat in stats {
        min_y = min_y.min(stat.favorable_pct);
        max_y = max_y.max(stat.favorable_pct);
    }

    // Add some padding
    let y_range = max_y - min_y;
    let y_min = (min_y - y_range * 0.1).max(0.0);
    let y_max = (max_y + y_range * 0.1).min(100.0);

    // Get x-axis range (logarithmic) - using wide-scale bit sizes
    let x_min = WIDE_SCALE_BIT_SIZE_LIMITS.first().copied().unwrap_or(1) as f64;
    let x_max = WIDE_SCALE_BIT_SIZE_LIMITS.last().copied().unwrap_or(1) as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Chance of Compression Being Favorable (Wide-Scale Sampling)"),
            ("sans-serif", CAPTION_FONT_SIZE).into_font(),
        )
        .margin(CHART_MARGIN)
        .x_label_area_size(260)
        .y_label_area_size(300)
        .build_cartesian_2d((x_min..x_max).log_scale(), (y_min..y_max).log_scale())?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    // Custom formatter for x-axis labels in scientific notation for large bit sizes
    let x_label_formatter = |x: &f64| {
        if *x == 0.0 {
            "0".to_string()
        } else {
            let exponent = x.log10().floor() as i32;
            let mantissa = x / 10_f64.powi(exponent);
            // Round mantissa to 1 decimal place if needed, otherwise show as integer
            let rounded_mantissa = mantissa.round();
            if (mantissa - rounded_mantissa).abs() < 1e-10 {
                format!("{}e{}", rounded_mantissa as i64, exponent)
            } else {
                format!("{:.1}e{}", mantissa, exponent)
            }
        }
    };

    chart
        .configure_mesh()
        .x_desc(format!(
            "Bit Size Limit (starting with {} samples), Log Scale",
            WIDE_SCALE_STARTING_SAMPLES_PER_BIT_SIZE.to_formatted_string(&Locale::en)
        ))
        .y_desc("Chance of Compression Being Favorable (%)")
        .x_label_formatter(&x_label_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Prepare data for favorable percentage series
    let favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.favorable_pct))
        .collect();

    // Draw the series
    chart
        .draw_series(LineSeries::new(
            favorable_pct_data.iter().copied(),
            RED.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Chance of compression being favorable (%)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                RED.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    const POINT_SIZE: u32 = 5;

    // Draw dots at each point
    chart.draw_series(
        favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, RED.filled())),
    )?;

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .margin(LEGEND_MARGIN)
        .label_font(("sans-serif", LEGEND_FONT_SIZE).into_font())
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!(
        "Wide-scale sampled favorable percentages plot saved to {}",
        filename
    );
    let end_time = Instant::now();
    println!(
        "Time taken to plot wide-scale sampled favorable percentages: {:?}",
        end_time.duration_since(start_time)
    );
    Ok(())
}
