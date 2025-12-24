//! Binary for generating plots using plotters
//!
//! This binary requires the `plotting` feature to be enabled.
//! Run with: `cargo run --release --bin plot --features plotting`

use num_bigint::BigUint;
use plotters::prelude::*;
use std::time::Instant;
use zeckendorf_rs::*;

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
    plot_fibonacci_numbers("plots/fibonacci_plot.png", 0..30)
        .expect("Failed to plot Fibonacci numbers");

    // Example: Plot compression ratios
    plot_compression_ratios("plots/compression_ratios_0_to_100.png", 0..100)
        .expect("Failed to plot compression ratios");
    // plot_compression_ratios("plots/compression_ratios_0_to_257.png", 0..257)
    //     .expect("Failed to plot compression ratios");
    // plot_compression_ratios("plots/compression_ratios_0_to_1_000.png", 0..1_000)
    //     .expect("Failed to plot compression ratios");
    // plot_compression_ratios("plots/compression_ratios_0_to_10_000.png", 0..10_000)
    //     .expect("Failed to plot compression ratios");
    // plot_compression_ratios("plots/compression_ratios_0_to_100_000.png", 0..100_000)
    //     .expect("Failed to plot compression ratios");
    // // Takes about 1 second to generate
    // plot_compression_ratios("plots/compression_ratios_0_to_1_000_000.png", 0..1_000_000)
    //     .expect("Failed to plot compression ratios");
    // // Takes about 9 seconds to generate
    // plot_compression_ratios(
    //     "plots/compression_ratios_0_to_10_000_000.png",
    //     0..10_000_000,
    // )
    // .expect("Failed to plot compression ratios");
    // // Takes about 100 seconds and 22 GB of memory to generate
    // plot_compression_ratios(
    //     "plots/compression_ratios_0_to_100_000_000.png",
    //     0..100_000_000,
    // )
    // .expect("Failed to plot compression ratios");
    // // ⚠️ Unable to plot 1 billion inputs because it takes too long and uses too much memory. The process was killed by the OS (exit code 137) after about an hour and using 190 GB of memory + swap space.
    // plot_compression_ratios(
    //     "plots/compression_ratios_0_to_1_000_000_000.png",
    //     0..1_000_000_000,
    // )
    // .expect("Failed to plot compression ratios");

    let end_time = Instant::now();
    println!("Time taken: {:?}", end_time.duration_since(start_time));
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
        .map(|i| memoized_fibonacci_recursive(i))
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
            let fib = memoized_fibonacci_recursive(i);
            (i as f64, fib as f64)
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
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
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

fn plot_compression_ratios(
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
            let compressed_as_zeckendorf_data = zeckendorf_compress_be(&data_bytes);
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
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
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
