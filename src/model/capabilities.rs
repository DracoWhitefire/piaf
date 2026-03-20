use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::ParseWarning;
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

/// Sink for mode and warning writes from extension handlers.
///
/// Implemented by [`DisplayCapabilities`] (alloc builds, writes to `Vec`) and
/// `StaticDisplayCapabilities` (no_alloc builds, writes to a fixed array). Both handler traits —
/// [`ExtensionHandler`][crate::ExtensionHandler] and `StaticExtensionHandler` — write their output
/// through this trait so the core parsing logic is shared across build configurations.
pub trait ModeSink {
    /// Push a decoded video mode. Duplicate modes (same width, height, refresh rate, and
    /// interlace flag) are silently ignored.
    fn push_mode(&mut self, mode: VideoMode);
    /// Push a non-fatal warning encountered while processing a block.
    fn push_warning(&mut self, w: EdidWarning);
}

/// Output context passed to [`StaticExtensionHandler::process`][crate::StaticExtensionHandler].
///
/// Handlers write decoded data through this type rather than directly to a sink trait object.
/// This allows the context to grow over time — additional sink types can be added as
/// `Option` fields without changing the handler trait signature.
///
/// Currently wraps a [`ModeSink`] for video mode and warning output. Future fields will
/// provide access to identity, colorimetry, and other sink types as the static pipeline
/// is extended.
pub struct StaticContext<'a> {
    modes: &'a mut dyn ModeSink,
}

impl<'a> StaticContext<'a> {
    /// Creates a new context backed by `modes`.
    pub fn new(modes: &'a mut dyn ModeSink) -> Self {
        Self { modes }
    }

    /// Pushes a decoded video mode. Duplicate modes are silently ignored.
    pub fn push_mode(&mut self, mode: VideoMode) {
        self.modes.push_mode(mode);
    }

    /// Pushes a non-fatal warning encountered while processing a block.
    pub fn push_warning(&mut self, w: EdidWarning) {
        self.modes.push_warning(w);
    }
}

impl ModeSink for StaticContext<'_> {
    fn push_mode(&mut self, mode: VideoMode) {
        self.modes.push_mode(mode);
    }

    fn push_warning(&mut self, w: EdidWarning) {
        self.modes.push_warning(w);
    }
}

