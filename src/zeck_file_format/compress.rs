//! Compression functions for the .zeck file format

use crate::zeck_file_format::error::ZeckFormatError;
use crate::zeck_file_format::file::ZeckFile;
use crate::{
    PadlessCompressionResult, padless_zeckendorf_compress_be_dangerous,
    padless_zeckendorf_compress_best_dangerous, padless_zeckendorf_compress_le_dangerous,
};
use std::convert::TryFrom;

/// Result of best compression attempt, containing the best compressed zeck file and the size for the other endianness attempt, or if neither produced compression (both were larger than the original).
#[derive(Debug, Clone, PartialEq)]
pub enum BestCompressionResult {
    /// Big endian compression produced the smallest output.
    /// Contains the compressed data and the size of the little endian attempt for comparison.
    BigEndianBest {
        /// The compressed ZeckFile using big endian interpretation
        zeck_file: ZeckFile,
        /// Compressed size using little endian interpretation (for comparison)
        le_size: usize,
    },
    /// Little endian compression produced the smallest output.
    /// Contains the compressed data and the size of the big endian attempt for comparison.
    LittleEndianBest {
        /// The compressed ZeckFile using little endian interpretation
        zeck_file: ZeckFile,
        /// Compressed size using big endian interpretation (for comparison)
        be_size: usize,
    },
    /// Neither compression method produced a smaller output than the original.
    /// Contains sizes for both attempts.
    Neither {
        /// Compressed size using big endian interpretation
        be_size: usize,
        /// Compressed size using little endian interpretation
        le_size: usize,
    },
}

/// Compresses data using the Zeckendorf algorithm with automatic endianness selection,
/// and stores the result in a [`BestCompressionResult`] struct.
///
/// This function attempts compression with both big endian and little endian interpretations,
/// and returns the best result, or if neither produced compression (both were larger than the original).
///
/// # ⚠️ Warning
///
/// **Compressing or decompressing data larger than 10KB (10,000 bytes) is unstable due to time and memory pressure.**
/// The library may experience performance issues, excessive memory usage, or failures when processing data exceeding this size.
///
/// # Examples
///
/// ```
/// # use zeck::zeck_file_format::compress::compress_zeck_best;
/// # use zeck::zeck_file_format::compress::BestCompressionResult;
/// # use zeck::zeck_file_format::decompress::decompress_zeck_file;
/// let data = vec![0, 1]; // Compresses best interpreted as a big endian integer
/// match compress_zeck_best(&data) {
///     Ok(best_compression_result) => {
///         match best_compression_result {
///             BestCompressionResult::BigEndianBest { zeck_file, le_size } => {
///                 let decompressed = decompress_zeck_file(&zeck_file).unwrap();
///                 assert_eq!(decompressed, data);
///             }
///             BestCompressionResult::LittleEndianBest { zeck_file, be_size } => {
///                 assert!(false);
///             }
///             BestCompressionResult::Neither { be_size, le_size } => {
///                 assert!(false);
///             }
///         }
///     }
///     Err(e) => {
///         assert!(false);
///     }
/// }
///
/// let data = vec![1, 0]; // Compresses best interpreted as a little endian integer
/// match compress_zeck_best(&data) {
///     Ok(best_compression_result) => {
///         match best_compression_result {
///             BestCompressionResult::BigEndianBest { zeck_file, le_size } => {
///                 assert!(false);
///             }
///             BestCompressionResult::LittleEndianBest { zeck_file, be_size } => {
///                 let decompressed = decompress_zeck_file(&zeck_file).unwrap();
///                 assert_eq!(decompressed, data);
///             }
///             BestCompressionResult::Neither { be_size, le_size } => {
///                 assert!(false);
///             }
///         }
///     }
///     Err(e) => {
///         assert!(false);
///     }
/// }
/// ```
pub fn compress_zeck_best(data: &[u8]) -> Result<BestCompressionResult, ZeckFormatError> {
    let original_size = u64::try_from(data.len())
        .map_err(|_| ZeckFormatError::DataSizeTooLarge { size: data.len() })?;
    let result = padless_zeckendorf_compress_best_dangerous(data);

    match result {
        PadlessCompressionResult::BigEndianBest {
            compressed_data,
            le_size,
        } => Ok(BestCompressionResult::BigEndianBest {
            zeck_file: ZeckFile::new(original_size, compressed_data, true),
            le_size,
        }),
        PadlessCompressionResult::LittleEndianBest {
            compressed_data,
            be_size,
        } => Ok(BestCompressionResult::LittleEndianBest {
            zeck_file: ZeckFile::new(original_size, compressed_data, false),
            be_size,
        }),
        PadlessCompressionResult::Neither { be_size, le_size } => {
            Ok(BestCompressionResult::Neither { be_size, le_size })
        }
    }
}

/// Compresses data using the Zeckendorf algorithm with little endian interpretation,
/// and stores the result in a [`ZeckFile`] struct.
///
/// # ⚠️ Warning
///
/// **Compressing or decompressing data larger than 10KB (10,000 bytes) is unstable due to time and memory pressure.**
/// The library may experience performance issues, excessive memory usage, or failures when processing data exceeding this size.
///
/// # Examples
///
/// ```
/// # use zeck::zeck_file_format::compress::compress_zeck_le;
/// let data = vec![1, 0];
/// match compress_zeck_le(&data) {
///     Ok(zeck_file) => {
///         // Access file information
///         println!("Original size: {} bytes", zeck_file.original_size);
///         println!("Compressed size: {} bytes", zeck_file.compressed_data.len());
///         // Serialize to bytes for writing to file
///         let bytes = zeck_file.to_bytes();
///     }
///     Err(e) => {
///         // Handle error (e.g., data size too large)
///     }
/// }
/// ```
pub fn compress_zeck_le(data: &[u8]) -> Result<ZeckFile, ZeckFormatError> {
    let original_size = u64::try_from(data.len())
        .map_err(|_| ZeckFormatError::DataSizeTooLarge { size: data.len() })?;
    let compressed_data = padless_zeckendorf_compress_le_dangerous(data);
    Ok(ZeckFile::new(original_size, compressed_data, false))
}

/// Compresses data using the Zeckendorf algorithm with big endian interpretation,
/// and stores the result in a [`ZeckFile`] struct.
///
/// # ⚠️ Warning
///
/// **Compressing or decompressing data larger than 10KB (10,000 bytes) is unstable due to time and memory pressure.**
/// The library may experience performance issues, excessive memory usage, or failures when processing data exceeding this size.
///
/// # Examples
///
/// ```
/// # use zeck::zeck_file_format::compress::compress_zeck_be;
/// let data = vec![1, 0];
/// match compress_zeck_be(&data) {
///     Ok(zeck_file) => {
///         // Access file information
///         println!("Original size: {} bytes", zeck_file.original_size);
///         println!("Is big endian: {}", zeck_file.is_big_endian());
///         // Serialize to bytes for writing to file
///         let bytes = zeck_file.to_bytes();
///     }
///     Err(e) => {
///         // Handle error (e.g., data size too large)
///     }
/// }
/// ```
pub fn compress_zeck_be(data: &[u8]) -> Result<ZeckFile, ZeckFormatError> {
    let original_size = u64::try_from(data.len())
        .map_err(|_| ZeckFormatError::DataSizeTooLarge { size: data.len() })?;
    let compressed_data = padless_zeckendorf_compress_be_dangerous(data);
    Ok(ZeckFile::new(original_size, compressed_data, true))
}
