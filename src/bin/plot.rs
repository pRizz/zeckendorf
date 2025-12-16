//! Binary for generating plots using plotters
//!
//! This binary requires the `plotting` feature to be enabled.
//! Run with: cargo run --bin plot --features plotting

use num_bigint::BigUint;
use std::time::Instant;
use plotters::prelude::*;
use zeckendorf_rs::*;

#[cfg(feature = "plotting")]
fn main() {
    let start_time = Instant::now();

    // Ensure plots directory exists
    std::fs::create_dir_all("plots").expect("Failed to create plots directory");

    // Example: Plot Fibonacci numbers
    plot_fibonacci_numbers("plots/fibonacci_plot.png", 0..30)
        .expect("Failed to plot Fibonacci numbers");

    // Example: Plot compression ratios
    // plot_compression_ratios("plots/compression_ratios_0_to_100.png", 0..100).expect("Failed to plot compression ratios");
    plot_compression_ratios("plots/compression_ratios_0_to_257.png", 0..257).expect("Failed to plot compression ratios");
    // plot_compression_ratios("plots/compression_ratios_0_to_1_000.png", 0..1_000).expect("Failed to plot compression ratios");
    // plot_compression_ratios("plots/compression_ratios_0_to_10_000.png", 0..10_000).expect("Failed to plot compression ratios");
    // plot_compression_ratios("plots/compression_ratios_0_to_100_000.png", 0..100_000).expect("Failed to plot compression ratios");
    // // Takes about 30 seconds to generate
    // plot_compression_ratios("plots/compression_ratios_0_to_1_000_000.png", 0..1_000_000).expect("Failed to plot compression ratios");
    // // Takes about 300 seconds to generate
    // plot_compression_ratios("plots/compression_ratios_0_to_10_000_000.png", 0..10_000_000).expect("Failed to plot compression ratios");

    let end_time = Instant::now();
    println!("Time taken: {:?}", end_time.duration_since(start_time));
}

#[cfg(feature = "plotting")]
fn plot_fibonacci_numbers(
    filename: &str,
    range: std::ops::Range<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("Plotting Fibonacci numbers for range {:?}", range);
    let root = BitMapBackend::new(filename, (3840, 2160)).into_drawing_area();
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
            ("sans-serif", 50).into_font(),
        )
        .margin(5)
        .x_label_area_size(60)
        .y_label_area_size(120)
        .build_cartesian_2d(
            range.start as f64..range.end as f64,
            (1f64..max_fib).log_scale(),
        )?;

    chart.configure_mesh().draw()?;

    // Filter out zero values since log(0) is undefined
    let data: Vec<(f64, f64)> = range.clone()
        .map(|i| {
            let fib = memoized_fibonacci_recursive(i);
            (i as f64, fib as f64)
        })
        .filter(|(_, y)| *y > 0.0)
        .collect();

    // Draw the line
    chart
        .draw_series(LineSeries::new(data.iter().copied(), &RED))?
        .label("Fibonacci Numbers")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    // Draw dots at each point
    chart.draw_series(
        data.iter()
            .map(|point| Circle::new(*point, 3, RED.filled())),
    )?;

    // Draw text labels above each point showing x,y coordinates
    for (x, y) in &data {
        let label = format!("({:.0}, {:.0})", x, y);
        let text_x = *x + 0.3;
        let text_y = *y * 1.0;
        chart.draw_series(std::iter::once(Text::new(
            label,
            (text_x, text_y),
            ("sans-serif", 32).into_font(),
        )))?;
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("Fibonacci plot saved to {}", filename);
    let end_time = Instant::now();
    println!("Time taken to plot Fibonacci numbers for range {:?}: {:?}", range, end_time.duration_since(start_time));
    Ok(())
}

#[cfg(feature = "plotting")]
fn plot_compression_ratios(
    filename: &str,
    range: std::ops::Range<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    println!("Plotting compression ratios for range {:?}", range);
    let root = BitMapBackend::new(filename, (3840, 2160)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Zeckendorf Compression Ratios",
            ("sans-serif", 50).into_font(),
        )
        .margin(5)
        .x_label_area_size(60)
        .y_label_area_size(120)
        .build_cartesian_2d(range.start as f64..range.end as f64, 0.0f64..2.0f64)?;

    chart.configure_mesh().draw()?;

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

    chart
        .draw_series(LineSeries::new(data, &BLUE))?
        .label("Compression Ratio")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // Draw a line at ratio 1.0 (no compression benefit)
    chart.draw_series(LineSeries::new(
        vec![(range.start as f64, 1.0), (range.end as f64, 1.0)],
        GREEN.mix(0.5).stroke_width(3),
    ))?;

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("Compression ratio plot saved to {}", filename);
    let end_time = Instant::now();
    println!("Time taken to plot compression ratios for range {:?}: {:?}", range, end_time.duration_since(start_time));
    Ok(())
}
