//! Fixed evaluator for the autoresearch-style parameter sweep.
//!
//! The evaluator walks a deterministic corpus, runs the research pipeline, and
//! reports one primary metric: total compressed bytes divided by total input
//! bytes. Compression failures are scored as ratio `1.0` without aborting the
//! whole run so the outer experiment loop can record and discard bad configs.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use crate::research::{config::ResearchConfig, pipeline};

const SYNTHETIC_CORPUS_RELATIVE: &[&str] =
    &[".cache", "zeckendorf-research", "corpus", "synthetic"];
const REAL_CORPUS_DIR: &str = "research/corpus/real";
const FNV_OFFSET_BASIS: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;

#[derive(Debug, Clone, PartialEq)]
pub struct RunReport {
    pub ratio: f64,
    pub uncompressed_bytes: u64,
    pub compressed_bytes: u64,
    pub roundtrip_ok: bool,
    pub wall_seconds: f64,
    pub peak_rss_mb: f64,
    pub config_hash: String,
}

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("failed to locate home directory for synthetic corpus")]
    MissingHomeDir,
    #[error("missing corpus directory: {0}")]
    MissingCorpusDir(PathBuf),
    #[error("corpus selection `{selection}` matched no files")]
    EmptyCorpus { selection: String },
    #[error("failed to read corpus directory {path}: {source}")]
    ReadDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to read corpus file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to hash config: {0}")]
    ConfigHash(toml::ser::Error),
}

pub fn run(cfg: &ResearchConfig) -> Result<RunReport, EvalError> {
    let synthetic_dir = default_synthetic_corpus_dir()?;
    run_with_roots(cfg, &synthetic_dir, Path::new(REAL_CORPUS_DIR))
}

pub fn run_with_roots(
    cfg: &ResearchConfig,
    synthetic_dir: &Path,
    real_dir: &Path,
) -> Result<RunReport, EvalError> {
    let started_at = Instant::now();
    let corpus_files = collect_corpus_files(&cfg.corpus_set, synthetic_dir, real_dir)?;
    let mut uncompressed_bytes = 0u64;
    let mut compressed_bytes = 0u64;
    let mut roundtrip_ok = true;

    for path in corpus_files {
        let input = fs::read(&path).map_err(|source| EvalError::ReadFile {
            path: path.clone(),
            source,
        })?;
        let compressed = pipeline::compress(cfg, &input);
        let recovered = pipeline::decompress(cfg, &compressed);

        uncompressed_bytes += input.len() as u64;
        compressed_bytes += compressed.len() as u64;

        match recovered {
            Ok(bytes) if bytes == input => {}
            Ok(_) | Err(_) => {
                roundtrip_ok = false;
            }
        }
    }

    let ratio = if roundtrip_ok && uncompressed_bytes > 0 {
        compressed_bytes as f64 / uncompressed_bytes as f64
    } else {
        1.0
    };

    Ok(RunReport {
        ratio,
        uncompressed_bytes,
        compressed_bytes,
        roundtrip_ok,
        wall_seconds: started_at.elapsed().as_secs_f64(),
        peak_rss_mb: peak_rss_mb(),
        config_hash: config_hash(cfg)?,
    })
}

pub fn format_metric_block(report: &RunReport) -> String {
    format!(
        "\
---
ratio:               {ratio:.6}
uncompressed_bytes:  {uncompressed_bytes}
compressed_bytes:    {compressed_bytes}
roundtrip_ok:        {roundtrip_ok}
wall_seconds:        {wall_seconds:.3}
peak_rss_mb:         {peak_rss_mb:.1}
config_hash:         {config_hash}
",
        ratio = report.ratio,
        uncompressed_bytes = report.uncompressed_bytes,
        compressed_bytes = report.compressed_bytes,
        roundtrip_ok = report.roundtrip_ok,
        wall_seconds = report.wall_seconds,
        peak_rss_mb = report.peak_rss_mb,
        config_hash = report.config_hash,
    )
}

fn collect_corpus_files(
    selection: &str,
    synthetic_dir: &Path,
    real_dir: &Path,
) -> Result<Vec<PathBuf>, EvalError> {
    let mut files = Vec::new();
    match selection {
        "all" => {
            push_files_from_dir(&mut files, synthetic_dir, CorpusKind::Synthetic)?;
            push_files_from_dir(&mut files, real_dir, CorpusKind::Real)?;
        }
        "synthetic" => {
            push_files_from_dir(&mut files, synthetic_dir, CorpusKind::Synthetic)?;
        }
        "real" => {
            push_files_from_dir(&mut files, real_dir, CorpusKind::Real)?;
        }
        shape_prefix => {
            push_files_from_dir(
                &mut files,
                synthetic_dir,
                CorpusKind::SyntheticPrefix(shape_prefix),
            )?;
        }
    }

    if files.is_empty() {
        return Err(EvalError::EmptyCorpus {
            selection: selection.to_string(),
        });
    }

    files.sort();
    Ok(files)
}

