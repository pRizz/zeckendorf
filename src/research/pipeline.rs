//! Compression pipeline: preprocessors → zeckendorf → frame header.
//!
//! The frame is research-internal. It is NOT the on-disk `ZeckFile` format —
//! production code paths are unaffected by this module.
//!
//! Frame layout (single-block mode):
//! ```text
//! [flags(1)] [preprocessed_len(8 BE u64)] [zeckendorf_payload...]
//! ```
//! `flags & 1` records the endianness actually used (1 = BE, 0 = LE), so the
//! frame is self-describing on the decompress side.

use crate::research::config::{Endianness, ResearchConfig};
use crate::research::preprocessors::{PreprocessError, Preprocessor};
use crate::{
    PadlessCompressionResult, padless_zeckendorf_compress_be_dangerous,
    padless_zeckendorf_compress_best_dangerous, padless_zeckendorf_compress_le_dangerous,
    padless_zeckendorf_decompress_be_dangerous, padless_zeckendorf_decompress_le_dangerous,
};

const FLAG_BIG_ENDIAN: u8 = 0b0000_0001;
const FRAME_HEADER_SIZE: usize = 1 + 8;

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("frame header truncated: need {FRAME_HEADER_SIZE} bytes, got {0}")]
    FrameHeaderTruncated(usize),
    #[error("decompressed payload exceeds recorded length: got {got}, expected {expected}")]
    PayloadOversized { got: usize, expected: usize },
    #[error("preprocessor inverse failed: {0}")]
    Preprocess(#[from] PreprocessError),
}

pub fn compress(cfg: &ResearchConfig, input: &[u8]) -> Vec<u8> {
    if let Some(block_size) = cfg.block_size_bytes
        && block_size > 0
    {
        return compress_blocks(cfg, input, block_size);
    }
    let preprocessed = apply_preprocessors(&cfg.preprocessors, input);
    encode_frame(cfg.endianness, &preprocessed)
}

pub fn decompress(cfg: &ResearchConfig, frame: &[u8]) -> Result<Vec<u8>, PipelineError> {
    if let Some(block_size) = cfg.block_size_bytes
        && block_size > 0
    {
        return decompress_blocks(cfg, frame);
    }
    let preprocessed = decode_frame(frame)?;
    invert_preprocessors(&cfg.preprocessors, preprocessed)
}

/// Block-mode output: for each chunk, `[u64 BE block_len] [block_frame]`.
/// Block size 0 is treated as "no block mode" by the dispatcher above.
fn compress_blocks(cfg: &ResearchConfig, input: &[u8], block_size: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for chunk in input.chunks(block_size) {
        let preprocessed = apply_preprocessors(&cfg.preprocessors, chunk);
        let block_frame = encode_frame(cfg.endianness, &preprocessed);
        out.extend_from_slice(&(block_frame.len() as u64).to_be_bytes());
        out.extend_from_slice(&block_frame);
    }
    out
}

fn decompress_blocks(cfg: &ResearchConfig, mut bytes: &[u8]) -> Result<Vec<u8>, PipelineError> {
    let mut out = Vec::new();
    while !bytes.is_empty() {
        if bytes.len() < 8 {
            return Err(PipelineError::FrameHeaderTruncated(bytes.len()));
        }
        let block_len = u64::from_be_bytes(bytes[..8].try_into().unwrap()) as usize;
        bytes = &bytes[8..];
        if bytes.len() < block_len {
            return Err(PipelineError::FrameHeaderTruncated(bytes.len()));
        }
        let (frame, rest) = bytes.split_at(block_len);
        let preprocessed = decode_frame(frame)?;
        let chunk = invert_preprocessors(&cfg.preprocessors, preprocessed)?;
        out.extend_from_slice(&chunk);
        bytes = rest;
    }
    Ok(out)
}

fn apply_preprocessors(preprocessors: &[Preprocessor], input: &[u8]) -> Vec<u8> {
    let mut data = input.to_vec();
    for p in preprocessors {
        data = p.apply(&data);
    }
    data
}

fn invert_preprocessors(
    preprocessors: &[Preprocessor],
    mut data: Vec<u8>,
) -> Result<Vec<u8>, PipelineError> {
    for p in preprocessors.iter().rev() {
        data = p.invert(&data)?;
    }
    Ok(data)
}

