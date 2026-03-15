#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;
use crate::model::manufacture::{ManufacturerId, MonitorString};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::Arc;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::Vec;
#[cfg(any(feature = "alloc", feature = "std"))]
use core::any::Any;

/// Trait for typed data stored in [`DisplayCapabilities::extension_data`] by custom handlers.
///
/// A blanket implementation covers any type that is `Any + Debug + Send + Sync`, so consumers
/// do not need to implement this trait manually — `#[derive(Debug)]` on a `Send + Sync` type
/// is sufficient.
#[cfg(any(feature = "alloc", feature = "std"))]
pub trait ExtensionData: Any + core::fmt::Debug + Send + Sync {
    /// Returns `self` as `&dyn Any` to enable downcasting.
    fn as_any(&self) -> &dyn Any;
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<T: Any + core::fmt::Debug + Send + Sync> ExtensionData for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Stereo viewing support decoded from DTD byte 17 bits 6, 5, and 0.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StereoMode {
    /// Normal display; no stereo (bits 6–5 = `0b00`; bit 0 is don't-care).
    #[default]
    None,
    /// Field-sequential stereo, right image when stereo sync = 1 (bits 6–5 = `0b01`, bit 0 = 0).
    FieldSequentialRightFirst,
    /// Field-sequential stereo, left image when stereo sync = 1 (bits 6–5 = `0b10`, bit 0 = 0).
    FieldSequentialLeftFirst,
    /// 2-way interleaved stereo, right image on even lines (bits 6–5 = `0b01`, bit 0 = 1).
    TwoWayInterleavedRightEven,
    /// 2-way interleaved stereo, left image on even lines (bits 6–5 = `0b10`, bit 0 = 1).
    TwoWayInterleavedLeftEven,
    /// 4-way interleaved stereo (bits 6–5 = `0b11`, bit 0 = 0).
    FourWayInterleaved,
    /// Side-by-side interleaved stereo (bits 6–5 = `0b11`, bit 0 = 1).
    SideBySideInterleaved,
}

/// Sync signal definition decoded from DTD byte 17 bits 4–1.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDefinition {
    /// Analog composite sync (bit 4 = 0, bit 3 = 0).
    AnalogComposite {
        /// H-sync pulse present during V-sync (serrations).
        serrations: bool,
        /// Sync on all three RGB signals (`true`) or green only (`false`).
        sync_on_all_rgb: bool,
    },
    /// Bipolar analog composite sync (bit 4 = 0, bit 3 = 1).
    BipolarAnalogComposite {
        /// H-sync pulse present during V-sync (serrations).
        serrations: bool,
        /// Sync on all three RGB signals (`true`) or green only (`false`).
        sync_on_all_rgb: bool,
    },
    /// Digital composite sync on H-sync pin (bit 4 = 1, bit 3 = 0).
    DigitalComposite {
        /// H-sync pulse present during V-sync (serrations).
        serrations: bool,
        /// H-sync polarity outside V-sync: `true` = positive.
        h_sync_positive: bool,
    },
    /// Digital separate sync (bit 4 = 1, bit 3 = 1).
    DigitalSeparate {
        /// V-sync polarity: `true` = positive.
        v_sync_positive: bool,
        /// H-sync polarity: `true` = positive.
        h_sync_positive: bool,
    },
}

/// A display video mode expressed as resolution, refresh rate, and scan type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VideoMode {
    /// Horizontal resolution in pixels.
    pub width: u16,
    /// Vertical resolution in pixels.
    pub height: u16,
    /// Refresh rate in Hz.
    pub refresh_rate: u8,
    /// `true` for interlaced modes; `false` for progressive (the common case).
    pub interlaced: bool,
    /// Horizontal front porch in pixels (0 when not decoded from a DTD).
    pub h_front_porch: u16,
    /// Horizontal sync pulse width in pixels (0 when not decoded from a DTD).
    pub h_sync_width: u16,
    /// Vertical front porch in lines (0 when not decoded from a DTD).
    pub v_front_porch: u16,
    /// Vertical sync pulse width in lines (0 when not decoded from a DTD).
    pub v_sync_width: u16,
    /// Horizontal border width in pixels on each side of the active area (0 when not from a DTD).
    pub h_border: u8,
    /// Vertical border height in lines on each side of the active area (0 when not from a DTD).
    pub v_border: u8,
    /// Stereo viewing support (default `None` for non-DTD modes).
    pub stereo: StereoMode,
    /// Sync signal definition (`None` for non-DTD modes).
    pub sync: Option<SyncDefinition>,
}

