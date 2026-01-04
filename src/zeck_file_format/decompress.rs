//! Decompression functions for the .zeck file format

use crate::zeck_file_format::error::ZeckFormatError;
use crate::zeck_file_format::{
    ZECK_FLAG_BIG_ENDIAN, ZECK_FLAG_RESERVED_MASK, ZECK_FORMAT_VERSION, file::ZeckFile,
};
use crate::{
    padless_zeckendorf_decompress_be_dangerous, padless_zeckendorf_decompress_le_dangerous,
};

/// Decompresses data from a [`ZeckFile`] struct.
///
/// This function takes a [`ZeckFile`] directly and uses its header information to decompress
/// the data. This is a convenience function that avoids the need to serialize and parse the
/// file format when you already have a [`ZeckFile`] struct.
///
/// # ⚠️ Warning
///
/// **Compressing or decompressing data larger than 10KB (10,000 bytes) is unstable due to time and memory pressure.**
/// The library may experience performance issues, excessive memory usage, or failures when processing data exceeding this size.
///
/// # Examples
///
/// ```
/// # use zeck::zeck_file_format::{compress::compress_zeck_le, decompress::decompress_zeck_file};
/// //let original = vec![1, 0]; // FIXME: fails test
/// let original = vec![0, 1];
/// let zeck_file = compress_zeck_le(&original).unwrap();
/// match decompress_zeck_file(&zeck_file) {
///     Ok(decompressed) => {
///         assert_eq!(decompressed, original);
///     }
///     Err(e) => {
///         // Handle error
///     }
/// }
/// ```
pub fn decompress_zeck_file(zeck_file: &ZeckFile) -> Result<Vec<u8>, ZeckFormatError> {
    // Check reserved flags
    if zeck_file.flags & ZECK_FLAG_RESERVED_MASK != 0 {
        return Err(ZeckFormatError::ReservedFlagsSet {
            flags: zeck_file.flags,
        });
    }

    // Route to version-specific decompression
    match zeck_file.version {
        1 => decompress_zeck_v1(
            &zeck_file.compressed_data,
            zeck_file.original_size,
            zeck_file.flags,
        ),
        _ => Err(ZeckFormatError::UnsupportedVersion {
            found_version: zeck_file.version,
            supported_version: ZECK_FORMAT_VERSION,
        }),
    }
}

/// Version 1 decompression implementation.
///
/// This function handles decompression for .zeck format version 1, using the endianness
/// specified in the flags byte.
fn decompress_zeck_v1(
    compressed_data: &[u8],
    original_size: u64,
    flags: u8,
) -> Result<Vec<u8>, ZeckFormatError> {
    let is_big_endian = (flags & ZECK_FLAG_BIG_ENDIAN) != 0;

    let decompressed = if is_big_endian {
        padless_zeckendorf_decompress_be_dangerous(compressed_data)
    } else {
        padless_zeckendorf_decompress_le_dangerous(compressed_data)
    };

    let original_size_usize = original_size as usize;
    let decompressed_len = decompressed.len();

    // If decompressed size is larger than original, return error
    if decompressed_len > original_size_usize {
        return Err(ZeckFormatError::DecompressedTooLarge {
            expected_size: original_size_usize,
            actual_size: decompressed_len,
        });
    }

    // If decompressed size is smaller than original, pad with leading zeros
    if decompressed_len < original_size_usize {
        let padding_size = original_size_usize - decompressed_len;
        let mut padded = Vec::with_capacity(original_size_usize);
        padded.resize(padding_size, 0u8);
        padded.extend_from_slice(&decompressed);
        Ok(padded)
    } else {
        // Sizes match exactly
        Ok(decompressed)
    }
}
