#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::capabilities::DisplayCapabilities;
use crate::model::capabilities::{ModeSink, StaticContext, StereoMode, SyncDefinition, VideoMode};
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::ParseWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionHandler;
use crate::model::extension::StaticExtensionHandler;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::{Arc, Vec};

/// Rich capabilities extracted from a DisplayID extension section.
///
/// Stored in [`DisplayCapabilities`] via `set_extension_data(0x70, ...)` by the dynamic
/// pipeline; retrieve with `caps.get_extension_data::<DisplayIdCapabilities>(0x70)`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayIdCapabilities {
    /// DisplayID version byte (0x10–0x1F for v1.x, 0x20 for v2.x).
    pub version: u8,
    /// Display product primary use case (bits 2:0 of header byte 3).
    pub product_type: u8,
}

/// Processes DisplayID extension blocks (tag `0x70`).
///
/// A single handler handles both DisplayID 1.x and 2.x — both versions use tag `0x70`
/// and the dispatch layer cannot distinguish them before the handler receives the payload.
/// Version-specific logic is selected internally after inspecting the version byte.
#[derive(Debug)]
pub struct DisplayIdHandler;

/// Minimum version byte for DisplayID 1.x (0x10 = version 1, revision 0).
const DISPLAYID_V1_MIN: u8 = 0x10;
/// Maximum version byte for DisplayID 1.x (0x1F = version 1, revision 15).
const DISPLAYID_V1_MAX: u8 = 0x1F;
/// Version byte for DisplayID 2.x.
const DISPLAYID_V2: u8 = 0x20;

/// Data block tag for the Detailed Timings Block (Type I descriptors, DisplayID 1.x §4.4.2).
const TAG_TYPE_I_TIMING: u8 = 0x03;

/// Parses the 4-byte section header common to all DisplayID fragments.
///
/// Returns `(version, section_byte_count, product_type, extension_count)`.
/// - `version`: byte 1 of the block (DisplayID version/revision)
/// - `section_byte_count`: byte 2, count of data block bytes in this fragment
/// - `product_type`: bits 2:0 of byte 3 (display product primary use case)
/// - `extension_count`: bits 7:3 of byte 3 (number of continuation blocks after the first)
fn parse_section_header(block: &[u8; 128]) -> (u8, u8, u8, u8) {
    let version = block[1];
    let section_byte_count = block[2];
    let packed = block[3];
    let product_type = packed & 0x07;
    let extension_count = (packed >> 3) & 0x1F;
    (version, section_byte_count, product_type, extension_count)
}