/// Consumer-facing display capability model derived from a parsed EDID.
///
/// All fields defined by the relevant specification are decoded and exposed here.
/// No field is omitted because it appears obscure or unlikely to be needed — that
/// judgement belongs to the consumer, not the library.
///
/// Fields are `Option` where the underlying EDID data may be absent or undecodable.
/// `None` means the value was not present or could not be reliably determined; it does
/// not imply the field is unimportant. The library never invents or defaults data.
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
    /// Physical display technology (e.g. TFT, OLED, PDP), from DisplayID 0x0C byte 0 bits 7:4.
    /// `None` when the Display Device Data Block is absent.
    pub display_technology: Option<crate::model::panel::DisplayTechnology>,
    /// Technology-specific sub-type code, from DisplayID 0x0C byte 0 bits 3:0 (raw, 0–15).
    /// `None` when the Display Device Data Block is absent.
    pub display_subtype: Option<u8>,
    /// Panel operating mode (continuous or non-continuous refresh), from DisplayID 0x0C byte 1 bits 3:0.
    /// `None` when the Display Device Data Block is absent.
    pub operating_mode: Option<crate::model::panel::OperatingMode>,
    /// Backlight type, from DisplayID 0x0C byte 1 bits 5:4.
    /// `None` when the Display Device Data Block is absent.
    pub backlight_type: Option<crate::model::panel::BacklightType>,
    /// Whether the panel uses a Data Enable (DE) signal, from DisplayID 0x0C byte 1 bit 6.
    /// `None` when the Display Device Data Block is absent.
    pub data_enable_used: Option<bool>,
    /// Data Enable signal polarity: `true` = positive, `false` = negative.
    /// From DisplayID 0x0C byte 1 bit 7. Valid only when `data_enable_used` is `Some(true)`.
    /// `None` when the Display Device Data Block is absent.
    pub data_enable_positive: Option<bool>,
    /// Native pixel format `(width_px, height_px)` from DisplayID 0x0C bytes 2–5.
    /// `None` when the Display Device Data Block is absent or either dimension is zero.
    pub native_pixels: Option<(u16, u16)>,
    /// Panel aspect ratio encoded as `(AR − 1) × 100` (raw byte), from DisplayID 0x0C byte 6.
    /// For example `77` represents approximately 16:9 (AR ≈ 1.77). `None` when the block is absent.
    pub panel_aspect_ratio_100: Option<u8>,
    /// Physical mounting orientation of the panel, from DisplayID 0x0C byte 7 bits 1:0.
    /// `None` when the Display Device Data Block is absent.
    pub physical_orientation: Option<crate::model::panel::PhysicalOrientation>,
    /// Panel rotation capability, from DisplayID 0x0C byte 7 bits 3:2.
    /// `None` when the Display Device Data Block is absent.
    pub rotation_capability: Option<crate::model::panel::RotationCapability>,
    /// Location of the zero (origin) pixel in the framebuffer, from DisplayID 0x0C byte 7 bits 5:4.
    /// `None` when the Display Device Data Block is absent.
    pub zero_pixel_location: Option<crate::model::panel::ZeroPixelLocation>,
    /// Fast-scan direction relative to H-sync, from DisplayID 0x0C byte 7 bits 7:6.
    /// `None` when the Display Device Data Block is absent.
    pub scan_direction: Option<crate::model::panel::ScanDirection>,
    /// Sub-pixel color filter arrangement, from DisplayID 0x0C byte 8.
    /// `None` when the Display Device Data Block is absent.
    pub subpixel_layout: Option<crate::model::panel::SubpixelLayout>,
    /// Pixel pitch `(horizontal_hundredths_mm, vertical_hundredths_mm)` in 0.01 mm units,
    /// from DisplayID 0x0C bytes 9–10. `None` when the Display Device Data Block is absent or
    /// either pitch is zero.
    pub pixel_pitch_hundredths_mm: Option<(u8, u8)>,
    /// Pixel response time in milliseconds, from DisplayID 0x0C byte 12.
    /// `None` when the Display Device Data Block is absent or the value is zero.
    pub pixel_response_time_ms: Option<u8>,
    /// Interface power sequencing timing parameters from DisplayID 0x0D.
    /// `None` when the Interface Power Sequencing Block is absent.
    pub power_sequencing: Option<crate::model::panel::PowerSequencing>,
    /// Display luminance transfer function from DisplayID 0x0E.
    /// `None` when the Transfer Characteristics Block is absent.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub transfer_characteristic: Option<crate::model::transfer::DisplayIdTransferCharacteristic>,
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
    ///
    /// In `alloc`/`std` builds this is a `Vec` and contains every warning.
    /// In bare `no_std` builds this is a fixed array capped at 8 entries;
    /// warnings beyond the first 8 are silently dropped.
    /// Use [`iter_warnings`][Self::iter_warnings] for build-portable access.
    #[cfg(any(feature = "alloc", feature = "std"))]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub warnings: Vec<ParseWarning>,
    /// Non-fatal conditions collected during base block decoding (bare `no_std` build).
    ///
    /// Capped at 8 entries. Use [`iter_warnings`][Self::iter_warnings] for
    /// build-portable access.
    #[cfg(not(any(feature = "alloc", feature = "std")))]
    pub warnings: [Option<EdidWarning>; 8],
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

