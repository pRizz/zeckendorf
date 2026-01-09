//! Binary for generating plots using plotters
//!
//! This binary requires the `plotting` and `development_tools` features to be enabled.
//! Run with: `cargo run --release --bin zeck-plot --features plotting,development_tools`

use num_bigint::BigUint;
use num_format::{Locale, ToFormattedString};
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::sync::Arc;
use std::time::Instant;
use zeck::*;

const AXIS_FONT_SIZE: u32 = 100;
const AXIS_TICK_FONT_SIZE: u32 = 64;
const CAPTION_FONT_SIZE: u32 = 160;
const LEGEND_FONT_SIZE: u32 = 80;
const POINT_LABEL_FONT_SIZE: u32 = 40;
const CHART_MARGIN: u32 = 120;
const PLOT_WIDTH: u32 = 3840;
const PLOT_HEIGHT: u32 = 2160;
const LEGEND_MARGIN: u32 = 50;
const SERIES_LINE_STROKE_WIDTH: u32 = 3;
const SERIES_LINE_DOT_SIZE: u32 = 5;
const LEGEND_PATH_LEFT_OFFSET: i32 = 30;
const LEGEND_PATH_RIGHT_OFFSET: i32 = 10;

fn main() {
    let start_time = Instant::now();

    // Ensure plots directory exists
    std::fs::create_dir_all("plots").expect("Failed to create plots directory");

    // Example: Plot Fibonacci numbers
    plot_fibonacci_numbers("plots/fibonacci_plot_0_to_30.png", 0..31)
        .expect("Failed to plot Fibonacci numbers");

    // Example: Plot Fibonacci, binary, and all-ones Zeckendorf numbers
    plot_fibonacci_binary_all_ones("plots/fibonacci_binary_all_ones_0_to_30.png", 0..31)
        .expect("Failed to plot Fibonacci, binary, and all-ones Zeckendorf numbers");

    // Example: Plot Fibonacci, binary, all-ones Zeckendorf, and 3^n numbers
    plot_fibonacci_binary_all_ones_power3(
        "plots/fibonacci_binary_all_ones_power3_0_to_30.png",
        0..31,
    )
    .expect("Failed to plot Fibonacci, binary, all-ones Zeckendorf, and 3^n numbers");

    // Example: Plot Fibonacci, binary, all-ones Zeckendorf, 3^n, φⁿ, and φ²ⁿ numbers
    plot_fibonacci_binary_all_ones_power3_phi_phi_squared(
        "plots/fibonacci_binary_all_ones_power3_phi_phi_squared_0_to_30.png",
        0..31,
    )
    .expect("Failed to plot Fibonacci, binary, all-ones Zeckendorf, 3^n, φⁿ, and φ²ⁿ numbers");

    // _plot_all_compression_ratios();

    // _plot_all_histograms();

    _plot_all_square_root_error_convergences();

    let end_time = Instant::now();
    println!("Time taken: {:?}", end_time.duration_since(start_time));
}

fn _plot_all_square_root_error_convergences() {
    // Plot with different n-value strategies
    plot_fibonacci_square_root_error_convergence(
        "plots/fibonacci_square_root_error_convergence_start2_step2_log_y.png",
        true,
        NValueStrategy::Start2Step2,
    )
    .expect("Failed to plot Fibonacci square root error convergence");
    plot_fibonacci_square_root_error_convergence(
        "plots/fibonacci_square_root_error_convergence_start2_step2_linear_y.png",
        false,
        NValueStrategy::Start2Step2,
    )
    .expect("Failed to plot Fibonacci square root error convergence");
    // We cannot print a log graph for start1_step2 because the numbers are negative,
    // and log(negative number) is undefined.
    // plot_fibonacci_square_root_ratio_error(
    //     "plots/fibonacci_square_root_error_convergence_start1_step2_log_y.png",
    //     true,
    //     NValueStrategy::Start1Step2,
    // )
    // .expect("Failed to plot Fibonacci square root error convergence");
    plot_fibonacci_square_root_error_convergence(
        "plots/fibonacci_square_root_error_convergence_start1_step2_linear_y.png",
        false,
        NValueStrategy::Start1Step2,
    )
    .expect("Failed to plot Fibonacci square root error convergence");
    // We cannot print a log graph for start1_step1 because the numbers oscillate between positive and negative values,
    // and log(0) and log(negative number) are undefined.
    // plot_fibonacci_square_root_ratio_error(
    //     "plots/fibonacci_square_root_error_convergence_start1_step1_log_y.png",
    //     true,
    //     NValueStrategy::Start1Step1,
    // )
    // .expect("Failed to plot Fibonacci square root error convergence");
    plot_fibonacci_square_root_error_convergence(
        "plots/fibonacci_square_root_error_convergence_start1_step1_linear_y.png",
        false,
        NValueStrategy::Start1Step1,
    )
    .expect("Failed to plot Fibonacci square root error convergence");
}

fn _plot_all_compression_ratios() {
    _plot_compression_ratios("plots/compression_ratios_0_to_100.png", 0..100)
        .expect("Failed to plot compression ratios");
    _plot_compression_ratios("plots/compression_ratios_0_to_257.png", 0..257)
        .expect("Failed to plot compression ratios");
    _plot_compression_ratios("plots/compression_ratios_0_to_1_000.png", 0..1_000)
        .expect("Failed to plot compression ratios");
    _plot_compression_ratios("plots/compression_ratios_0_to_10_000.png", 0..10_000)
        .expect("Failed to plot compression ratios");
    _plot_compression_ratios("plots/compression_ratios_0_to_100_000.png", 0..100_000)
        .expect("Failed to plot compression ratios");
    // Takes about 1 second to generate
    _plot_compression_ratios("plots/compression_ratios_0_to_1_000_000.png", 0..1_000_000)
        .expect("Failed to plot compression ratios");
    // Takes about 9 seconds to generate
    _plot_compression_ratios(
        "plots/compression_ratios_0_to_10_000_000.png",
        0..10_000_000,
    )
    .expect("Failed to plot compression ratios");
    // Takes about 100 seconds and 22 GB of memory to generate
    _plot_compression_ratios(
        "plots/compression_ratios_0_to_100_000_000.png",
        0..100_000_000,
    )
    .expect("Failed to plot compression ratios");
    // // ⚠️ Unable to plot 1 billion inputs because it takes too long and uses too much memory. The process was killed by the OS (exit code 137) after about an hour and using 190 GB of memory + swap space.
    // _plot_compression_ratios(
    //     "plots/compression_ratios_0_to_1_000_000_000.png",
    //     0..1_000_000_000,
    // )
    // .expect("Failed to plot compression ratios");
}

// Plot histogram of compressed bit counts from random 64-bit integers
fn _plot_all_histograms() {
    // Takes a few seconds to generate
    for i in 3..=6 {
        let samples = 10_u64.pow(i as u32);
        _plot_compressed_bits_histogram(
            &format!("plots/compressed_bits_histogram_{samples}_random_u64s.png"),
            samples as usize,
        )
        .expect("Failed to plot compressed bits histogram");
    }
}

