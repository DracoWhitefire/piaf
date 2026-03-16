#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::capabilities::DisplayCapabilities;
use crate::model::capabilities::{ModeSink, StaticContext, StereoMode, SyncDefinition, VideoMode};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::color::Chromaticity;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::color::{ChromaticityPoint, ColorBitDepth};
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::ParseWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionHandler;
use crate::model::extension::StaticExtensionHandler;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::manufacture::{ManufactureDate, ManufacturerId, MonitorString};
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

/// Decodes one 11-byte Type II Video Timing descriptor and pushes a mode to `sink`.
///
/// Descriptor layout (DisplayID 1.x §4.4.3):
/// - Bytes 0–2: Pixel clock in 10 kHz units (little-endian 24-bit; `actual = (raw + 1) × 10 000 Hz`)
/// - Byte 3:    Flags: [7]=preferred, [6:5]=stereo, [4]=interlaced, [3]=HS polarity (+), [2]=VS polarity (+)
/// - Byte 4:    H-active bits 7:0  (9-bit mantissa, 8-pixel granule; `h = 8 + 8 × mantissa`)
/// - Byte 5:    Bit 0 = H-active bit 8; bits 7:1 = H-blank mantissa (7-bit, same granule)
/// - Byte 6:    Bits 7:4 = H-offset mantissa (4-bit); bits 3:0 = H-sync-width mantissa (4-bit)
/// - Byte 7:    V-active bits 7:0  (12-bit mantissa; `v = 1 + mantissa`)
/// - Byte 8:    Bits 3:0 = V-active bits 11:8; bits 7:4 = reserved
/// - Byte 9:    Full byte = V-blank mantissa (`v_blank = 1 + byte`);
///   bits 7:4 = V-offset mantissa (`v_fp = 1 + nibble`);
///   bits 3:0 = V-sync-width mantissa (`v_sw = 1 + nibble`)
/// - Byte 10:   Reserved
fn decode_type_ii_descriptor(d: &[u8; 11], sink: &mut dyn ModeSink) {
    let raw_pixel_clock = (d[0] as u32) | ((d[1] as u32) << 8) | ((d[2] as u32) << 16);
    let pixel_clock_10khz = 1u64 + raw_pixel_clock as u64;

    let flags = d[3];
    let interlaced = (flags & 0x10) != 0;
    let h_sync_positive = (flags & 0x08) != 0;
    let v_sync_positive = (flags & 0x04) != 0;

    // Horizontal: 8-pixel granule, each value = 8 + 8 × mantissa.
    let ha_raw = (d[4] as u16) | (((d[5] & 0x01) as u16) << 8);
    let h_active = 8u16 + 8 * ha_raw;

    let hb_raw = ((d[5] >> 1) & 0x7F) as u16;
    let h_blank = 8u16 + 8 * hb_raw;

    let h_front_porch = 8u16 + 8 * ((d[6] >> 4) as u16);
    let h_sync_width = 8u16 + 8 * ((d[6] & 0x0F) as u16);

    // Vertical: 1-line granule, each value = 1 + mantissa.
    let va_raw = (d[7] as u16) | (((d[8] & 0x0F) as u16) << 8);
    let v_active = 1u16 + va_raw;

    // Byte 9 dual-role: full byte encodes v_blank; nibbles encode v_front_porch and v_sync_width.
    let v_blank = 1u16 + d[9] as u16;
    let v_front_porch = 1u16 + ((d[9] >> 4) as u16);
    let v_sync_width = 1u16 + ((d[9] & 0x0F) as u16);

    let h_total = h_active as u64 + h_blank as u64;
    let v_total = v_active as u64 + v_blank as u64;
    if h_total == 0 || v_total == 0 {
        return;
    }

    let pixel_clock_hz = pixel_clock_10khz * 10_000;
    let refresh_rate = (pixel_clock_hz / (h_total * v_total)).min(255) as u8;

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

/// Calls `f(tag, block_payload)` for each well-formed data block in `payload`.
///
/// Stops at the end-of-section sentinel (tag `0x00`, length `0`) or when a block's
/// declared length would extend past the available payload.
fn for_each_data_block(payload: &[u8], mut f: impl FnMut(u8, &[u8])) {
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

        f(tag, &payload[offset + 3..block_end]);
        offset = block_end;
    }
}

