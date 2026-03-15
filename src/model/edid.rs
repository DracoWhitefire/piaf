#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::ParseWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::Vec;

/// EDID specification version and revision, decoded from base block bytes 18–19.
///
/// Most displays in use report version 1 with revision 3 or 4.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdidVersion {
    /// EDID version number (byte 18). Always `1` for all current displays.
    pub version: u8,
    /// EDID revision number (byte 19).
    pub revision: u8,
}

impl core::fmt::Display for EdidVersion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}", self.version, self.revision)
    }
}

/// The output of [`parse_edid`][crate::parse_edid].
///
/// Stays close to the source structure: the base block is preserved as a raw 128-byte array,
/// extension blocks are stored the same way. Pass this to [`capabilities_from_edid`][crate::capabilities_from_edid]
/// to derive a consumer-friendly [`DisplayCapabilities`][crate::DisplayCapabilities].
#[derive(Debug, Clone)]
pub struct ParsedEdid {
    /// The raw 128-byte EDID base block, validated and checksum-verified.
    pub base_block: [u8; 128],
    /// Raw 128-byte extension blocks, in the order they appear in the stream.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub extensions: Vec<[u8; 128]>,
    /// Non-fatal conditions encountered during parsing (e.g. unrecognised extension tags).
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub warnings: Vec<ParseWarning>,
}
