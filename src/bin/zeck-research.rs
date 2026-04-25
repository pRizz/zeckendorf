//! `zeck-research` — autoresearch-style parameter-sweep harness.

use std::{error::Error, fs, path::PathBuf};

use clap::Parser;
use zeck::research::{config::ResearchConfig, eval};

#[derive(Debug, Parser)]
#[command(about = "Evaluate a Zeckendorf research config against the fixed corpus")]
struct Args {
    /// TOML config to evaluate.
    #[arg(long)]
    config: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let config_toml = fs::read_to_string(&args.config)?;
    let cfg: ResearchConfig = toml::from_str(&config_toml)?;
    let report = eval::run(&cfg)?;
    print!("{}", eval::format_metric_block(&report));
    Ok(())
}
