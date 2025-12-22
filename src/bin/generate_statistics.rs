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
//! Run with: `cargo run --release --bin generate_statistics --features plotting`

use plotters::prelude::*;

use num_bigint::BigUint;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::{cmp::Ordering, fs, path::Path, time::Instant};
use zeckendorf_rs::zeckendorf_compress_be;

const AXIS_FONT_SIZE: u32 = 100;
const AXIS_TICK_FONT_SIZE: u32 = 64;
const CAPTION_FONT_SIZE: u32 = 160;
const LEGEND_FONT_SIZE: u32 = 70;
const CHART_MARGIN: u32 = 120;
const PLOT_WIDTH: u32 = 3840;
const PLOT_HEIGHT: u32 = 2160;
const LEGEND_MARGIN: u32 = 50;

// Time taken to generate bit limit statistics: 111.330666ms
const INPUT_LIMITS: [u64; 5] = [10, 100, 1_000, 10_000, 100_000];

// Time taken to generate bit limit statistics: 861.613791ms
// const INPUT_LIMITS: [u64; 6] = [10, 100, 1_000, 10_000, 100_000, 1_000_000];

// Time taken to generate bit limit statistics: 110.184340667s; 110 / 100000000 ~ 1.1 microseconds per input on average.
// const INPUT_LIMITS: [u64; 8] = [10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000];

// Sampled statistics configuration
// Time taken to generate sampled statistics: 16.027274541s
const BIT_SIZE_LIMITS: [u64; 8] = [20, 50, 100, 200, 500, 1_000, 2_000, 5_000];
const SAMPLES_PER_BIT_SIZE: u64 = 100_000;

// Seed for the random number generator to ensure reproducible results
const RNG_SEED: u64 = 42;

#[derive(Debug, Clone)]
struct CompressionStats {
    limit: u64,
    favorable_pct: f64,
    average_pct: f64,
    median_pct: f64,
    best_compressed_input: Option<u64>,
    best_compression_amount: f64,
    average_favorable_pct: f64,
    median_favorable_pct: f64,
}

fn main() {
    let start_time = Instant::now();

    generate_bit_limit_stats();
    generate_sampled_bit_limit_stats();

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
            stat.best_compression_amount,
            stat.best_compressed_input
                .map(|input| input.to_string())
                .unwrap_or_else(|| "".to_string()),
            stat.average_favorable_pct,
            stat.median_favorable_pct
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
    let csv_header = "compression up to input,chance of compression being favorable,average compression amount in percent,median compression amount in percent,best compression amount in percent,best compression input,average favorable compression amount in percent,median favorable compression amount in percent\n";

    let all_stats = INPUT_LIMITS
        .iter()
        .map(|&limit| gather_stats_for_limit(limit))
        .collect::<Vec<CompressionStats>>();
    let statistics_file_name = format!("statistics_up_to_{}_inputs", INPUT_LIMITS.last().unwrap());
    let csv_content = generate_stats_csv(&all_stats, csv_header);
    write_stats_csv(&csv_content, &statistics_file_name);

    let plot_filename = format!(
        "plots/compression_statistics_up_to_{}_inputs.png",
        INPUT_LIMITS.last().unwrap()
    );
    if let Err(e) = plot_statistics(&plot_filename, &all_stats) {
        eprintln!("Error: Failed to plot statistics: {e}");
    }
    let end_time = Instant::now();
    println!(
        "Time taken to generate bit limit statistics: {:?}",
        end_time.duration_since(start_time)
    );
}

