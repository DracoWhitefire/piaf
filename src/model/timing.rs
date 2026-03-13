/// Video timing support reported in the display range limits descriptor (`0xFD`), byte 10.
///
/// Indicates which timing generation formula (if any) the display supports beyond the
/// explicitly listed modes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimingFormula {
    /// Default GTF supported (byte 10 = `0x00`).
    ///
    /// The display accepts any timing within its range limits that satisfies the
    /// default GTF parameters. Requires bit 0 of the Feature Support byte (`0x18`) to be set.
    DefaultGtf,
    /// Range limits only; no secondary timing formula (byte 10 = `0x01`).
    ///
    /// The display supports only the video timing modes explicitly listed in the EDID.
    RangeLimitsOnly,
    /// Secondary GTF curve supported (byte 10 = `0x02`).
    ///
    /// The display accepts timings using either the default GTF or the secondary GTF curve
    /// whose parameters are stored in bytes 12–17.
    SecondaryGtf(GtfSecondaryParams),
    /// CVT timing supported (byte 10 = `0x04`).
    ///
    /// The display accepts Coordinated Video Timings within its range limits.
    /// Requires bit 0 of the Feature Support byte (`0x18`) to be set.
    Cvt,
}

/// GTF secondary curve parameters decoded from a display range limits descriptor (`0xFD`).
///
/// Used when [`TimingFormula::SecondaryGtf`] is active (byte 10 = `0x02`).
/// The GTF formula selects the secondary curve for horizontal frequencies above
/// [`start_freq_khz`][Self::start_freq_khz] and the default curve below it.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GtfSecondaryParams {
    /// Start break frequency in kHz (byte 12 × 2).
    ///
    /// The secondary GTF curve is applied for horizontal frequencies at or above this value.
    pub start_freq_khz: u16,
    /// GTF `C` parameter (0–127); byte 13 ÷ 2.
    pub c: u8,
    /// GTF `M` parameter (0–65535); bytes 14–15, little-endian.
    pub m: u16,
    /// GTF `K` parameter (0–255); byte 16.
    pub k: u8,
    /// GTF `J` parameter (0–127); byte 17 ÷ 2.
    pub j: u8,
}

impl TimingFormula {
    /// Decodes the timing formula from bytes 10–17 of a `0xFD` descriptor.
    pub(crate) fn from_descriptor_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes[10] {
            0x00 => Some(Self::DefaultGtf),
            0x01 => Some(Self::RangeLimitsOnly),
            0x02 => Some(Self::SecondaryGtf(GtfSecondaryParams {
                start_freq_khz: (bytes[12] as u16) * 2,
                c: bytes[13] / 2,
                m: ((bytes[15] as u16) << 8) | (bytes[14] as u16),
                k: bytes[16],
                j: bytes[17] / 2,
            })),
            0x04 => Some(Self::Cvt),
            _ => None,
        }
    }
}