fn encode_frame(endianness: Endianness, preprocessed: &[u8]) -> Vec<u8> {
    let preprocessed_len = preprocessed.len();
    // Zeckendorf of "no bytes" is degenerate; the underlying padless decompressor
    // doesn't always round-trip an empty payload back to empty. Frame an empty
    // input directly with a recorded length of 0 and an empty payload.
    let (used_be, payload) = if preprocessed.is_empty() {
        (true, Vec::new())
    } else {
        match endianness {
            Endianness::Be => (true, padless_zeckendorf_compress_be_dangerous(preprocessed)),
            Endianness::Le => (
                false,
                padless_zeckendorf_compress_le_dangerous(preprocessed),
            ),
            Endianness::Best => match padless_zeckendorf_compress_best_dangerous(preprocessed) {
                PadlessCompressionResult::BigEndianBest {
                    compressed_data, ..
                } => (true, compressed_data),
                PadlessCompressionResult::LittleEndianBest {
                    compressed_data, ..
                } => (false, compressed_data),
                // Both attempts grew the data; pick BE so the frame is still valid and round-trips.
                // The metric will simply reflect that ratio > 1.0 for this input.
                PadlessCompressionResult::Neither { .. } => {
                    (true, padless_zeckendorf_compress_be_dangerous(preprocessed))
                }
            },
        }
    };

    let mut out = Vec::with_capacity(FRAME_HEADER_SIZE + payload.len());
    out.push(if used_be { FLAG_BIG_ENDIAN } else { 0 });
    out.extend_from_slice(&(preprocessed_len as u64).to_be_bytes());
    out.extend_from_slice(&payload);
    out
}

