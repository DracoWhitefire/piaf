/// Display technology type, decoded from Display Device Data Block (0x0C) byte 0 bits 7:4.
///
/// Identifies the physical display technology used by an embedded panel.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayTechnology {
    /// Thin-film transistor LCD, unspecified sub-type (`0`).
    Tft,
    /// DSTN or STN (dual-scan or super-twisted nematic LCD) (`1`).
    DstnStn,
    /// TFT-IPS or super-TFT (in-plane switching) (`2`).
    TftIps,
    /// TFT-MVA or TFT-PVA (multi-domain / patterned vertical alignment) (`3`).
    TftMva,
    /// CRT (cathode ray tube) (`4`).
    Crt,
    /// PDP (plasma display panel) (`5`).
    Pdp,
    /// OLED or ELED (organic light emitting) (`6`).
    Oled,
    /// EL (electroluminescent) (`7`).
    El,
    /// FED or SED (field emission / surface-conduction electron emission) (`8`).
    FedSed,
    /// LCoS (liquid crystal on silicon) (`9`).
    Lcos,
    /// Reserved or undefined value (`10`–`15`).
    Unknown(u8),
}

impl DisplayTechnology {
    /// Decodes the display technology from a 4-bit nibble (bits 7:4 of byte 0).
    pub fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0 => Self::Tft,
            1 => Self::DstnStn,
            2 => Self::TftIps,
            3 => Self::TftMva,
            4 => Self::Crt,
            5 => Self::Pdp,
            6 => Self::Oled,
            7 => Self::El,
            8 => Self::FedSed,
            9 => Self::Lcos,
            v => Self::Unknown(v),
        }
    }
}

/// Panel operating mode, decoded from Display Device Data Block (0x0C) byte 1 bits 3:0.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingMode {
    /// Continuous (free-running) refresh (`0`).
    Continuous,
    /// Non-continuous (event-driven or line-at-a-time) refresh (`1`).
    NonContinuous,
    /// Reserved or undefined value (`2`–`15`).
    Unknown(u8),
}

impl OperatingMode {
    /// Decodes the operating mode from the lower 4 bits of byte 1.
    pub fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0 => Self::Continuous,
            1 => Self::NonContinuous,
            v => Self::Unknown(v),
        }
    }
}

/// Backlight type, decoded from Display Device Data Block (0x0C) byte 1 bits 5:4.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BacklightType {
    /// No backlight, or not applicable (`0`).
    None,
    /// AC fluorescent (CCFL) backlight (`1`).
    AcFluorescent,
    /// DC-powered backlight (LED or other) (`2`).
    Dc,
    /// Reserved value (`3`).
    Unknown(u8),
}

impl BacklightType {
    /// Decodes the backlight type from a 2-bit value (bits 5:4 of byte 1).
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Self::None,
            1 => Self::AcFluorescent,
            2 => Self::Dc,
            v => Self::Unknown(v),
        }
    }
}

/// Physical mounting orientation of the panel, decoded from Display Device Data Block (0x0C)
/// byte 7 bits 1:0.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalOrientation {
    /// Landscape — wider than tall (`0`).
    Landscape,
    /// Portrait — taller than wide (`1`).
    Portrait,
    /// Orientation not defined; may be freely rotated (`2`).
    NotDefined,
    /// Undefined / reserved encoding (`3`).
    Undefined,
}

impl PhysicalOrientation {
    /// Decodes the physical orientation from a 2-bit value (bits 1:0 of byte 7).
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Self::Landscape,
            1 => Self::Portrait,
            2 => Self::NotDefined,
            _ => Self::Undefined,
        }
    }
}

/// Rotation capability, decoded from Display Device Data Block (0x0C) byte 7 bits 3:2.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationCapability {
    /// No display rotation supported (`0`).
    None,
    /// 90° clockwise rotation supported (`1`).
    Cw90,
    /// 180° rotation supported (`2`).
    Deg180,
    /// 270° clockwise (90° counter-clockwise) rotation supported (`3`).
    Cw270,
}

impl RotationCapability {
    /// Decodes the rotation capability from a 2-bit value (bits 3:2 of byte 7).
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Self::None,
            1 => Self::Cw90,
            2 => Self::Deg180,
            _ => Self::Cw270,
        }
    }
}

/// Location of the zero pixel (the upper-left pixel in the framebuffer), decoded from
/// Display Device Data Block (0x0C) byte 7 bits 5:4.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroPixelLocation {
    /// Zero pixel is at the upper-left corner (`0`).
    UpperLeft,
    /// Zero pixel is at the upper-right corner (`1`).
    UpperRight,
    /// Zero pixel is at the lower-left corner (`2`).
    LowerLeft,
    /// Zero pixel is at the lower-right corner (`3`).
    LowerRight,
}

impl ZeroPixelLocation {
    /// Decodes the zero pixel location from a 2-bit value (bits 5:4 of byte 7).
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Self::UpperLeft,
            1 => Self::UpperRight,
            2 => Self::LowerLeft,
            _ => Self::LowerRight,
        }
    }
}

/// Scan direction of the fast (horizontal) scan relative to H-sync, decoded from
/// Display Device Data Block (0x0C) byte 7 bits 7:6.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanDirection {
    /// Scan direction not defined (`0`).
    NotDefined,
    /// Fast scan follows the H-sync direction; slow scan follows V-sync direction (`1`).
    Normal,
    /// Fast scan direction is opposite to H-sync; slow scan opposite to V-sync (`2`).
    Reversed,
    /// Reserved value (`3`).
    Reserved,
}

impl ScanDirection {
    /// Decodes the scan direction from a 2-bit value (bits 7:6 of byte 7).
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Self::NotDefined,
            1 => Self::Normal,
            2 => Self::Reversed,
            _ => Self::Reserved,
        }
    }
}

/// Sub-pixel layout, decoded from Display Device Data Block (0x0C) byte 8.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubpixelLayout {
    /// Sub-pixel arrangement not defined (`0x00`).
    NotDefined,
    /// RGB vertical stripes (`0x01`).
    RgbVertical,
    /// BGR vertical stripes (`0x02`).
    BgrVertical,
    /// RGB horizontal stripes (`0x03`).
    RgbHorizontal,
    /// BGR horizontal stripes (`0x04`).
    BgrHorizontal,
    /// Quad arrangement: RGBG (`0x05`).
    QuadRgbg,
    /// Quad arrangement: BGRG (`0x06`).
    QuadBgrg,
    /// Delta (triangular) RGB arrangement (`0x07`).
    DeltaRgb,
    /// Delta (triangular) BGR arrangement (`0x08`).
    DeltaBgr,
    /// Reserved or proprietary layout (`0x09`–`0xFF`).
    Unknown(u8),
}

impl SubpixelLayout {
    /// Decodes the sub-pixel layout from the raw byte 8 value.
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => Self::NotDefined,
            0x01 => Self::RgbVertical,
            0x02 => Self::BgrVertical,
            0x03 => Self::RgbHorizontal,
            0x04 => Self::BgrHorizontal,
            0x05 => Self::QuadRgbg,
            0x06 => Self::QuadBgrg,
            0x07 => Self::DeltaRgb,
            0x08 => Self::DeltaBgr,
            v => Self::Unknown(v),
        }
    }
}
