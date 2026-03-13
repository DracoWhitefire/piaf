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
            Self::Depth6  =>  6,
            Self::Depth8  =>  8,
            Self::Depth10 => 10,
            Self::Depth12 => 12,
            Self::Depth14 => 14,
            Self::Depth16 => 16,
        }
    }
}