/// Iterates DisplayID 1.x data blocks within a fragment's payload region and pushes
/// decoded modes to `sink`.
///
/// `payload` must be the data-block region: bytes `block[4..4+section_byte_count]`,
/// clamped to `block[4..127]` to exclude the checksum byte.
fn process_data_blocks(payload: &[u8], sink: &mut dyn ModeSink) {
    for_each_data_block(payload, |tag, block_payload| {
        if tag == TAG_TYPE_I_TIMING {
            let mut i = 0;
            while i + 20 <= block_payload.len() {
                let descriptor: &[u8; 20] = block_payload[i..i + 20].try_into().unwrap();
                decode_type_i_descriptor(descriptor, sink);
                i += 20;
            }
        } else if tag == TAG_TYPE_II_TIMING {
            let mut i = 0;
            while i + 11 <= block_payload.len() {
                let descriptor: &[u8; 11] = block_payload[i..i + 11].try_into().unwrap();
                decode_type_ii_descriptor(descriptor, sink);
                i += 11;
            }
        }
        // Unknown block tags are silently skipped.
    });
}

/// Decodes a Product Identification Block payload into `caps`.
///
/// Payload layout (DisplayID 1.x §4.2):
/// - Bytes 0–1: Manufacturer ID (2-byte PNP-encoded, same as EDID base block)
/// - Bytes 2–3: Product code (little-endian uint16)
/// - Bytes 4–7: Serial number (little-endian uint32; `0` = not specified)
/// - Byte  8:   Week of manufacture (`0` = unspecified, `0xFF` = model year)
/// - Byte  9:   Year (`byte + 1990`; when week = `0xFF`, this is the model year)
/// - Bytes 10+: ASCII product name (space-padded, `0x0A`-terminated; may be absent)
///
/// Fields are written only when the payload is long enough to contain them.
/// If the block is already populated (e.g. by the EDID base block), the values
/// are overwritten by the DisplayID data.
#[cfg(any(feature = "alloc", feature = "std"))]
fn decode_product_id_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    // Manufacturer ID — 2-byte packed PNP encoding (same as EDID base block bytes 0x08–0x09).
    if payload.len() >= 2 {
        let id_raw = ((payload[0] as u16) << 8) | (payload[1] as u16);
        let char1 = ((id_raw >> 10) & 0x1F) as u8;
        let char2 = ((id_raw >> 5) & 0x1F) as u8;
        let char3 = (id_raw & 0x1F) as u8;
        if (1..=26).contains(&char1) && (1..=26).contains(&char2) && (1..=26).contains(&char3) {
            caps.manufacturer = Some(ManufacturerId([
                char1 + b'A' - 1,
                char2 + b'A' - 1,
                char3 + b'A' - 1,
            ]));
        }
    }

    // Product code (LE uint16).
    if payload.len() >= 4 {
        caps.product_code = Some(u16::from_le_bytes([payload[2], payload[3]]));
    }

    // Serial number (LE uint32; 0 = not specified).
    if payload.len() >= 8 {
        let sn = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
        if sn != 0 {
            caps.serial_number = Some(sn);
        }
    }

    // Manufacture / model year.
    if payload.len() >= 10 {
        caps.manufacture_date = Some(ManufactureDate::from_edid_bytes(payload[8], payload[9]));
    }

    // Product name: bytes 10+ (ASCII, 0x0A-terminated, space-padded; max 13 bytes stored).
    if payload.len() >= 11 {
        let name_bytes = &payload[10..];
        let mut buf = [b' '; 13];
        let copy_len = name_bytes.len().min(13);
        buf[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        // Ensure there is a 0x0A terminator within the buffer.
        if !buf.contains(&0x0A) {
            let term_pos = copy_len.min(12);
            buf[term_pos] = 0x0A;
        }
        caps.display_name = Some(MonitorString(buf));
    }
}

/// Scans all data blocks in `payload` for a Product Identification Block (tag `0x00`)
/// and decodes the first one found into `caps`.
#[cfg(any(feature = "alloc", feature = "std"))]
fn scan_product_id_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    for_each_data_block(payload, |tag, block_payload| {
        if tag == TAG_PRODUCT_ID {
            decode_product_id_block(block_payload, caps);
        }
    });
}

