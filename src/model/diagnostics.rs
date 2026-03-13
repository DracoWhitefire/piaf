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
