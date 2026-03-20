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

/// Physical interface standard type, decoded from Display Interface Data Block (0x0F)
/// byte 0 bits 3:0.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayInterfaceType {
    /// Undefined / not specified (`0x0`).
    Undefined,
    /// Analog (VGA) interface (`0x1`).
    Analog,
    /// LVDS single link (`0x2`).
    LvdsSingle,
    /// LVDS dual link (`0x3`).
    LvdsDual,
    /// TMDS single link — DVI-D single or HDMI (`0x4`).
    TmdsSingle,
    /// TMDS dual link — DVI-DL or HDMI dual (`0x5`).
    TmdsDual,
    /// Embedded DisplayPort (eDP) (`0x6`).
    EmbeddedDisplayPort,
    /// External DisplayPort (DP) (`0x7`).
    DisplayPort,
    /// Proprietary interface (`0x8`).
    Proprietary,
    /// Reserved or unrecognized value (`0x9`–`0xF`).
    Reserved(u8),
}

impl DisplayInterfaceType {
    /// Decodes the interface type from the lower 4 bits of byte 0.
    pub fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0x0 => Self::Undefined,
            0x1 => Self::Analog,
            0x2 => Self::LvdsSingle,
            0x3 => Self::LvdsDual,
            0x4 => Self::TmdsSingle,
            0x5 => Self::TmdsDual,
            0x6 => Self::EmbeddedDisplayPort,
            0x7 => Self::DisplayPort,
            0x8 => Self::Proprietary,
            v => Self::Reserved(v),
        }
    }
}

/// Content protection mechanism supported on the display interface, decoded from Display
/// Interface Data Block (0x0F) byte 6 bits 1:0.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceContentProtection {
    /// No content protection (`0`).
    None,
    /// High-bandwidth Digital Content Protection (HDCP) (`1`).
    Hdcp,
    /// DisplayPort Content Protection (DPCP) (`2`).
    Dpcp,
    /// Reserved or unrecognized value (`3`).
    Reserved(u8),
}

impl InterfaceContentProtection {
    /// Decodes the content protection type from a 2-bit value (bits 1:0 of byte 6).
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Self::None,
            1 => Self::Hdcp,
            2 => Self::Dpcp,
            v => Self::Reserved(v),
        }
    }
}

/// Display interface capabilities, decoded from the Display Interface Data Block
/// (DisplayID 1.x `0x0F`).
///
/// Identifies the physical interface type, link characteristics, pixel clock range,
/// and supported content protection mechanism.
///
/// Stored in [`DisplayCapabilities::display_id_interface`][crate::DisplayCapabilities::display_id_interface].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayIdInterface {
    /// Physical interface standard (LVDS, eDP, DisplayPort, TMDS, etc.).
    pub interface_type: DisplayInterfaceType,
    /// Whether spread-spectrum clocking is supported on this interface.
    pub spread_spectrum: bool,
    /// Number of data lanes or LVDS pairs (raw count from byte 1 bits 3:0).
    pub num_lanes: u8,
    /// Minimum pixel clock in units of 10 kHz (from bytes 2–3, LE uint16).
    pub min_pixel_clock_10khz: u32,
    /// Maximum pixel clock in units of 10 kHz (from bytes 4–5, LE uint16).
    pub max_pixel_clock_10khz: u32,
    /// Content protection mechanism supported on this interface.
    pub content_protection: InterfaceContentProtection,
}

/// Panel interface power sequencing timing parameters, decoded from the Interface Power
/// Sequencing Block (DisplayID 1.x `0x0D`).
///
/// Describes the minimum delays required when powering the display panel on and off.
/// All fields are raw counts in **2 ms units** per the DisplayID 1.x §4.11 specification;
/// multiply by 2 to obtain milliseconds.
///
/// The six parameters (T1–T6) follow the standard LVDS/eDP power sequencing model:
///
/// ```text
/// Power-on:   [VCC on] →T1→ [Signal on] →T2→ [Backlight on]
/// Power-off:  [Backlight off] →T3→ [Signal off] →T4→ [VCC off]
/// Minimum off time: T5 (VCC), T6 (Backlight)
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PowerSequencing {
    /// T1: minimum delay from power supply enable to interface signal valid (2 ms units).
    pub t1_power_to_signal: u8,
    /// T2: minimum delay from interface signal enable to backlight enable (2 ms units).
    pub t2_signal_to_backlight: u8,
    /// T3: minimum delay from backlight disable to interface signal disable (2 ms units).
    pub t3_backlight_to_signal_off: u8,
    /// T4: minimum delay from interface signal disable to power supply disable (2 ms units).
    pub t4_signal_to_power_off: u8,
    /// T5: minimum power supply off time before power can be re-applied (2 ms units).
    pub t5_power_off_min: u8,
    /// T6: minimum backlight off time (2 ms units).
    pub t6_backlight_off_min: u8,
}