/// Consumer-facing display capability model derived from a parsed EDID.
///
/// Fields are `Option` where the underlying EDID data may be absent or undecodable.
/// `None` means the information was not present or could not be reliably determined —
/// the library never invents data.
///
/// Produced by [`capabilities_from_edid`][crate::capabilities_from_edid].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct DisplayCapabilities {
    /// Three-character PNP manufacturer ID (e.g. `GSM` for LG, `SAM` for Samsung).
    pub manufacturer: Option<ManufacturerId>,
    /// Manufacture date or model year, decoded from bytes 16–17.
    pub manufacture_date: Option<crate::model::manufacture::ManufactureDate>,
    /// EDID specification version and revision, decoded from bytes 18–19.
    pub edid_version: Option<crate::model::edid::EdidVersion>,
    /// Manufacturer-assigned product code.
    pub product_code: Option<u16>,
    /// Manufacturer-assigned serial number, if encoded numerically in the base block.
    pub serial_number: Option<u32>,
    /// Serial number string from the monitor serial number descriptor (`0xFF`), if present.
    pub serial_number_string: Option<MonitorString>,
    /// Human-readable display name from the monitor name descriptor, if present.
    pub display_name: Option<MonitorString>,
    /// Unspecified ASCII text strings from `0xFE` descriptors, in descriptor slot order.
    ///
    /// Up to four entries (one per descriptor slot). Each slot is `None` if the corresponding
    /// descriptor was not a `0xFE` entry.
    pub unspecified_text: [Option<MonitorString>; 4],
    /// Additional white points from the `0xFB` descriptor.
    ///
    /// Up to two entries (the EDID `0xFB` descriptor has two fixed slots). Each slot is
    /// `None` if the corresponding entry was unused (index byte `0x00`).
    pub white_points: [Option<crate::model::color::WhitePoint>; 2],
    /// `true` if the display uses a digital input interface.
    pub digital: bool,
    /// Color bit depth per primary channel, decoded from byte `0x14` bits 6–4.
    /// `None` for analog displays or when the field is undefined or reserved.
    pub color_bit_depth: Option<crate::model::color::ColorBitDepth>,
    /// CIE xy chromaticity coordinates for the color primaries and white point,
    /// decoded from bytes `0x19`–`0x22`.
    pub chromaticity: crate::model::color::Chromaticity,
    /// Display gamma from byte `0x17`. `None` if the display did not specify a gamma value.
    pub gamma: Option<crate::model::color::DisplayGamma>,
    /// Display feature support flags from byte `0x18`.
    pub display_features: Option<crate::model::features::DisplayFeatureFlags>,
    /// Supported color encoding formats, decoded from byte `0x18` bits 4–3.
    /// Only populated for EDID 1.4+ digital displays.
    pub digital_color_encoding: Option<crate::model::color::DigitalColorEncoding>,
    /// Color type, decoded from byte `0x18` bits 4–3.
    /// Only populated for analog displays; `None` for the undefined value (`0b11`).
    pub analog_color_type: Option<crate::model::color::AnalogColorType>,
    /// Video interface type, decoded from byte `0x14` bits 3–0.
    /// `None` for analog displays or when the field is undefined or reserved.
    pub video_interface: Option<crate::model::input::VideoInterface>,
    /// Analog sync and video white levels, decoded from byte `0x14` bits 6–5.
    /// Only populated for analog displays.
    pub analog_sync_level: Option<crate::model::input::AnalogSyncLevel>,
    /// Physical screen dimensions or aspect ratio, decoded from bytes `0x15`–`0x16`.
    /// `None` when both bytes are zero (undefined).
    pub screen_size: Option<crate::model::screen::ScreenSize>,
    /// Minimum supported vertical refresh rate in Hz.
    pub min_v_rate: Option<u16>,
    /// Maximum supported vertical refresh rate in Hz.
    pub max_v_rate: Option<u16>,
    /// Minimum supported horizontal scan rate in kHz.
    pub min_h_rate_khz: Option<u16>,
    /// Maximum supported horizontal scan rate in kHz.
    pub max_h_rate_khz: Option<u16>,
    /// Maximum pixel clock in MHz.
    pub max_pixel_clock_mhz: Option<u16>,
    /// Physical image area dimensions in millimetres `(width_mm, height_mm)`, decoded from
    /// the first detailed timing descriptor that provides non-zero values (bytes 12–14).
    ///
    /// More precise than [`screen_size`][Self::screen_size] (which is in cm from the base
    /// block header). `None` when all DTD image-size fields are zero.
    pub preferred_image_size_mm: Option<(u16, u16)>,
    /// Video timing formula reported in the display range limits descriptor (`0xFD`), byte 10.
    pub timing_formula: Option<crate::model::timing::TimingFormula>,
    /// DCM polynomial coefficients decoded from a Color Management Data descriptor (`0xF9`).
    pub color_management: Option<crate::model::color::ColorManagementData>,
    /// Video modes decoded from standard timing and detailed timing descriptors.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub supported_modes: Vec<VideoMode>,
    /// Non-fatal conditions collected from the parser and all handlers.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub warnings: Vec<EdidWarning>,
    /// Typed data attached by extension handlers, keyed by extension tag byte.
    ///
    /// Uses a `Vec` of `(tag, data)` pairs rather than a `HashMap` so that this field is
    /// available in `alloc`-only (no_std) builds. The number of distinct extension tags in
    /// any real EDID is small enough that linear scan is negligible.
    ///
    /// Not serialized — use a custom handler to map this to a serializable form.
    #[cfg(any(feature = "alloc", feature = "std"))]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub extension_data: Vec<(u8, Arc<dyn ExtensionData>)>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl DisplayCapabilities {
    /// Store typed data from a handler, keyed by an extension tag.
    /// Replaces any previously stored entry for the same tag.
    pub fn set_extension_data<T: ExtensionData>(&mut self, tag: u8, data: T) {
        if let Some(entry) = self.extension_data.iter_mut().find(|(t, _)| *t == tag) {
            entry.1 = Arc::new(data);
        } else {
            self.extension_data.push((tag, Arc::new(data)));
        }
    }

    /// Retrieve typed data previously stored by a handler for the given tag.
    /// Returns `None` if no data is stored for the tag or the type does not match.
    pub fn get_extension_data<T: Any>(&self, tag: u8) -> Option<&T> {
        self.extension_data
            .iter()
            .find(|(t, _)| *t == tag)
            // `**data` deref-chains through `&` then through Arc's Deref to reach
            // `dyn ExtensionData`, forcing vtable dispatch for `as_any()`.
            // Calling `.as_any()` on `&Arc<dyn ExtensionData>` would hit the blanket
            // `ExtensionData` impl for Arc itself and return the wrong TypeId.
            .and_then(|(_, data)| (**data).as_any().downcast_ref::<T>())
    }
}