impl DisplayCapabilities {
    /// Returns an iterator over all collected warnings.
    ///
    /// Portable across all build configurations. In bare `no_std` builds only the first
    /// 8 warnings are retained; prefer this method over accessing `warnings` directly
    /// to write code that compiles in all configurations.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn iter_warnings(&self) -> impl Iterator<Item = &ParseWarning> {
        self.warnings.iter()
    }

    /// Returns an iterator over all collected warnings.
    ///
    /// Portable across all build configurations. In bare `no_std` builds only the first
    /// 8 warnings are retained; prefer this method over accessing `warnings` directly
    /// to write code that compiles in all configurations.
    #[cfg(not(any(feature = "alloc", feature = "std")))]
    pub fn iter_warnings(&self) -> impl Iterator<Item = &EdidWarning> {
        self.warnings.iter().flatten()
    }

    /// Appends a warning. In bare `no_std` builds warnings beyond the first 8 are
    /// silently dropped.
    ///
    /// In `alloc`/`std` builds the value is boxed into a [`ParseWarning`]; handlers that
    /// already hold a `ParseWarning` can push directly to the `&mut Vec<ParseWarning>`
    /// parameter instead. This method exists for code paths (such as internal DTD decoding)
    /// that need a single call site portable across build configurations.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub(crate) fn push_warning(&mut self, w: impl core::error::Error + Send + Sync + 'static) {
        self.warnings.push(Arc::new(w));
    }

    /// Appends a warning. In bare `no_std` builds warnings beyond the first 8 are
    /// silently dropped.
    #[cfg(not(any(feature = "alloc", feature = "std")))]
    pub(crate) fn push_warning(&mut self, w: EdidWarning) {
        for slot in self.warnings.iter_mut() {
            if slot.is_none() {
                *slot = Some(w);
                return;
            }
        }
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl ModeSink for DisplayCapabilities {
    fn push_mode(&mut self, mode: VideoMode) {
        if !self.supported_modes.contains(&mode) {
            self.supported_modes.push(mode);
        }
    }

    fn push_warning(&mut self, w: EdidWarning) {
        self.warnings.push(Arc::new(w));
    }
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

/// No-alloc display capability model derived from a parsed EDID.
///
/// Contains the same scalar fields as [`DisplayCapabilities`] plus a fixed-capacity mode list.
/// Use `MAX_MODES` to set the maximum number of video modes that can be stored; 64 is a
/// reasonable default for most displays (real displays rarely declare more than ~40 modes).
/// Modes beyond the capacity are silently dropped, matching the behaviour of the 8-entry
/// warning cap.
///
/// Produced by `capabilities_from_edid_static`.
///
/// # Stack size
///
/// At `MAX_MODES = 64` this struct is approximately 3 KB. On targets with very limited stack
/// space, consider placing the value in a `static mut` or a statically-allocated buffer
/// rather than on the stack.
#[derive(Debug, Clone)]
pub struct StaticDisplayCapabilities<const MAX_MODES: usize> {
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
    /// Physical display technology (e.g. TFT, OLED, PDP), from DisplayID 0x0C byte 0 bits 7:4.
    /// `None` when the Display Device Data Block is absent.
    pub display_technology: Option<crate::model::panel::DisplayTechnology>,
    /// Technology-specific sub-type code, from DisplayID 0x0C byte 0 bits 3:0 (raw, 0–15).
    /// `None` when the Display Device Data Block is absent.
    pub display_subtype: Option<u8>,
    /// Panel operating mode (continuous or non-continuous refresh), from DisplayID 0x0C byte 1 bits 3:0.
    /// `None` when the Display Device Data Block is absent.
    pub operating_mode: Option<crate::model::panel::OperatingMode>,
    /// Backlight type, from DisplayID 0x0C byte 1 bits 5:4.
    /// `None` when the Display Device Data Block is absent.
    pub backlight_type: Option<crate::model::panel::BacklightType>,
    /// Whether the panel uses a Data Enable (DE) signal, from DisplayID 0x0C byte 1 bit 6.
    /// `None` when the Display Device Data Block is absent.
    pub data_enable_used: Option<bool>,
    /// Data Enable signal polarity: `true` = positive, `false` = negative.
    /// From DisplayID 0x0C byte 1 bit 7. Valid only when `data_enable_used` is `Some(true)`.
    /// `None` when the Display Device Data Block is absent.
    pub data_enable_positive: Option<bool>,
    /// Native pixel format `(width_px, height_px)` from DisplayID 0x0C bytes 2–5.
    /// `None` when the Display Device Data Block is absent or either dimension is zero.
    pub native_pixels: Option<(u16, u16)>,
    /// Panel aspect ratio encoded as `(AR − 1) × 100` (raw byte), from DisplayID 0x0C byte 6.
    /// For example `77` represents approximately 16:9 (AR ≈ 1.77). `None` when the block is absent.
    pub panel_aspect_ratio_100: Option<u8>,
    /// Physical mounting orientation of the panel, from DisplayID 0x0C byte 7 bits 1:0.
    /// `None` when the Display Device Data Block is absent.
    pub physical_orientation: Option<crate::model::panel::PhysicalOrientation>,
    /// Panel rotation capability, from DisplayID 0x0C byte 7 bits 3:2.
    /// `None` when the Display Device Data Block is absent.
    pub rotation_capability: Option<crate::model::panel::RotationCapability>,
    /// Location of the zero (origin) pixel in the framebuffer, from DisplayID 0x0C byte 7 bits 5:4.
    /// `None` when the Display Device Data Block is absent.
    pub zero_pixel_location: Option<crate::model::panel::ZeroPixelLocation>,
    /// Fast-scan direction relative to H-sync, from DisplayID 0x0C byte 7 bits 7:6.
    /// `None` when the Display Device Data Block is absent.
    pub scan_direction: Option<crate::model::panel::ScanDirection>,
    /// Sub-pixel color filter arrangement, from DisplayID 0x0C byte 8.
    /// `None` when the Display Device Data Block is absent.
    pub subpixel_layout: Option<crate::model::panel::SubpixelLayout>,
    /// Pixel pitch `(horizontal_hundredths_mm, vertical_hundredths_mm)` in 0.01 mm units,
    /// from DisplayID 0x0C bytes 9–10. `None` when the Display Device Data Block is absent or
    /// either pitch is zero.
    pub pixel_pitch_hundredths_mm: Option<(u8, u8)>,
    /// Pixel response time in milliseconds, from DisplayID 0x0C byte 12.
    /// `None` when the Display Device Data Block is absent or the value is zero.
    pub pixel_response_time_ms: Option<u8>,
    /// Interface power sequencing timing parameters from DisplayID 0x0D.
    /// `None` when the Interface Power Sequencing Block is absent.
    pub power_sequencing: Option<crate::model::panel::PowerSequencing>,
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
    /// Video modes decoded from the base block and extension handlers.
    ///
    /// Populated up to `MAX_MODES` entries. Use [`iter_modes`][Self::iter_modes] for a
    /// safe iterator that respects [`num_modes`][Self::num_modes].
    pub supported_modes: [Option<VideoMode>; MAX_MODES],
    /// Number of valid entries in [`supported_modes`][Self::supported_modes].
    pub num_modes: usize,
    /// Non-fatal conditions collected from the parser and all handlers.
    ///
    /// Capped at 8 entries; warnings beyond the first 8 are silently dropped.
    /// Use [`iter_warnings`][Self::iter_warnings] for a safe iterator.
    pub warnings: [Option<EdidWarning>; 8],
    /// Number of valid entries in [`warnings`][Self::warnings].
    pub num_warnings: usize,
}

impl<const MAX_MODES: usize> Default for StaticDisplayCapabilities<MAX_MODES> {
    fn default() -> Self {
        Self {
            manufacturer: None,
            manufacture_date: None,
            edid_version: None,
            product_code: None,
            serial_number: None,
            serial_number_string: None,
            display_name: None,
            unspecified_text: Default::default(),
            white_points: Default::default(),
            digital: false,
            color_bit_depth: None,
            display_technology: None,
            display_subtype: None,
            operating_mode: None,
            backlight_type: None,
            data_enable_used: None,
            data_enable_positive: None,
            native_pixels: None,
            panel_aspect_ratio_100: None,
            physical_orientation: None,
            rotation_capability: None,
            zero_pixel_location: None,
            scan_direction: None,
            subpixel_layout: None,
            pixel_pitch_hundredths_mm: None,
            pixel_response_time_ms: None,
            power_sequencing: None,
            chromaticity: Default::default(),
            gamma: None,
            display_features: None,
            digital_color_encoding: None,
            analog_color_type: None,
            video_interface: None,
            analog_sync_level: None,
            screen_size: None,
            min_v_rate: None,
            max_v_rate: None,
            min_h_rate_khz: None,
            max_h_rate_khz: None,
            max_pixel_clock_mhz: None,
            preferred_image_size_mm: None,
            timing_formula: None,
            color_management: None,
            // `Option<VideoMode>` is not `Copy` so array repeat syntax is unavailable;
            // `core::array::from_fn` constructs each slot individually.
            supported_modes: core::array::from_fn(|_| None),
            num_modes: 0,
            warnings: Default::default(),
            num_warnings: 0,
        }
    }
}

impl<const MAX_MODES: usize> StaticDisplayCapabilities<MAX_MODES> {
    /// Returns an iterator over decoded video modes.
    pub fn iter_modes(&self) -> impl Iterator<Item = &VideoMode> {
        self.supported_modes[..self.num_modes].iter().flatten()
    }

    /// Returns an iterator over collected warnings.
    pub fn iter_warnings(&self) -> impl Iterator<Item = &EdidWarning> {
        self.warnings[..self.num_warnings].iter().flatten()
    }
}

impl<const MAX_MODES: usize> ModeSink for StaticDisplayCapabilities<MAX_MODES> {
    fn push_mode(&mut self, mode: VideoMode) {
        if self.num_modes < MAX_MODES {
            // Dedup by (width, height, refresh_rate, interlaced) — the same criterion used
            // by the alloc pipeline when processing CEA SVDs.  The first entry for a given
            // resolution/rate wins, so a DTD-derived mode with full timing detail takes
            // precedence over a later SVD-derived mode with sparse timing detail.
            for i in 0..self.num_modes {
                if let Some(existing) = &self.supported_modes[i] {
                    if existing.width == mode.width
                        && existing.height == mode.height
                        && existing.refresh_rate == mode.refresh_rate
                        && existing.interlaced == mode.interlaced
                    {
                        return;
                    }
                }
            }
            self.supported_modes[self.num_modes] = Some(mode);
            self.num_modes += 1;
        }
        // Modes beyond capacity are silently dropped.
    }

    fn push_warning(&mut self, w: EdidWarning) {
        if self.num_warnings < 8 {
            self.warnings[self.num_warnings] = Some(w);
            self.num_warnings += 1;
        }
        // Warnings beyond capacity are silently dropped.
    }
}
