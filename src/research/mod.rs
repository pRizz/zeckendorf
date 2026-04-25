//! Parameter-sweep research harness for Zeckendorf compression.
//!
//! Gated by the `research` feature. The autoresearch-style flow:
//! an agent edits a TOML config, the `zeck-research` binary runs the
//! evaluator against a fixed corpus, and prints a single primary
//! metric (total compression ratio) plus secondary signals.
//!
//! Nothing here is part of the public library API surface. The default
//! build excludes this module entirely.

pub mod config;
pub mod eval;
pub mod pipeline;
pub mod preprocessors;