fn plot_fibonacci_numbers(
    filename: &str,
    range: std::ops::Range<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("Plotting Fibonacci numbers for range {:?}", range);
    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    // Find the maximum Fibonacci value in the range to set the log scale upper bound
    let max_fib = range
        .clone()
        .map(|i| {
            let fib = memoized_fast_doubling_fibonacci_biguint(i);
            biguint_to_u64(&fib)
        })
        .max()
        .unwrap_or(1) as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Fibonacci Numbers (Log Scale)",
            ("sans-serif", CAPTION_FONT_SIZE).into_font(),
        )
        .margin(CHART_MARGIN)
        .x_label_area_size(200)
        .y_label_area_size(300)
        .build_cartesian_2d(
            range.start as f64..range.end as f64,
            (1f64..max_fib).log_scale(),
        )?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    chart
        .configure_mesh()
        .x_desc("Fibonacci Index")
        .y_desc("Fibonacci Number")
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Filter out zero values since log(0) is undefined
    let data: Vec<(f64, f64)> = range
        .clone()
        .map(|i| {
            let fib = memoized_fast_doubling_fibonacci_biguint(i);
            let fib_u64 = biguint_to_u64(&fib);
            (i as f64, fib_u64 as f64)
        })
        .filter(|(_, y)| *y > 0.0)
        .collect();

    // Draw the line
    chart
        .draw_series(LineSeries::new(
            data.iter().copied(),
            RED.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Fibonacci Numbers")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                RED.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw dots at each point
    chart.draw_series(
        data.iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, RED.filled())),
    )?;

    // Draw text labels above each point showing x,y coordinates
    for (x, y) in &data {
        let label = format!("({:.0}, {:.0})", x, y);
        let text_x = *x + 0.3;
        let text_y = *y * 1.0;
        chart.draw_series(std::iter::once(Text::new(
            label,
            (text_x, text_y),
            ("sans-serif", POINT_LABEL_FONT_SIZE).into_font(),
        )))?;
    }

    chart
        .configure_series_labels()
        .margin(LEGEND_MARGIN)
        .label_font(("sans-serif", LEGEND_FONT_SIZE).into_font())
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    println!("Fibonacci plot saved to {}", filename);
    let end_time = Instant::now();
    println!(
        "Time taken to plot Fibonacci numbers for range {:?}: {:?}",
        range,
        end_time.duration_since(start_time)
    );
    Ok(())
}

