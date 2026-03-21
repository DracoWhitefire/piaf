/// Display technology type, decoded from Display Device Data Block (0x0C) byte 0 bits 7:4.
///
/// Identifies the physical display technology used by an embedded panel.
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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

/// Behavior when one or more tiles are missing from a tiled display, decoded from Tiled
/// Display Topology Data Block (0x12) byte 0 bits 5:4.
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileTopologyBehavior {
    /// Behavior is undefined (`0`).
    Undefined,
    /// No image is shown until all tiles are present and operational (`1`).
    RequireAllTiles,
    /// The image is scaled to fit whatever tiles are currently present (`2`).
    ScaleWhenMissing,
    /// Reserved or unrecognized value (`3`).
    Reserved(u8),
}

impl TileTopologyBehavior {
    /// Decodes the topology behavior from a 2-bit value (bits 5:4 of byte 0).
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Self::Undefined,
            1 => Self::RequireAllTiles,
            2 => Self::ScaleWhenMissing,
            v => Self::Reserved(v),
        }
    }
}

/// Bezel sizes around a single tile, decoded from the optional bezel bytes of the Tiled
/// Display Topology Data Block (0x12) when the `has_bezel_info` flag is set.
///
/// Each field is the bezel width or height in pixels at the tile's native resolution.
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileBezelInfo {
    /// Top bezel height in pixels.
    pub top_px: u8,
    /// Bottom bezel height in pixels.
    pub bottom_px: u8,
    /// Right bezel width in pixels.
    pub right_px: u8,
    /// Left bezel width in pixels.
    pub left_px: u8,
}

/// Tiled display topology, decoded from the Tiled Display Topology Data Block
/// (DisplayID 1.x `0x12`).
///
/// A tiled display is composed of multiple physical panels (tiles) arranged in a
/// rectangular grid. Each tile reports its own position and dimensions; the host
/// assembles the full image across all tiles.
///
/// Stored in [`DisplayCapabilities::tiled_topology`][crate::DisplayCapabilities::tiled_topology].
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayIdTiledTopology {
    /// All tiles are housed in a single physical enclosure.
    pub single_enclosure: bool,
    /// How the display behaves when one or more tiles are missing.
    pub topology_behavior: TileTopologyBehavior,
    /// Total number of horizontal tiles in the grid (1–16).
    pub h_tile_count: u8,
    /// Total number of vertical tiles in the grid (1–16).
    pub v_tile_count: u8,
    /// Zero-based column index of this tile within the grid.
    pub h_tile_location: u8,
    /// Zero-based row index of this tile within the grid.
    pub v_tile_location: u8,
    /// Native pixel width of this tile.
    pub tile_width_px: u16,
    /// Native pixel height of this tile.
    pub tile_height_px: u16,
    /// Per-edge bezel sizes, present when the block's `has_bezel_info` flag is set.
    pub bezel: Option<TileBezelInfo>,
}

/// Stereo content format, decoded from Stereo Display Interface Data Block (0x10) byte 0
/// bits 3:0.
///
/// Describes how left-eye and right-eye images are encoded in the video signal.
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StereoViewingMode {
    /// Field sequential — left and right frames alternate at double frame rate (`0`).
    ///
    /// Requires an external sync signal to the glasses. The polarity of that signal
    /// is encoded in [`DisplayIdStereoInterface::sync_polarity_positive`].
    FieldSequential,
    /// Side-by-side — left and right images packed horizontally at half width each (`1`).
    SideBySide,
    /// Top-and-bottom — left and right images packed vertically at half height each (`2`).
    TopAndBottom,
    /// Row interleaved — odd display rows carry the left eye, even rows the right eye (`3`).
    RowInterleaved,
    /// Column interleaved — odd display columns carry the left eye, even columns the right (`4`).
    ColumnInterleaved,
    /// Pixel interleaved / checkerboard — left and right pixels alternate in a
    /// checkerboard pattern (`5`).
    PixelInterleaved,
    /// Reserved or unrecognized value (`6`–`15`).
    Reserved(u8),
}

impl StereoViewingMode {
    /// Decodes the stereo viewing mode from the lower 4 bits of byte 0.
    pub fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0 => Self::FieldSequential,
            1 => Self::SideBySide,
            2 => Self::TopAndBottom,
            3 => Self::RowInterleaved,
            4 => Self::ColumnInterleaved,
            5 => Self::PixelInterleaved,
            v => Self::Reserved(v),
        }
    }
}

/// Physical interface used to deliver stereo synchronization to the glasses, decoded from
/// Stereo Display Interface Data Block (0x10) byte 1.
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StereoSyncInterface {
    /// Sync delivered via the display's own video connector — no dedicated stereo port (`0`).
    DisplayConnector,
    /// VESA 3-pin DIN stereo connector (`1`).
    VesaDin,
    /// Infrared (IR) wireless sync (`2`).
    Infrared,
    /// Radio frequency (RF) wireless sync (`3`).
    RadioFrequency,
    /// Reserved or unrecognized value (`4`–`255`).
    Reserved(u8),
}

impl StereoSyncInterface {
    /// Decodes the stereo sync interface from the raw byte 1 value.
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => Self::DisplayConnector,
            1 => Self::VesaDin,
            2 => Self::Infrared,
            3 => Self::RadioFrequency,
            v => Self::Reserved(v),
        }
    }
}

/// Stereo display interface parameters, decoded from the Stereo Display Interface Data Block
/// (DisplayID 1.x `0x10`).
///
/// Describes how stereoscopic 3D content is encoded and how synchronization is delivered
/// to active-shutter glasses.
///
/// Stored in [`DisplayCapabilities::stereo_interface`][crate::DisplayCapabilities::stereo_interface].
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayIdStereoInterface {
    /// How left-eye and right-eye images are encoded in the video signal.
    pub viewing_mode: StereoViewingMode,
    /// Polarity of the 3D sync signal sent to the glasses.
    ///
    /// `true` = positive (glasses open left eye on high); `false` = negative.
    /// Only meaningful for [`StereoViewingMode::FieldSequential`].
    pub sync_polarity_positive: bool,
    /// Physical channel used to deliver the sync signal to the glasses.
    pub sync_interface: StereoSyncInterface,
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
#[non_exhaustive]
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
