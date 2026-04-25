//! Reversible byte-level preprocessing transforms.
//!
//! Each transform must round-trip losslessly: `invert(apply(x)) == x` for any input.
//! Variants are serde-tagged so they can be composed in a TOML config file.

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum PreprocessError {
    #[error("RLE: truncated input — expected count/byte pair")]
    RleTruncated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Preprocessor {
    /// No-op. Useful as a baseline pipeline entry.
    Identity,
    /// Run-length encoding: emit `(count, byte)` pairs. Runs longer than 255 are split.
    /// Worst case (no runs at all) doubles size; the harness's metric will reflect that.
    Rle,
}

impl Preprocessor {
    pub fn apply(&self, input: &[u8]) -> Vec<u8> {
        match self {
            Preprocessor::Identity => input.to_vec(),
            Preprocessor::Rle => rle_encode(input),
        }
    }

    pub fn invert(&self, input: &[u8]) -> Result<Vec<u8>, PreprocessError> {
        match self {
            Preprocessor::Identity => Ok(input.to_vec()),
            Preprocessor::Rle => rle_decode(input),
        }
    }
}

fn rle_encode(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() * 2);
    let mut i = 0;
    while i < input.len() {
        let byte = input[i];
        let mut run_len = 1usize;
        while i + run_len < input.len() && input[i + run_len] == byte && run_len < 255 {
            run_len += 1;
        }
        out.push(run_len as u8);
        out.push(byte);
        i += run_len;
    }
    out
}

fn rle_decode(input: &[u8]) -> Result<Vec<u8>, PreprocessError> {
    if !input.len().is_multiple_of(2) {
        return Err(PreprocessError::RleTruncated);
    }
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        let count = input[i] as usize;
        let byte = input[i + 1];
        for _ in 0..count {
            out.push(byte);
        }
        i += 2;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, SeedableRng, rngs::StdRng};

    fn roundtrip(p: &Preprocessor, input: &[u8]) {
        // Arrange
        let original = input.to_vec();

        // Act
        let encoded = p.apply(&original);
        let decoded = p.invert(&encoded).expect("invert must succeed");

        // Assert
        assert_eq!(decoded, original);
    }

    #[test]
    fn identity_roundtrip_random() {
        let mut rng = StdRng::seed_from_u64(1);
        let bytes: Vec<u8> = (0..1024).map(|_| rng.random::<u8>()).collect();

        roundtrip(&Preprocessor::Identity, &bytes);
    }

    #[test]
    fn identity_roundtrip_empty() {
        roundtrip(&Preprocessor::Identity, &[]);
    }

    #[test]
    fn rle_roundtrip_random() {
        let mut rng = StdRng::seed_from_u64(2);
        let bytes: Vec<u8> = (0..1024).map(|_| rng.random::<u8>()).collect();

        roundtrip(&Preprocessor::Rle, &bytes);
    }

    #[test]
    fn rle_roundtrip_long_run() {
        let bytes = vec![0x42u8; 1000];

        roundtrip(&Preprocessor::Rle, &bytes);
    }

    #[test]
    fn rle_roundtrip_empty() {
        roundtrip(&Preprocessor::Rle, &[]);
    }

    #[test]
    fn rle_roundtrip_alternating() {
        let bytes: Vec<u8> = (0..512).map(|i| (i & 1) as u8).collect();

        roundtrip(&Preprocessor::Rle, &bytes);
    }

    #[test]
    fn rle_compresses_long_runs() {
        // Arrange
        let bytes = vec![0x7Fu8; 512];

        // Act
        let encoded = Preprocessor::Rle.apply(&bytes);

        // Assert: 512 bytes → 3 pairs (255,127), (255,127), (2,127) = 6 bytes
        assert_eq!(encoded.len(), 6);
    }
}
