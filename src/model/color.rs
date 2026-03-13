/// Display gamma, decoded from EDID base block byte `0x17`.
///
/// Gamma is encoded as `(value * 100) - 100`, so a stored byte of `120` represents
/// gamma 2.20. A byte value of `0xFF` means gamma is undefined; use `None` on
/// [`DisplayCapabilities`][crate::DisplayCapabilities] in that case.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayGamma(u8);

impl DisplayGamma {
    /// Decodes EDID byte `0x17` into a `DisplayGamma`.
    ///
    /// Returns `None` if the byte is `0xFF` (gamma not specified).
    pub fn from_edid_byte(byte: u8) -> Option<Self> {
        if byte == 0xFF {
            None
        } else {
            Some(Self(byte))
        }
    }

    /// Returns the raw encoded byte.
    pub fn raw(&self) -> u8 {
        self.0
    }

    /// Returns the gamma value as a floating-point number (e.g. `2.20`).
    pub fn value(&self) -> f32 {
        (self.0 as f32 + 100.0) / 100.0
    }
}

/// Supported color encoding formats for a digital display, decoded from EDID base block
/// byte `0x18` bits 4–3.
///
/// Defined for EDID 1.4+ digital inputs. On EDID 1.3 displays this field has a different
/// meaning and is not decoded here.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigitalColorEncoding {
    /// Only RGB 4:4:4 is supported.
    Rgb444,
    /// RGB 4:4:4 and YCbCr 4:4:4 are supported.
    Rgb444YCbCr444,
    /// RGB 4:4:4 and YCbCr 4:2:2 are supported.
    Rgb444YCbCr422,
    /// RGB 4:4:4, YCbCr 4:4:4, and YCbCr 4:2:2 are supported.
    Rgb444YCbCr444YCbCr422,
}

impl DigitalColorEncoding {
    /// Decodes bits 4–3 of EDID byte `0x18` for a digital display.
    pub(crate) fn from_edid_bits(bits: u8) -> Self {
        match (bits >> 3) & 0x03 {
            0b00 => Self::Rgb444,
            0b01 => Self::Rgb444YCbCr444,
            0b10 => Self::Rgb444YCbCr422,
            _    => Self::Rgb444YCbCr444YCbCr422,
        }
    }
}

/// Display color type for an analog display, decoded from EDID base block byte `0x18` bits 4–3.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalogColorType {
    /// Monochrome or grayscale display.
    Monochrome,
    /// RGB color display.
    Rgb,
    /// Non-RGB multicolor display.
    NonRgb,
}

impl AnalogColorType {
    /// Decodes bits 4–3 of EDID byte `0x18` for an analog display.
    ///
    /// Returns `None` for the undefined value (`0b11`).
    pub(crate) fn from_edid_bits(bits: u8) -> Option<Self> {
        match (bits >> 3) & 0x03 {
            0b00 => Some(Self::Monochrome),
            0b01 => Some(Self::Rgb),
            0b10 => Some(Self::NonRgb),
            _    => None,
        }
    }
}

/// Color bit depth per primary color channel, decoded from EDID base block byte `0x14` bits 6–4.
///
/// Only valid for digital input displays. `None` is used for the undefined (0b000) and
/// reserved (0b111) values.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorBitDepth {
    /// 6 bits per primary color channel.
    Depth6,
    /// 8 bits per primary color channel.
    Depth8,
    /// 10 bits per primary color channel.
    Depth10,
    /// 12 bits per primary color channel.
    Depth12,
    /// 14 bits per primary color channel.
    Depth14,
    /// 16 bits per primary color channel.
    Depth16,
}

impl ColorBitDepth {
    /// Decodes bits 6–4 of EDID byte `0x14` into a `ColorBitDepth`.
    ///
    /// Returns `None` for the undefined (0b000) and reserved (0b111) values.
    pub(crate) fn from_edid_bits(bits: u8) -> Option<Self> {
        match bits & 0x07 {
            0b001 => Some(Self::Depth6),
            0b010 => Some(Self::Depth8),
            0b011 => Some(Self::Depth10),
            0b100 => Some(Self::Depth12),
            0b101 => Some(Self::Depth14),
            0b110 => Some(Self::Depth16),
            _ => None, // 0b000 = undefined, 0b111 = reserved
        }
    }

    /// Returns the number of bits per primary color channel.
    pub fn bits_per_primary(&self) -> u8 {
        match self {
            Self::Depth6 => 6,
            Self::Depth8 => 8,
            Self::Depth10 => 10,
            Self::Depth12 => 12,
            Self::Depth14 => 14,
            Self::Depth16 => 16,
        }
    }
}
