mod metadata;
mod timing;

#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::capabilities::DisplayCapabilities;
use crate::model::capabilities::StaticContext;
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::ParseWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionHandler;
use crate::model::extension::StaticExtensionHandler;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::{Arc, Vec};

use metadata::{
    scan_ascii_string_blocks, scan_color_characteristics_block, scan_display_device_data_block,
    scan_display_interface_block, scan_display_params_block, scan_power_sequencing_block,
    scan_product_id_block, scan_serial_number_block, scan_stereo_display_interface_block,
    scan_tiled_topology_block, scan_transfer_characteristics_block, scan_video_timing_range_block,
};
use timing::process_data_blocks;

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

/// Data block tag for the Product Identification Block (DisplayID 1.x §4.2).
const TAG_PRODUCT_ID: u8 = 0x00;

/// Data block tag for the Display Parameters Block (DisplayID 1.x §4.3).
const TAG_DISPLAY_PARAMS: u8 = 0x01;

/// Data block tag for the Color Characteristics Block (DisplayID 1.x §4.4).
const TAG_COLOR_CHARACTERISTICS: u8 = 0x02;

/// Data block tag for the Detailed Timings Block (Type I descriptors, DisplayID 1.x §4.4.2).
const TAG_TYPE_I_TIMING: u8 = 0x03;

/// Data block tag for the Video Timing Modes Type II — Detailed Timings Block (DisplayID 1.x §4.4.3).
const TAG_TYPE_II_TIMING: u8 = 0x04;

/// Data block tag for the Video Timing Modes Type III — Short Timings Block (DisplayID 1.x §4.4.4).
const TAG_TYPE_III_TIMING: u8 = 0x05;

/// Data block tag for the Video Timing Modes Type IV — DMT/VIC Code Block (DisplayID 1.x §4.4.5).
const TAG_TYPE_IV_TIMING: u8 = 0x06;

/// Data block tag for the VESA Video Timing Block (DisplayID 1.x §4.4.6).
///
/// Payload is a compact presence bitmap: up to 10 bytes encoding DMT IDs 0x01–0x50.
/// Bit `i` (0-indexed, LSB-first within each byte) is set if DMT ID `i + 1` is supported.
const TAG_VESA_VIDEO_TIMING: u8 = 0x07;

/// Data block tag for the CTA-861 Video Timing Block (DisplayID 1.x §4.4.7).
///
/// Payload is a compact presence bitmap: up to 8 bytes encoding CTA-861 VIC codes 1–64.
/// Bit `i` (0-indexed, LSB-first within each byte) is set if VIC `i + 1` is supported.
const TAG_CTA_VIDEO_TIMING: u8 = 0x08;

/// Data block tag for the Video Timing Range Limits Block (DisplayID 1.x §4.5).
const TAG_VIDEO_TIMING_RANGE: u8 = 0x09;

/// Data block tag for the Product Serial Number Block (DisplayID 1.x §4.8).
const TAG_SERIAL_NUMBER: u8 = 0x0A;

/// Data block tag for the General Purpose ASCII String Block (DisplayID 1.x §4.9).
const TAG_ASCII_STRING: u8 = 0x0B;

/// Data block tag for the Display Device Data Block (DisplayID 1.x §4.10).
const TAG_DISPLAY_DEVICE_DATA: u8 = 0x0C;

/// Data block tag for the Interface Power Sequencing Block (DisplayID 1.x §4.11).
const TAG_POWER_SEQUENCING: u8 = 0x0D;

/// Data block tag for the Transfer Characteristics Block (DisplayID 1.x §4.12).
const TAG_TRANSFER_CHARACTERISTICS: u8 = 0x0E;

/// Data block tag for the Display Interface Data Block (DisplayID 1.x §4.13).
const TAG_DISPLAY_INTERFACE: u8 = 0x0F;

/// Data block tag for the Stereo Display Interface Data Block (DisplayID 1.x §4.14).
const TAG_STEREO_DISPLAY_INTERFACE: u8 = 0x10;

/// Data block tag for the Tiled Display Topology Data Block (DisplayID 1.x §4.15).
const TAG_TILED_TOPOLOGY: u8 = 0x12;

/// Data block tag for the Video Timing Modes Type V — Short Timings Block (DisplayID 1.x §4.6).
const TAG_TYPE_V_TIMING: u8 = 0x11;

/// Data block tag for the Video Timing Modes Type VI — Detailed Timings Block (DisplayID 1.x §4.7).
const TAG_TYPE_VI_TIMING: u8 = 0x13;