/// Decodes one 20-byte Type I Video Timing descriptor and pushes a mode to `sink`.
///
/// Descriptor layout (DisplayID 1.x §4.4.2):
/// - Byte 0: Options (reserved; bit 7 = preferred timing)
/// - Bytes 1–2: Pixel clock in 10 kHz units (little-endian uint16)
/// - Bytes 3–4: Horizontal Active in pixels (exact, little-endian uint16)
/// - Bytes 5–6: Horizontal Blank in pixels (exact, little-endian uint16)
/// - Bytes 7–8: Horizontal Front Porch in pixels (exact, little-endian uint16)
/// - Bytes 9–10: Horizontal Sync Width in pixels (exact, little-endian uint16)
/// - Bytes 11–12: Vertical Active in lines (exact, little-endian uint16)
/// - Bytes 13–14: Vertical Blank in lines (exact, little-endian uint16)
/// - Bytes 15–16: Vertical Front Porch in lines (exact, little-endian uint16)
/// - Bytes 17–18: Vertical Sync Width in lines (exact, little-endian uint16)
/// - Byte 19: Flags: [0]=interlaced, [2:1]=sync type, [3]=HS polarity, [4]=VS polarity
///
/// Null descriptors (pixel clock = 0) are silently skipped; degenerate total sizes are skipped.
fn decode_type_i_descriptor(d: &[u8; 20], sink: &mut dyn ModeSink) {
    let pixel_clock_10khz = u16::from_le_bytes([d[1], d[2]]);
    if pixel_clock_10khz == 0 {
        return; // null descriptor
    }

    let h_active = u16::from_le_bytes([d[3], d[4]]);
    let h_blank = u16::from_le_bytes([d[5], d[6]]);
    let h_front_porch = u16::from_le_bytes([d[7], d[8]]);
    let h_sync_width = u16::from_le_bytes([d[9], d[10]]);
    let v_active = u16::from_le_bytes([d[11], d[12]]);
    let v_blank = u16::from_le_bytes([d[13], d[14]]);
    let v_front_porch = u16::from_le_bytes([d[15], d[16]]);
    let v_sync_width = u16::from_le_bytes([d[17], d[18]]);
    let flags = d[19];

    let h_total = h_active as u32 + h_blank as u32;
    let v_total = v_active as u32 + v_blank as u32;
    if h_total == 0 || v_total == 0 {
        return; // degenerate descriptor
    }

    let pixel_clock_hz = pixel_clock_10khz as u32 * 10_000;
    let refresh_rate = (pixel_clock_hz / (h_total * v_total)).min(255) as u8;

    let interlaced = (flags & 0x01) != 0;
    // Bytes 19 bits [3:2]: HS positive (bit 3), VS positive (bit 2)... or [4:3] in some docs.
    // DisplayID 1.3 §4.4.2 table: bit 3 = H-sync polarity (1=positive), bit 4 = V-sync polarity.
    let h_sync_positive = (flags & 0x08) != 0;
    let v_sync_positive = (flags & 0x10) != 0;

    sink.push_mode(VideoMode {
        width: h_active,
        height: v_active,
        refresh_rate,
        interlaced,
        h_front_porch,
        h_sync_width,
        v_front_porch,
        v_sync_width,
        h_border: 0,
        v_border: 0,
        stereo: StereoMode::None,
        sync: Some(SyncDefinition::DigitalSeparate {
            v_sync_positive,
            h_sync_positive,
        }),
    });
}

/// Iterates DisplayID 1.x data blocks within a fragment's payload region and pushes
/// decoded modes to `sink`.
///
/// `payload` must be the data-block region: bytes `block[4..4+section_byte_count]`,
/// clamped to `block[4..127]` to exclude the checksum byte.
fn process_data_blocks(payload: &[u8], sink: &mut dyn ModeSink) {
    let mut offset = 0;
    while offset + 3 <= payload.len() {
        let tag = payload[offset];
        let length = payload[offset + 2] as usize;

        // End-of-section sentinel: tag 0x00 with length 0.
        if tag == 0x00 && length == 0 {
            break;
        }

        let block_end = offset + 3 + length;
        if block_end > payload.len() {
            // Malformed block — extends past payload; stop iterating.
            break;
        }

        let block_payload = &payload[offset + 3..block_end];
        if tag == TAG_TYPE_I_TIMING {
            let mut i = 0;
            while i + 20 <= block_payload.len() {
                let descriptor: &[u8; 20] = block_payload[i..i + 20].try_into().unwrap();
                decode_type_i_descriptor(descriptor, sink);
                i += 20;
            }
        }
        // Unknown block tags are silently skipped.

        offset = block_end;
    }
}

