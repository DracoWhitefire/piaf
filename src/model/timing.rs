/// Video timing support reported in the display range limits descriptor (`0xFD`), byte 10.
///
/// Indicates which timing generation formula (if any) the display supports beyond the
/// explicitly listed modes.
#[non_exhaustive]
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
    /// CVT timing supported (byte 10 = `0x04`), with parameters from bytes 11–17.
    ///
    /// The display accepts Coordinated Video Timings within its range limits.
    /// Requires bit 0 of the Feature Support byte (`0x18`) to be set.
    Cvt(CvtSupportParams),
}

/// GTF secondary curve parameters decoded from a display range limits descriptor (`0xFD`).
///
/// Used when [`TimingFormula::SecondaryGtf`] is active (byte 10 = `0x02`).
/// The GTF formula selects the secondary curve for horizontal frequencies at or above
/// [`start_freq_khz`][Self::start_freq_khz] and the default curve below it.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GtfSecondaryParams {
    /// Start break frequency in kHz (byte 12 × 2).
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

/// CVT support parameters decoded from a display range limits descriptor (`0xFD`).
///
/// Used when [`TimingFormula::Cvt`] is active (byte 10 = `0x04`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CvtSupportParams {
    /// CVT standard version, encoded as two BCD nibbles (e.g., `0x11` = version 1.1).
    pub version: u8,
    /// Additional pixel clock precision: 6-bit value from byte 12 bits 7–2.
    ///
    /// The maximum pixel clock is: `(descriptor byte 9 × 10 MHz) − (pixel_clock_adjust × 0.25 MHz)`.
    /// When all six bits are set (`63`), byte 9 was already rounded up to the nearest 10 MHz.
    pub pixel_clock_adjust: u8,
    /// Maximum number of horizontal active pixels, or `None` if there is no limit.
    ///
    /// Computed as `8 × (byte 13 + 256 × (byte 12 bits 1–0))`. `None` when the 10-bit
    /// combined value is zero.
    pub max_h_active_pixels: Option<u16>,
    /// Aspect ratios the display supports for CVT-generated timings.
    pub supported_aspect_ratios: CvtAspectRatios,
    /// Preferred aspect ratio for CVT-generated timings, or `None` for a reserved value.
    pub preferred_aspect_ratio: Option<CvtAspectRatio>,
    /// Standard CVT blanking (normal blanking) is supported.
    pub standard_blanking: bool,
    /// Reduced CVT blanking is supported (preferred over standard blanking).
    pub reduced_blanking: bool,
    /// Display scaling capabilities.
    pub scaling: CvtScaling,
    /// Preferred vertical refresh rate in Hz, or `None` if byte 17 = `0x00` (reserved).
    pub preferred_v_rate: Option<u8>,
}

bitflags::bitflags! {
    /// Aspect ratios supported for CVT-generated timings (byte 14 of a `0xFD` descriptor).
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CvtAspectRatios: u8 {
        /// 4∶3 aspect ratio supported.
        const R4_3   = 0x80;
        /// 16∶9 aspect ratio supported.
        const R16_9  = 0x40;
        /// 16∶10 aspect ratio supported.
        const R16_10 = 0x20;
        /// 5∶4 aspect ratio supported.
        const R5_4   = 0x10;
        /// 15∶9 aspect ratio supported.
        const R15_9  = 0x08;
    }
}

bitflags::bitflags! {
    /// Display scaling capabilities reported in byte 16 of a `0xFD` CVT descriptor.
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CvtScaling: u8 {
        /// Input horizontal active pixels can exceed the display's preferred horizontal count.
        const HORIZONTAL_SHRINK  = 0x80;
        /// Input horizontal active pixels can be fewer than the display's preferred horizontal count.
        const HORIZONTAL_STRETCH = 0x40;
        /// Input vertical active lines can exceed the display's preferred vertical count.
        const VERTICAL_SHRINK    = 0x20;
        /// Input vertical active lines can be fewer than the display's preferred vertical count.
        const VERTICAL_STRETCH   = 0x10;
    }
}

/// Preferred aspect ratio for CVT-generated timings, decoded from byte 15 bits 7–5.
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CvtAspectRatio {
    /// 4∶3 preferred aspect ratio.
    R4_3,
    /// 16∶9 preferred aspect ratio.
    R16_9,
    /// 16∶10 preferred aspect ratio.
    R16_10,
    /// 5∶4 preferred aspect ratio.
    R5_4,
    /// 15∶9 preferred aspect ratio.
    R15_9,
}

impl CvtAspectRatio {
    fn from_bits(bits: u8) -> Option<Self> {
        match (bits >> 5) & 0x07 {
            0b000 => Some(Self::R4_3),
            0b001 => Some(Self::R16_9),
            0b010 => Some(Self::R16_10),
            0b011 => Some(Self::R5_4),
            0b100 => Some(Self::R15_9),
            _ => None, // 0b101–0b111 reserved
        }
    }
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
            0x04 => {
                let pixel_clock_adjust = (bytes[12] >> 2) & 0x3F;
                let h_raw = (bytes[13] as u16) + (((bytes[12] & 0x03) as u16) << 8);
                let max_h_active_pixels = if h_raw == 0 { None } else { Some(h_raw * 8) };
                Some(Self::Cvt(CvtSupportParams {
                    version: bytes[11],
                    pixel_clock_adjust,
                    max_h_active_pixels,
                    supported_aspect_ratios: CvtAspectRatios::from_bits_truncate(bytes[14]),
                    preferred_aspect_ratio: CvtAspectRatio::from_bits(bytes[15]),
                    standard_blanking: bytes[15] & 0x08 != 0,
                    reduced_blanking: bytes[15] & 0x10 != 0,
                    scaling: CvtScaling::from_bits_truncate(bytes[16]),
                    preferred_v_rate: if bytes[17] != 0 {
                        Some(bytes[17])
                    } else {
                        None
                    },
                }))
            }
            _ => None,
        }
    }
}
