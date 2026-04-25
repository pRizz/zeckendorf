//! Research-harness configuration. Loaded from TOML by `zeck-research`.

use serde::{Deserialize, Serialize};

use crate::research::preprocessors::Preprocessor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Endianness {
    Be,
    Le,
    #[default]
    Best,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchConfig {
    #[serde(default)]
    pub endianness: Endianness,
    /// If set, input is split into chunks of this many bytes before compression.
    /// Each chunk is framed independently and length-prefixed in the output.
    #[serde(default)]
    pub block_size_bytes: Option<usize>,
    /// Preprocessors applied left-to-right before zeckendorf compression. Inverted
    /// in reverse order on decompression.
    #[serde(default)]
    pub preprocessors: Vec<Preprocessor>,
    /// Which corpus subset to evaluate against. `"all"` means every available file.
    #[serde(default = "default_corpus_set")]
    pub corpus_set: String,
}

fn default_corpus_set() -> String {
    "all".to_string()
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            endianness: Endianness::default(),
            block_size_bytes: None,
            preprocessors: Vec::new(),
            corpus_set: default_corpus_set(),
        }
    }
}
