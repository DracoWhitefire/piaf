/// A non-fatal condition encountered while parsing or processing an EDID block.
///
/// Warnings are collected into [`ParsedEdid::warnings`][crate::ParsedEdid] (from the parser)
/// and into [`DisplayCapabilities::warnings`][crate::DisplayCapabilities] (from handlers).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EdidWarning {
    /// An extension block with an unrecognised tag was encountered.
    /// The inner value is the tag byte.
    #[error("unknown extension block tag: {0:#04x}")]
    UnknownExtension(u8),
    /// A 18-byte descriptor could not be decoded.
    #[error("descriptor could not be parsed")]
    DescriptorParseFailed,
    /// The manufacturer ID bytes do not encode a valid PNP ID.
    ///
    /// Each of the three 5-bit fields must be in the range 1–26 (A–Z). Values of 0 or
    /// 27–31 indicate a corrupted or unprogrammed EEPROM.
    /// [`DisplayCapabilities::manufacturer`][crate::DisplayCapabilities::manufacturer]
    /// is left as `None`.
    #[error("manufacturer ID bytes do not encode a valid PNP ID")]
    InvalidManufacturerId,
    /// A data block inside an extension block declared a length that extends past the
    /// end of the data block collection. Remaining data blocks in the collection are skipped.
    #[error("data block length exceeds collection boundary")]
    MalformedDataBlock,
    /// The byte slice length does not match the size implied by the extension count.
    ///
    /// The EDID header declares `extension_count` extension blocks, so the expected
    /// size is `(1 + extension_count) × 128` bytes. Extra bytes are ignored but may
    /// indicate a driver bug, a KVM device, or a hotplug race.
    #[error("EDID byte length {actual} does not match expected {expected}")]
    SizeMismatch {
        /// The size implied by the extension count: `(1 + extension_count) × 128`.
        expected: usize,
        /// The actual length of the byte slice passed to [`crate::parse_edid`].
        actual: usize,
    },
}

/// A fatal error that prevents useful parsing of an EDID byte stream.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EdidError {
    /// The byte slice is shorter than a complete EDID block (128 bytes per block).
    #[error("input is shorter than a complete EDID block")]
    InvalidLength,
    /// The fixed 8-byte EDID header was not found at offset 0.
    #[error("EDID header not found at offset 0")]
    InvalidHeader,
    /// The checksum byte does not make the block sum to zero.
    #[error("block checksum does not sum to zero")]
    ChecksumMismatch,
}
