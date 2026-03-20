use super::{
    TAG_CTA_VIDEO_TIMING, TAG_TYPE_I_TIMING, TAG_TYPE_II_TIMING, TAG_TYPE_III_TIMING,
    TAG_TYPE_IV_TIMING, TAG_VESA_VIDEO_TIMING, for_each_data_block,
};
use crate::capabilities::cea861::{dmt_to_mode, vic_to_mode};
use crate::model::capabilities::{ModeSink, StereoMode, SyncDefinition, VideoMode};

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
pub(super) fn decode_type_i_descriptor(d: &[u8; 20], sink: &mut dyn ModeSink) {
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
pub(super) fn decode_type_ii_descriptor(d: &[u8; 11], sink: &mut dyn ModeSink) {
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

/// Decodes one 3-byte Type III Short Video Timing descriptor and pushes a mode to `sink`.
///
/// Descriptor layout (DisplayID 1.x §4.4.4):
/// - Byte 0:  Bit 7 = preferred; bits 6:4 = CVT algorithm (0=standard, 1=reduced blanking);
///   bits 3:0 = aspect ratio code (see table below)
/// - Byte 1:  Horizontal active = `(byte + 1) × 8` pixels
/// - Byte 2:  Bit 7 = interlaced; bits 6:0 = refresh rate = `bits + 1` Hz
///
/// Aspect ratio codes:
/// `0`=1:1, `1`=5:4, `2`=4:3, `3`=15:9, `4`=16:9, `5`=16:10, `6`=64:27, `7`=256:135, `8`=undefined
///
/// Vertical active is derived from horizontal active and the aspect ratio. Descriptors
/// with reserved or undefined aspect ratios (code > 7) are silently skipped, as are
/// descriptors where the height calculation does not yield a whole number of lines.
pub(super) fn decode_type_iii_descriptor(d: &[u8; 3], sink: &mut dyn ModeSink) {
    let aspect_code = d[0] & 0x0F;
    let h_active = ((d[1] as u16) + 1) * 8;
    let interlaced = (d[2] & 0x80) != 0;
    let refresh_rate = (d[2] & 0x7F) + 1;

    // Compute v_active from the aspect ratio (width:height → v = h × height / width).
    // Aspect code 8 = undefined; codes 9–15 = reserved; both are skipped.
    let (ar_h, ar_w): (u32, u32) = match aspect_code {
        0 => (1, 1),     // 1:1
        1 => (4, 5),     // 5:4
        2 => (3, 4),     // 4:3
        3 => (9, 15),    // 15:9
        4 => (9, 16),    // 16:9
        5 => (10, 16),   // 16:10
        6 => (27, 64),   // 64:27
        7 => (135, 256), // 256:135
        _ => return,     // undefined or reserved
    };
    let v_raw = (h_active as u32) * ar_h;
    if v_raw % ar_w != 0 {
        return; // h_active is not a valid value for this aspect ratio
    }
    let v_active = (v_raw / ar_w) as u16;
    if v_active == 0 {
        return;
    }

    sink.push_mode(VideoMode {
        width: h_active,
        height: v_active,
        refresh_rate,
        interlaced,
        h_front_porch: 0,
        h_sync_width: 0,
        v_front_porch: 0,
        v_sync_width: 0,
        h_border: 0,
        v_border: 0,
        stereo: StereoMode::None,
        sync: None,
    });
}

/// Decodes a Type IV DMT/VIC Code Block payload and pushes resolved modes to `sink`.
///
/// The code space is determined by the revision byte's upper 2 bits, passed in as `code_type`:
/// - `0` — VESA DMT IDs (1-byte each); resolved via the DMT table
/// - `1` — CTA-861 VIC codes (1-byte each); resolved via the VIC table
/// - `2` — HDMI VIC codes (1-byte each); 4 codes defined: 1–4
/// - `3` — reserved; descriptors silently skipped
///
/// Codes not found in the respective table are silently skipped.
pub(super) fn decode_type_iv_block(payload: &[u8], code_type: u8, sink: &mut dyn ModeSink) {
    for &code in payload {
        let mode = match code_type {
            0 => dmt_to_mode(code as u16),
            1 => vic_to_mode(code),
            2 => hdmi_vic_to_mode(code),
            _ => None,
        };
        if let Some(m) = mode {
            sink.push_mode(m);
        }
    }
}

/// Returns the `VideoMode` for an HDMI VIC code (1–4), or `None`.
///
/// HDMI 1.4 defines four extended resolution codes for 4K/UHD modes:
/// - 1: 3840×2160@30 Hz
/// - 2: 3840×2160@25 Hz
/// - 3: 3840×2160@24 Hz
/// - 4: 4096×2160@24 Hz
fn hdmi_vic_to_mode(code: u8) -> Option<VideoMode> {
    let (w, h, r): (u16, u16, u8) = match code {
        1 => (3840, 2160, 30),
        2 => (3840, 2160, 25),
        3 => (3840, 2160, 24),
        4 => (4096, 2160, 24),
        _ => return None,
    };
    Some(VideoMode {
        width: w,
        height: h,
        refresh_rate: r,
        ..Default::default()
    })
}

/// Iterates the VESA Video Timing Block presence bitmap and pushes supported modes to `sink`.
///
/// The payload is a bitmap of up to 10 bytes (80 bits). Bit `i` (0-indexed, LSB-first within
/// each byte) corresponds to DMT ID `i + 1`. A set bit means the display supports that mode.
/// Bytes beyond the first 10 are ignored; unknown or unset bits produce no output.
///
/// Source: edid-decode (timvideos/edid-decode); confirmed against `DATA_BLOCK_VESA_TIMING`
/// in the Linux kernel `drivers/gpu/drm/drm_displayid_internal.h`.
pub(super) fn decode_vesa_video_timing_block(payload: &[u8], sink: &mut dyn ModeSink) {
    for (byte_idx, &byte) in payload.iter().take(10).enumerate() {
        for bit in 0u16..8 {
            if byte & (1 << bit) != 0 {
                let dmt_id = (byte_idx as u16) * 8 + bit + 1;
                if let Some(mode) = dmt_to_mode(dmt_id) {
                    sink.push_mode(mode);
                }
            }
        }
    }
}

/// Iterates the CTA-861 Video Timing Block presence bitmap and pushes supported modes to `sink`.
///
/// The payload is a bitmap of up to 8 bytes (64 bits). Bit `i` (0-indexed, LSB-first within
/// each byte) corresponds to VIC `i + 1`. A set bit means the display supports that mode.
/// Bytes beyond the first 8 are ignored; unset bits and unrecognised VICs produce no output.
///
/// Source: edid-decode (swick/edid-decode), `parse-displayid-block.cpp`.
pub(super) fn decode_cta_video_timing_block(payload: &[u8], sink: &mut dyn ModeSink) {
    for (byte_idx, &byte) in payload.iter().take(8).enumerate() {
        for bit in 0u8..8 {
            if byte & (1 << bit) != 0 {
                let vic = (byte_idx as u8) * 8 + bit + 1;
                if let Some(mode) = vic_to_mode(vic) {
                    sink.push_mode(mode);
                }
            }
        }
    }
}

/// Iterates DisplayID 1.x data blocks within a fragment's payload region and pushes
/// decoded modes to `sink`.
///
/// `payload` must be the data-block region: bytes `block[4..4+section_byte_count]`,
/// clamped to `block[4..127]` to exclude the checksum byte.
pub(super) fn process_data_blocks(payload: &[u8], sink: &mut dyn ModeSink) {
    for_each_data_block(payload, |tag, revision, block_payload| {
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
        } else if tag == TAG_TYPE_III_TIMING {
            let mut i = 0;
            while i + 3 <= block_payload.len() {
                let descriptor: &[u8; 3] = block_payload[i..i + 3].try_into().unwrap();
                decode_type_iii_descriptor(descriptor, sink);
                i += 3;
            }
        } else if tag == TAG_TYPE_IV_TIMING {
            // Revision byte bits 7:6 select the code space (0=DMT, 1=VIC, 2=HDMI VIC).
            let code_type = (revision >> 6) & 0x03;
            decode_type_iv_block(block_payload, code_type, sink);
        } else if tag == TAG_VESA_VIDEO_TIMING {
            decode_vesa_video_timing_block(block_payload, sink);
        } else if tag == TAG_CTA_VIDEO_TIMING {
            decode_cta_video_timing_block(block_payload, sink);
        }
        // Unknown block tags are silently skipped.
    });
}

#[cfg(test)]
#[cfg(any(feature = "alloc", feature = "std"))]
mod tests {
    use super::*;
    use crate::model::capabilities::{
        DisplayCapabilities, StaticContext, StaticDisplayCapabilities, SyncDefinition,
    };

    // -----------------------------------------------------------------------
    // Shared test helpers
    // -----------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
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

    fn make_type_iii_descriptor(
        preferred: bool,
        algo: u8,
        aspect: u8,
        h_raw: u8,
        interlaced: bool,
        refresh_raw: u8,
    ) -> [u8; 3] {
        let byte0 = ((preferred as u8) << 7) | ((algo & 0x07) << 4) | (aspect & 0x0F);
        let byte2 = ((interlaced as u8) << 7) | (refresh_raw & 0x7F);
        [byte0, h_raw, byte2]
    }

    // -----------------------------------------------------------------------
    // Type I Video Timing (tag 0x03)
    // -----------------------------------------------------------------------

    #[test]
    fn test_type_i_timing_decoded() {
        // 1920×1080@60 Hz: pixel clock ≈ 148.5 MHz = 14850 × 10 kHz
        // h_total = 2200, v_total = 1125 → 148500000 / (2200 * 1125) ≈ 60 Hz
        let d = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let mut caps = DisplayCapabilities::default();
        decode_type_i_descriptor(&d, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        let mode = &caps.supported_modes[0];
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
        assert!(!mode.interlaced);
    }

    #[test]
    fn test_type_i_null_descriptor_skipped() {
        let null_descriptor = [0u8; 20];
        let mut caps = DisplayCapabilities::default();
        decode_type_i_descriptor(&null_descriptor, &mut caps);
        assert!(caps.supported_modes.is_empty());
    }

    #[test]
    fn test_type_i_interlaced_flag_decoded() {
        // flags byte 19 bit 0 = interlaced
        let d = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x01);
        let mut caps = DisplayCapabilities::default();
        decode_type_i_descriptor(&d, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        assert!(caps.supported_modes[0].interlaced);
    }

    #[test]
    fn test_type_i_static_pipeline() {
        let d = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let mut payload = vec![0x03u8, 0x00, 20];
        payload.extend_from_slice(&d);
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
    }

    // -----------------------------------------------------------------------
    // process_data_blocks dispatcher
    // -----------------------------------------------------------------------

    #[test]
    fn test_multiple_descriptors_in_block() {
        // Two 20-byte descriptors in one Type I data block.
        let desc1 = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let desc2 = make_type_i_descriptor(22118, 2560, 440, 80, 32, 1440, 41, 4, 5, 0x00);
        let mut payload = vec![0x03u8, 0x00, 40]; // Type I tag, revision, 40-byte payload
        payload.extend_from_slice(&desc1);
        payload.extend_from_slice(&desc2);
        let mut caps = DisplayCapabilities::default();
        process_data_blocks(&payload, &mut caps);
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
        let payload = [0x03u8, 0x00, 50]; // claims 50 bytes; 0 remain after the 3-byte header
        let mut caps = DisplayCapabilities::default();
        process_data_blocks(&payload, &mut caps);
        assert!(caps.supported_modes.is_empty());
    }

    #[test]
    fn test_eos_sentinel_stops_iteration() {
        // A valid Type I descriptor followed by an EOS sentinel (tag=0, length=0).
        // A second descriptor after the sentinel must not be decoded.
        let desc = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let desc2 = make_type_i_descriptor(22118, 2560, 440, 80, 32, 1440, 41, 4, 5, 0x00);
        let mut payload = vec![0x03u8, 0x00, 20];
        payload.extend_from_slice(&desc);
        payload.extend_from_slice(&[0x00, 0x00, 0x00]); // EOS sentinel
        payload.extend_from_slice(&[0x03, 0x00, 20]);
        payload.extend_from_slice(&desc2);
        let mut caps = DisplayCapabilities::default();
        process_data_blocks(&payload, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 1920);
    }

    #[test]
    fn test_unknown_data_block_tag_skipped() {
        // An unknown data block (tag 0xFF, 10-byte payload) followed by a valid Type I block.
        let desc = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let mut payload = vec![0xFFu8, 0x00, 10];
        payload.extend_from_slice(&[0u8; 10]);
        payload.extend_from_slice(&[0x03, 0x00, 20]);
        payload.extend_from_slice(&desc);
        let mut caps = DisplayCapabilities::default();
        process_data_blocks(&payload, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 1920);
    }

    // -----------------------------------------------------------------------
    // Type II Video Timing (tag 0x04)
    // -----------------------------------------------------------------------

    #[test]
    fn test_type_ii_timing_decoded() {
        // 1920×1080@60 Hz via Type II encoding.
        let d = make_type_ii_descriptor(15153, 239, 34, 10, 5, 1079, 0x43, 0x0C);
        let mut caps = DisplayCapabilities::default();
        decode_type_ii_descriptor(&d, &mut caps);
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
    fn test_type_ii_interlaced_flag() {
        // flags byte 3 bit 4 = interlaced
        let d = make_type_ii_descriptor(15153, 239, 34, 10, 5, 1079, 0x43, 0x10);
        let mut caps = DisplayCapabilities::default();
        decode_type_ii_descriptor(&d, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        assert!(caps.supported_modes[0].interlaced);
    }

    #[test]
    fn test_type_ii_multiple_descriptors() {
        let desc1 = make_type_ii_descriptor(15153, 239, 34, 10, 5, 1079, 0x43, 0x0C);
        let desc2 = make_type_ii_descriptor(26819, 319, 54, 10, 4, 1439, 0x31, 0x0C);
        let mut payload = vec![0x04u8, 0x00, 22]; // Type II tag, 22-byte payload
        payload.extend_from_slice(&desc1);
        payload.extend_from_slice(&desc2);
        let mut caps = DisplayCapabilities::default();
        process_data_blocks(&payload, &mut caps);
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
        // Only 10 bytes — one short of a full descriptor; no mode should be produced.
        let mut payload = vec![0x04u8, 0x00, 10];
        payload.extend_from_slice(&[0u8; 10]);
        let mut caps = DisplayCapabilities::default();
        process_data_blocks(&payload, &mut caps);
        assert!(caps.supported_modes.is_empty());
    }

    #[test]
    fn test_type_ii_static_pipeline() {
        let d = make_type_ii_descriptor(15153, 239, 34, 10, 5, 1079, 0x43, 0x0C);
        let mut payload = vec![0x04u8, 0x00, 11];
        payload.extend_from_slice(&d);
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
    }

    // -----------------------------------------------------------------------
    // Type III Short Video Timing (tag 0x05)
    // -----------------------------------------------------------------------

    #[test]
    fn test_type_iii_16_9_decoded() {
        // 1920×1080@60 Hz: aspect=16:9(0x04), h_raw=239→h=1920, refresh_raw=59→60 Hz
        let d = make_type_iii_descriptor(false, 0, 0x04, 239, false, 59);
        let mut caps = DisplayCapabilities::default();
        decode_type_iii_descriptor(&d, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        let mode = &caps.supported_modes[0];
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
        assert!(!mode.interlaced);
    }

    #[test]
    fn test_type_iii_4_3_decoded() {
        // 1024×768@75 Hz: aspect=4:3(0x02), h_raw=127→h=1024, refresh_raw=74→75 Hz
        let d = make_type_iii_descriptor(false, 0, 0x02, 127, false, 74);
        let mut caps = DisplayCapabilities::default();
        decode_type_iii_descriptor(&d, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 1024);
        assert_eq!(caps.supported_modes[0].height, 768);
        assert_eq!(caps.supported_modes[0].refresh_rate, 75);
    }

    #[test]
    fn test_type_iii_interlaced_flag() {
        let d = make_type_iii_descriptor(false, 0, 0x04, 239, true, 59);
        let mut caps = DisplayCapabilities::default();
        decode_type_iii_descriptor(&d, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        assert!(caps.supported_modes[0].interlaced);
    }

    #[test]
    fn test_type_iii_undefined_aspect_skipped() {
        // Aspect code 0x08 = undefined; descriptor must be skipped.
        let d = make_type_iii_descriptor(false, 0, 0x08, 239, false, 59);
        let mut caps = DisplayCapabilities::default();
        decode_type_iii_descriptor(&d, &mut caps);
        assert!(caps.supported_modes.is_empty());
    }

    #[test]
    fn test_type_iii_non_integer_height_skipped() {
        // 16:9 requires h_active divisible by 16. h_raw=0→h=8, 8*9/16=4.5 — not integer; skip.
        let d = make_type_iii_descriptor(false, 0, 0x04, 0, false, 59);
        let mut caps = DisplayCapabilities::default();
        decode_type_iii_descriptor(&d, &mut caps);
        assert!(caps.supported_modes.is_empty());
    }

    #[test]
    fn test_type_iii_multiple_aspect_ratios() {
        let d1 = make_type_iii_descriptor(false, 0, 0x04, 239, false, 59); // 1920×1080@60
        let d2 = make_type_iii_descriptor(false, 0, 0x02, 159, false, 59); // 1280×960@60
        let d3 = make_type_iii_descriptor(false, 0, 0x05, 159, false, 59); // 1280×800@60
        let mut caps = DisplayCapabilities::default();
        decode_type_iii_descriptor(&d1, &mut caps);
        decode_type_iii_descriptor(&d2, &mut caps);
        decode_type_iii_descriptor(&d3, &mut caps);
        assert_eq!(caps.supported_modes.len(), 3);
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 1920 && m.height == 1080)
        );
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 1280 && m.height == 960)
        );
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 1280 && m.height == 800)
        );
    }

    #[test]
    fn test_type_iii_static_pipeline() {
        let d = make_type_iii_descriptor(false, 0, 0x04, 239, false, 59);
        let mut payload = vec![0x05u8, 0x00, 3];
        payload.extend_from_slice(&d);
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
    }

    // -----------------------------------------------------------------------
    // Type IV DMT/VIC Code Block (tag 0x06)
    // -----------------------------------------------------------------------

    #[test]
    fn test_type_iv_dmt_decoded() {
        // DMT 0x52 = 1920×1080@60 Hz; verify full timing detail from the DMT table.
        let mut caps = DisplayCapabilities::default();
        decode_type_iv_block(&[0x52], 0, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        let mode = &caps.supported_modes[0];
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
        assert!(!mode.interlaced);
        assert_eq!(mode.h_front_porch, 88);
        assert_eq!(mode.h_sync_width, 44);
        assert_eq!(mode.v_front_porch, 4);
        assert_eq!(mode.v_sync_width, 5);
        assert_eq!(
            mode.sync,
            Some(SyncDefinition::DigitalSeparate {
                h_sync_positive: true,
                v_sync_positive: true,
            })
        );
    }

    #[test]
    fn test_type_iv_dmt_interlaced() {
        // DMT 0x0F = 1024×768i@43 Hz (interlaced)
        let mut caps = DisplayCapabilities::default();
        decode_type_iv_block(&[0x0F], 0, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        let mode = &caps.supported_modes[0];
        assert_eq!(mode.width, 1024);
        assert_eq!(mode.height, 768);
        assert_eq!(mode.refresh_rate, 43);
        assert!(mode.interlaced);
    }

    #[test]
    fn test_type_iv_dmt_static_pipeline() {
        // DMT 0x09 = 800×600@60 Hz
        let payload = vec![0x06u8, 0x00, 1, 0x09]; // code_type=0 (DMT) in revision bits 7:6
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 800);
        assert_eq!(mode.height, 600);
        assert_eq!(mode.refresh_rate, 60);
    }

    #[test]
    fn test_type_iv_vic_decoded() {
        // VIC 1 = 640×480@60 Hz
        let mut caps = DisplayCapabilities::default();
        decode_type_iv_block(&[1], 1, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 640);
        assert_eq!(caps.supported_modes[0].height, 480);
        assert_eq!(caps.supported_modes[0].refresh_rate, 60);
    }

    #[test]
    fn test_type_iv_hdmi_vic_decoded() {
        // HDMI VIC 1 = 3840×2160@30 Hz
        let mut caps = DisplayCapabilities::default();
        decode_type_iv_block(&[1], 2, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 3840);
        assert_eq!(caps.supported_modes[0].height, 2160);
        assert_eq!(caps.supported_modes[0].refresh_rate, 30);
    }

    #[test]
    fn test_type_iv_unknown_dmt_skipped() {
        let mut caps = DisplayCapabilities::default();
        decode_type_iv_block(&[0xFF], 0, &mut caps);
        assert!(caps.supported_modes.is_empty());
    }

    #[test]
    fn test_type_iv_multiple_codes() {
        let mut caps = DisplayCapabilities::default();
        decode_type_iv_block(&[0x10, 0x23, 0x52], 0, &mut caps); // 1024×768@60, 1280×1024@60, 1920×1080@60
        assert_eq!(caps.supported_modes.len(), 3);
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 1024 && m.height == 768)
        );
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 1280 && m.height == 1024)
        );
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 1920 && m.height == 1080)
        );
    }

    #[test]
    fn test_type_iv_mixed_known_unknown_dmt_codes() {
        // Unknown codes sandwiched around a known one — only the known one yields a mode.
        let mut caps = DisplayCapabilities::default();
        decode_type_iv_block(&[0xFE, 0x23, 0xFF], 0, &mut caps); // 0x23 = 1280×1024@60
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 1280);
        assert_eq!(caps.supported_modes[0].height, 1024);
        assert_eq!(caps.supported_modes[0].refresh_rate, 60);
    }

    #[test]
    fn test_type_iv_reserved_code_type_skipped() {
        // Code type 3 is reserved; no mode should be produced.
        let mut caps = DisplayCapabilities::default();
        decode_type_iv_block(&[0x52], 3, &mut caps);
        assert!(caps.supported_modes.is_empty());
    }

    // -----------------------------------------------------------------------
    // VESA Video Timing Block (tag 0x07)
    // -----------------------------------------------------------------------

    #[test]
    fn test_vesa_timing_single_bit_set() {
        // Bitmap byte 0, bit 3 (0-indexed) = DMT ID 0x04 = 640×480@60 Hz.
        let mut caps = DisplayCapabilities::default();
        decode_vesa_video_timing_block(&[0b0000_1000], &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        let mode = &caps.supported_modes[0];
        assert_eq!(mode.width, 640);
        assert_eq!(mode.height, 480);
        assert_eq!(mode.refresh_rate, 60);
        assert_eq!(mode.h_front_porch, 16);
        assert_eq!(mode.h_sync_width, 96);
        assert_eq!(mode.v_front_porch, 10);
        assert_eq!(mode.v_sync_width, 2);
        assert_eq!(
            mode.sync,
            Some(SyncDefinition::DigitalSeparate {
                h_sync_positive: false,
                v_sync_positive: false,
            })
        );
    }

    #[test]
    fn test_vesa_timing_multiple_modes() {
        // Byte 0 bit 0 = DMT 0x01 (640×350@85), byte 1 bit 0 = DMT 0x09 (800×600@60).
        let mut caps = DisplayCapabilities::default();
        decode_vesa_video_timing_block(&[0x01, 0x01], &mut caps);
        assert_eq!(caps.supported_modes.len(), 2);
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 640 && m.height == 350)
        );
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 800 && m.height == 600)
        );
    }

    #[test]
    fn test_vesa_timing_empty_payload() {
        let mut caps = DisplayCapabilities::default();
        decode_vesa_video_timing_block(&[], &mut caps);
        assert!(caps.supported_modes.is_empty());
    }

    #[test]
    fn test_vesa_timing_payload_truncated_at_10_bytes() {
        // 11-byte payload; the 11th byte has bits set but must be ignored.
        let mut bitmap = vec![0u8; 11];
        bitmap[10] = 0xFF; // outside the 10-byte window
        bitmap[0] = 0x01; // DMT 0x01 = 640×350@85 — should be decoded
        let mut caps = DisplayCapabilities::default();
        decode_vesa_video_timing_block(&bitmap, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 640);
        assert_eq!(caps.supported_modes[0].height, 350);
    }

    #[test]
    fn test_vesa_timing_static_pipeline() {
        // Byte 1 bit 0 = DMT 0x09 = 800×600@60 Hz.
        let payload = vec![0x07u8, 0x00, 2, 0x00, 0x01];
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 800);
        assert_eq!(mode.height, 600);
        assert_eq!(mode.refresh_rate, 60);
    }

    // -----------------------------------------------------------------------
    // CTA-861 Video Timing Block (tag 0x08)
    // -----------------------------------------------------------------------

    #[test]
    fn test_cta_timing_single_bit_set() {
        // Byte 0 bit 0 = VIC 1 = 640×480@60 Hz.
        let mut caps = DisplayCapabilities::default();
        decode_cta_video_timing_block(&[0x01], &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        let mode = &caps.supported_modes[0];
        assert_eq!(mode.width, 640);
        assert_eq!(mode.height, 480);
        assert_eq!(mode.refresh_rate, 60);
        // VIC lookups carry full timing detail.
        assert!(mode.h_front_porch != 0 || mode.h_sync_width != 0);
    }

    #[test]
    fn test_cta_timing_multiple_modes() {
        // Byte 0 = 0x03 → bits 0 and 1 set → VIC 1 (640×480@60) and VIC 2 (480p@60).
        let mut caps = DisplayCapabilities::default();
        decode_cta_video_timing_block(&[0x03], &mut caps);
        assert_eq!(caps.supported_modes.len(), 2);
    }

    #[test]
    fn test_cta_timing_empty_payload() {
        let mut caps = DisplayCapabilities::default();
        decode_cta_video_timing_block(&[], &mut caps);
        assert!(caps.supported_modes.is_empty());
    }

    #[test]
    fn test_cta_timing_payload_truncated_at_8_bytes() {
        // 9-byte payload; the 9th byte is outside the 8-byte window and must be ignored.
        let mut bitmap = vec![0u8; 9];
        bitmap[8] = 0xFF; // outside the limit
        bitmap[0] = 0x01; // VIC 1 — should be decoded
        let mut caps = DisplayCapabilities::default();
        decode_cta_video_timing_block(&bitmap, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 640);
    }

    #[test]
    fn test_cta_timing_static_pipeline() {
        let payload = vec![0x08u8, 0x00, 1, 0x01]; // VIC 1
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        assert_eq!(caps.supported_modes[0].as_ref().unwrap().width, 640);
    }
}
