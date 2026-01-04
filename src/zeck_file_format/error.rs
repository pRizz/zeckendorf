/// Errors that can occur when parsing or processing .zeck files.
#[derive(Debug, Clone, PartialEq)]
pub enum ZeckFormatError {
    /// The input data is too short to contain a valid header.
    HeaderTooShort {
        /// The actual length of the input data
        actual_length: usize,
        /// The minimum required length for a header
        required_length: usize,
    },
    /// The file format version in the header is not supported.
    UnsupportedVersion {
        /// The version found in the header
        found_version: u8,
        /// The maximum supported version
        supported_version: u8,
    },
    /// The reserved flags in the header are set (indicating a newer format version).
    ReservedFlagsSet {
        /// The flags byte from the header
        flags: u8,
    },
    /// Compression did not reduce the size of the data.
    CompressionFailed {
        /// Original data size
        original_size: usize,
        /// Big endian compressed size
        be_size: usize,
        /// Little endian compressed size
        le_size: usize,
    },
    /// Decompressed data size is larger than the original size specified in the header.
    DecompressedTooLarge {
        /// Expected size from header
        expected_size: usize,
        /// Actual decompressed size
        actual_size: usize,
    },
    /// The input data size is too large to be represented in the file format header.
    DataSizeTooLarge {
        /// The size that could not be converted
        size: usize,
    },
}

impl std::fmt::Display for ZeckFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZeckFormatError::HeaderTooShort {
                actual_length,
                required_length,
            } => {
                write!(
                    f,
                    "Header too short: got {} bytes, need at least {} bytes",
                    actual_length, required_length
                )
            }
            ZeckFormatError::UnsupportedVersion {
                found_version,
                supported_version,
            } => {
                write!(
                    f,
                    "Unsupported file format version: found {}, maximum supported is {}",
                    found_version, supported_version
                )
            }
            ZeckFormatError::ReservedFlagsSet { flags } => {
                write!(
                    f,
                    "Reserved flags are set in header (flags: 0x{:02x}), indicating a newer format version",
                    flags
                )
            }
            ZeckFormatError::CompressionFailed {
                original_size,
                be_size,
                le_size,
            } => {
                write!(
                    f,
                    "Compression failed: original size {} bytes, big endian compressed size {} bytes, little endian compressed size {} bytes",
                    original_size, be_size, le_size
                )
            }
            ZeckFormatError::DecompressedTooLarge {
                expected_size,
                actual_size,
            } => {
                write!(
                    f,
                    "Decompressed data is too large: expected {} bytes, got {} bytes",
                    expected_size, actual_size
                )
            }
            ZeckFormatError::DataSizeTooLarge { size } => {
                write!(
                    f,
                    "Data size {} bytes is too large to be represented in the file format header",
                    size
                )
            }
        }
    }
}

impl std::error::Error for ZeckFormatError {}
