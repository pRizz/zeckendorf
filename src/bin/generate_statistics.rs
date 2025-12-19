//! Binary for generating statistics about the compression ratio of the Zeckendorf representation
//!
//! The statistics are saved in the statistics_history directory in a file named statistics_up_to_<limit>_inputs.csv
//!
//! The purpose of this binary is to determine the average compression ratio, median compression ratio, best compression ratio, and chance of compression being favorable for a given limit. As we compress to higher limits, the statistics should become more stable.
//!
//! The Zeckendorf compression oscillates between being favorable and unfavorable, as the data changes, and the statistics are used to determine the average and median compression ratios, and the chance of compression being favorable. See this crate's `plot` binary for more details about the oscillation and to visualize the compression ratios.
//!
//! The meaning of "compression up to input" in the csv header is such that the statistics are gathered for all inputs up to and including the given limit. For example, "compression up to 100" means that the corresponding statistics in that row in the csv are gathered for all inputs from 1 to 100.
//!
//! Run with: `cargo run --bin generate_statistics --features plotting`

use plotters::prelude::*;

use num_bigint::BigUint;
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

// Time taken to generate statistics for limits [10, 100, 1000, 10000, 100000]: 1.337864666s
const INPUT_LIMITS: [u64; 5] = [10, 100, 1_000, 10_000, 100_000];

// Time taken to generate statistics for limits [10, 100, 1000, 10000, 100000, 1000000]: 9.208086875s
// const INPUT_LIMITS: [u64; 6] = [10, 100, 1_000, 10_000, 100_000, 1_000_000];

// Time taken to generate statistics for limits [10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000]: 1142.74973475s or ~19 minutes
// const INPUT_LIMITS: [u64; 8] = [10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000];

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
    let mut output = String::new();
    let csv_header = "compression up to input,chance of compression being favorable,average compression amount in percent,median compression amount in percent,best compression amount in percent,best compression input,average favorable compression amount in percent,median favorable compression amount in percent\n";
    output.push_str(csv_header);
    println!("{}", csv_header);

    let mut all_stats = Vec::new();
    for &limit in INPUT_LIMITS.iter() {
        let stats = gather_stats(limit);
        all_stats.push(stats.clone());
        // println!("Statistics for limit {:?}: {:?}", limit, stats);
        let line = format!(
            "{},{:.6},{:.6},{:.6},{:.6},{},{:.6},{:.6}",
            stats.limit,
            stats.favorable_pct,
            stats.average_pct,
            stats.median_pct,
            stats.best_compression_amount,
            stats
                .best_compressed_input
                .map(|input| input.to_string())
                .unwrap_or_else(|| "".to_string()),
            stats.average_favorable_pct,
            stats.median_favorable_pct
        );
        println!("{}", line);
        output.push_str(&line);
        output.push_str("\n");
    }
    let statistics_directory = Path::new("statistics_history");
    if let Err(e) = fs::create_dir_all(statistics_directory) {
        eprintln!("Error: Failed to create directory 'statistics_history': {e}");
        std::process::exit(1);
    }
    let statistics_file_name = format!(
        "statistics_up_to_{}_inputs.csv",
        INPUT_LIMITS.last().unwrap()
    );
    println!(
        "Writing statistics to '{}'",
        statistics_directory.join(&statistics_file_name).display()
    );
    fs::write(statistics_directory.join(&statistics_file_name), output)
        .expect("Failed to write statistics to file");

    {
        let plot_filename = format!(
            "plots/compression_statistics_up_to_{}_inputs.png",
            INPUT_LIMITS.last().unwrap()
        );
        if let Err(e) = plot_statistics(&plot_filename, &all_stats) {
            eprintln!("Error: Failed to plot statistics: {e}");
        }
    }

    let end_time = Instant::now();
    println!(
        "Time taken to generate statistics for limits {:?}: {:?}",
        INPUT_LIMITS,
        end_time.duration_since(start_time)
    );
}

fn gather_stats(limit: u64) -> CompressionStats {
    let start_time = Instant::now();
    let mut compression_amounts = Vec::new();
    let mut maybe_best_value_amount_pair: Option<(u64, f64)> = None;

    for value_to_compress in 1..=limit {
        if let Some(compression_amount) = compression_amount_percent(value_to_compress) {
            compression_amounts.push(compression_amount);
            maybe_best_value_amount_pair = match maybe_best_value_amount_pair {
                Some((current_best_compressed_value, current_best_compression_amount)) => {
                    // If the new amount is better than the current best, update the best
                    if compression_amount > current_best_compression_amount {
                        Some((value_to_compress, compression_amount))
                    }
                    // Otherwise, keep the current best
                    else {
                        Some((
                            current_best_compressed_value,
                            current_best_compression_amount,
                        ))
                    }
                }
                None => Some((value_to_compress, compression_amount)),
            };
        }
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

    chart
        .configure_mesh()
        .x_desc("Input Limit")
        .y_desc("Compression Statistics (%)")
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