fn generate_sampled_bit_limit_stats() {
    let csv_header = "max bit size,chance of compression being favorable,average compression amount in percent,median compression amount in percent,best compression amount in percent,best compression input,average favorable compression amount in percent,median favorable compression amount in percent\n";

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

    let plot_filename = format!(
        "plots/compression_statistics_sampled_up_to_{}_bits.png",
        BIT_SIZE_LIMITS.last().unwrap()
    );
    if let Err(e) = plot_sampled_statistics(&plot_filename, &sampled_stats) {
        eprintln!("Error: Failed to plot sampled statistics: {e}");
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
                if compression_amount > current_best_compression_amount {
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
            best_compression_amount: 0.0,
            best_compressed_input: None,
            average_favorable_pct: 0.0,
            median_favorable_pct: 0.0,
        };
    }

    let total = compression_amounts.len() as f64;
    let favorable_count = compression_amounts
        .iter()
        .filter(|amount| **amount > 0.0)
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
        .filter(|amount| *amount > 0.0)
        .collect();

    let average_favorable_pct = if favorable_amounts.is_empty() {
        0.0
    } else {
        favorable_amounts.iter().sum::<f64>() / favorable_amounts.len() as f64
    };

    let maybe_favorable_median = median(&mut favorable_amounts);
    let median_favorable_pct = if let Some(value) = maybe_favorable_median {
        value
    } else {
        0.0
    };

    let (best_compressed_input, best_compression_amount) =
        if let Some((best_value_input, best_compression_amount)) = maybe_best_value_amount_pair {
            (Some(best_value_input), best_compression_amount)
        } else {
            (None, 0.0)
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
        best_compressed_input,
        best_compression_amount,
        average_favorable_pct,
        median_favorable_pct,
    }
}

/// Calculates the compression amount in percent for a given input value, converting the input to a bigint as big endian bytes and then compressing it.
///
/// Returns:
/// - Some(f64) if the compression is possible. The compression amount in percent as a positive number, or a negative number if the compression is unfavorable (increases the size of the data).
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
    Some((1.0 - ratio) * 100.0)
}

/// Calculates the compression amount in percent for a given data in bytes.
///
/// Returns:
/// - Some(f64) if the compression is possible. The compression amount in percent as a positive number, or a negative number if the compression is unfavorable (increases the size of the data).
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
    Some((1.0 - ratio) * 100.0)
}

/// Generates a random bytes array with roughly the specified number of bits (the number of bits is rounded up to the nearest byte).
fn generate_random_bytes_of_roughly_bit_size(bit_size: u64, rng: &mut StdRng) -> Vec<u8> {
    // Generate random bytes to cover the bit size
    let num_bytes = ((bit_size + 7) / 8) as usize;
    let mut bytes = vec![0u8; num_bytes];
    rng.fill(&mut bytes[..]);

    bytes
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
                if compression_amount > current_best {
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
            best_compression_amount: 0.0,
            best_compressed_input: None,
            average_favorable_pct: 0.0,
            median_favorable_pct: 0.0,
        };
    }

    let total = compression_amounts.len() as f64;
    let favorable_count = compression_amounts
        .iter()
        .filter(|amount| **amount > 0.0)
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
        .filter(|amount| *amount > 0.0)
        .collect();

    let average_favorable_pct = if favorable_amounts.is_empty() {
        0.0
    } else {
        favorable_amounts.iter().sum::<f64>() / favorable_amounts.len() as f64
    };

    let maybe_favorable_median = median(&mut favorable_amounts);
    let median_favorable_pct = if let Some(value) = maybe_favorable_median {
        value
    } else {
        0.0
    };

    let best_compression_amount = maybe_best_compression_amount.unwrap_or(0.0);

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
        best_compressed_input: None,
        best_compression_amount,
        average_favorable_pct,
        median_favorable_pct,
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