/// Decodes a Display Parameters Block payload into `caps`.
///
/// Payload layout (DisplayID 1.x §4.3):
/// - Bytes 0–1: Horizontal image size in mm (little-endian uint16; `0` = not defined)
/// - Bytes 2–3: Vertical image size in mm (little-endian uint16; `0` = not defined)
/// - Byte  4:   Display technology (bits 7:4) and feature support flags (bits 3:0)
/// - Byte  5:   Color bit depth — bits 4:0 use the same `001=6bpc … 110=16bpc` encoding
///   as EDID base block byte `0x14` bits 6:4
///
/// When both image size fields are non-zero they are written to `preferred_image_size_mm`.
/// Color bit depth is written to `color_bit_depth` when the field decodes to a known value.
#[cfg(any(feature = "alloc", feature = "std"))]
fn decode_display_params_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    // Physical image size (bytes 0–3).
    if payload.len() >= 4 {
        let h_mm = u16::from_le_bytes([payload[0], payload[1]]);
        let v_mm = u16::from_le_bytes([payload[2], payload[3]]);
        if h_mm != 0 && v_mm != 0 {
            caps.preferred_image_size_mm = Some((h_mm, v_mm));
        }
    }

    // Color bit depth (byte 5, bits 4:0).
    if payload.len() >= 6 {
        caps.color_bit_depth = ColorBitDepth::from_edid_bits(payload[5] & 0x1F);
    }
}

/// Scans all data blocks in `payload` for a Display Parameters Block (tag `0x01`)
/// and decodes the first one found into `caps`.
#[cfg(any(feature = "alloc", feature = "std"))]
fn scan_display_params_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    for_each_data_block(payload, |tag, block_payload| {
        if tag == TAG_DISPLAY_PARAMS {
            decode_display_params_block(block_payload, caps);
        }
    });
}

/// Decodes a Color Characteristics Block payload into `caps.chromaticity`.
///
/// Payload layout (DisplayID 1.x §4.4):
/// - Bytes  0–1:  Red primary x   (little-endian uint16; value × 1/1024 = CIE x)
/// - Bytes  2–3:  Red primary y
/// - Bytes  4–5:  Green primary x
/// - Bytes  6–7:  Green primary y
/// - Bytes  8–9:  Blue primary x
/// - Bytes 10–11: Blue primary y
/// - Bytes 12–13: White point x
/// - Bytes 14–15: White point y
///
/// The 16-bit values use the same 1/1024 scale as the 10-bit EDID base block encoding.
/// Only the lower 10 bits are significant; upper bits are reserved and masked out.
/// The full 16 bytes must be present; short payloads are ignored.
#[cfg(any(feature = "alloc", feature = "std"))]
fn decode_color_characteristics_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    if payload.len() < 16 {
        return;
    }

    let read_point = |i: usize| ChromaticityPoint {
        x_raw: u16::from_le_bytes([payload[i], payload[i + 1]]) & 0x03FF,
        y_raw: u16::from_le_bytes([payload[i + 2], payload[i + 3]]) & 0x03FF,
    };

    caps.chromaticity = Chromaticity {
        red: read_point(0),
        green: read_point(4),
        blue: read_point(8),
        white: read_point(12),
    };
}

/// Scans all data blocks in `payload` for a Color Characteristics Block (tag `0x02`)
/// and decodes the first one found into `caps`.
#[cfg(any(feature = "alloc", feature = "std"))]
fn scan_color_characteristics_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    for_each_data_block(payload, |tag, block_payload| {
        if tag == TAG_COLOR_CHARACTERISTICS {
            decode_color_characteristics_block(block_payload, caps);
        }
    });
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
    TAG_PRODUCT_ID,            // 0x00 — Product Identification Block
    TAG_DISPLAY_PARAMS,        // 0x01 — Display Parameters Block
    TAG_COLOR_CHARACTERISTICS, // 0x02 — Color Characteristics Block
    TAG_TYPE_I_TIMING,         // 0x03 — Detailed Timings Block (Type I descriptors)
    TAG_TYPE_II_TIMING,        // 0x04 — Video Timing Modes Type II — Detailed Timings Block
];

