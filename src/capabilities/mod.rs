mod base;
mod cea861;

#[cfg(any(feature = "alloc", feature = "std"))]
pub use base::BaseBlockHandler;
pub use cea861::Cea861Flags;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use cea861::{
    AudioFormat, AudioFormatInfo, AudioSampleRates, Cea861Capabilities, ColorimetryBlock,
    ColorimetryFlags, DtcPointEncoding, HdmiAudioBlock, HdmiDscMaxSlices, HdmiForumDsc,
    HdmiForumFrl, HdmiForumSinkCap, HdmiVsdb, HdmiVsdbFlags, HdrDynamicMetadataDescriptor, HdrEotf,
    HdrStaticMetadata, InfoFrameDescriptor, RoomConfigurationBlock, ShortAudioDescriptor,
    SpeakerAllocation, SpeakerAllocationFlags, SpeakerAllocationFlags2, SpeakerAllocationFlags3,
    SpeakerLocationEntry, T7VtdbBlock, T8VtdbBlock, T10VtdbBlock, T10VtdbEntry,
    VendorSpecificBlock, VesaDisplayDeviceBlock, VesaTransferCharacteristic, VideoCapability,
    VideoCapabilityFlags, VtbExtBlock, infoframe_type,
};
pub use cea861::{CEA861_HANDLER, Cea861Handler, STANDARD_HANDLERS};

use crate::model::capabilities::{DisplayCapabilities, ModeSink, StaticDisplayCapabilities};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::ParseWarning;
use crate::model::edid::EdidSource;
use crate::model::extension::{ExtensionLibrary, StaticExtensionHandler};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::{Box, Vec};

#[cfg(any(feature = "alloc", feature = "std"))]
impl ExtensionLibrary {
    /// Creates a library pre-loaded with the built-in [`BaseBlockHandler`] and [`Cea861Handler`].
    ///
    /// This is the recommended starting point for most consumers. Additional handlers can be
    /// added after construction via [`add_base_handler`][ExtensionLibrary::add_base_handler]
    /// and [`register`][ExtensionLibrary::register].
    pub fn with_standard_handlers() -> Self {
        let mut lib = Self::with_standard_extensions();
        lib.add_base_handler(BaseBlockHandler);
        if let Some(cea) = lib.extensions.iter_mut().find(|e| e.tag == 0x02) {
            cea.handler = Some(Box::new(Cea861Handler));
        }
        lib
    }
}

/// Derives [`DisplayCapabilities`] from any [`EdidSource`] by running all registered handlers.
///
/// Base handlers are called first (in registration order), then extension block handlers are
/// called for each extension block whose tag matches a registered entry in `library`.
/// Warnings from all handlers are collected into [`DisplayCapabilities::warnings`].
pub fn capabilities_from_edid<T: EdidSource>(
    edid: &T,
    library: &ExtensionLibrary,
) -> DisplayCapabilities {
    #[cfg(any(feature = "alloc", feature = "std"))]
    let mut caps = DisplayCapabilities::default();
    #[cfg(not(any(feature = "alloc", feature = "std")))]
    let mut caps = DisplayCapabilities::default();

    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        let mut warnings: Vec<ParseWarning> = Vec::new();

        // 1. Process Base Block through all registered base handlers, in order.
        // Each handler receives a single-element slice containing the base block.
        let base = edid.base_block();
        for handler in &library.base_handlers {
            handler.process(&[base], &mut caps, &mut warnings);
        }

        // 2. Dispatch extension blocks: for each registered handler, collect all
        // blocks with its tag in stream order and call the handler once with the
        // full slice. Handlers are responsible for any multi-block reassembly.
        for metadata in &library.extensions {
            if let Some(handler) = &metadata.handler {
                let blocks: Vec<&[u8; 128]> = edid
                    .extension_blocks()
                    .filter(|b| b[0] == metadata.tag)
                    .collect();
                if !blocks.is_empty() {
                    handler.process(&blocks, &mut caps, &mut warnings);
                }
            }
        }

        edid.propagate_parse_warnings(&mut caps);
        caps.warnings.extend(warnings);
    }

    #[cfg(not(any(feature = "alloc", feature = "std")))]
    {
        let _ = library;
        base::decode_base_block(edid.base_block(), &mut caps);
    }

    caps
}