fn plot_statistics(
    filename: &str,
    stats: &[CompressionStats],
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("Plotting compression statistics");

    // Ensure plots directory exists
    std::fs::create_dir_all("plots").expect("Failed to create plots directory");

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    // Find the min and max values for y-axis
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for stat in stats {
        min_y = min_y
            .min(stat.favorable_pct)
            .min(stat.average_pct)
            .min(stat.median_pct)
            .min(stat.average_favorable_pct)
            .min(stat.median_favorable_pct);
        max_y = max_y
            .max(stat.favorable_pct)
            .max(stat.average_pct)
            .max(stat.median_pct)
            .max(stat.average_favorable_pct)
            .max(stat.median_favorable_pct);
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
            "Zeckendorf Compression Statistics",
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
        .y_desc("Compression Amount (%)")
        .x_label_formatter(&x_label_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Prepare data for each series
    let favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.favorable_pct))
        .collect();

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
        .map(|s| (s.limit as f64, s.average_favorable_pct))
        .collect();

    let median_favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.median_favorable_pct))
        .collect();

    const STROKE_WIDTH: u32 = 3;
    const LEGEND_PATH_LEFT_OFFSET: i32 = 30;
    const LEGEND_PATH_RIGHT_OFFSET: i32 = 10;

    // Draw each series with different colors
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

    chart
        .draw_series(LineSeries::new(
            average_pct_data.iter().copied(),
            BLUE.stroke_width(STROKE_WIDTH),
        ))?
        .label("Average compression amount (%)")
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
        .label("Median compression amount (%)")
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
        .label("Average favorable compression amount (%)")
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
        .label("Median favorable compression amount (%)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                CYAN.stroke_width(STROKE_WIDTH),
            )
        });

    const POINT_SIZE: u32 = 5;

    // Draw dots at each point
    chart.draw_series(
        favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, RED.filled())),
    )?;

    chart.draw_series(
        average_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, BLUE.filled())),
    )?;

    chart.draw_series(
        median_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, GREEN.filled())),
    )?;

    chart.draw_series(
        average_favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, MAGENTA.filled())),
    )?;

    chart.draw_series(
        median_favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, CYAN.filled())),
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
    println!("Compression statistics plot saved to {}", filename);
    let end_time = Instant::now();
    println!(
        "Time taken to plot compression statistics: {:?}",
        end_time.duration_since(start_time)
    );
    Ok(())
}

fn plot_sampled_statistics(
    filename: &str,
    stats: &[CompressionStats],
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("Plotting sampled compression statistics");

    // Ensure plots directory exists
    std::fs::create_dir_all("plots").expect("Failed to create plots directory");

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    // Find the min and max values for y-axis
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for stat in stats {
        min_y = min_y
            .min(stat.favorable_pct)
            .min(stat.average_pct)
            .min(stat.median_pct)
            .min(stat.average_favorable_pct)
            .min(stat.median_favorable_pct);
        max_y = max_y
            .max(stat.favorable_pct)
            .max(stat.average_pct)
            .max(stat.median_pct)
            .max(stat.average_favorable_pct)
            .max(stat.median_favorable_pct);
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
            format!(
                "Zeckendorf Compression Statistics\n(Sampled, {} samples per bit size limit)",
                SAMPLES_PER_BIT_SIZE
            ),
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
        .x_desc("Bit Size Limit")
        .y_desc("Compression Amount (%)")
        .x_label_formatter(&x_label_bits_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Prepare data for each series
    let favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.favorable_pct))
        .collect();

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
        .map(|s| (s.limit as f64, s.average_favorable_pct))
        .collect();

    let median_favorable_pct_data: Vec<(f64, f64)> = stats
        .iter()
        .map(|s| (s.limit as f64, s.median_favorable_pct))
        .collect();

    const STROKE_WIDTH: u32 = 3;
    const LEGEND_PATH_LEFT_OFFSET: i32 = 30;
    const LEGEND_PATH_RIGHT_OFFSET: i32 = 10;

    // Draw each series with different colors
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

    chart
        .draw_series(LineSeries::new(
            average_pct_data.iter().copied(),
            BLUE.stroke_width(STROKE_WIDTH),
        ))?
        .label("Average compression amount (%)")
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
        .label("Median compression amount (%)")
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
        .label("Average favorable compression amount (%)")
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
        .label("Median favorable compression amount (%)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                CYAN.stroke_width(STROKE_WIDTH),
            )
        });

    const POINT_SIZE: u32 = 5;

    // Draw dots at each point
    chart.draw_series(
        favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, RED.filled())),
    )?;

    chart.draw_series(
        average_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, BLUE.filled())),
    )?;

    chart.draw_series(
        median_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, GREEN.filled())),
    )?;

    chart.draw_series(
        average_favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, MAGENTA.filled())),
    )?;

    chart.draw_series(
        median_favorable_pct_data
            .iter()
            .map(|point| Circle::new(*point, POINT_SIZE, CYAN.filled())),
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
    println!("Sampled compression statistics plot saved to {}", filename);
    let end_time = Instant::now();
    println!(
        "Time taken to plot sampled compression statistics: {:?}",
        end_time.duration_since(start_time)
    );
    Ok(())
}
