#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::Vec;

/// Bit-depth encoding for transfer characteristic sample points, decoded from
/// the Transfer Characteristics Block (DisplayID 1.x `0x0E`) byte 0 bits 7:6.
///
/// The same three encodings are used by the CEA-861 VESA Display Transfer
/// Characteristic Data Block (standard tag `0x05`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferPointEncoding {
    /// 8 bits per luminance point: one byte per sample, range 0–255.
    Bits8,
    /// 10 bits per luminance point: 5 bytes packed MSB-first per 4 samples, range 0–1023.
    Bits10,
    /// 12 bits per luminance point: 3 bytes packed MSB-first per 2 samples, range 0–4095.
    Bits12,
}

/// Transfer curve sample data decoded from the DisplayID Transfer Characteristics
/// Block (`0x0E`).
///
/// Sample values are normalized to `[0.0, 1.0]` and represent evenly-spaced input
/// levels from black (`0`) to white (`1`).
#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum TransferCurve {
    /// Single luminance transfer curve (grayscale / single-channel mode).
    Luminance(Vec<f32>),
    /// Separate per-primary transfer curves (multi-channel mode, byte 0 bit 5 set).
    Rgb {
        /// Red primary transfer curve.
        red: Vec<f32>,
        /// Green primary transfer curve.
        green: Vec<f32>,
        /// Blue primary transfer curve.
        blue: Vec<f32>,
    },
}

/// Decoded Transfer Characteristics Block (DisplayID 1.x `0x0E`).
///
/// Encodes the display's native luminance transfer function as a sequence of sample
/// points at evenly-spaced input levels from 0 (black) to 1 (white). Stored in
/// [`DisplayCapabilities::transfer_characteristic`][crate::DisplayCapabilities::transfer_characteristic].
#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayIdTransferCharacteristic {
    /// Bit depth used to pack each sample value.
    pub encoding: TransferPointEncoding,
    /// Sample data — either a single luminance curve or separate R, G, B curves.
    pub curve: TransferCurve,
}