fn decode_frame(frame: &[u8]) -> Result<Vec<u8>, PipelineError> {
    if frame.len() < FRAME_HEADER_SIZE {
        return Err(PipelineError::FrameHeaderTruncated(frame.len()));
    }
    let flags = frame[0];
    let is_be = (flags & FLAG_BIG_ENDIAN) != 0;
    let preprocessed_len =
        u64::from_be_bytes(frame[1..FRAME_HEADER_SIZE].try_into().unwrap()) as usize;
    let payload = &frame[FRAME_HEADER_SIZE..];

    if preprocessed_len == 0 {
        return Ok(Vec::new());
    }

    let mut data = if is_be {
        padless_zeckendorf_decompress_be_dangerous(payload)
    } else {
        padless_zeckendorf_decompress_le_dangerous(payload)
    };

    if data.len() > preprocessed_len {
        return Err(PipelineError::PayloadOversized {
            got: data.len(),
            expected: preprocessed_len,
        });
    }

    // Padless decompression drops bytes that became leading (BE) or trailing (LE)
    // zeros in the BigUint. Restore them from the recorded length.
    if data.len() < preprocessed_len {
        if is_be {
            let pad = preprocessed_len - data.len();
            let mut padded = vec![0u8; pad];
            padded.extend_from_slice(&data);
            data = padded;
        } else {
            data.resize(preprocessed_len, 0);
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(endianness: Endianness, preprocessors: Vec<Preprocessor>) -> ResearchConfig {
        ResearchConfig {
            endianness,
            block_size_bytes: None,
            preprocessors,
            corpus_set: "test".into(),
        }
    }

    fn cfg_blocks(
        endianness: Endianness,
        preprocessors: Vec<Preprocessor>,
        block_size: usize,
    ) -> ResearchConfig {
        ResearchConfig {
            endianness,
            block_size_bytes: Some(block_size),
            preprocessors,
            corpus_set: "test".into(),
        }
    }

    fn roundtrip(cfg: &ResearchConfig, input: &[u8]) {
        // Arrange / Act
        let frame = compress(cfg, input);
        let recovered = decompress(cfg, &frame).expect("decompress must succeed");

        // Assert
        assert_eq!(recovered, input);
    }

    #[test]
    fn roundtrip_identity_be_simple() {
        let cfg = cfg(Endianness::Be, vec![Preprocessor::Identity]);
        let bytes: Vec<u8> = (0..16u8).collect();

        roundtrip(&cfg, &bytes);
    }

    #[test]
    fn roundtrip_identity_le_simple() {
        let cfg = cfg(Endianness::Le, vec![Preprocessor::Identity]);
        let bytes: Vec<u8> = (0..16u8).collect();

        roundtrip(&cfg, &bytes);
    }

    #[test]
    fn roundtrip_no_preprocessors_best() {
        let cfg = cfg(Endianness::Best, vec![]);
        let bytes: Vec<u8> = (0..256u32).map(|i| (i % 256) as u8).collect();

        roundtrip(&cfg, &bytes);
    }

    #[test]
    fn roundtrip_with_leading_zeros_be() {
        let cfg = cfg(Endianness::Be, vec![]);
        let bytes = vec![0u8, 0u8, 0u8, 0x42u8, 0xFFu8, 0xFFu8];

        roundtrip(&cfg, &bytes);
    }

    #[test]
    fn roundtrip_with_trailing_zeros_le() {
        let cfg = cfg(Endianness::Le, vec![]);
        let bytes = vec![0x42u8, 0xFFu8, 0xFFu8, 0u8, 0u8, 0u8];

        roundtrip(&cfg, &bytes);
    }

    #[test]
    fn roundtrip_rle_then_zeckendorf() {
        let cfg = cfg(Endianness::Best, vec![Preprocessor::Rle]);
        let bytes = vec![0xAAu8; 200];

        roundtrip(&cfg, &bytes);
    }

    #[test]
    fn roundtrip_chained_preprocessors() {
        let cfg = cfg(
            Endianness::Best,
            vec![
                Preprocessor::Identity,
                Preprocessor::Rle,
                Preprocessor::Identity,
            ],
        );
        let bytes: Vec<u8> = (0..512u32).map(|i| ((i / 8) % 256) as u8).collect();

        roundtrip(&cfg, &bytes);
    }

    #[test]
    fn roundtrip_empty_input() {
        let cfg = cfg(Endianness::Best, vec![]);

        roundtrip(&cfg, &[]);
    }

    #[test]
    fn neither_endianness_helps_still_round_trips() {
        // Single byte: Zeckendorf almost always inflates this. Forces the Neither path.
        let cfg = cfg(Endianness::Best, vec![]);
        let bytes = vec![0xFFu8];

        roundtrip(&cfg, &bytes);
    }

    #[test]
    fn frame_too_short_errors() {
        let cfg = cfg(Endianness::Best, vec![]);

        let result = decompress(&cfg, &[0u8; 4]);

        assert!(matches!(
            result,
            Err(PipelineError::FrameHeaderTruncated(4))
        ));
    }

    #[test]
    fn roundtrip_blocks_evenly_divisible() {
        let cfg = cfg_blocks(Endianness::Best, vec![], 64);
        let bytes: Vec<u8> = (0..256u32).map(|i| (i % 256) as u8).collect();

        roundtrip(&cfg, &bytes);
    }

    #[test]
    fn roundtrip_blocks_with_remainder() {
        let cfg = cfg_blocks(Endianness::Best, vec![], 100);
        // 250 bytes / 100 → blocks of [100, 100, 50]
        let bytes: Vec<u8> = (0..250u32).map(|i| (i % 256) as u8).collect();

        roundtrip(&cfg, &bytes);
    }

    #[test]
    fn roundtrip_blocks_with_preprocessor() {
        let cfg = cfg_blocks(Endianness::Be, vec![Preprocessor::Rle], 32);
        let bytes: Vec<u8> = (0..200u32).map(|i| ((i / 4) % 256) as u8).collect();

        roundtrip(&cfg, &bytes);
    }

    #[test]
    fn roundtrip_blocks_empty_input() {
        let cfg = cfg_blocks(Endianness::Best, vec![], 64);

        roundtrip(&cfg, &[]);
    }

    #[test]
    fn roundtrip_blocks_smaller_than_one_block() {
        let cfg = cfg_blocks(Endianness::Best, vec![], 1024);
        let bytes: Vec<u8> = (0..50u8).collect();

        roundtrip(&cfg, &bytes);
    }
}
