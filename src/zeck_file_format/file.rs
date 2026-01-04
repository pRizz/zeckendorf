//! Zeck file structure and serialization

use crate::zeck_file_format::{
    ZECK_FLAG_BIG_ENDIAN, ZECK_FORMAT_VERSION, ZECK_HEADER_SIZE, error::ZeckFormatError,
};

/// Represents a .zeck file with its header information and compressed data.
///
/// This struct holds all the information needed to reconstruct a .zeck file,
/// including the format version, original file size, endianness flags, and
/// the compressed data itself.
#[derive(Debug, Clone, PartialEq)]
pub struct ZeckFile {
    /// File format version
    pub version: u8,
    /// Original uncompressed file size in bytes
    pub original_size: u64,
    /// Flags byte (bit 0 = big endian, bits 1-7 reserved)
    pub flags: u8,
    /// Compressed data (without header)
    pub compressed_data: Vec<u8>,
}

impl ZeckFile {
    /// Creates a new ZeckFile with the default version and specified parameters.
    pub(crate) fn new(original_size: u64, compressed_data: Vec<u8>, is_big_endian: bool) -> Self {
        let mut flags = 0u8;
        if is_big_endian {
            flags |= ZECK_FLAG_BIG_ENDIAN;
        }
        Self {
            version: ZECK_FORMAT_VERSION,
            original_size,
            flags,
            compressed_data,
        }
    }

    /// Returns whether the data was compressed using big endian interpretation.
    pub fn is_big_endian(&self) -> bool {
        (self.flags & ZECK_FLAG_BIG_ENDIAN) != 0
    }

    /// Serializes the ZeckFile to a byte vector in .zeck file format.
    ///
    /// This creates a complete .zeck file with header followed by compressed data,
    /// suitable for writing to disk or transmitting over a network.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zeck::zeck_file_format::{compress::compress_zeck_be, ZeckFile};
    /// let data = vec![1, 2, 3];
    /// let zeck_file = compress_zeck_be(&data).expect("Compression failed");
    /// let bytes = zeck_file.to_bytes();
    /// // bytes can now be written to a file
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(ZECK_HEADER_SIZE + self.compressed_data.len());

        // Version (1 byte)
        output.push(self.version);

        // Original size (8 bytes, little endian)
        output.extend_from_slice(&self.original_size.to_le_bytes());

        // Flags (1 byte)
        output.push(self.flags);

        // Compressed data
        output.extend_from_slice(&self.compressed_data);

        output
    }

    /// Returns the total size of the serialized file (header + compressed data).
    pub fn total_size(&self) -> usize {
        ZECK_HEADER_SIZE + self.compressed_data.len()
    }
}

impl std::fmt::Display for ZeckFile {
    /// Formats the ZeckFile for display, showing key information.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zeck::zeck_file_format::{compress::compress_zeck_be, ZeckFile};
    /// let data = vec![1, 2, 3];
    /// let zeck_file = compress_zeck_be(&data).expect("Compression failed");
    /// println!("{}", zeck_file);
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ZeckFile {{ version: {}, original_size: {} bytes, compressed_size: {} bytes, endianness: {} }}",
            self.version,
            self.original_size,
            self.compressed_data.len(),
            if self.is_big_endian() {
                "big"
            } else {
                "little"
            }
        )
    }
}

/// Deserializes a .zeck file from raw bytes into a [`ZeckFile`] struct.
///
/// This function reads the header to determine the file format version, original size, and endianness,
/// and constructs a [`ZeckFile`] struct. To decompress the data, call [`crate::zeck_file_format::decompress::decompress_zeck_file`] on the result.
///
/// # Examples
///
/// ```
/// # use zeck::zeck_file_format::{compress::compress_zeck_le, file::deserialize_zeck_file, decompress::decompress_zeck_file};
/// let original = vec![0, 1];
/// let zeck_file = compress_zeck_le(&original).unwrap();
/// let zeck_file_bytes = zeck_file.to_bytes();
/// match deserialize_zeck_file(&zeck_file_bytes) {
///     Ok(zeck_file) => {
///         let decompressed = decompress_zeck_file(&zeck_file).unwrap();
///         assert_eq!(decompressed, original);
///     }
///     Err(e) => {
///         // Handle error
///         assert!(false);
///     }
/// }
/// ```
pub fn deserialize_zeck_file(zeck_file_data: &[u8]) -> Result<ZeckFile, ZeckFormatError> {
    // Check header size
    if zeck_file_data.len() < ZECK_HEADER_SIZE {
        return Err(ZeckFormatError::HeaderTooShort {
            actual_length: zeck_file_data.len(),
            required_length: ZECK_HEADER_SIZE,
        });
    }

    // Parse header
    let version = zeck_file_data[0];
    let original_size = u64::from_le_bytes([
        zeck_file_data[1],
        zeck_file_data[2],
        zeck_file_data[3],
        zeck_file_data[4],
        zeck_file_data[5],
        zeck_file_data[6],
        zeck_file_data[7],
        zeck_file_data[8],
    ]);
    let flags = zeck_file_data[9];

    // Extract compressed data (everything after the header)
    let compressed_data = zeck_file_data[ZECK_HEADER_SIZE..].to_vec();

    // Construct and return ZeckFile
    Ok(ZeckFile {
        version,
        original_size,
        flags,
        compressed_data,
    })
}