/// Returns the data-block payload slice for a single DisplayID fragment.
///
/// Extracts `block[4..end]` where `end = min(4 + section_byte_count, 127)`.
fn fragment_payload(block: &[u8; 128]) -> &[u8] {
    let section_byte_count = block[2] as usize;
    let end = (4 + section_byte_count).min(127);
    if end > 4 { &block[4..end] } else { &[] }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl ExtensionHandler for DisplayIdHandler {
    fn process(
        &self,
        blocks: &[&[u8; 128]],
        caps: &mut DisplayCapabilities,
        warnings: &mut Vec<ParseWarning>,
    ) {
        let Some(first) = blocks.first() else { return };

        let (version, _section_byte_count, product_type, extension_count) =
            parse_section_header(first);

        // Validate version range.
        match version {
            DISPLAYID_V1_MIN..=DISPLAYID_V1_MAX | DISPLAYID_V2 => {}
            v => {
                warnings.push(Arc::new(EdidWarning::DisplayIdVersionUnknown(v)));
                return;
            }
        }

        // Validate extension count vs actual number of continuation blocks.
        let actual_continuation = blocks.len().saturating_sub(1);
        if extension_count as usize != actual_continuation {
            warnings.push(Arc::new(EdidWarning::DisplayIdExtensionCountMismatch {
                declared: extension_count,
                found: actual_continuation as u8,
            }));
        }

        // Store rich capabilities.
        caps.set_extension_data(
            0x70,
            DisplayIdCapabilities {
                version,
                product_type,
            },
        );

        // Process data blocks from all fragments.
        for block in blocks {
            process_data_blocks(fragment_payload(block), caps);
        }
    }
}

impl StaticExtensionHandler for DisplayIdHandler {
    fn tag(&self) -> u8 {
        0x70
    }

    fn process(&self, blocks: &[&[u8; 128]], ctx: &mut StaticContext<'_>) {
        let Some(first) = blocks.first() else { return };

        let (version, _section_byte_count, _product_type, extension_count) =
            parse_section_header(first);

        // Validate version range.
        match version {
            DISPLAYID_V1_MIN..=DISPLAYID_V1_MAX | DISPLAYID_V2 => {}
            v => {
                ctx.push_warning(EdidWarning::DisplayIdVersionUnknown(v));
                return;
            }
        }

        // Validate extension count vs actual number of continuation blocks.
        // In bare no_std the dispatch layer calls once per block, so blocks.len() == 1 always;
        // the mismatch warning may fire once per block in that case. In alloc builds the full
        // slice is provided and the warning fires at most once.
        let actual_continuation = blocks.len().saturating_sub(1);
        if extension_count as usize != actual_continuation {
            ctx.push_warning(EdidWarning::DisplayIdExtensionCountMismatch {
                declared: extension_count,
                found: actual_continuation as u8,
            });
        }

        // Process data blocks from all fragments.
        for block in blocks {
            process_data_blocks(fragment_payload(block), ctx);
        }
    }
}

/// Data block tags decoded by this handler.
///
/// Must be kept in sync with the `if tag ==` dispatch in `process_data_blocks`.
/// `test_all_block_tags_accounted_for` verifies that the union of implemented,
/// deferred, and reserved ranges covers every value 0x00–0xFF.
///
/// Note: tag assignments should be verified against the VESA DisplayID 1.3
/// specification once a real DisplayID fixture is available.
#[cfg(test)]
const IMPLEMENTED_BLOCK_TAGS: &[u8] = &[
    TAG_TYPE_I_TIMING, // 0x03 — Detailed Timings Block (Type I descriptors)
];

/// DisplayID 1.x data block tags that are defined by the specification but not
/// yet decoded, plus tag ranges reserved or unassigned by the specification.
///
/// Each entry is an inclusive `(first, last)` range. When a new block type is
/// implemented, remove its tag from here and add it to `IMPLEMENTED_BLOCK_TAGS`.
#[cfg(test)]
const DEFERRED_OR_RESERVED_TAG_RANGES: &[(u8, u8)] = &[
    (0x00, 0x00), // Product Identification (EOS sentinel when length=0; data block otherwise)
    (0x01, 0x01), // Display Parameters Block
    (0x02, 0x02), // Color Characteristics Block
    (0x04, 0x13), // Type II–VI timings, interface and identity blocks, Tiled Display Topology
    (0x14, 0x7E), // Reserved for future use in DisplayID 1.x
    (0x7F, 0x7F), // Vendor-specific
    (0x80, 0xFF), // Undefined (outside the DisplayID 1.x tag space)
];

/// Pre-built static reference to the built-in DisplayID handler.
///
/// Suitable for inclusion in a `&[&dyn StaticExtensionHandler]` slice alongside
/// [`CEA861_HANDLER`][crate::CEA861_HANDLER].
pub static DISPLAYID_HANDLER: &dyn StaticExtensionHandler = &DisplayIdHandler;

#[cfg(test)]
#[cfg(any(feature = "alloc", feature = "std"))]
mod tests {
    use super::*;
    use crate::model::capabilities::StaticDisplayCapabilities;
    use crate::model::extension::ExtensionHandler;

    fn make_displayid_block(version: u8, data_blocks: &[u8]) -> [u8; 128] {
        let mut block = [0u8; 128];
        block[0] = 0x70; // extension tag
        block[1] = version;
        block[2] = data_blocks.len().min(122) as u8; // section_byte_count
        block[3] = 0x00; // product_type=0, extension_count=0
        let end = (4 + data_blocks.len()).min(127);
        block[4..end].copy_from_slice(&data_blocks[..end - 4]);
        block
    }

    fn make_type_i_descriptor(
        pixel_clock_10khz: u16,
        h_active: u16,
        h_blank: u16,
        h_fp: u16,
        h_sw: u16,
        v_active: u16,
        v_blank: u16,
        v_fp: u16,
        v_sw: u16,
        flags: u8,
    ) -> [u8; 20] {
        let mut d = [0u8; 20];
        d[0] = 0x00; // options
        d[1..3].copy_from_slice(&pixel_clock_10khz.to_le_bytes());
        d[3..5].copy_from_slice(&h_active.to_le_bytes());
        d[5..7].copy_from_slice(&h_blank.to_le_bytes());
        d[7..9].copy_from_slice(&h_fp.to_le_bytes());
        d[9..11].copy_from_slice(&h_sw.to_le_bytes());
        d[11..13].copy_from_slice(&v_active.to_le_bytes());
        d[13..15].copy_from_slice(&v_blank.to_le_bytes());
        d[15..17].copy_from_slice(&v_fp.to_le_bytes());
        d[17..19].copy_from_slice(&v_sw.to_le_bytes());
        d[19] = flags;
        d
    }

    fn make_type_i_data_block(descriptor: &[u8; 20]) -> [u8; 23] {
        let mut db = [0u8; 23];
        db[0] = TAG_TYPE_I_TIMING;
        db[1] = 0x00; // revision
        db[2] = 20; // payload length
        db[3..23].copy_from_slice(descriptor);
        db
    }

    #[test]
    fn test_unknown_version_emits_warning() {
        let block = make_displayid_block(0x05, &[]);
        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);
        assert_eq!(warnings.len(), 1);
        let w = (*warnings[0]).downcast_ref::<EdidWarning>().unwrap();
        assert_eq!(*w, EdidWarning::DisplayIdVersionUnknown(0x05));
    }

    #[test]
    fn test_extension_count_mismatch_warning() {
        // Declare 1 continuation block but provide none.
        let mut block = make_displayid_block(0x10, &[]);
        block[3] = 0x08; // extension_count = 1, product_type = 0
        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);
        assert_eq!(warnings.len(), 1);
        let w = (*warnings[0]).downcast_ref::<EdidWarning>().unwrap();
        assert_eq!(
            *w,
            EdidWarning::DisplayIdExtensionCountMismatch {
                declared: 1,
                found: 0
            }
        );
    }

    #[test]
    fn test_type_i_timing_decoded() {
        // 1920×1080@60 Hz: pixel clock ≈ 148.5 MHz = 14850 × 10 kHz
        // h_total = 2200, v_total = 1125 → 148500000 / (2200 * 1125) ≈ 60 Hz
        let descriptor = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let data_block = make_type_i_data_block(&descriptor);
        let block = make_displayid_block(0x10, &data_block);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);
        assert_eq!(caps.supported_modes.len(), 1);
        let mode = &caps.supported_modes[0];
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
        assert!(!mode.interlaced);
    }

    #[test]
    fn test_type_i_static_pipeline() {
        let descriptor = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let data_block = make_type_i_data_block(&descriptor);
        let block = make_displayid_block(0x10, &data_block);

        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        StaticExtensionHandler::process(&DisplayIdHandler, &[&block], &mut ctx);

        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
    }

    #[test]
    fn test_v2_accepted_without_warning() {
        let block = make_displayid_block(0x20, &[]);
        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_displayid_capabilities_stored() {
        // version 0x13 (DisplayID 1.3), product_type = 2 (packed in byte 3 as 0x02)
        let mut block = make_displayid_block(0x13, &[]);
        block[3] = 0x02; // extension_count=0, product_type=2
        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);
        assert!(warnings.is_empty());
        let did = caps
            .get_extension_data::<DisplayIdCapabilities>(0x70)
            .unwrap();
        assert_eq!(did.version, 0x13);
        assert_eq!(did.product_type, 2);
    }

    #[test]
    fn test_null_descriptor_skipped() {
        // A descriptor with pixel_clock = 0 is a null entry and must not produce a mode.
        let null_descriptor = [0u8; 20];
        let data_block = make_type_i_data_block(&null_descriptor);
        let block = make_displayid_block(0x10, &data_block);
        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);
        assert!(warnings.is_empty());
        assert!(caps.supported_modes.is_empty());
    }

    #[test]
    fn test_interlaced_flag_decoded() {
        // flags byte 19 bit 0 = interlaced
        let descriptor = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x01);
        let data_block = make_type_i_data_block(&descriptor);
        let block = make_displayid_block(0x10, &data_block);
        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);
        assert!(warnings.is_empty());
        assert_eq!(caps.supported_modes.len(), 1);
        assert!(caps.supported_modes[0].interlaced);
    }

    #[test]
    fn test_multiple_descriptors_in_block() {
        // Two 20-byte descriptors packed into a single Type I data block (40-byte payload).
        let desc1 = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        // 2560×1440@60: h_total=3000, v_total=1481 → clock≈2560×1440×60/10000≈22118 × 10 kHz
        let desc2 = make_type_i_descriptor(22118, 2560, 440, 80, 32, 1440, 41, 4, 5, 0x00);
        let mut db = [0u8; 43];
        db[0] = TAG_TYPE_I_TIMING;
        db[1] = 0x00;
        db[2] = 40; // 2 × 20 bytes
        db[3..23].copy_from_slice(&desc1);
        db[23..43].copy_from_slice(&desc2);
        let block = make_displayid_block(0x10, &db);
        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);
        assert!(warnings.is_empty());
        assert_eq!(caps.supported_modes.len(), 2);
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 1920 && m.height == 1080)
        );
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 2560 && m.height == 1440)
        );
    }

    #[test]
    fn test_multi_fragment_reassembly() {
        // First fragment: 1920×1080@60. Declares one continuation block.
        let desc1 = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let db1 = make_type_i_data_block(&desc1);
        let mut block1 = make_displayid_block(0x10, &db1);
        block1[3] = 0x08; // extension_count=1 (bits 7:3), product_type=0

        // Second fragment (continuation): 2560×1440@60.
        let desc2 = make_type_i_descriptor(22118, 2560, 440, 80, 32, 1440, 41, 4, 5, 0x00);
        let db2 = make_type_i_data_block(&desc2);
        let block2 = make_displayid_block(0x10, &db2);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(
            &DisplayIdHandler,
            &[&block1, &block2],
            &mut caps,
            &mut warnings,
        );

        assert!(warnings.is_empty(), "unexpected warnings: {:?}", warnings);
        assert_eq!(caps.supported_modes.len(), 2);
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 1920 && m.height == 1080)
        );
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 2560 && m.height == 1440)
        );
    }

    #[test]
    fn test_malformed_data_block_stops_iteration() {
        // A data block that claims a length extending past the payload boundary.
        // Iteration must stop without producing modes or panicking.
        let mut payload = [0u8; 6];
        payload[0] = TAG_TYPE_I_TIMING;
        payload[1] = 0x00;
        payload[2] = 50; // claims 50 bytes; only 3 remain after the header
        let block = make_displayid_block(0x10, &payload);
        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);
        assert!(warnings.is_empty());
        assert!(caps.supported_modes.is_empty());
    }

    #[test]
    fn test_eos_sentinel_stops_iteration() {
        // A valid Type I descriptor followed by an EOS sentinel (tag=0, length=0).
        // A second descriptor after the sentinel must not be decoded.
        let desc = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let db = make_type_i_data_block(&desc);
        let desc2 = make_type_i_descriptor(22118, 2560, 440, 80, 32, 1440, 41, 4, 5, 0x00);
        let db2 = make_type_i_data_block(&desc2);

        let mut payload = Vec::new();
        payload.extend_from_slice(&db);
        payload.extend_from_slice(&[0x00, 0x00, 0x00]); // EOS sentinel
        payload.extend_from_slice(&db2);

        let block = make_displayid_block(0x10, &payload);
        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 1920);
    }

    #[test]
    fn test_unknown_data_block_tag_skipped() {
        // An unknown data block (tag 0xFF, 10-byte payload) followed by a valid Type I block.
        // The unknown block must be skipped and iteration must continue.
        let desc = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let db = make_type_i_data_block(&desc);

        let mut payload = Vec::new();
        payload.extend_from_slice(&[0xFF, 0x00, 10]); // unknown tag, 10-byte payload
        payload.extend_from_slice(&[0u8; 10]);
        payload.extend_from_slice(&db);

        let block = make_displayid_block(0x10, &payload);
        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 1920);
    }

    // -----------------------------------------------------------------------
    // Block tag coverage
    // -----------------------------------------------------------------------

    #[test]
    fn test_all_block_tags_accounted_for() {
        // Every value 0x00–0xFF must appear in either IMPLEMENTED_BLOCK_TAGS or
        // DEFERRED_OR_RESERVED_TAG_RANGES. If this test fails after a spec update,
        // add the new tag to IMPLEMENTED_BLOCK_TAGS (and implement it) or extend
        // DEFERRED_OR_RESERVED_TAG_RANGES.
        for tag in 0u16..=255 {
            let tag = tag as u8;
            let implemented = IMPLEMENTED_BLOCK_TAGS.contains(&tag);
            let deferred_or_reserved = DEFERRED_OR_RESERVED_TAG_RANGES
                .iter()
                .any(|&(lo, hi)| tag >= lo && tag <= hi);
            assert!(
                implemented || deferred_or_reserved,
                "DisplayID block tag 0x{:02X} is unaccounted for: \
                 add it to IMPLEMENTED_BLOCK_TAGS or DEFERRED_OR_RESERVED_TAG_RANGES",
                tag
            );
        }
    }

    #[test]
    fn test_implemented_and_deferred_are_disjoint() {
        for &tag in IMPLEMENTED_BLOCK_TAGS {
            let in_deferred = DEFERRED_OR_RESERVED_TAG_RANGES
                .iter()
                .any(|&(lo, hi)| tag >= lo && tag <= hi);
            assert!(
                !in_deferred,
                "DisplayID block tag 0x{:02X} appears in both IMPLEMENTED_BLOCK_TAGS \
                 and DEFERRED_OR_RESERVED_TAG_RANGES",
                tag
            );
        }
    }
}