/// DisplayID 1.x data block tags that are defined by the specification but not
/// yet decoded, plus tag ranges reserved or unassigned by the specification.
///
/// Each entry is an inclusive `(first, last)` range. When a new block type is
/// implemented, remove its tag from here and add it to `IMPLEMENTED_BLOCK_TAGS`.
#[cfg(test)]
const DEFERRED_OR_RESERVED_TAG_RANGES: &[(u8, u8)] = &[
    (0x05, 0x13), // Type III–VI timings, interface and identity blocks, Tiled Display Topology
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
    use crate::model::color::{Chromaticity, ChromaticityPoint, ColorBitDepth};
    use crate::model::extension::ExtensionHandler;
    use crate::model::manufacture::{ManufactureDate, ManufacturerId};

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
    // Product Identification Block (tag 0x00)
    // -----------------------------------------------------------------------

    fn make_product_id_payload(
        manufacturer_raw: u16, // packed PNP encoding
        product_code: u16,
        serial: u32,
        week: u8,
        year_offset: u8, // actual year = offset + 1990
        name: Option<&[u8]>,
    ) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&manufacturer_raw.to_be_bytes());
        v.extend_from_slice(&product_code.to_le_bytes());
        v.extend_from_slice(&serial.to_le_bytes());
        v.push(week);
        v.push(year_offset);
        if let Some(n) = name {
            v.extend_from_slice(n);
        }
        v
    }

    fn make_product_id_data_block(payload: &[u8]) -> Vec<u8> {
        let mut db = Vec::new();
        db.push(TAG_PRODUCT_ID); // tag 0x00
        db.push(0x00); // revision
        db.push(payload.len() as u8);
        db.extend_from_slice(payload);
        db
    }

    /// Pack three ASCII uppercase letters into the 2-byte PNP manufacturer ID encoding.
    fn pack_manufacturer_id(a: u8, b: u8, c: u8) -> u16 {
        let ca = (a - b'A' + 1) as u16;
        let cb = (b - b'A' + 1) as u16;
        let cc = (c - b'A' + 1) as u16;
        (ca << 10) | (cb << 5) | cc
    }

    #[test]
    fn test_product_id_manufacturer_and_product_code() {
        // Encode "SAM" and product code 0x1234.
        let packed = pack_manufacturer_id(b'S', b'A', b'M');
        let payload = make_product_id_payload(packed, 0x1234, 0, 0, 0, None);
        let db = make_product_id_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.manufacturer, Some(ManufacturerId(*b"SAM")));
        assert_eq!(caps.product_code, Some(0x1234));
    }

    #[test]
    fn test_product_id_serial_number() {
        let packed = pack_manufacturer_id(b'D', b'E', b'L');
        let payload = make_product_id_payload(packed, 0x0001, 0xDEADBEEF, 0, 0, None);
        let db = make_product_id_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.serial_number, Some(0xDEAD_BEEF));
    }

    #[test]
    fn test_product_id_zero_serial_not_stored() {
        let packed = pack_manufacturer_id(b'G', b'S', b'M');
        let payload = make_product_id_payload(packed, 0x0001, 0, 0, 0, None);
        let db = make_product_id_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.serial_number, None);
    }

    #[test]
    fn test_product_id_manufacture_date() {
        let packed = pack_manufacturer_id(b'A', b'P', b'L');
        // Week 10, year 2020 → year_byte = 2020 - 1990 = 30
        let payload = make_product_id_payload(packed, 0x0001, 0, 10, 30, None);
        let db = make_product_id_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(
            caps.manufacture_date,
            Some(ManufactureDate::Manufactured {
                week: Some(10),
                year: 2020
            })
        );
    }

    #[test]
    fn test_product_id_display_name() {
        let packed = pack_manufacturer_id(b'H', b'W', b'P');
        // Name "Z27k G2" as ASCII with 0x0A terminator.
        let name: &[u8] = b"Z27k G2\x0a     ";
        let payload = make_product_id_payload(packed, 0x0042, 0, 0, 34, Some(name));
        let db = make_product_id_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.display_name.as_deref(), Some("Z27k G2"));
    }

    #[test]
    fn test_product_id_too_short_does_not_panic() {
        // A product ID block that is only 1 byte long — too short for any field.
        let mut db = [0u8; 4]; // tag, revision, length=1, one byte
        db[0] = TAG_PRODUCT_ID;
        db[1] = 0x00;
        db[2] = 1;
        db[3] = 0xFF;
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.manufacturer, None);
        assert_eq!(caps.product_code, None);
    }

    #[test]
    fn test_product_id_and_timing_in_same_block() {
        // Product ID block followed by a Type I timing block — both must be decoded.
        let packed = pack_manufacturer_id(b'S', b'A', b'M');
        let pid_payload = make_product_id_payload(packed, 0xABCD, 0, 0, 0, None);
        let pid_db = make_product_id_data_block(&pid_payload);

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
    // Display Parameters Block (tag 0x01)
    // -----------------------------------------------------------------------

    fn make_display_params_payload(
        h_mm: u16,
        v_mm: u16,
        tech_flags: u8,
        bit_depth_byte: u8,
    ) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&h_mm.to_le_bytes());
        v.extend_from_slice(&v_mm.to_le_bytes());
        v.push(tech_flags);
        v.push(bit_depth_byte);
        v
    }

    fn make_display_params_data_block(payload: &[u8]) -> Vec<u8> {
        let mut db = Vec::new();
        db.push(TAG_DISPLAY_PARAMS);
        db.push(0x00); // revision
        db.push(payload.len() as u8);
        db.extend_from_slice(payload);
        db
    }

    #[test]
    fn test_display_params_image_size_mm() {
        let payload = make_display_params_payload(597, 336, 0x10, 0x00);
        let db = make_display_params_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.preferred_image_size_mm, Some((597, 336)));
    }

    #[test]
    fn test_display_params_zero_size_not_stored() {
        // Both axes zero — undefined; preferred_image_size_mm must remain None.
        let payload = make_display_params_payload(0, 0, 0x10, 0x00);
        let db = make_display_params_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.preferred_image_size_mm, None);
    }

    #[test]
    fn test_display_params_partial_zero_size_not_stored() {
        // One axis zero — must not store a partial size.
        let payload = make_display_params_payload(597, 0, 0x10, 0x00);
        let db = make_display_params_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.preferred_image_size_mm, None);
    }

    #[test]
    fn test_display_params_color_bit_depth_8bpc() {
        // Bits 4:0 = 0b00010 = 8 bpc (same encoding as EDID base block).
        let payload = make_display_params_payload(597, 336, 0x10, 0b0000_0010);
        let db = make_display_params_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.color_bit_depth, Some(ColorBitDepth::Depth8));
    }

    #[test]
    fn test_display_params_color_bit_depth_10bpc() {
        // Bits 4:0 = 0b00011 = 10 bpc.
        let payload = make_display_params_payload(597, 336, 0x10, 0b0000_0011);
        let db = make_display_params_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.color_bit_depth, Some(ColorBitDepth::Depth10));
    }

    #[test]
    fn test_display_params_undefined_bit_depth_not_stored() {
        // Bits 4:0 = 0b00000 = undefined → color_bit_depth must remain None.
        let payload = make_display_params_payload(597, 336, 0x10, 0b0000_0000);
        let db = make_display_params_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.color_bit_depth, None);
    }

    #[test]
    fn test_display_params_too_short_does_not_panic() {
        // Only 3 bytes — too short for image size; must not panic.
        let mut db = [0u8; 6];
        db[0] = TAG_DISPLAY_PARAMS;
        db[1] = 0x00;
        db[2] = 3;
        db[3] = 0x55;
        db[4] = 0x01;
        db[5] = 0x00;
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.preferred_image_size_mm, None);
    }

    // -----------------------------------------------------------------------
    // Color Characteristics Block (tag 0x02)
    // -----------------------------------------------------------------------

    /// Build a minimal 16-byte Color Characteristics payload with the given primaries.
    /// Each argument is a (x_raw, y_raw) pair in 1/1024 CIE units (10-bit values).
    fn make_color_characteristics_payload(
        red: (u16, u16),
        green: (u16, u16),
        blue: (u16, u16),
        white: (u16, u16),
    ) -> [u8; 16] {
        let mut p = [0u8; 16];
        let write = |buf: &mut [u8], offset: usize, val: (u16, u16)| {
            buf[offset..offset + 2].copy_from_slice(&val.0.to_le_bytes());
            buf[offset + 2..offset + 4].copy_from_slice(&val.1.to_le_bytes());
        };
        write(&mut p, 0, red);
        write(&mut p, 4, green);
        write(&mut p, 8, blue);
        write(&mut p, 12, white);
        p
    }

    fn make_color_char_data_block(payload: &[u8]) -> Vec<u8> {
        let mut db = Vec::new();
        db.push(TAG_COLOR_CHARACTERISTICS);
        db.push(0x00); // revision
        db.push(payload.len() as u8);
        db.extend_from_slice(payload);
        db
    }

    #[test]
    fn test_color_characteristics_primaries_decoded() {
        // sRGB-like primaries: R(0.64, 0.33), G(0.30, 0.60), B(0.15, 0.06), D65(0.3127, 0.3290)
        // Scaled × 1024: R(655, 338), G(307, 614), B(154, 61), W(320, 337)
        let payload =
            make_color_characteristics_payload((655, 338), (307, 614), (154, 61), (320, 337));
        let db = make_color_char_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.chromaticity.red.x_raw, 655);
        assert_eq!(caps.chromaticity.red.y_raw, 338);
        assert_eq!(caps.chromaticity.green.x_raw, 307);
        assert_eq!(caps.chromaticity.green.y_raw, 614);
        assert_eq!(caps.chromaticity.blue.x_raw, 154);
        assert_eq!(caps.chromaticity.blue.y_raw, 61);
        assert_eq!(caps.chromaticity.white.x_raw, 320);
        assert_eq!(caps.chromaticity.white.y_raw, 337);
    }

    #[test]
    fn test_color_characteristics_upper_bits_masked() {
        // Bits above 10 (mask 0x03FF) should be stripped — store 0x04FF, expect 0x00FF.
        let payload = make_color_characteristics_payload(
            (0x04FF, 0x04FF),
            (0x04FF, 0x04FF),
            (0x04FF, 0x04FF),
            (0x04FF, 0x04FF),
        );
        let db = make_color_char_data_block(&payload);
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.chromaticity.red.x_raw, 0x00FF);
    }

    #[test]
    fn test_color_characteristics_short_payload_ignored() {
        // A 15-byte payload — one byte short of the minimum 16. Must not modify chromaticity.
        let mut db = [0u8; 18];
        db[0] = TAG_COLOR_CHARACTERISTICS;
        db[1] = 0x00;
        db[2] = 15; // one byte short
        let block = make_displayid_block(0x10, &db);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(
            caps.chromaticity,
            crate::model::color::Chromaticity::default()
        );
    }

    // -----------------------------------------------------------------------
    // Type II Video Timing Block (tag 0x04)
    // -----------------------------------------------------------------------

    /// Builds an 11-byte Type II timing descriptor.
    ///
    /// `pixel_clock_10khz`: raw 24-bit value (actual = `(raw + 1) × 10 kHz`).
    /// Horizontal values are in 8-pixel granules; vertical active in 1-line granule.
    /// `v_blank_byte`: raw byte 9 (v_blank = 1 + byte; upper nibble = v_fp - 1; lower = v_sw - 1).
    #[allow(clippy::too_many_arguments)]
    fn make_type_ii_descriptor(
        pixel_clock_10khz: u32, // raw 24-bit value
        ha_raw: u16,            // 9-bit h_active mantissa (h_active = 8 + 8 × ha_raw)
        hb_raw: u8,             // 7-bit h_blank mantissa (h_blank = 8 + 8 × hb_raw)
        hfp_raw: u8,            // 4-bit h_front_porch nibble
        hsw_raw: u8,            // 4-bit h_sync_width nibble
        va_raw: u16,            // 12-bit v_active mantissa (v_active = 1 + va_raw)
        v_blank_byte: u8,       // raw byte 9 (upper nibble = vfp-1, lower = vsw-1)
        flags: u8,
    ) -> [u8; 11] {
        let mut d = [0u8; 11];
        d[0] = (pixel_clock_10khz & 0xFF) as u8;
        d[1] = ((pixel_clock_10khz >> 8) & 0xFF) as u8;
        d[2] = ((pixel_clock_10khz >> 16) & 0xFF) as u8;
        d[3] = flags;
        d[4] = (ha_raw & 0xFF) as u8;
        d[5] = (((ha_raw >> 8) & 0x01) as u8) | ((hb_raw & 0x7F) << 1);
        d[6] = ((hfp_raw & 0x0F) << 4) | (hsw_raw & 0x0F);
        d[7] = (va_raw & 0xFF) as u8;
        d[8] = ((va_raw >> 8) & 0x0F) as u8;
        d[9] = v_blank_byte;
        d[10] = 0x00; // reserved
        d
    }

    fn make_type_ii_data_block(descriptor: &[u8; 11]) -> [u8; 14] {
        let mut db = [0u8; 14];
        db[0] = TAG_TYPE_II_TIMING;
        db[1] = 0x00; // revision
        db[2] = 11; // payload length
        db[3..14].copy_from_slice(descriptor);
        db
    }

    #[test]
    fn test_type_ii_timing_decoded() {
        // 1920×1080@60 Hz via Type II encoding.
        //
        // ha_raw = (1920 - 8) / 8 = 239
        // hb_raw = (280 - 8) / 8 = 34  → h_total = 1920 + 280 = 2200
        // hfp_raw = (88 - 8) / 8 = 10, hsw_raw = (48 - 8) / 8 = 5
        // va_raw = 1080 - 1 = 1079 = 0x437
        // v_blank_byte = 0x43 → v_blank = 68, v_fp = 5, v_sw = 4
        //   → v_total = 1080 + 68 = 1148
        // pixel_clock_10khz raw = 15153 → actual = 15154 × 10 kHz = 151 540 000 Hz
        //   → refresh = 151 540 000 / (2200 × 1148) ≈ 60 Hz
        let descriptor = make_type_ii_descriptor(
            15153, // pixel_clock_10khz raw
            239,   // ha_raw
            34,    // hb_raw
            10,    // hfp_raw
            5,     // hsw_raw
            1079,  // va_raw
            0x43,  // v_blank_byte
            0x0C,  // flags: H-sync+, V-sync+
        );
        let data_block = make_type_ii_data_block(&descriptor);
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
        assert_eq!(mode.h_front_porch, 88);
        assert_eq!(mode.h_sync_width, 48);
        assert_eq!(mode.v_front_porch, 5);
        assert_eq!(mode.v_sync_width, 4);
        assert!(!mode.interlaced);
        assert_eq!(
            mode.sync,
            Some(SyncDefinition::DigitalSeparate {
                h_sync_positive: true,
                v_sync_positive: true,
            })
        );
    }

    #[test]
    fn test_type_ii_static_pipeline() {
        let descriptor = make_type_ii_descriptor(15153, 239, 34, 10, 5, 1079, 0x43, 0x0C);
        let data_block = make_type_ii_data_block(&descriptor);
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
    fn test_type_ii_interlaced_flag() {
        // flags byte 3 bit 4 = interlaced
        let descriptor = make_type_ii_descriptor(15153, 239, 34, 10, 5, 1079, 0x43, 0x10);
        let data_block = make_type_ii_data_block(&descriptor);
        let block = make_displayid_block(0x10, &data_block);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(caps.supported_modes.len(), 1);
        assert!(caps.supported_modes[0].interlaced);
    }

    #[test]
    fn test_type_ii_multiple_descriptors() {
        // Two 11-byte descriptors packed into a single Type II block.
        let desc1 = make_type_ii_descriptor(15153, 239, 34, 10, 5, 1079, 0x43, 0x0C);
        // 2560×1440@60: ha_raw=(2560-8)/8=319, hb_raw=(440-8)/8=54 → h_total=3000
        // va_raw=1440-1=1439=0x59F, v_blank_byte=0x31→v_blank=50 → v_total=1490
        // clock: 60*3000*1490=268200000 Hz → 26820 × 10kHz → raw=26819
        let desc2 = make_type_ii_descriptor(26819, 319, 54, 10, 4, 1439, 0x31, 0x0C);

        let mut payload = Vec::new();
        // Single data block header with 22-byte payload (two 11-byte descriptors).
        payload.push(TAG_TYPE_II_TIMING);
        payload.push(0x00); // revision
        payload.push(22); // payload length
        payload.extend_from_slice(&desc1);
        payload.extend_from_slice(&desc2);

        let block = make_displayid_block(0x10, &payload);
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
    fn test_type_ii_partial_descriptor_ignored() {
        // A Type II block with only 10 bytes of payload — one short of a full descriptor.
        // No mode should be produced.
        let mut payload = [0u8; 13]; // header (3) + 10 bytes
        payload[0] = TAG_TYPE_II_TIMING;
        payload[1] = 0x00;
        payload[2] = 10;
        let block = make_displayid_block(0x10, &payload);

        let mut caps = DisplayCapabilities::default();
        let mut warnings: Vec<ParseWarning> = Vec::new();
        ExtensionHandler::process(&DisplayIdHandler, &[&block], &mut caps, &mut warnings);

        assert!(warnings.is_empty());
        assert!(caps.supported_modes.is_empty());
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
