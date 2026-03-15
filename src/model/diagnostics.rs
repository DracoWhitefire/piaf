/// A reference-counted, type-erased warning value.
///
/// Any type that implements [`core::error::Error`] + [`Send`] + [`Sync`] + `'static` can be
/// wrapped in a `ParseWarning`. The built-in library variants use [`EdidWarning`], but
/// custom handlers may push their own error types without wrapping them in `EdidWarning`.
///
/// Using [`Arc`][crate::model::prelude::Arc] (rather than `Box`) means `ParseWarning` is
/// [`Clone`], which lets warnings be copied from [`crate::ParsedEdid`] into
/// [`crate::DisplayCapabilities`] without consuming the parsed result.
///
/// To inspect a specific variant, use the inherent `downcast_ref` method available on
/// `dyn core::error::Error + Send + Sync + 'static` in `std` builds:
///
/// ```text
/// for w in caps.iter_warnings() {
///     if let Some(ew) = (**w).downcast_ref::<EdidWarning>() { ... }
/// }
/// ```
#[cfg(any(feature = "alloc", feature = "std"))]
pub type ParseWarning = crate::model::prelude::Arc<dyn core::error::Error + Send + Sync + 'static>;

/// A non-fatal condition encountered while parsing or processing an EDID block.
///
/// Warnings are collected into [`ParsedEdid::warnings`][crate::ParsedEdid] (from the parser)
/// and into [`DisplayCapabilities::warnings`][crate::DisplayCapabilities] (from handlers).
/// In `alloc`/`std` builds each entry is a [`ParseWarning`]; use `downcast_ref` to recover the
/// concrete type. In bare `no_std` builds this enum is used directly.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
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
    /// A Detailed Timing Descriptor slot was skipped because the byte slice passed to the
    /// decoder was shorter than the required 18 bytes.
    ///
    /// This indicates a malformed extension block that claimed to contain a DTD but did
    /// not supply enough data.
    #[error("DTD slot skipped: slice is shorter than the required 18 bytes")]
    DtdSlotTooShort,
    /// A Detailed Timing Descriptor slot was skipped because the pixel clock value
    /// would overflow during refresh rate calculation.
    ///
    /// This indicates a malformed or corrupted EDID: valid pixel clocks are at most
    /// a few hundred MHz (fits comfortably in a `u32` after scaling by 10 000).
    #[error("DTD slot skipped: pixel clock overflow during refresh rate calculation")]
    DtdPixelClockOverflow,
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