#[derive(Debug, Clone, Copy)]
enum CorpusKind<'a> {
    Synthetic,
    Real,
    SyntheticPrefix(&'a str),
}

fn push_files_from_dir(
    files: &mut Vec<PathBuf>,
    dir: &Path,
    kind: CorpusKind<'_>,
) -> Result<(), EvalError> {
    if !dir.is_dir() {
        return Err(EvalError::MissingCorpusDir(dir.to_path_buf()));
    }

    for entry in fs::read_dir(dir).map_err(|source| EvalError::ReadDir {
        path: dir.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| EvalError::ReadDir {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if file_matches_selection(&path, kind) {
            files.push(path);
        }
    }
    Ok(())
}

fn file_matches_selection(path: &Path, kind: CorpusKind<'_>) -> bool {
    match kind {
        CorpusKind::Synthetic | CorpusKind::Real => true,
        CorpusKind::SyntheticPrefix(shape_prefix) => path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(shape_prefix)),
    }
}

fn default_synthetic_corpus_dir() -> Result<PathBuf, EvalError> {
    let home = std::env::var_os("HOME").ok_or(EvalError::MissingHomeDir)?;
    let mut path = PathBuf::from(home);
    for part in SYNTHETIC_CORPUS_RELATIVE {
        path.push(part);
    }
    Ok(path)
}

fn config_hash(cfg: &ResearchConfig) -> Result<String, EvalError> {
    let toml = toml::to_string(cfg).map_err(EvalError::ConfigHash)?;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in toml.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    Ok(format!("{hash:08x}"))
}

fn peak_rss_mb() -> f64 {
    0.0
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;
    use crate::research::{config::Endianness, preprocessors::Preprocessor};

    #[test]
    fn run_with_roots_reports_parseable_ratio_for_tiny_corpus() {
        // Arrange
        let test_dir = unique_test_dir();
        let synthetic_dir = test_dir.join("synthetic");
        let real_dir = test_dir.join("real");
        fs::create_dir_all(&synthetic_dir).expect("synthetic test dir should be creatable");
        fs::create_dir_all(&real_dir).expect("real test dir should be creatable");
        let first = vec![0u8, 1, 1, 2, 3, 5, 8, 13];
        let second = b"aaaaabbbbbccccc".to_vec();
        fs::write(synthetic_dir.join("small_8.bin"), &first)
            .expect("first corpus file should be writable");
        fs::write(real_dir.join("sample.txt"), &second)
            .expect("second corpus file should be writable");
        let cfg = ResearchConfig {
            endianness: Endianness::Best,
            block_size_bytes: None,
            preprocessors: vec![Preprocessor::Identity],
            corpus_set: "all".to_string(),
        };
        let expected_compressed_bytes =
            pipeline::compress(&cfg, &first).len() + pipeline::compress(&cfg, &second).len();
        let expected_uncompressed_bytes = first.len() + second.len();
        let expected_ratio = expected_compressed_bytes as f64 / expected_uncompressed_bytes as f64;

        // Act
        let report =
            run_with_roots(&cfg, &synthetic_dir, &real_dir).expect("tiny corpus should run");
        let block = format_metric_block(&report);
        let ratio_line = block
            .lines()
            .find(|line| line.starts_with("ratio:"))
            .expect("metric block should include ratio line");
        let parsed_ratio: f64 = ratio_line["ratio:".len()..]
            .trim()
            .parse()
            .expect("ratio line should parse as f64");

        // Assert
        assert!(report.roundtrip_ok);
        assert_eq!(
            report.uncompressed_bytes,
            expected_uncompressed_bytes as u64
        );
        assert_eq!(report.compressed_bytes, expected_compressed_bytes as u64);
        assert!((parsed_ratio - expected_ratio).abs() < 0.000001);

        fs::remove_dir_all(&test_dir).expect("test dir cleanup should succeed");
    }

    fn unique_test_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "zeckendorf-research-eval-{}-{nanos}",
            std::process::id()
        ))
    }
}
