//! .zeck file format module
//!
//! This module provides functionality for compressing and decompressing data using the .zeck file format,
//! which includes a header containing format version, original file size, and endianness information.

pub mod compress;
pub mod decompress;
pub mod error;
pub mod file;

pub use error::ZeckFormatError;
pub use file::ZeckFile;

/// Current .zeck file format version.
pub const ZECK_FORMAT_VERSION: u8 = 1;

/// Size of the .zeck file format header in bytes.
pub const ZECK_HEADER_SIZE: usize = 10;

/// Bit flag in the flags byte indicating big endian interpretation.
/// If this bit is set (1), the data was compressed using big endian interpretation.
/// If this bit is clear (0), the data was compressed using little endian interpretation.
pub const ZECK_FLAG_BIG_ENDIAN: u8 = 0b0000_0001;

/// Reserved flags mask. Bits 1-7 are reserved for future use.
pub const ZECK_FLAG_RESERVED_MASK: u8 = 0b1111_1110;
