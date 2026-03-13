#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::Vec;

/// The output of [`parse_edid`][crate::parse_edid].
///
/// Stays close to the source structure: the base block is preserved as a raw 128-byte array,
/// extension blocks are stored the same way. Pass this to [`capabilities_from_edid`][crate::capabilities_from_edid]
/// to derive a consumer-friendly [`DisplayCapabilities`][crate::DisplayCapabilities].
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedEdid {
    /// The raw 128-byte EDID base block, validated and checksum-verified.
    pub base_block: [u8; 128],
    /// Raw 128-byte extension blocks, in the order they appear in the stream.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub extensions: Vec<[u8; 128]>,
    /// Non-fatal conditions encountered during parsing (e.g. unrecognised extension tags).
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub warnings: Vec<EdidWarning>,
}