/// Calls `f(tag, revision, block_payload)` for each well-formed data block in `payload`.
///
/// `revision` is the second byte of the 3-byte data block header and carries block-specific
/// flags (e.g., the code-space selector for Type IV timing blocks).
///
/// Stops at the end-of-section sentinel (tag `0x00`, length `0`) or when a block's
/// declared length would extend past the available payload.
fn for_each_data_block(payload: &[u8], mut f: impl FnMut(u8, u8, &[u8])) {
    let mut offset = 0;
    while offset + 3 <= payload.len() {
        let tag = payload[offset];
        let revision = payload[offset + 1];
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

        f(tag, revision, &payload[offset + 3..block_end]);
        offset = block_end;
    }
}

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
            let payload = fragment_payload(block);
            process_data_blocks(payload, caps);
            scan_product_id_block(payload, caps);
            scan_display_params_block(payload, caps);
            scan_color_characteristics_block(payload, caps);
            scan_video_timing_range_block(payload, caps);
            scan_serial_number_block(payload, caps);
            scan_ascii_string_blocks(payload, caps);
            scan_display_device_data_block(payload, caps);
            scan_power_sequencing_block(payload, caps);
            scan_transfer_characteristics_block(payload, caps);
            scan_display_interface_block(payload, caps);
            scan_stereo_display_interface_block(payload, caps);
            scan_tiled_topology_block(payload, caps);
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
/// Must be kept in sync with the `if tag ==` dispatch in `timing::process_data_blocks`.
/// `test_all_block_tags_accounted_for` verifies that the union of implemented,
/// deferred, and reserved ranges covers every value 0x00–0xFF.
#[cfg(test)]
const IMPLEMENTED_BLOCK_TAGS: &[u8] = &[
    TAG_PRODUCT_ID,               // 0x00 — Product Identification Block
    TAG_DISPLAY_PARAMS,           // 0x01 — Display Parameters Block
    TAG_COLOR_CHARACTERISTICS,    // 0x02 — Color Characteristics Block
    TAG_TYPE_I_TIMING,            // 0x03 — Detailed Timings Block (Type I descriptors)
    TAG_TYPE_II_TIMING,           // 0x04 — Video Timing Modes Type II — Detailed Timings Block
    TAG_TYPE_III_TIMING,          // 0x05 — Video Timing Modes Type III — Short Timings Block
    TAG_TYPE_IV_TIMING,           // 0x06 — Video Timing Modes Type IV — DMT/VIC Code Block
    TAG_VESA_VIDEO_TIMING,        // 0x07 — VESA Video Timing Block (DMT presence bitmap)
    TAG_CTA_VIDEO_TIMING,         // 0x08 — CTA-861 Video Timing Block (VIC presence bitmap)
    TAG_VIDEO_TIMING_RANGE,       // 0x09 — Video Timing Range Limits Block
    TAG_SERIAL_NUMBER,            // 0x0A — Product Serial Number Block
    TAG_ASCII_STRING,             // 0x0B — General Purpose ASCII String Block
    TAG_DISPLAY_DEVICE_DATA,      // 0x0C — Display Device Data Block
    TAG_POWER_SEQUENCING,         // 0x0D — Interface Power Sequencing Block
    TAG_TRANSFER_CHARACTERISTICS, // 0x0E — Transfer Characteristics Block
    TAG_DISPLAY_INTERFACE,        // 0x0F — Display Interface Data Block
    TAG_STEREO_DISPLAY_INTERFACE, // 0x10 — Stereo Display Interface Data Block
    TAG_TYPE_V_TIMING,            // 0x11 — Video Timing Modes Type V — Short Timings Block
    TAG_TILED_TOPOLOGY,           // 0x12 — Tiled Display Topology Data Block
    TAG_TYPE_VI_TIMING,           // 0x13 — Video Timing Modes Type VI — Detailed Timings Block
];

/// DisplayID 1.x data block tags that are defined by the specification but not
/// yet decoded, plus tag ranges reserved or unassigned by the specification.
///
/// Each entry is an inclusive `(first, last)` range. When a new block type is
/// implemented, remove its tag from here and add it to `IMPLEMENTED_BLOCK_TAGS`.
#[cfg(test)]
const DEFERRED_OR_RESERVED_TAG_RANGES: &[(u8, u8)] = &[
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
    use crate::model::extension::ExtensionHandler;
    use crate::model::manufacture::ManufacturerId;

    // -----------------------------------------------------------------------
    // Shared test helpers
    // -----------------------------------------------------------------------

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
        d[0] = 0x00;
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
        db[1] = 0x00;
        db[2] = 20;
        db[3..23].copy_from_slice(descriptor);
        db
    }

    // -----------------------------------------------------------------------
    // Handler-level tests (warnings, capabilities, multi-fragment)
    // -----------------------------------------------------------------------

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
    fn test_product_id_and_timing_in_same_block() {
        // Product ID block followed by a Type I timing block — both must be decoded.
        let ca = (b'S' - b'A' + 1) as u16;
        let cb = (b'A' - b'A' + 1) as u16;
        let cc = (b'M' - b'A' + 1) as u16;
        let packed: u16 = (ca << 10) | (cb << 5) | cc;
        let mut pid_payload = Vec::new();
        pid_payload.extend_from_slice(&packed.to_be_bytes());
        pid_payload.extend_from_slice(&0xABCDu16.to_le_bytes());
        pid_payload.extend_from_slice(&0u32.to_le_bytes()); // serial = 0
        pid_payload.push(0); // week
        pid_payload.push(0); // year
        let mut pid_db = vec![TAG_PRODUCT_ID, 0x00, pid_payload.len() as u8];
        pid_db.extend_from_slice(&pid_payload);

        let desc = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let timing_db = make_type_i_data_block(&desc);

        let mut payload = Vec::new();
        payload.extend_from_slice(&pid_db);
        payload.extend_from_slice(&timing_db);

        let block = make_displayid_block(0x10, &payload);
        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.manufacturer, Some(ManufacturerId(*b"SAM")));
        assert_eq!(caps.product_code, Some(0xABCD));
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 1920);
    }

    // -----------------------------------------------------------------------
    // Block tag coverage
    // -----------------------------------------------------------------------

    #[test]
    fn test_all_block_tags_accounted_for() {
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
