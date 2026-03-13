#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
/// A non-fatal condition encountered while parsing or processing an EDID block.
///
/// Warnings are collected into [`ParsedEdid::warnings`][crate::ParsedEdid] (from the parser)
/// and into [`DisplayCapabilities::warnings`][crate::DisplayCapabilities] (from handlers).
pub enum EdidWarning {
    /// An extension block with an unrecognised tag was encountered.
    /// The inner value is the tag byte.
    UnknownExtension(u8),
    /// A 18-byte descriptor could not be decoded.
    DescriptorParseFailed,
}

/// A fatal error that prevents useful parsing of an EDID byte stream.
#[derive(Debug, Clone, PartialEq)]
pub enum EdidError {
    /// The byte slice is shorter than a complete EDID block (128 bytes per block).
    InvalidLength,
    /// The fixed 8-byte EDID header was not found at offset 0.
    InvalidHeader,
    /// The checksum byte does not make the block sum to zero.
    ChecksumMismatch,
}
