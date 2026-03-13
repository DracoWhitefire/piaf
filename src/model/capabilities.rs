#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::{String, Vec};
#[cfg(feature = "std")]
use core::any::Any;
#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::sync::Arc;

/// Trait for typed data stored in [`DisplayCapabilities::extension_data`] by custom handlers.
///
/// A blanket implementation covers any type that is `Any + Debug + Send + Sync`, so consumers
/// do not need to implement this trait manually — `#[derive(Debug)]` on a `Send + Sync` type
/// is sufficient.
#[cfg(feature = "std")]
pub trait ExtensionData: Any + core::fmt::Debug + Send + Sync {
    /// Returns `self` as `&dyn Any` to enable downcasting.
    fn as_any(&self) -> &dyn Any;
}

#[cfg(feature = "std")]
impl<T: Any + core::fmt::Debug + Send + Sync> ExtensionData for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A display video mode expressed as resolution and refresh rate.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VideoMode {
    /// Horizontal resolution in pixels.
    pub width: u16,
    /// Vertical resolution in pixels.
    pub height: u16,
    /// Refresh rate in Hz.
    pub refresh_rate: u8,
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
    /// Three-character PNP manufacturer ID (e.g. `"GSM"` for LG, `"SAM"` for Samsung).
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub manufacturer: Option<String>,
    /// Manufacture date or model year, decoded from bytes 16–17.
    pub manufacture_date: Option<crate::model::manufacture::ManufactureDate>,
    /// EDID specification version and revision, decoded from bytes 18–19.
    pub edid_version: Option<crate::model::edid::EdidVersion>,
    /// Manufacturer-assigned product code.
    pub product_code: Option<u16>,
    /// Manufacturer-assigned serial number, if encoded numerically in the base block.
    pub serial_number: Option<u32>,
    /// Serial number string from the monitor serial number descriptor (`0xFF`), if present.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub serial_number_string: Option<String>,
    /// Human-readable display name from the monitor name descriptor, if present.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub display_name: Option<String>,
    /// Unspecified ASCII text strings from `0xFE` descriptors, in slot order.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub unspecified_text: Vec<String>,
    /// Additional white points from the `0xFB` descriptor, if present.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub white_points: Vec<crate::model::color::WhitePoint>,
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
    pub min_v_rate: Option<u8>,
    /// Maximum supported vertical refresh rate in Hz.
    pub max_v_rate: Option<u8>,
    /// Minimum supported horizontal scan rate in kHz.
    pub min_h_rate_khz: Option<u8>,
    /// Maximum supported horizontal scan rate in kHz.
    pub max_h_rate_khz: Option<u8>,
    /// Maximum pixel clock in MHz.
    pub max_pixel_clock_mhz: Option<u16>,
    /// Video modes decoded from standard timing and detailed timing descriptors.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub supported_modes: Vec<VideoMode>,
    /// `true` if the display reports basic audio support (set by a CEA-861 handler).
    pub has_audio: bool,
    /// Non-fatal conditions collected from the parser and all handlers.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub warnings: Vec<EdidWarning>,
    /// Typed data attached by custom extension handlers, keyed by extension tag byte.
    /// Not serialized — use a custom handler to map this to a serializable form.
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub extension_data: HashMap<u8, Arc<dyn ExtensionData>>,
}

#[cfg(feature = "std")]
impl DisplayCapabilities {
    /// Store typed data from a custom handler, keyed by an extension tag.
    pub fn set_extension_data<T: ExtensionData>(&mut self, tag: u8, data: T) {
        self.extension_data.insert(tag, Arc::new(data));
    }

    /// Retrieve typed data previously stored by a handler for the given tag.
    /// Returns `None` if no data is stored for the tag or the type does not match.
    pub fn get_extension_data<T: Any>(&self, tag: u8) -> Option<&T> {
        self.extension_data.get(&tag)?.as_any().downcast_ref::<T>()
    }
}
