use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionHandler;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::Vec;

mod descriptors;
mod header;
mod timings;

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
        warnings: &mut Vec<EdidWarning>,
    ) {
        header::decode_header_fields(base, caps, warnings);
        descriptors::decode_descriptors(base, caps);
        timings::decode_established_timings(base, caps);
        timings::decode_standard_timings(base, caps);
        timings::decode_detailed_timings(base, caps);
    }
}
