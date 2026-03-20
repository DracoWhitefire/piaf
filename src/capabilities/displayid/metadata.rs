use super::{
    TAG_COLOR_CHARACTERISTICS, TAG_DISPLAY_PARAMS, TAG_PRODUCT_ID, TAG_SERIAL_NUMBER,
    TAG_VIDEO_TIMING_RANGE, for_each_data_block,
};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::color::ColorBitDepth;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::color::{Chromaticity, ChromaticityPoint};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::manufacture::{ManufactureDate, ManufacturerId, MonitorString};

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
/// If the block is already populated (e.g., by the EDID base block), the values
/// are overwritten by the DisplayID data.
#[cfg(any(feature = "alloc", feature = "std"))]
pub(super) fn decode_product_id_block(payload: &[u8], caps: &mut DisplayCapabilities) {
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
pub(super) fn scan_product_id_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    for_each_data_block(payload, |tag, _revision, block_payload| {
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
pub(super) fn decode_display_params_block(payload: &[u8], caps: &mut DisplayCapabilities) {
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
pub(super) fn scan_display_params_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    for_each_data_block(payload, |tag, _revision, block_payload| {
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
pub(super) fn decode_color_characteristics_block(payload: &[u8], caps: &mut DisplayCapabilities) {
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
pub(super) fn scan_color_characteristics_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    for_each_data_block(payload, |tag, _revision, block_payload| {
        if tag == TAG_COLOR_CHARACTERISTICS {
            decode_color_characteristics_block(block_payload, caps);
        }
    });
}

/// Decodes a Video Timing Range Limits Block payload into `caps`.
///
/// Payload layout (DisplayID 1.x §4.5, 15 bytes):
/// - Bytes 0–2:  Minimum pixel clock in 10 kHz steps (24-bit LE; not stored)
/// - Bytes 3–5:  Maximum pixel clock in 10 kHz steps (24-bit LE; stored ÷ 100 → MHz)
/// - Byte  6:    Minimum horizontal frequency in kHz
/// - Byte  7:    Maximum horizontal frequency in kHz
/// - Bytes 8–9:  Minimum horizontal blanking in pixels (LE uint16; not stored)
/// - Byte  10:   Minimum vertical refresh rate in Hz
/// - Byte  11:   Maximum vertical refresh rate in Hz
/// - Bytes 12–13: Minimum vertical blanking in lines (LE uint16; not stored)
/// - Byte  14:   Video timing support flags (not stored)
///
/// Each field is written only when the payload is long enough to contain it.
#[cfg(any(feature = "alloc", feature = "std"))]
pub(super) fn decode_video_timing_range_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    // Maximum pixel clock (bytes 3–5): stored as MHz = raw_10khz / 100.
    if payload.len() >= 6 {
        let raw = (payload[3] as u32) | ((payload[4] as u32) << 8) | ((payload[5] as u32) << 16);
        caps.max_pixel_clock_mhz = Some((raw / 100) as u16);
    }

    // Horizontal frequency range (bytes 6–7).
    if payload.len() >= 8 {
        caps.min_h_rate_khz = Some(payload[6] as u16);
        caps.max_h_rate_khz = Some(payload[7] as u16);
    }

    // Vertical refresh rate range (bytes 10–11).
    if payload.len() >= 12 {
        caps.min_v_rate = Some(payload[10] as u16);
        caps.max_v_rate = Some(payload[11] as u16);
    }
}

/// Scans all data blocks in `payload` for a Video Timing Range Limits Block (tag `0x09`)
/// and decodes the first one found into `caps`.
#[cfg(any(feature = "alloc", feature = "std"))]
pub(super) fn scan_video_timing_range_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    for_each_data_block(payload, |tag, _revision, block_payload| {
        if tag == TAG_VIDEO_TIMING_RANGE {
            decode_video_timing_range_block(block_payload, caps);
        }
    });
}

/// Decodes a Product Serial Number Block payload into `caps.serial_number_string`.
///
/// Payload layout (DisplayID 1.x §4.8):
/// - Bytes 0+: ASCII serial number string (`0x0A`-terminated, space-padded).
///
/// The string is stored in the same `MonitorString` format used by EDID base-block
/// serial number descriptors (`0xFF`): up to 13 bytes, `0x0A`-terminated.
/// Empty payloads are silently ignored.
#[cfg(any(feature = "alloc", feature = "std"))]
pub(super) fn decode_serial_number_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    if payload.is_empty() {
        return;
    }
    let mut buf = [b' '; 13];
    let copy_len = payload.len().min(13);
    buf[..copy_len].copy_from_slice(&payload[..copy_len]);
    if !buf.contains(&0x0A) {
        buf[copy_len.min(12)] = 0x0A;
    }
    caps.serial_number_string = Some(MonitorString(buf));
}

/// Scans all data blocks in `payload` for a Product Serial Number Block (tag `0x0A`)
/// and decodes the first one found into `caps`.
#[cfg(any(feature = "alloc", feature = "std"))]
pub(super) fn scan_serial_number_block(payload: &[u8], caps: &mut DisplayCapabilities) {
    for_each_data_block(payload, |tag, _revision, block_payload| {
        if tag == TAG_SERIAL_NUMBER {
            decode_serial_number_block(block_payload, caps);
        }
    });
}

#[cfg(test)]
#[cfg(any(feature = "alloc", feature = "std"))]
mod tests {
    use super::*;
    use crate::model::color::{Chromaticity, ColorBitDepth};
    use crate::model::manufacture::{ManufactureDate, ManufacturerId};

    // -----------------------------------------------------------------------
    // Shared test helpers
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

    fn make_color_characteristics_payload(
        red: (u16, u16),
        green: (u16, u16),
        blue: (u16, u16),
        white: (u16, u16),
    ) -> [u8; 16] {
        let mut p = [0u8; 16];
        let mut write = |offset: usize, val: (u16, u16)| {
            p[offset..offset + 2].copy_from_slice(&val.0.to_le_bytes());
            p[offset + 2..offset + 4].copy_from_slice(&val.1.to_le_bytes());
        };
        write(0, red);
        write(4, green);
        write(8, blue);
        write(12, white);
        p
    }

    /// Pack three ASCII uppercase letters into the 2-byte PNP manufacturer ID encoding.
    fn pack_manufacturer_id(a: u8, b: u8, c: u8) -> u16 {
        let ca = (a - b'A' + 1) as u16;
        let cb = (b - b'A' + 1) as u16;
        let cc = (c - b'A' + 1) as u16;
        (ca << 10) | (cb << 5) | cc
    }

    // -----------------------------------------------------------------------
    // Product Identification Block (tag 0x00)
    // -----------------------------------------------------------------------

    #[test]
    fn test_product_id_manufacturer_and_product_code() {
        let packed = pack_manufacturer_id(b'S', b'A', b'M');
        let payload = make_product_id_payload(packed, 0x1234, 0, 0, 0, None);
        let mut caps = DisplayCapabilities::default();
        decode_product_id_block(&payload, &mut caps);
        assert_eq!(caps.manufacturer, Some(ManufacturerId(*b"SAM")));
        assert_eq!(caps.product_code, Some(0x1234));
    }

    #[test]
    fn test_product_id_serial_number() {
        let packed = pack_manufacturer_id(b'D', b'E', b'L');
        let payload = make_product_id_payload(packed, 0x0001, 0xDEADBEEF, 0, 0, None);
        let mut caps = DisplayCapabilities::default();
        decode_product_id_block(&payload, &mut caps);
        assert_eq!(caps.serial_number, Some(0xDEAD_BEEF));
    }

    #[test]
    fn test_product_id_zero_serial_not_stored() {
        let packed = pack_manufacturer_id(b'G', b'S', b'M');
        let payload = make_product_id_payload(packed, 0x0001, 0, 0, 0, None);
        let mut caps = DisplayCapabilities::default();
        decode_product_id_block(&payload, &mut caps);
        assert_eq!(caps.serial_number, None);
    }

    #[test]
    fn test_product_id_manufacture_date() {
        let packed = pack_manufacturer_id(b'A', b'P', b'L');
        // Week 10, year 2020 → year_byte = 2020 - 1990 = 30
        let payload = make_product_id_payload(packed, 0x0001, 0, 10, 30, None);
        let mut caps = DisplayCapabilities::default();
        decode_product_id_block(&payload, &mut caps);
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
        let name: &[u8] = b"Z27k G2\x0a     ";
        let payload = make_product_id_payload(packed, 0x0042, 0, 0, 34, Some(name));
        let mut caps = DisplayCapabilities::default();
        decode_product_id_block(&payload, &mut caps);
        assert_eq!(caps.display_name.as_deref(), Some("Z27k G2"));
    }

    #[test]
    fn test_product_id_too_short_does_not_panic() {
        // Only 1 byte — too short for any field.
        let payload = [0xFFu8];
        let mut caps = DisplayCapabilities::default();
        decode_product_id_block(&payload, &mut caps);
        assert_eq!(caps.manufacturer, None);
        assert_eq!(caps.product_code, None);
    }

    // -----------------------------------------------------------------------
    // Display Parameters Block (tag 0x01)
    // -----------------------------------------------------------------------

    #[test]
    fn test_display_params_image_size_mm() {
        let payload = make_display_params_payload(597, 336, 0x10, 0x00);
        let mut caps = DisplayCapabilities::default();
        decode_display_params_block(&payload, &mut caps);
        assert_eq!(caps.preferred_image_size_mm, Some((597, 336)));
    }

    #[test]
    fn test_display_params_zero_size_not_stored() {
        let payload = make_display_params_payload(0, 0, 0x10, 0x00);
        let mut caps = DisplayCapabilities::default();
        decode_display_params_block(&payload, &mut caps);
        assert_eq!(caps.preferred_image_size_mm, None);
    }

    #[test]
    fn test_display_params_partial_zero_size_not_stored() {
        let payload = make_display_params_payload(597, 0, 0x10, 0x00);
        let mut caps = DisplayCapabilities::default();
        decode_display_params_block(&payload, &mut caps);
        assert_eq!(caps.preferred_image_size_mm, None);
    }

    #[test]
    fn test_display_params_color_bit_depth_8bpc() {
        // Bits 4:0 = 0b00010 = 8 bpc
        let payload = make_display_params_payload(597, 336, 0x10, 0b0000_0010);
        let mut caps = DisplayCapabilities::default();
        decode_display_params_block(&payload, &mut caps);
        assert_eq!(caps.color_bit_depth, Some(ColorBitDepth::Depth8));
    }

    #[test]
    fn test_display_params_color_bit_depth_10bpc() {
        // Bits 4:0 = 0b00011 = 10 bpc
        let payload = make_display_params_payload(597, 336, 0x10, 0b0000_0011);
        let mut caps = DisplayCapabilities::default();
        decode_display_params_block(&payload, &mut caps);
        assert_eq!(caps.color_bit_depth, Some(ColorBitDepth::Depth10));
    }

    #[test]
    fn test_display_params_undefined_bit_depth_not_stored() {
        // Bits 4:0 = 0b00000 = undefined
        let payload = make_display_params_payload(597, 336, 0x10, 0b0000_0000);
        let mut caps = DisplayCapabilities::default();
        decode_display_params_block(&payload, &mut caps);
        assert_eq!(caps.color_bit_depth, None);
    }

    #[test]
    fn test_display_params_too_short_does_not_panic() {
        // Only 3 bytes — too short for image size.
        let payload = [0x55u8, 0x01, 0x00];
        let mut caps = DisplayCapabilities::default();
        decode_display_params_block(&payload, &mut caps);
        assert_eq!(caps.preferred_image_size_mm, None);
    }

    // -----------------------------------------------------------------------
    // Color Characteristics Block (tag 0x02)
    // -----------------------------------------------------------------------

    #[test]
    fn test_color_characteristics_primaries_decoded() {
        // sRGB-like primaries: R(0.64, 0.33), G(0.30, 0.60), B(0.15, 0.06), D65(0.3127, 0.3290)
        // Scaled × 1024: R(655, 338), G(307, 614), B(154, 61), W(320, 337)
        let payload =
            make_color_characteristics_payload((655, 338), (307, 614), (154, 61), (320, 337));
        let mut caps = DisplayCapabilities::default();
        decode_color_characteristics_block(&payload, &mut caps);
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
        let mut caps = DisplayCapabilities::default();
        decode_color_characteristics_block(&payload, &mut caps);
        assert_eq!(caps.chromaticity.red.x_raw, 0x00FF);
    }

    #[test]
    fn test_color_characteristics_short_payload_ignored() {
        // A 15-byte payload — one byte short of the minimum 16. Must not modify chromaticity.
        let payload = [0u8; 15];
        let mut caps = DisplayCapabilities::default();
        decode_color_characteristics_block(&payload, &mut caps);
        assert_eq!(caps.chromaticity, Chromaticity::default());
    }

    // -----------------------------------------------------------------------
    // Video Timing Range Limits Block (tag 0x09)
    // -----------------------------------------------------------------------

    fn make_video_timing_range_payload(
        min_pixel_clock_10khz: u32,
        max_pixel_clock_10khz: u32,
        min_h_khz: u8,
        max_h_khz: u8,
        min_h_blank: u16,
        min_v_rate: u8,
        max_v_rate: u8,
        min_v_blank: u16,
        flags: u8,
    ) -> [u8; 15] {
        let mut p = [0u8; 15];
        p[0] = (min_pixel_clock_10khz & 0xFF) as u8;
        p[1] = ((min_pixel_clock_10khz >> 8) & 0xFF) as u8;
        p[2] = ((min_pixel_clock_10khz >> 16) & 0xFF) as u8;
        p[3] = (max_pixel_clock_10khz & 0xFF) as u8;
        p[4] = ((max_pixel_clock_10khz >> 8) & 0xFF) as u8;
        p[5] = ((max_pixel_clock_10khz >> 16) & 0xFF) as u8;
        p[6] = min_h_khz;
        p[7] = max_h_khz;
        p[8..10].copy_from_slice(&min_h_blank.to_le_bytes());
        p[10] = min_v_rate;
        p[11] = max_v_rate;
        p[12..14].copy_from_slice(&min_v_blank.to_le_bytes());
        p[14] = flags;
        p
    }

    #[test]
    fn test_video_timing_range_all_fields_decoded() {
        // max_pixel_clock = 33750 × 10 kHz = 337.5 MHz → stored as 337 MHz
        // h range: 30–135 kHz, v range: 48–240 Hz
        let payload = make_video_timing_range_payload(1000, 33750, 30, 135, 160, 48, 240, 45, 0);
        let mut caps = DisplayCapabilities::default();
        decode_video_timing_range_block(&payload, &mut caps);
        assert_eq!(caps.max_pixel_clock_mhz, Some(337));
        assert_eq!(caps.min_h_rate_khz, Some(30));
        assert_eq!(caps.max_h_rate_khz, Some(135));
        assert_eq!(caps.min_v_rate, Some(48));
        assert_eq!(caps.max_v_rate, Some(240));
    }

    #[test]
    fn test_video_timing_range_typical_monitor() {
        // 1920×1080@60 Hz monitor: max clock ~148.5 MHz = 14850 × 10 kHz → 148 MHz
        // h: 30–83 kHz, v: 56–75 Hz
        let payload = make_video_timing_range_payload(3000, 14850, 30, 83, 160, 56, 75, 45, 0x00);
        let mut caps = DisplayCapabilities::default();
        decode_video_timing_range_block(&payload, &mut caps);
        assert_eq!(caps.max_pixel_clock_mhz, Some(148));
        assert_eq!(caps.min_h_rate_khz, Some(30));
        assert_eq!(caps.max_h_rate_khz, Some(83));
        assert_eq!(caps.min_v_rate, Some(56));
        assert_eq!(caps.max_v_rate, Some(75));
    }

    #[test]
    fn test_video_timing_range_short_payload_partial_decode() {
        // Only 8 bytes — enough for max_pixel_clock and h rates, not v rates.
        let mut payload = [0u8; 8];
        payload[3] = 0x52; // max_pixel_clock low byte
        payload[4] = 0x39; // max_pixel_clock mid byte (0x3952 = 14674 × 10 kHz → 146 MHz)
        payload[6] = 25; // min_h_rate_khz
        payload[7] = 90; // max_h_rate_khz
        let mut caps = DisplayCapabilities::default();
        decode_video_timing_range_block(&payload, &mut caps);
        assert_eq!(caps.max_pixel_clock_mhz, Some(146));
        assert_eq!(caps.min_h_rate_khz, Some(25));
        assert_eq!(caps.max_h_rate_khz, Some(90));
        assert_eq!(caps.min_v_rate, None);
        assert_eq!(caps.max_v_rate, None);
    }

    #[test]
    fn test_video_timing_range_too_short_does_not_panic() {
        // Only 5 bytes — too short even for max_pixel_clock.
        let payload = [0u8; 5];
        let mut caps = DisplayCapabilities::default();
        decode_video_timing_range_block(&payload, &mut caps);
        assert_eq!(caps.max_pixel_clock_mhz, None);
        assert_eq!(caps.min_h_rate_khz, None);
        assert_eq!(caps.min_v_rate, None);
    }

    // -----------------------------------------------------------------------
    // Product Serial Number Block (tag 0x0A)
    // -----------------------------------------------------------------------

    #[test]
    fn test_serial_number_short_string() {
        let payload = b"SN12345\x0a     ";
        let mut caps = DisplayCapabilities::default();
        decode_serial_number_block(payload, &mut caps);
        assert_eq!(caps.serial_number_string.as_deref(), Some("SN12345"));
    }

    #[test]
    fn test_serial_number_full_13_bytes_no_terminator_gets_one_added() {
        // 13 bytes of non-0x0A content — a terminator must be inserted at position 12.
        let payload = b"ABCDEFGHIJKLM";
        let mut caps = DisplayCapabilities::default();
        decode_serial_number_block(payload, &mut caps);
        let s = caps.serial_number_string.unwrap();
        assert_eq!(s.0[12], 0x0A);
    }

    #[test]
    fn test_serial_number_truncated_at_13_bytes() {
        // Payload longer than 13 bytes; only the first 13 bytes are copied into the buffer,
        // but the 13th is overwritten by the 0x0A terminator, so 12 visible characters remain.
        let payload = b"ABCDEFGHIJKLMNOPQRST";
        let mut caps = DisplayCapabilities::default();
        decode_serial_number_block(payload, &mut caps);
        assert_eq!(caps.serial_number_string.as_deref(), Some("ABCDEFGHIJKL"));
    }

    #[test]
    fn test_serial_number_empty_payload_not_stored() {
        let payload: &[u8] = &[];
        let mut caps = DisplayCapabilities::default();
        decode_serial_number_block(payload, &mut caps);
        assert_eq!(caps.serial_number_string, None);
    }
}