/// Derives [`StaticDisplayCapabilities`] from any [`EdidSource`] without heap allocation.
///
/// `N` is the maximum number of video modes the result can hold; 64 is a reasonable default
/// for most displays. Modes beyond `N` are silently dropped, matching the 8-entry warning cap.
///
/// Extension blocks are dispatched to the first handler in `handlers` whose
/// [`tag`][StaticExtensionHandler::tag] matches the block's tag byte. Only mode-producing
/// data (SVDs, DTDs, VTB-EXT timings) is extracted; rich metadata such as audio descriptors
/// and colorimetry blocks requires the alloc pipeline.
///
/// This function is a replacement for [`capabilities_from_edid`], not a complement —
/// calling both on the same EDID will process extension blocks twice.
pub fn capabilities_from_edid_static<const N: usize, T: EdidSource>(
    parsed: &T,
    handlers: &[&dyn StaticExtensionHandler],
) -> StaticDisplayCapabilities<N> {
    // Step 1 — Decode the base block into a temporary DisplayCapabilities.
    // This gives us all scalar fields and preferred_image_size_mm "for free" from the
    // existing base-block decoder, without duplicating its logic here.
    let mut base_caps = DisplayCapabilities::default();

    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        use crate::model::extension::ExtensionHandler;
        let mut w: Vec<ParseWarning> = Vec::new();
        base::BaseBlockHandler.process(&[parsed.base_block()], &mut base_caps, &mut w);
        // base_caps now holds scalar fields, preferred_image_size_mm, and supported_modes.
        // Handler-level warnings in `w` (Arc-boxed) are not copied to the static output;
        // use capabilities_from_edid if full warning detail is needed.
    }

    #[cfg(not(any(feature = "alloc", feature = "std")))]
    {
        base::decode_base_block(parsed.base_block(), &mut base_caps);
    }

    // Step 2 — Copy all scalar fields from the temporary into the static output.
    let mut caps = StaticDisplayCapabilities::<N> {
        manufacturer: base_caps.manufacturer,
        manufacture_date: base_caps.manufacture_date,
        edid_version: base_caps.edid_version,
        product_code: base_caps.product_code,
        serial_number: base_caps.serial_number,
        serial_number_string: base_caps.serial_number_string,
        display_name: base_caps.display_name,
        unspecified_text: base_caps.unspecified_text,
        white_points: base_caps.white_points,
        digital: base_caps.digital,
        color_bit_depth: base_caps.color_bit_depth,
        chromaticity: base_caps.chromaticity,
        gamma: base_caps.gamma,
        display_features: base_caps.display_features,
        digital_color_encoding: base_caps.digital_color_encoding,
        analog_color_type: base_caps.analog_color_type,
        video_interface: base_caps.video_interface,
        analog_sync_level: base_caps.analog_sync_level,
        screen_size: base_caps.screen_size,
        min_v_rate: base_caps.min_v_rate,
        max_v_rate: base_caps.max_v_rate,
        min_h_rate_khz: base_caps.min_h_rate_khz,
        max_h_rate_khz: base_caps.max_h_rate_khz,
        max_pixel_clock_mhz: base_caps.max_pixel_clock_mhz,
        preferred_image_size_mm: base_caps.preferred_image_size_mm,
        timing_formula: base_caps.timing_formula,
        color_management: base_caps.color_management,
        ..Default::default()
    };

    // Step 3 — Populate modes.
    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        // base_caps.supported_modes was populated by BaseBlockHandler above.
        for mode in base_caps.supported_modes {
            caps.push_mode(mode);
        }
    }

    #[cfg(not(any(feature = "alloc", feature = "std")))]
    {
        // No supported_modes in bare no_std DisplayCapabilities; call ungated functions directly.
        base::decode_base_modes(parsed.base_block(), &mut caps);
    }

    // Step 4 — Copy base-block warnings (bare no_std only; alloc warnings are Arc-boxed).
    #[cfg(not(any(feature = "alloc", feature = "std")))]
    {
        for w in base_caps.warnings.iter().flatten().copied() {
            caps.push_warning(w);
        }
    }

    // Step 5 — Dispatch extension blocks to the static handler slice.
    for ext in parsed.extension_blocks() {
        let tag = ext[0];
        if let Some(handler) = handlers.iter().find(|h| h.tag() == tag) {
            handler.process(ext, &mut caps);
        }
    }

    caps
}

#[cfg(test)]
#[cfg(any(feature = "alloc", feature = "std"))]
mod tests {
    use super::*;
    use crate::model::extension::ExtensionTagRegistry;
    use crate::parser::parse_edid;

    // Verifies that with_standard_handlers() wires BaseBlockHandler into the pipeline.
    // Handler-level assertions live in base.rs and cea861.rs.
    #[test]
    fn test_standard_handlers_are_wired() {
        let mut bytes = [0u8; 128];
        bytes[0..8].copy_from_slice(&crate::parser::EDID_HEADER);
        bytes[0x14] = 0x80; // Digital input flag

        let mut sum = 0u8;
        for &b in bytes[..127].iter() {
            sum = sum.wrapping_add(b);
        }
        bytes[127] = 0u8.wrapping_sub(sum);

        let registry = ExtensionTagRegistry::new();
        let parsed = parse_edid(&bytes, &registry).unwrap();
        let caps = capabilities_from_edid(&parsed, &ExtensionLibrary::with_standard_handlers());

        assert!(caps.digital);
    }
}
