use crate::model::capabilities::DisplayCapabilities;
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::ParseWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionHandler;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::{Arc, Vec};

mod descriptors;
mod header;
pub(crate) mod timings;

/// Decodes the base block fields that are available in all build configurations,
/// including bare `no_std` without `alloc`.
///
/// Called by [`capabilities_from_edid`][crate::capabilities_from_edid] in `no_std` builds
/// where the handler pipeline is unavailable. In `std`/`alloc` builds the full
/// [`BaseBlockHandler`] is used instead, which additionally decodes variable-length fields
/// and emits diagnostics.
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub(super) fn decode_base_block(base: &[u8; 128], caps: &mut DisplayCapabilities) {
    if !header::decode_header_fields(base, caps) {
        caps.push_warning(EdidWarning::InvalidManufacturerId);
    }
    descriptors::decode_descriptors_meta(base, caps);
}

/// Decodes the EDID base block into [`DisplayCapabilities`].
///
/// Extracts manufacturer ID, product code, serial number, input type, physical dimensions,
/// monitor name and range limit descriptors, standard timings, and detailed timing descriptors.
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug)]
pub struct BaseBlockHandler;

#[cfg(any(feature = "alloc", feature = "std"))]
impl ExtensionHandler for BaseBlockHandler {
    fn process(
        &self,
        base: &[u8; 128],
        caps: &mut DisplayCapabilities,
        warnings: &mut Vec<ParseWarning>,
    ) {
        if !header::decode_header_fields(base, caps) {
            warnings.push(Arc::new(EdidWarning::InvalidManufacturerId));
        }
        descriptors::decode_descriptors_meta(base, caps);
        descriptors::decode_descriptors_modes(base, caps);
        timings::decode_established_timings(base, caps);
        timings::decode_standard_timings(base, caps);
        // Use decode_dtd_slot (not decode_detailed_timings) to preserve
        // preferred_image_size_mm population and upgrade-in-place semantics.
        for i in 0..4 {
            let offset = 0x36 + (i * 18);
            timings::decode_dtd_slot(&base[offset..offset + 18], caps);
        }
    }
}