/// Plots three number sequences on a log scale: Fibonacci numbers, binary numbers (2^n), and all-ones Zeckendorf numbers.
///
/// This function creates a comparison plot showing how these three different number sequences grow:
/// - **Fibonacci numbers**: F(n) where n is the Fibonacci index
/// - **Binary numbers**: 2^n where n is the exponent
/// - **All-ones Zeckendorf numbers**: Numbers with n ones in their Zeckendorf representation
///
/// The "all-ones" Zeckendorf numbers are created by generating a Zeckendorf representation with n consecutive
/// ones (in the Effective Zeckendorf Bits Ascending format), then converting that representation back to
/// the actual number value. This is useful for understanding how Zeckendorf representations behave
/// when they contain many ones.
///
/// The plot uses a logarithmic scale on the y-axis to better visualize the growth patterns of these sequences.
/// Each series is displayed with a different color and includes both lines and dots at each data point.
///
/// # Arguments
///
/// * `filename` - The path where the plot image will be saved (e.g., "plots/comparison.png")
/// * `range` - The range of input values n to plot (e.g., 0..31)
///
/// # Returns
///
/// Returns `Ok(())` if the plot was successfully created, or an error if plotting failed.
///
/// # Examples
///
/// ```
/// plot_fibonacci_binary_all_ones("plots/comparison_0_to_30.png", 0..31)?;
/// ```
fn plot_fibonacci_binary_all_ones(
    filename: &str,
    range: std::ops::Range<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!(
        "Plotting Fibonacci, binary, and all-ones Zeckendorf numbers for range {:?}",
        range
    );

    // Prepare Fibonacci data
    let fibonacci_data: Vec<(f64, f64)> = range
        .clone()
        .filter_map(|i| {
            let fib = memoized_fast_doubling_fibonacci_biguint(i);
            let fib_f64 = biguint_to_approximate_f64(&fib);
            if fib_f64 > 0.0 && fib_f64.is_finite() {
                Some((i as f64, fib_f64))
            } else {
                None
            }
        })
        .collect();

    // Prepare binary data (2^n)
    let binary_data: Vec<(f64, f64)> = range
        .clone()
        .map(|i| {
            let binary_value = 2_f64.powi(i as i32);
            (i as f64, binary_value)
        })
        .filter(|(_, y)| *y > 0.0 && y.is_finite())
        .collect();

    // Prepare all-ones Zeckendorf data
    let all_ones_data: Vec<(f64, f64)> = range
        .clone()
        .filter_map(|i| {
            if i == 0 {
                return None; // Skip 0 as it would result in an empty Zeckendorf representation
            }
            let all_ones_biguint = all_ones_zeckendorf_to_biguint(i as usize);
            let all_ones_f64 = biguint_to_approximate_f64(&all_ones_biguint);
            if all_ones_f64 > 0.0 && all_ones_f64.is_finite() {
                Some((i as f64, all_ones_f64))
            } else {
                None
            }
        })
        .collect();

    // Find the maximum value from all three series for y-axis range
    let max_value = fibonacci_data
        .iter()
        .chain(binary_data.iter())
        .chain(all_ones_data.iter())
        .map(|(_, y)| *y)
        .fold(1.0f64, |acc, y| acc.max(y));

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Fibonacci, Binary, and All-Ones Zeckendorf Numbers (Log Scale)",
            ("sans-serif", CAPTION_FONT_SIZE).into_font(),
        )
        .margin(CHART_MARGIN)
        .x_label_area_size(260)
        .y_label_area_size(300)
        .build_cartesian_2d(
            range.start as f64..range.end as f64,
            (1f64..max_value).log_scale(),
        )?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    // Custom formatter for y-axis labels in scientific notation
    // Example: 1000000 -> 1e6
    let y_label_formatter = |y: &f64| {
        if *y == 0.0 {
            "0".to_string()
        } else {
            let exponent = y.log10().floor() as i32;
            let mantissa = y / 10_f64.powi(exponent);
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
        .x_desc("Input n")
        .y_desc("Number Value (Log Scale)")
        .y_label_formatter(&y_label_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Draw Fibonacci series
    chart
        .draw_series(LineSeries::new(
            fibonacci_data.iter().copied(),
            RED.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Fibonacci Numbers F(n)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                RED.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw binary series
    chart
        .draw_series(LineSeries::new(
            binary_data.iter().copied(),
            BLUE.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Binary Numbers 2^n")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                BLUE.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw all-ones Zeckendorf series
    chart
        .draw_series(LineSeries::new(
            all_ones_data.iter().copied(),
            GREEN.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("All-Ones Zeckendorf (n ones)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                GREEN.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw dots at each point for Fibonacci
    chart.draw_series(
        fibonacci_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, RED.filled())),
    )?;

    // Draw dots at each point for binary
    chart.draw_series(
        binary_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, BLUE.filled())),
    )?;

    // Draw dots at each point for all-ones
    chart.draw_series(
        all_ones_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, GREEN.filled())),
    )?;

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .margin(LEGEND_MARGIN)
        .label_font(("sans-serif", LEGEND_FONT_SIZE).into_font())
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    println!(
        "Fibonacci, binary, and all-ones Zeckendorf plot saved to {}",
        filename
    );
    let end_time = Instant::now();
    println!(
        "Time taken to plot for range {:?}: {:?}",
        range,
        end_time.duration_since(start_time)
    );
    Ok(())
}

/// Plots four number sequences on a log scale: Fibonacci numbers, binary numbers (2^n), all-ones Zeckendorf numbers, and powers of 3 (3^n).
///
/// This function creates a comparison plot showing how these four different number sequences grow:
/// - **Fibonacci numbers**: F(n) where n is the Fibonacci index
/// - **Binary numbers**: 2^n where n is the exponent
/// - **All-ones Zeckendorf numbers**: Numbers with n ones in their Zeckendorf representation
/// - **Powers of 3**: 3^n where n is the exponent
///
/// The "all-ones" Zeckendorf numbers are created by generating a Zeckendorf representation with n consecutive
/// ones (in the Effective Zeckendorf Bits Ascending format), then converting that representation back to
/// the actual number value. This is useful for understanding how Zeckendorf representations behave
/// when they contain many ones.
///
/// The plot uses a logarithmic scale on the y-axis to better visualize the growth patterns of these sequences.
/// Each series is displayed with a different color and includes both lines and dots at each data point.
///
/// # Arguments
///
/// * `filename` - The path where the plot image will be saved (e.g., "plots/comparison.png")
/// * `range` - The range of input values n to plot (e.g., 0..31)
///
/// # Returns
///
/// Returns `Ok(())` if the plot was successfully created, or an error if plotting failed.
///
/// # Examples
///
/// ```
/// plot_fibonacci_binary_all_ones_power3("plots/comparison_0_to_30.png", 0..31)?;
/// ```
fn plot_fibonacci_binary_all_ones_power3(
    filename: &str,
    range: std::ops::Range<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!(
        "Plotting Fibonacci, binary, all-ones Zeckendorf, and 3^n numbers for range {:?}",
        range
    );

    // Prepare Fibonacci data
    let fibonacci_data: Vec<(f64, f64)> = range
        .clone()
        .filter_map(|i| {
            let fib = memoized_fast_doubling_fibonacci_biguint(i);
            let fib_f64 = biguint_to_approximate_f64(&fib);
            if fib_f64 > 0.0 && fib_f64.is_finite() {
                Some((i as f64, fib_f64))
            } else {
                None
            }
        })
        .collect();

    // Prepare binary data (2^n)
    let binary_data: Vec<(f64, f64)> = range
        .clone()
        .map(|i| {
            let binary_value = 2_f64.powi(i as i32);
            (i as f64, binary_value)
        })
        .filter(|(_, y)| *y > 0.0 && y.is_finite())
        .collect();

    // Prepare all-ones Zeckendorf data
    let all_ones_data: Vec<(f64, f64)> = range
        .clone()
        .filter_map(|i| {
            if i == 0 {
                return None; // Skip 0 as it would result in an empty Zeckendorf representation
            }
            let all_ones_biguint = all_ones_zeckendorf_to_biguint(i as usize);
            let all_ones_f64 = biguint_to_approximate_f64(&all_ones_biguint);
            if all_ones_f64 > 0.0 && all_ones_f64.is_finite() {
                Some((i as f64, all_ones_f64))
            } else {
                None
            }
        })
        .collect();

    // Prepare power of 3 data (3^n)
    let power3_data: Vec<(f64, f64)> = range
        .clone()
        .map(|i| {
            let power3_value = 3_f64.powi(i as i32);
            (i as f64, power3_value)
        })
        .filter(|(_, y)| *y > 0.0 && y.is_finite())
        .collect();

    // Find the maximum value from all four series for y-axis range
    let max_value = fibonacci_data
        .iter()
        .chain(binary_data.iter())
        .chain(all_ones_data.iter())
        .chain(power3_data.iter())
        .map(|(_, y)| *y)
        .fold(1.0f64, |acc, y| acc.max(y));

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Fibonacci, Binary, All-Ones Zeckendorf, and 3^n Numbers (Log Scale)",
            ("sans-serif", CAPTION_FONT_SIZE).into_font(),
        )
        .margin(CHART_MARGIN)
        .x_label_area_size(260)
        .y_label_area_size(300)
        .build_cartesian_2d(
            range.start as f64..range.end as f64,
            (1f64..max_value).log_scale(),
        )?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    // Custom formatter for y-axis labels in scientific notation
    // Example: 1000000 -> 1e6
    let y_label_formatter = |y: &f64| {
        if *y == 0.0 {
            "0".to_string()
        } else {
            let exponent = y.log10().floor() as i32;
            let mantissa = y / 10_f64.powi(exponent);
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
        .x_desc("Input n")
        .y_desc("Number Value (Log Scale)")
        .y_label_formatter(&y_label_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Draw Fibonacci series
    chart
        .draw_series(LineSeries::new(
            fibonacci_data.iter().copied(),
            RED.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Fibonacci Numbers F(n)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                RED.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw binary series
    chart
        .draw_series(LineSeries::new(
            binary_data.iter().copied(),
            BLUE.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Binary Numbers 2^n")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                BLUE.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw all-ones Zeckendorf series
    chart
        .draw_series(LineSeries::new(
            all_ones_data.iter().copied(),
            GREEN.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("All-Ones Zeckendorf (n ones)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                GREEN.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw power of 3 series
    chart
        .draw_series(LineSeries::new(
            power3_data.iter().copied(),
            MAGENTA.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Powers of 3 (3^n)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                MAGENTA.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw dots at each point for Fibonacci
    chart.draw_series(
        fibonacci_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, RED.filled())),
    )?;

    // Draw dots at each point for binary
    chart.draw_series(
        binary_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, BLUE.filled())),
    )?;

    // Draw dots at each point for all-ones
    chart.draw_series(
        all_ones_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, GREEN.filled())),
    )?;

    // Draw dots at each point for power of 3
    chart.draw_series(
        power3_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, MAGENTA.filled())),
    )?;

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .margin(LEGEND_MARGIN)
        .label_font(("sans-serif", LEGEND_FONT_SIZE).into_font())
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    println!(
        "Fibonacci, binary, all-ones Zeckendorf, and 3^n plot saved to {}",
        filename
    );
    let end_time = Instant::now();
    println!(
        "Time taken to plot for range {:?}: {:?}",
        range,
        end_time.duration_since(start_time)
    );
    Ok(())
}

/// Plots six number sequences on a log scale: Fibonacci numbers, binary numbers (2^n), all-ones Zeckendorf numbers, powers of 3 (3^n), phi to the n (φⁿ), and phi squared to the n (φ²ⁿ).
///
/// This function creates a comparison plot showing how these six different number sequences grow:
/// - **Fibonacci numbers**: F(n) where n is the Fibonacci index
/// - **Binary numbers**: 2^n where n is the exponent
/// - **All-ones Zeckendorf numbers**: Numbers with n ones in their Zeckendorf representation
/// - **Powers of 3**: 3^n where n is the exponent
/// - **Phi to the n**: φⁿ where φ is the golden ratio (approximately 1.618)
/// - **Phi squared to the n**: φ²ⁿ where φ is the golden ratio (approximately 1.618) and φ² ≈ 2.618
///
/// The "all-ones" Zeckendorf numbers are created by generating a Zeckendorf representation with n consecutive
/// ones (in the Effective Zeckendorf Bits Ascending format), then converting that representation back to
/// the actual number value. This is useful for understanding how Zeckendorf representations behave
/// when they contain many ones.
///
/// I discovered that the ratio of the all ones Zeckendorf numbers to the previous all ones Zeckendorf numbers converges to the golden ratio squared, so this plot is useful for seeing how close these numbers are by comparison.
///
/// The plot uses a logarithmic scale on the y-axis to better visualize the growth patterns of these sequences.
/// Each series is displayed with a different color and includes both lines and dots at each data point.
///
/// # Arguments
///
/// * `filename` - The path where the plot image will be saved (e.g., "plots/comparison.png")
/// * `range` - The range of input values n to plot (e.g., 0..31)
///
/// # Returns
///
/// Returns `Ok(())` if the plot was successfully created, or an error if plotting failed.
///
/// # Examples
///
/// ```
/// plot_fibonacci_binary_all_ones_power3_phi_phi_squared("plots/comparison_0_to_30.png", 0..31)?;
/// ```
fn plot_fibonacci_binary_all_ones_power3_phi_phi_squared(
    filename: &str,
    range: std::ops::Range<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!(
        "Plotting Fibonacci, binary, all-ones Zeckendorf, 3^n, φⁿ, and φ²ⁿ numbers for range {:?}",
        range
    );

    // Prepare Fibonacci data
    let fibonacci_data: Vec<(f64, f64)> = range
        .clone()
        .filter_map(|i| {
            let fib = memoized_fast_doubling_fibonacci_biguint(i);
            let fib_f64 = biguint_to_approximate_f64(&fib);
            if fib_f64 > 0.0 && fib_f64.is_finite() {
                Some((i as f64, fib_f64))
            } else {
                None
            }
        })
        .collect();

    // Prepare binary data (2^n)
    let binary_data: Vec<(f64, f64)> = range
        .clone()
        .map(|i| {
            let binary_value = 2_f64.powi(i as i32);
            (i as f64, binary_value)
        })
        .filter(|(_, y)| *y > 0.0 && y.is_finite())
        .collect();

    // Prepare all-ones Zeckendorf data
    let all_ones_data: Vec<(f64, f64)> = range
        .clone()
        .filter_map(|i| {
            if i == 0 {
                return None; // Skip 0 as it would result in an empty Zeckendorf representation
            }
            let all_ones_biguint = all_ones_zeckendorf_to_biguint(i as usize);
            let all_ones_f64 = biguint_to_approximate_f64(&all_ones_biguint);
            if all_ones_f64 > 0.0 && all_ones_f64.is_finite() {
                Some((i as f64, all_ones_f64))
            } else {
                None
            }
        })
        .collect();

    // Prepare power of 3 data (3^n)
    let power3_data: Vec<(f64, f64)> = range
        .clone()
        .map(|i| {
            let power3_value = 3_f64.powi(i as i32);
            (i as f64, power3_value)
        })
        .filter(|(_, y)| *y > 0.0 && y.is_finite())
        .collect();

    // Prepare phi to the n data (φⁿ)
    let phi_data: Vec<(f64, f64)> = range
        .clone()
        .map(|i| {
            let phi_value = PHI.powi(i as i32);
            (i as f64, phi_value)
        })
        .filter(|(_, y)| *y > 0.0 && y.is_finite())
        .collect();

    // Prepare phi squared to the n data (φ²ⁿ)
    let phi_squared_data: Vec<(f64, f64)> = range
        .clone()
        .map(|i| {
            let phi_squared_value = PHI_SQUARED.powi(i as i32);
            (i as f64, phi_squared_value)
        })
        .filter(|(_, y)| *y > 0.0 && y.is_finite())
        .collect();

    // Find the maximum value from all six series for y-axis range
    let max_value = fibonacci_data
        .iter()
        .chain(binary_data.iter())
        .chain(all_ones_data.iter())
        .chain(power3_data.iter())
        .chain(phi_data.iter())
        .chain(phi_squared_data.iter())
        .map(|(_, y)| *y)
        .fold(1.0f64, |acc, y| acc.max(y));

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Various Number Sequences (Log Scale)",
            ("sans-serif", CAPTION_FONT_SIZE).into_font(),
        )
        .margin(CHART_MARGIN)
        .x_label_area_size(260)
        .y_label_area_size(300)
        .build_cartesian_2d(
            range.start as f64..range.end as f64,
            (1f64..max_value).log_scale(),
        )?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    // Custom formatter for y-axis labels in scientific notation
    // Example: 1000000 -> 1e6
    let y_label_formatter = |y: &f64| {
        if *y == 0.0 {
            "0".to_string()
        } else {
            let exponent = y.log10().floor() as i32;
            let mantissa = y / 10_f64.powi(exponent);
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
        .x_desc("Input n")
        .y_desc("Number Value (Log Scale)")
        .y_label_formatter(&y_label_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Draw power of 3 series
    chart
        .draw_series(LineSeries::new(
            power3_data.iter().copied(),
            MAGENTA.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Powers of 3 (3^n)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                MAGENTA.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw phi squared to the n series
    chart
        .draw_series(LineSeries::new(
            phi_squared_data.iter().copied(),
            CYAN.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Phi Squared to the n (φ²ⁿ)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                CYAN.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw all-ones Zeckendorf series
    chart
        .draw_series(LineSeries::new(
            all_ones_data.iter().copied(),
            GREEN.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("All-Ones Zeckendorf (n ones)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                GREEN.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw binary series
    chart
        .draw_series(LineSeries::new(
            binary_data.iter().copied(),
            BLUE.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Binary Numbers 2^n")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                BLUE.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw phi to the n series
    chart
        .draw_series(LineSeries::new(
            phi_data.iter().copied(),
            BLACK.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Phi to the n (φⁿ)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                BLACK.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw Fibonacci series
    chart
        .draw_series(LineSeries::new(
            fibonacci_data.iter().copied(),
            RED.stroke_width(SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Fibonacci Numbers F(n)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                RED.stroke_width(SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw dots at each point for Fibonacci
    chart.draw_series(
        fibonacci_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, RED.filled())),
    )?;

    // Draw dots at each point for binary
    chart.draw_series(
        binary_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, BLUE.filled())),
    )?;

    // Draw dots at each point for all-ones
    chart.draw_series(
        all_ones_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, GREEN.filled())),
    )?;

    // Draw dots at each point for power of 3
    chart.draw_series(
        power3_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, MAGENTA.filled())),
    )?;

    // Draw dots at each point for phi
    chart.draw_series(
        phi_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, BLACK.filled())),
    )?;

    // Draw dots at each point for phi squared
    chart.draw_series(
        phi_squared_data
            .iter()
            .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, CYAN.filled())),
    )?;

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .margin(LEGEND_MARGIN)
        .label_font(("sans-serif", LEGEND_FONT_SIZE).into_font())
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    println!(
        "Fibonacci, binary, all-ones Zeckendorf, 3^n, φⁿ, and φ²ⁿ plot saved to {}",
        filename
    );
    let end_time = Instant::now();
    println!(
        "Time taken to plot for range {:?}: {:?}",
        range,
        end_time.duration_since(start_time)
    );
    Ok(())
}

fn _plot_compression_ratios(
    filename: &str,
    range: std::ops::Range<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    use num_format::{Locale, ToFormattedString};

    let start_time = Instant::now();
    println!("Plotting compression ratios for range {:?}", range);
    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "Zeckendorf Compression Ratios from {} to {}",
                range.start,
                range.end.to_formatted_string(&Locale::en)
            ),
            ("sans-serif", CAPTION_FONT_SIZE).into_font(),
        )
        .margin(CHART_MARGIN)
        .x_label_area_size(260)
        .y_label_area_size(260)
        .build_cartesian_2d(range.start as f64..range.end as f64, 0.0f64..2.0f64)?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    // Custom formatter for x-axis labels: use scientific notation for values >= 1000
    // Example: 1000 -> 1e3, 300000 -> 3e5
    let x_label_formatter = |x: &f64| {
        if *x >= 1000.0 {
            let exponent = x.log10().floor() as i32;
            let mantissa = x / 10_f64.powi(exponent);
            // Round mantissa to 1 decimal place if needed, otherwise show as integer
            let rounded_mantissa = mantissa.round();
            if (mantissa - rounded_mantissa).abs() < 1e-10 {
                format!("{}e{}", rounded_mantissa as i64, exponent)
            } else {
                format!("{:.1}e{}", mantissa, exponent)
            }
        } else {
            format!("{:.0}", x)
        }
    };

    chart
        .configure_mesh()
        .x_desc("Input Value")
        .y_desc("Compression Ratio (Compressed / Original)")
        .x_label_formatter(&x_label_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    let data: Vec<(f64, f64)> = range
        .clone()
        .filter_map(|i| {
            let original_number = BigUint::from(i);
            // println!("Original number: {:?}", original_number);
            // Calculate bits required to represent the original number
            let original_bit_size = original_number.bits() as f64;
            // println!("Original bit size: {:?}", original_bit_size);
            let data_bytes = original_number.to_bytes_be();
            // println!("Data bytes as big endian: {:?}", data_bytes);
            let compressed_as_zeckendorf_data =
                padless_zeckendorf_compress_be_dangerous(&data_bytes);
            // println!("Compressed: {:?}", compressed_as_zeckendorf_data);
            // Since the last step of the compression outputs the data with the least significant bits and bytes first, we need to interpret the data as little endian.
            let compressed_as_bigint = BigUint::from_bytes_le(&compressed_as_zeckendorf_data);
            // println!("Compressed as bigint: {:?}", compressed_as_bigint);
            // Calculate bits required to store the compressed representation
            let compressed_bit_size = compressed_as_bigint.bits() as f64;
            // println!("Compressed bit size: {:?}", compressed_bit_size);
            if original_bit_size > 0.0 {
                Some((i as f64, compressed_bit_size / original_bit_size))
            } else {
                None
            }
        })
        .collect();

    const THINNER_SERIES_LINE_STROKE_WIDTH: u32 = 1;
    chart
        .draw_series(LineSeries::new(
            data,
            BLUE.stroke_width(THINNER_SERIES_LINE_STROKE_WIDTH),
        ))?
        .label("Compression Ratio")
        .legend(|(x, y)| {
            PathElement::new(
                vec![
                    (x - LEGEND_PATH_LEFT_OFFSET, y),
                    (x + LEGEND_PATH_RIGHT_OFFSET, y),
                ],
                BLUE.stroke_width(THINNER_SERIES_LINE_STROKE_WIDTH),
            )
        });

    // Draw a line at ratio 1.0 (no compression benefit)
    chart.draw_series(LineSeries::new(
        vec![(range.start as f64, 1.0), (range.end as f64, 1.0)],
        GREEN.mix(0.5).stroke_width(SERIES_LINE_STROKE_WIDTH),
    ))?;

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .margin(LEGEND_MARGIN)
        .label_font(("sans-serif", LEGEND_FONT_SIZE).into_font())
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    println!("Compression ratio plot saved to {}", filename);
    let end_time = Instant::now();
    println!(
        "Time taken to plot compression ratios for range {:?}: {:?}",
        range,
        end_time.duration_since(start_time)
    );
    Ok(())
}

/// Helper function to convert Arc<BigUint> to u64 for plotting.
/// Panics if the value doesn't fit in u64.
fn biguint_to_u64(value: &Arc<BigUint>) -> u64 {
    let digits = value.to_u64_digits();
    if digits.len() == 1 {
        digits[0]
    } else if digits.is_empty() {
        0
    } else {
        panic!("Fibonacci value too large to fit in u64");
    }
}

/// Helper function to convert BigUint to f64 for plotting.
/// For values that don't fit in f64, uses an approximation based on bits, but capped at 1023 bits to avoid overflow.
fn biguint_to_approximate_f64(value: &BigUint) -> f64 {
    // Try to convert to u64 first
    let digits = value.to_u64_digits();
    if digits.len() == 1 {
        digits[0] as f64
    } else if digits.is_empty() {
        0.0
    } else {
        // For very large numbers, approximate using bits
        // We'll use: value ≈ 2^bits, but cap to avoid overflow
        let bits = value.bits() as f64;
        // f64::MAX is around 1.8e308, which corresponds to 2^1024 - 1
        // So we cap bits at 1023 to avoid overflow
        let capped_bits = bits.min(1023.0);
        2_f64.powf(capped_bits)
    }
}

/// Calculates the mean (average) of a slice of u64 values.
///
/// # Arguments
///
/// * `values` - A slice of u64 values
///
/// # Returns
///
/// Returns the mean as an f64, or 0.0 if the slice is empty.
///
/// # Examples
///
/// ```
/// # fn calculate_mean(values: &[u64]) -> f64 {
/// #     if values.is_empty() { return 0.0; }
/// #     values.iter().sum::<u64>() as f64 / values.len() as f64
/// # }
/// assert_eq!(calculate_mean(&[1, 2, 3, 4, 5]), 3.0);
/// assert_eq!(calculate_mean(&[10, 20, 30]), 20.0);
/// assert_eq!(calculate_mean(&[]), 0.0);
/// ```
fn _calculate_mean(values: &[u64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<u64>() as f64 / values.len() as f64
}

/// Calculates the median of a slice of u64 values.
///
/// # Arguments
///
/// * `values` - A slice of u64 values
///
/// # Returns
///
/// Returns the median as an f64, or 0.0 if the slice is empty.
///
/// # Examples
///
/// ```
/// # fn calculate_median(values: &[u64]) -> f64 {
/// #     if values.is_empty() { return 0.0; }
/// #     let mut sorted_values = values.to_vec();
/// #     sorted_values.sort();
/// #     let mid = sorted_values.len() / 2;
/// #     if sorted_values.len() % 2 == 0 {
/// #         (sorted_values[mid - 1] + sorted_values[mid]) as f64 / 2.0
/// #     } else {
/// #         sorted_values[mid] as f64
/// #     }
/// # }
/// // Odd number of elements
/// assert_eq!(calculate_median(&[1, 2, 3, 4, 5]), 3.0);
/// // Even number of elements (average of middle two)
/// assert_eq!(calculate_median(&[1, 2, 3, 4]), 2.5);
/// assert_eq!(calculate_median(&[10, 20, 30, 40]), 25.0);
/// assert_eq!(calculate_median(&[]), 0.0);
/// ```
fn _calculate_median(values: &[u64]) -> f64 {
    // println!("Calculating median of {:?}", values);
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted_values = values.to_vec();
    sorted_values.sort();
    let mid = sorted_values.len() / 2;
    // println!("Mid index: {:?}", mid);
    if sorted_values.len().is_multiple_of(2) {
        // println!("Even number of elements, returning average of middle two: {:?}", (sorted_values[mid - 1] + sorted_values[mid]) as f64 / 2.0);
        (sorted_values[mid - 1] + sorted_values[mid]) as f64 / 2.0
    } else {
        // println!("Odd number of elements, returning middle element: {:?}", sorted_values[mid] as f64);
        sorted_values[mid] as f64
    }
}

/// Calculates the standard deviation of a slice of u64 values.
///
/// # Arguments
///
/// * `values` - A slice of u64 values
/// * `mean` - The mean of the values (pre-calculated for efficiency)
///
/// # Returns
///
/// Returns the standard deviation as an f64, or 0.0 if the slice is empty or has only one element.
///
/// # Examples
///
/// ```
/// # fn calculate_mean(values: &[u64]) -> f64 {
/// #     if values.is_empty() { return 0.0; }
/// #     values.iter().sum::<u64>() as f64 / values.len() as f64
/// # }
/// # fn calculate_standard_deviation(values: &[u64], mean: f64) -> f64 {
/// #     if values.is_empty() || values.len() == 1 { return 0.0; }
/// #     let variance = values.iter().map(|&v| { let d = v as f64 - mean; d * d }).sum::<f64>() / (values.len() - 1) as f64;
/// #     variance.sqrt()
/// # }
/// let values = &[2, 4, 4, 4, 5, 5, 7, 9];
/// let mean = calculate_mean(values);
/// let std_dev = calculate_standard_deviation(values, mean);
/// // Standard deviation should be approximately 2.0 for this dataset
/// assert!((std_dev - 2.0).abs() < 0.1);
/// assert_eq!(calculate_standard_deviation(&[], 0.0), 0.0);
/// assert_eq!(calculate_standard_deviation(&[5], 5.0), 0.0);
/// ```
fn _calculate_standard_deviation(values: &[u64], mean: f64) -> f64 {
    if values.is_empty() || values.len() == 1 {
        return 0.0;
    }
    let variance = values
        .iter()
        .map(|&value| {
            let diff = value as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / (values.len() - 1) as f64;
    variance.sqrt()
}

/// Draws a text box with multiple lines of text on a chart. Uses monospace font.
///
/// This function draws a styled text box containing multiple lines of text. The box size is
/// automatically calculated based on the number of lines provided.
///
/// # Arguments
///
/// * `chart` - The chart context to draw on
/// * `lines` - A slice of strings to display, one per line
/// * `box_top_right` - The top-right corner position of the box (x, y) in chart coordinates
/// * `x_range` - The width of the x-axis range (used for proportional sizing)
/// * `y_max_val` - The maximum y value (used for proportional sizing)
/// * `box_width_fraction` - Width of the box as a fraction of x_range (e.g., 0.15 for 15%)
/// * `margin_right_fraction` - Right margin as a fraction of x_range
/// * `margin_top_fraction` - Top margin as a fraction of y_max_val
/// * `padding_x_fraction` - Internal horizontal padding as a fraction of x_range
/// * `padding_y_fraction` - Internal vertical padding as a fraction of y_max_val
/// * `line_height_fraction` - Height between lines as a fraction of y_max_val
/// * `font_size` - Font size for the text
///
/// # Returns
///
/// Returns `Ok(())` if the text box was successfully drawn, or an error if drawing failed.
#[allow(clippy::too_many_arguments)]
fn _draw_text_box<'a>(
    chart: &mut ChartContext<'a, BitMapBackend<'a>, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
    lines: &[String],
    box_top_right: (f64, f64),
    x_range: f64,
    y_max_val: f64,
    box_width_fraction: f64,
    margin_right_fraction: f64,
    margin_top_fraction: f64,
    padding_x_fraction: f64,
    padding_y_fraction: f64,
    line_height_fraction: f64,
    font_size: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    if lines.is_empty() {
        return Ok(());
    }

    let (x_max, _) = box_top_right;
    let num_lines = lines.len() as f64;

    // Calculate box position and size
    let box_width = x_range * box_width_fraction;
    let box_left = x_max - box_width - (x_range * margin_right_fraction);
    let box_right = x_max - (x_range * margin_right_fraction);
    let box_top = y_max_val - (y_max_val * margin_top_fraction);

    // Text positioning with padding
    let text_x = box_left + (x_range * padding_x_fraction);
    let line_height = y_max_val * line_height_fraction;
    let text_y_start = box_top - (y_max_val * padding_y_fraction);

    // Calculate box height based on content (number of lines + padding)
    let box_height = (y_max_val * padding_y_fraction * 2.0) + (line_height * num_lines);
    let box_bottom = box_top - box_height;

    // Draw background rectangle
    chart.draw_series(std::iter::once(Rectangle::new(
        [(box_left, box_bottom), (box_right, box_top)],
        WHITE.mix(0.95).filled(),
    )))?;

    // Draw border around the background
    chart.draw_series(std::iter::once(PathElement::new(
        vec![
            (box_left, box_bottom),
            (box_right, box_bottom),
            (box_right, box_top),
            (box_left, box_top),
            (box_left, box_bottom),
        ],
        BLACK.stroke_width(2),
    )))?;

    // Draw each line of text
    for (i, line) in lines.iter().enumerate() {
        let y_pos = text_y_start - (line_height * (i as f64));
        chart.draw_series(std::iter::once(Text::new(
            line.clone(),
            (text_x, y_pos),
            (FontFamily::Monospace, font_size).into_font(),
        )))?;
    }

    Ok(())
}

/// Plots a histogram of compressed bit counts from seeded random 64-bit unsigned integers.
///
/// This function:
/// 1. Generates a specified number of seeded random 64-bit unsigned integers (for repeatability)
/// 2. Converts them to BigUint
/// 3. Compresses them using `padless_zeckendorf_compress_le_dangerous`
/// 4. Reads the compressed data back into a new BigUint (little endian)
/// 5. Gets the number of bits using `.bits()`
/// 6. Plots a histogram with bucket sizes of 1
///
/// # Arguments
///
/// * `filename` - The path where the plot image will be saved (e.g., "plots/histogram.png")
/// * `count` - The number of random 64-bit integers to generate and compress
///
/// # Returns
///
/// Returns `Ok(())` if the plot was successfully created, or an error if plotting failed.
fn _plot_compressed_bits_histogram(
    filename: &str,
    count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!(
        "Plotting compressed bits histogram from {} random 64-bit integers",
        count
    );

    // Create seeded RNG for repeatability
    let seed = [42u8; 32]; // Fixed seed for repeatability
    let mut rng = StdRng::from_seed(seed);

    // Generate random 64-bit unsigned integers
    let random_numbers: Vec<u64> = (0..count).map(|_| rng.random::<u64>()).collect();

    // Convert to BigUint, compress, and collect bit counts
    let mut bit_counts: Vec<u64> = Vec::with_capacity(count);
    for number in &random_numbers {
        let biguint = BigUint::from(*number);
        let data_bytes = biguint.to_bytes_le();
        let compressed_data = padless_zeckendorf_compress_le_dangerous(&data_bytes);
        let compressed_biguint = BigUint::from_bytes_le(&compressed_data);
        let bits = compressed_biguint.bits();
        bit_counts.push(bits);
    }

    // Create histogram buckets (bucket size 1)
    let min_bits = *bit_counts.iter().min().unwrap_or(&0);
    let max_bits = *bit_counts.iter().max().unwrap_or(&0);
    let bucket_count = (max_bits - min_bits + 1) as usize;

    let mut histogram: Vec<u64> = vec![0; bucket_count];
    for &bits in &bit_counts {
        let bucket_index = (bits - min_bits) as usize;
        histogram[bucket_index] += 1;
    }

    // Find max frequency for y-axis
    let max_frequency = *histogram.iter().max().unwrap_or(&1) as f64;

    // Calculate statistics
    let mean = _calculate_mean(&bit_counts);
    let median = _calculate_median(&bit_counts);
    let std_dev = _calculate_standard_deviation(&bit_counts, mean);

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "Histogram of Compressed Bit Counts ({count} Random u64's)",
                count = count.to_formatted_string(&Locale::en)
            ),
            ("sans-serif", 110.0).into_font(),
        )
        .margin(CHART_MARGIN)
        .x_label_area_size(260)
        .y_label_area_size(300)
        .build_cartesian_2d(
            (min_bits as f64 - 0.5)..(max_bits as f64 + 0.5),
            0.0..(max_frequency * 1.1),
        )?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    // Create formatters for x and y axes with 0 decimal places and comma separators
    let axis_label_formatter = |value: &f64| {
        let rounded = value.round() as u64;
        rounded.to_formatted_string(&Locale::en)
    };

    chart
        .configure_mesh()
        .x_desc("Compressed Bit Count")
        .y_desc("Frequency")
        .x_label_formatter(&axis_label_formatter)
        .y_label_formatter(&axis_label_formatter)
        .label_style(axis_tick_style)
        .axis_desc_style(axis_label_style)
        .draw()?;

    // Draw histogram bars with gaps between them
    const BAR_WIDTH: f64 = 0.8; // Width of each bar (less than 1.0 to create gaps)
    const BAR_GAP: f64 = (1.0 - BAR_WIDTH) / 2.0; // Gap on each side of the bar

    for (bucket_index, &frequency) in histogram.iter().enumerate() {
        if frequency > 0 {
            let bits_value = min_bits + bucket_index as u64;
            let bar_left = bits_value as f64 - 0.5 + BAR_GAP;
            let bar_right = bits_value as f64 - 0.5 + BAR_GAP + BAR_WIDTH;
            let bar_height = frequency as f64;

            // Draw filled rectangle using Rectangle
            // Rectangle takes bottom-left and top-right corners
            // FIXME: refactor this to use the idiomatic Histogram API; example: https://github.com/plotters-rs/plotters/blob/0f195eadaac7d9a2390a3707fbe192f8e2645d34/plotters/examples/histogram.rs
            // Refactoring will also require refactoring the legend box drawing code, since the Histogram chart can't `.drawSeries(...)`.
            chart.draw_series(std::iter::once(Rectangle::new(
                [(bar_left, 0.0), (bar_right, bar_height)],
                BLUE.filled(),
            )))?;
        }
    }

    // Add statistics legend in top right
    // Calculate position in top right (using chart coordinates)
    // Vibey numbers 😬
    let x_max = max_bits as f64 + 0.5;
    let y_max = max_frequency * 1.1;
    let x_range = x_max - (min_bits as f64 - 10.0);

    // Prepare statistics lines
    let stats_lines = vec![
        format!("Mean:   {:.2}", mean),
        format!("Median: {:.2}", median),
        format!("Std:    {:.2}", std_dev),
        format!("Min:    {:.0}", min_bits),
        format!("Max:    {:.0}", max_bits),
    ];

    // Draw text box with statistics
    _draw_text_box(
        &mut chart,
        &stats_lines,
        (x_max, y_max),
        x_range,
        y_max,
        0.15,  // box_width_fraction
        0.025, // margin_right_fraction
        0.015, // margin_top_fraction
        0.015, // padding_x_fraction
        0.015, // padding_y_fraction
        0.04,  // line_height_fraction
        LEGEND_FONT_SIZE,
    )?;

    root.present()?;
    println!("Compressed bits histogram saved to {}", filename);
    let end_time = Instant::now();
    println!(
        "Time taken to plot compressed bits histogram: {:?}",
        end_time.duration_since(start_time)
    );
    Ok(())
}

/// I discovered a new relation between certain square roots of Fibonacci numbers by looking and inspecting
/// the Zeckendorf Spiral at the website I made below:
/// Website: https://zeckendorf.lovable.app/
/// Source code: https://github.com/pRizz/zeckendorf-spiral
/// Namely:
/// Sqrt(Fibonacci(n)) ≈ Sqrt(Fibonacci(n + 4)) - Sqrt(Fibonacci(n + 2))
/// Alternatively, when we divide both sides by Sqrt(Fibonacci(n)):
/// 1 ≈ (Sqrt(Fibonacci(n + 4)) - Sqrt(Fibonacci(n + 2))) / Sqrt(Fibonacci(n))
/// In the graph for this function, we subtract 1.0 from both sides so that the numbers are more readable on the log graph
/// as they approach 0:
/// 0 ≈ (Sqrt(Fibonacci(n + 4)) - Sqrt(Fibonacci(n + 2))) / Sqrt(Fibonacci(n)) - 1.0
/// Alternatively, when we subtract Sqrt(Fibonacci(n)) from both sides, from the first equation:
/// 0 ≈ Sqrt(Fibonacci(n + 4)) - Sqrt(Fibonacci(n + 2)) - Sqrt(Fibonacci(n))
/// This ratio was found by noticing that the gaps in the middle of the Zeckendorf Spiral
/// shrink as the square roots of the zeckendorf squares get larger.
/// And I was thinking that there is probably a relationship between the sides of certain squares in the Zeckendorf Spiral,
/// which, when measured, gives the relation above.
/// The purpose of this plot is to visualize the error converging to 0 in the formula
/// above as we input larger and larger n values.
/// It is kinda fascinating that this produces a closed form equation for producing arbitrarily
/// small irrational numbers. It is irrational because the formula equates to only square roots of Fibonacci numbers.
/// Technically, there are perfect squares when the fibonacci is 1, 4, or 144, but those seem to be the only
/// ones and they only appear at the start of the spiral.
/// Also starting and stepping at different values produces different graphs.
#[derive(Clone, Copy, Debug)]
enum NValueStrategy {
    /// Start at 2, step by 2 (e.g., 2, 4, 6, 8, ...)
    /// This produces a graph with positive values, and it converges to 0.
    /// This is the default spiral pattern that is used at https://github.com/pRizz/zeckendorf-spiral
    Start2Step2,
    /// Start at 1, step by 2 (e.g., 1, 3, 5, 7, ...)
    /// This produces a graph with negative values, and it converges to 0.
    Start1Step2,
    /// Start at 1, step by 1 (e.g., 1, 2, 3, 4, ...)
    /// Fascinatingly, this produces a graph that oscillates between positive and negative values, and it converges to 0.
    /// It would be fun to generate a graph that oscillates between exactly 1 and -1 by dividing the formula
    /// by the square root of the square of the formula (produces the absolute value of the formula).
    /// I need to write an article about this lol.
    Start1Step1,
}

impl NValueStrategy {
    fn generate_n_values(&self, count: u64) -> Vec<u64> {
        match self {
            NValueStrategy::Start2Step2 => (1..=count).map(|i| i * 2).collect(),
            NValueStrategy::Start1Step2 => (0..count).map(|i| i * 2 + 1).collect(),
            NValueStrategy::Start1Step1 => (1..=count).collect(),
        }
    }
}

fn plot_fibonacci_square_root_error_convergence(
    filename: &str,
    log_y: bool,
    n_strategy: NValueStrategy,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("Plotting Fibonacci square root error convergence to {}", filename);

    // Generate n values based on the strategy
    let n_values: Vec<u64> = match n_strategy {
        NValueStrategy::Start2Step2 => n_strategy.generate_n_values(15),
        NValueStrategy::Start1Step2 => n_strategy.generate_n_values(15),
        NValueStrategy::Start1Step1 => n_strategy.generate_n_values(30),
    };

    // Calculate the ratio for each n
    let data: Vec<(f64, f64)> = n_values
        .iter()
        .filter_map(|&n| {
            // Calculate Fibonacci(n), Fibonacci(n+2), Fibonacci(n+4)
            let fib_n = memoized_fast_doubling_fibonacci_biguint(n);
            let fib_n_plus_2 = memoized_fast_doubling_fibonacci_biguint(n + 2);
            let fib_n_plus_4 = memoized_fast_doubling_fibonacci_biguint(n + 4);

            // Convert to f64
            let fib_n_f64 = biguint_to_approximate_f64(&fib_n);
            let fib_n_plus_2_f64 = biguint_to_approximate_f64(&fib_n_plus_2);
            let fib_n_plus_4_f64 = biguint_to_approximate_f64(&fib_n_plus_4);

            // Check for valid values after conversion to f64 (positive and finite)
            if fib_n_f64 <= 0.0
                || fib_n_plus_2_f64 <= 0.0
                || fib_n_plus_4_f64 <= 0.0
                || !fib_n_f64.is_finite()
                || !fib_n_plus_2_f64.is_finite()
                || !fib_n_plus_4_f64.is_finite()
            {
                return None;
            }

            // Calculate square roots
            let sqrt_fib_n = fib_n_f64.sqrt();
            let sqrt_fib_n_plus_2 = fib_n_plus_2_f64.sqrt();
            let sqrt_fib_n_plus_4 = fib_n_plus_4_f64.sqrt();

            if sqrt_fib_n == 0.0 {
                return None;
            }

            // First equation; but this seems to produce floating point errors when the numbers are large.
            // let ratio = (sqrt_fib_n_plus_4 - sqrt_fib_n_plus_2) / sqrt_fib_n - 1.0;
            // Second equation; this produces better results when the numbers are large.
            let ratio = sqrt_fib_n_plus_4 - sqrt_fib_n_plus_2 - sqrt_fib_n;

            if ratio.is_finite() {
                Some((n as f64, ratio))
            } else {
                None
            }
        })
        .collect();

    if data.is_empty() {
        return Err("No valid data points calculated".into());
    }

    // Find the min and max values for y-axis range
    let min_y = data
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::INFINITY, |acc, y| acc.min(y));
    println!("Min y: {}", min_y);
    let max_y = data
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, |acc, y| acc.max(y));
    println!("Max y: {}", max_y);

    // Determine y-axis range with some padding
    let y_padding = (max_y - min_y) * 0.1;
    let y_min = min_y - y_padding;
    println!("Calculated y min: {}", y_min);
    let y_max = max_y + y_padding;
    println!("Calculated y max: {}", y_max);

    let root = BitMapBackend::new(filename, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    let axis_label_style =
        TextStyle::from(("sans-serif", AXIS_FONT_SIZE).into_font()).color(&BLACK);
    let axis_tick_style =
        TextStyle::from(("sans-serif", AXIS_TICK_FONT_SIZE).into_font()).color(&BLACK);

    let y_label_formatter = |y: &f64| {
        if *y == 0.0 {
            return "0".to_string();
        }

        let abs_y = y.abs();
        if abs_y < 1e-12 {
            return "0".to_string();
        }

        if abs_y < 1e-3 {
            // Use scientific notation for small values
            let sign = if *y < 0.0 { "-" } else { "" };
            let exponent = abs_y.log10().floor() as i32;
            let mantissa = abs_y / 10_f64.powi(exponent);
            // Round mantissa to 1 decimal place if needed, otherwise show as integer
            let rounded_mantissa = mantissa.round();
            if (mantissa - rounded_mantissa).abs() < 1e-10 {
                return format!("{}{}e{}", sign, rounded_mantissa as i64, exponent);
            }
            return format!("{}{:.1}e{}", sign, mantissa, exponent);
        }

        // Use regular decimal notation for larger values, removing trailing zeros
        let formatted = format!("{:.3}", y);
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    };

    // Macro to draw the series (works with any chart type, log or linear y-axis; otherwise we get type errors when supplying
    // the y-axis range to the chart builder; linear y-axis and log y-axis ranges produce different types,
    // so we need to use a macro to avoid type errors and not duplicate code.
    macro_rules! draw_series_content {
        ($chart:ident) => {
            $chart
                .configure_mesh()
                .x_desc("n")
                .y_desc("Sqrt(F(n+4)) - Sqrt(F(n+2)) - Sqrt(F(n))")
                .y_label_formatter(&y_label_formatter)
                .label_style(axis_tick_style)
                .axis_desc_style(axis_label_style)
                .draw()?;

            // Draw the line
            $chart
                .draw_series(LineSeries::new(
                    data.iter().copied(),
                    BLUE.stroke_width(SERIES_LINE_STROKE_WIDTH),
                ))?
                .label("Ratio")
                .legend(|(x, y)| {
                    PathElement::new(
                        vec![
                            (x - LEGEND_PATH_LEFT_OFFSET, y),
                            (x + LEGEND_PATH_RIGHT_OFFSET, y),
                        ],
                        BLUE.stroke_width(SERIES_LINE_STROKE_WIDTH),
                    )
                });

            // Draw dots at each point
            $chart.draw_series(
                data.iter()
                    .map(|point| Circle::new(*point, SERIES_LINE_DOT_SIZE, BLUE.filled())),
            )?;

            $chart
                .configure_series_labels()
                .position(SeriesLabelPosition::UpperRight)
                .margin(LEGEND_MARGIN)
                .label_font(("sans-serif", LEGEND_FONT_SIZE).into_font())
                .background_style(WHITE.mix(0.8))
                .border_style(BLACK)
                .draw()?;
        };
    }

    let n_strategy_str = match n_strategy {
        NValueStrategy::Start2Step2 => "Start at 2, Step by 2",
        NValueStrategy::Start1Step2 => "Start at 1, Step by 2",
        NValueStrategy::Start1Step1 => "Start at 1, Step by 1",
    };
    // Build chart with conditional y-axis scaling
    if log_y {
        // Manually set the y-axis range to start at 1e-10 to have a nicer plot.
        let log_y_range = (y_min.max(1e-10)..y_max).log_scale();
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("Fibonacci Square Root Error Convergence ({n_strategy_str}) (Log Y-Axis)"),
                ("sans-serif", 100).into_font(),
            )
            .margin(CHART_MARGIN)
            .x_label_area_size(260)
            .y_label_area_size(300)
            .build_cartesian_2d(0.0f64..32.0f64, log_y_range)?;
        draw_series_content!(chart);
    } else {
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("Fibonacci Square Root Error Convergence ({n_strategy_str})"),
                ("sans-serif", 100).into_font(),
            )
            .margin(CHART_MARGIN)
            .x_label_area_size(260)
            .y_label_area_size(260)
            .build_cartesian_2d(0.0f64..32.0f64, y_min..y_max)?;
        draw_series_content!(chart);
    }

    root.present()?;
    let end_time = Instant::now();
    println!(
        "Time taken to plot Fibonacci square root error convergence to {}: {:?}",
        filename,
        end_time.duration_since(start_time)
    );
    Ok(())
}
