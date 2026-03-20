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

#[cfg(test)]
#[cfg(any(feature = "alloc", feature = "std"))]
mod tests {
    use super::*;
    use crate::model::capabilities::{DisplayCapabilities, SyncDefinition};

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

    #[allow(clippy::too_many_arguments)]
    fn make_type_ii_descriptor(
        pixel_clock_10khz: u32,
        ha_raw: u16,
        hb_raw: u8,
        hfp_raw: u8,
        hsw_raw: u8,
        va_raw: u16,
        v_blank_byte: u8,
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
        d[10] = 0x00;
        d
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
        // Two descriptors decoded in sequence.
        // 2560×1440@60: ha_raw=(2560-8)/8=319, hb_raw=(440-8)/8=54 → h_total=3000
        // va_raw=1440-1=1439=0x59F, v_blank_byte=0x31→v_blank=50 → v_total=1490
        let desc1 = make_type_ii_descriptor(15153, 239, 34, 10, 5, 1079, 0x43, 0x0C);
        let desc2 = make_type_ii_descriptor(26819, 319, 54, 10, 4, 1439, 0x31, 0x0C);
        let mut caps = DisplayCapabilities::default();
        decode_type_ii_descriptor(&desc1, &mut caps);
        decode_type_ii_descriptor(&desc2, &mut caps);
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
}
