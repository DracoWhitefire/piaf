use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::capabilities::VideoMode;
use crate::model::color::ChromaticityPoint;
use crate::model::color::{ColorManagementData, DcmChannel, DisplayGamma, WhitePoint};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::String;
use crate::model::timing::TimingFormula;

/// Decodes the four 18-byte monitor descriptor slots (offsets 0x36, 0x48, 0x5A, 0x6C).
/// Handles monitor name (0xFC) and range limits (0xFD) descriptor types.
#[cfg(any(feature = "alloc", feature = "std"))]
pub(super) fn decode_descriptors(base: &[u8; 128], caps: &mut DisplayCapabilities) {
    for i in 0..4 {
        let offset = 0x36 + (i * 18);
        let descriptor = &base[offset..offset + 18];

        // Additional White Point Descriptor: tag 0xFB
        // Contains up to two white point entries at byte offsets 5 and 12.
        // Each entry: index (1), lsb (1), x_msb (1), y_msb (1), gamma (1). Index 0 = unused.
        if descriptor[0..4] == [0x00, 0x00, 0x00, 0xFB] {
            for entry_off in [5usize, 12usize] {
                let index = descriptor[entry_off];
                if index == 0 {
                    continue;
                }
                let lsb = descriptor[entry_off + 1];
                let x_raw = ((descriptor[entry_off + 2] as u16) << 2) | ((lsb >> 2) & 0x03) as u16;
                let y_raw = ((descriptor[entry_off + 3] as u16) << 2) | (lsb & 0x03) as u16;
                caps.white_points.push(WhitePoint {
                    index,
                    chromaticity: ChromaticityPoint { x_raw, y_raw },
                    gamma: DisplayGamma::from_edid_byte(descriptor[entry_off + 4]),
                });
            }
        }

        // Established Timings III Descriptor: tag 0xF7
        // Byte 5 must be revision 0x0A. Bytes 6-11 are a 44-bit timing bitmap.
        if descriptor[0..5] == [0x00, 0x00, 0x00, 0xF7, 0x00] && descriptor[5] == 0x0A {
            const ET3: &[(usize, u8, u16, u16, u8)] = &[
                // Byte 6
                (6, 0x80, 640, 350, 85),
                (6, 0x40, 640, 400, 85),
                (6, 0x20, 720, 400, 85),
                (6, 0x10, 640, 480, 85),
                (6, 0x08, 848, 480, 60),
                (6, 0x04, 800, 600, 85),
                (6, 0x02, 1024, 768, 85),
                (6, 0x01, 1152, 864, 75),
                // Byte 7
                (7, 0x80, 1280, 768, 60), // RB — same mode as non-RB below; deduplicates
                (7, 0x40, 1280, 768, 60),
                (7, 0x20, 1280, 768, 75),
                (7, 0x10, 1280, 768, 85),
                (7, 0x08, 1280, 960, 60),
                (7, 0x04, 1280, 960, 85),
                (7, 0x02, 1280, 1024, 60),
                (7, 0x01, 1280, 1024, 85),
                // Byte 8
                (8, 0x80, 1360, 768, 60),
                (8, 0x40, 1440, 900, 60), // RB
                (8, 0x20, 1440, 900, 60),
                (8, 0x10, 1440, 900, 75),
                (8, 0x08, 1440, 900, 85),
                (8, 0x04, 1400, 1050, 60), // RB
                (8, 0x02, 1400, 1050, 60),
                (8, 0x01, 1400, 1050, 75),
                // Byte 9
                (9, 0x80, 1400, 1050, 85),
                (9, 0x40, 1680, 1050, 60), // RB
                (9, 0x20, 1680, 1050, 60),
                (9, 0x10, 1680, 1050, 75),
                (9, 0x08, 1680, 1050, 85),
                (9, 0x04, 1600, 1200, 60),
                (9, 0x02, 1600, 1200, 65),
                (9, 0x01, 1600, 1200, 70),
                // Byte 10
                (10, 0x80, 1600, 1200, 75),
                (10, 0x40, 1600, 1200, 85),
                (10, 0x20, 1792, 1344, 60),
                (10, 0x10, 1792, 1344, 75),
                (10, 0x08, 1856, 1392, 60),
                (10, 0x04, 1856, 1392, 75),
                (10, 0x02, 1920, 1200, 60), // RB
                (10, 0x01, 1920, 1200, 60),
                // Byte 11 (bits 3-0 reserved)
                (11, 0x80, 1920, 1200, 75),
                (11, 0x40, 1920, 1200, 85),
                (11, 0x20, 1920, 1440, 60),
                (11, 0x10, 1920, 1440, 75),
            ];
            for &(byte_off, mask, w, h, rate) in ET3 {
                if descriptor[byte_off] & mask != 0 {
                    let mode = VideoMode {
                        width: w,
                        height: h,
                        refresh_rate: rate,
                        interlaced: false,
                        ..Default::default()
                    };
                    if !caps.supported_modes.contains(&mode) {
                        caps.supported_modes.push(mode);
                    }
                }
            }
        }

        // CVT 3 Byte Code Descriptor: tag 0xF8
        // Byte 5 must be version 0x01. Bytes 6-17 hold up to 4 entries of 3 bytes each.
        // An entry of (00 00 00) is unused. Byte 0 of an entry = 0x00 is reserved.
        if descriptor[0..5] == [0x00, 0x00, 0x00, 0xF8, 0x00] && descriptor[5] == 0x01 {
            for entry in 0..4usize {
                let off = 6 + entry * 3;
                let b0 = descriptor[off];
                let b1 = descriptor[off + 1];
                let b2 = descriptor[off + 2];

                if b0 == 0 {
                    continue; // unused or reserved
                }

                // Reconstruct vertical addressable lines:
                // stored = (VAdd / 2) - 1  →  VAdd = (stored + 1) * 2
                let lines_raw = (((b1 as u16) & 0xF0) << 4) | (b0 as u16);
                let v_add = (lines_raw + 1) * 2;

                // HAdd = 8 * floor((VAdd * AR) / 8)
                let h_add = {
                    let v = v_add as u32;
                    let h = match (b1 >> 2) & 0x03 {
                        0b00 => v * 4 / 3,   // 4:3
                        0b01 => v * 16 / 9,  // 16:9
                        0b10 => v * 16 / 10, // 16:10
                        _ => v * 15 / 9,     // 15:9
                    };
                    ((h / 8) * 8) as u16
                };

                // Rate bits 4-0: 50Hz std, 60Hz std, 75Hz std, 85Hz std, 60Hz RB
                // 60Hz RB deduplicates against 60Hz std since VideoMode has no RB flag.
                for (mask, rate) in [
                    (0x10u8, 50u8),
                    (0x08, 60),
                    (0x04, 75),
                    (0x02, 85),
                    (0x01, 60),
                ] {
                    if b2 & mask != 0 {
                        let mode = VideoMode {
                            width: h_add,
                            height: v_add,
                            refresh_rate: rate,
                            interlaced: false,
                            ..Default::default()
                        };
                        if !caps.supported_modes.contains(&mode) {
                            caps.supported_modes.push(mode);
                        }
                    }
                }
            }
        }

        // Additional Standard Timing Descriptor: tag 0xFA
        // Bytes 5-16 contain up to 6 standard timing entries (2 bytes each).
        if descriptor[0..4] == [0x00, 0x00, 0x00, 0xFA] {
            for j in 0..6 {
                let base_off = 5 + (j * 2);
                if let Some(mode) = super::timings::decode_standard_timing_entry(
                    descriptor[base_off],
                    descriptor[base_off + 1],
                ) {
                    if !caps.supported_modes.contains(&mode) {
                        caps.supported_modes.push(mode);
                    }
                }
            }
        }

        // Serial Number Descriptor: tag 0xFF
        if descriptor[0..4] == [0x00, 0x00, 0x00, 0xFF] {
            let s = String::from_utf8_lossy(&descriptor[5..18]);
            let trimmed = s.trim().to_string();
            if !trimmed.is_empty() {
                caps.serial_number_string = Some(trimmed);
            }
        }

        // Unspecified ASCII Text Descriptor: tag 0xFE
        if descriptor[0..4] == [0x00, 0x00, 0x00, 0xFE] {
            let s = String::from_utf8_lossy(&descriptor[5..18]);
            let trimmed = s.trim().to_string();
            if !trimmed.is_empty() {
                caps.unspecified_text.push(trimmed);
            }
        }

        // Color Management Data: tag 0xF9
        // Byte 5 = version (must be 0x03). Bytes 6-17: three pairs of (a3 LSB, a3 MSB, a2 LSB, a2 MSB)
        // for red, green, blue in that order.
        if descriptor[0..5] == [0x00, 0x00, 0x00, 0xF9, 0x00] && descriptor[5] == 0x03 {
            caps.color_management = Some(ColorManagementData {
                red: DcmChannel {
                    a3: ((descriptor[7] as u16) << 8) | (descriptor[6] as u16),
                    a2: ((descriptor[9] as u16) << 8) | (descriptor[8] as u16),
                },
                green: DcmChannel {
                    a3: ((descriptor[11] as u16) << 8) | (descriptor[10] as u16),
                    a2: ((descriptor[13] as u16) << 8) | (descriptor[12] as u16),
                },
                blue: DcmChannel {
                    a3: ((descriptor[15] as u16) << 8) | (descriptor[14] as u16),
                    a2: ((descriptor[17] as u16) << 8) | (descriptor[16] as u16),
                },
            });
        }

        // Monitor Name Descriptor: tag 0xFC
        if descriptor[0..4] == [0x00, 0x00, 0x00, 0xFC] {
            let name_bytes = &descriptor[5..18];
            let name = String::from_utf8_lossy(name_bytes);
            let trimmed = name.trim().to_string();
            if !trimmed.is_empty() {
                caps.display_name = Some(trimmed);
            }
        }

        // Monitor Range Limits: tag 0xFD
        // Byte 4 holds rate offset flags — bits 1:0 = vert offsets, bits 3:2 = horiz offsets.
        // When a "max + 255" bit is set, add 255 to the encoded byte value for that field.
        // When a "min + 255" bit is set, add 255 to the encoded byte value for that field.
        if descriptor[0..4] == [0x00, 0x00, 0x00, 0xFD] {
            let offsets = descriptor[4];
            let min_v_off: u16 = if offsets & 0x01 != 0 { 255 } else { 0 };
            let max_v_off: u16 = if offsets & 0x02 != 0 { 255 } else { 0 };
            let min_h_off: u16 = if offsets & 0x04 != 0 { 255 } else { 0 };
            let max_h_off: u16 = if offsets & 0x08 != 0 { 255 } else { 0 };

            caps.min_v_rate = Some(descriptor[5] as u16 + min_v_off);
            caps.max_v_rate = Some(descriptor[6] as u16 + max_v_off);
            caps.min_h_rate_khz = Some(descriptor[7] as u16 + min_h_off);
            caps.max_h_rate_khz = Some(descriptor[8] as u16 + max_h_off);
            caps.max_pixel_clock_mhz = Some((descriptor[9] as u16) * 10);
            caps.timing_formula = TimingFormula::from_descriptor_bytes(descriptor);
        }
    }
}

#[cfg(test)]
#[cfg(any(feature = "alloc", feature = "std"))]
mod tests {
    use crate::capabilities::base::BaseBlockHandler;
    use crate::model::capabilities::{DisplayCapabilities, VideoMode};
    use crate::model::color::{ChromaticityPoint, DisplayGamma};
    use crate::model::extension::ExtensionHandler;
    use crate::model::prelude::Vec;

    #[test]
    fn test_additional_white_point() {
        let mut base = [0u8; 128];

        // 0xFB descriptor at 0x36: two white point entries
        base[0x36..0x3A].copy_from_slice(&[0x00, 0x00, 0x00, 0xFB]);
        base[0x3A] = 0x00; // reserved

        // Entry 1 at offset 5 (0x3B): index=1, lsb=0x05 (x[1:0]=1, y[1:0]=1), x_msb=80, y_msb=84, gamma=120
        base[0x3B] = 1; // index
        base[0x3C] = 0x05; // lsb: bits 3-2 = 01 (x), bits 1-0 = 01 (y)
        base[0x3D] = 80; // x MSB → x_raw = 80*4+1 = 321
        base[0x3E] = 84; // y MSB → y_raw = 84*4+1 = 337
        base[0x3F] = 120; // gamma = 2.20

        // Entry 2 at offset 12 (0x42): index=2, unused gamma (0xFF)
        base[0x42] = 2;
        base[0x43] = 0x00;
        base[0x44] = 96; // x_raw = 384
        base[0x45] = 32; // y_raw = 128
        base[0x46] = 0xFF; // gamma undefined

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert_eq!(caps.white_points.len(), 2);
        assert_eq!(caps.white_points[0].index, 1);
        assert_eq!(
            caps.white_points[0].chromaticity,
            ChromaticityPoint {
                x_raw: 321,
                y_raw: 337
            }
        );
        assert_eq!(
            caps.white_points[0].gamma,
            DisplayGamma::from_edid_byte(120)
        );
        assert_eq!(caps.white_points[1].index, 2);
        assert_eq!(
            caps.white_points[1].chromaticity,
            ChromaticityPoint {
                x_raw: 384,
                y_raw: 128
            }
        );
        assert_eq!(caps.white_points[1].gamma, None);
    }

    #[test]
    fn test_unspecified_text_descriptor() {
        let mut base = [0u8; 128];

        // Two 0xFE descriptors in slots 0 and 1
        base[0x36..0x3A].copy_from_slice(&[0x00, 0x00, 0x00, 0xFE]);
        base[0x3A] = 0x00;
        base[0x3B..0x3F].copy_from_slice(b"ABCD");
        base[0x3F] = 0x0A;
        for b in &mut base[0x40..0x48] {
            *b = 0x20;
        }

        base[0x48..0x4C].copy_from_slice(&[0x00, 0x00, 0x00, 0xFE]);
        base[0x4C] = 0x00;
        base[0x4D..0x51].copy_from_slice(b"EFGH");
        base[0x51] = 0x0A;
        for b in &mut base[0x52..0x5A] {
            *b = 0x20;
        }

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert_eq!(caps.unspecified_text.len(), 2);
        assert_eq!(caps.unspecified_text[0], "ABCD");
        assert_eq!(caps.unspecified_text[1], "EFGH");
    }

    #[test]
    fn test_serial_number_descriptor() {
        let mut base = [0u8; 128];

        // Serial number descriptor at 0x36: "SN12345678"
        base[0x36..0x3A].copy_from_slice(&[0x00, 0x00, 0x00, 0xFF]);
        base[0x3A] = 0x00; // reserved
        base[0x3B..0x3F].copy_from_slice(b"SN12");
        base[0x3F..0x43].copy_from_slice(b"3456");
        base[0x43] = 0x0A; // newline terminator
        for b in &mut base[0x44..0x48] {
            *b = 0x20;
        }

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.serial_number_string, Some("SN123456".to_string()));
    }

    #[test]
    fn test_established_timings_iii() {
        let mut base = [0u8; 128];

        base[0x36..0x3B].copy_from_slice(&[0x00, 0x00, 0x00, 0xF7, 0x00]);
        base[0x3B] = 0x0A; // revision

        // Byte 6: set 1024x768@85 (bit 1) and 1152x864@75 (bit 0)
        base[0x3C] = 0x03;
        // Byte 7: set 1280x1024@60 (bit 1)
        base[0x3D] = 0x02;
        // Byte 9: set 1600x1200@60 (bit 2)
        base[0x3F] = 0x04;

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1024,
            height: 768,
            refresh_rate: 85,
            ..Default::default()
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1152,
            height: 864,
            refresh_rate: 75,
            ..Default::default()
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1280,
            height: 1024,
            refresh_rate: 60,
            ..Default::default()
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1600,
            height: 1200,
            refresh_rate: 60,
            ..Default::default()
        }));
        assert_eq!(caps.supported_modes.len(), 4);
    }

    #[test]
    fn test_additional_standard_timings() {
        let mut base = [0u8; 128];

        // 0xFA descriptor at 0x36 with two valid entries and four unused (0x01 0x01)
        base[0x36..0x3A].copy_from_slice(&[0x00, 0x00, 0x00, 0xFA]);
        base[0x3A] = 0x00; // reserved
                           // 1920x1080@60: b1 = 1920/8 - 31 = 209, b2 = 16:9 (3<<6) | 0
        base[0x3B] = 209;
        base[0x3C] = 0xC0;
        // 1280x720@60: b1 = 1280/8 - 31 = 129, b2 = 16:9 (3<<6) | 0
        base[0x3D] = 129;
        base[0x3E] = 0xC0;
        // remaining 4 entries unused
        for i in 0..4 {
            base[0x3F + (i * 2)] = 0x01;
            base[0x40 + (i * 2)] = 0x01;
        }

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            ..Default::default()
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1280,
            height: 720,
            refresh_rate: 60,
            ..Default::default()
        }));
    }

    #[test]
    fn test_range_limits_offset_flags() {
        use crate::model::timing::TimingFormula;

        let mut base = [0u8; 128];
        // 0xFD descriptor at 0x36
        // Byte 4 = 0x0B = 0b00001011: max_v+255, min_v+255, max_h+255, min_h not offset
        base[0x36..0x3B].copy_from_slice(&[0x00, 0x00, 0x00, 0xFD, 0x0B]);
        base[0x3B] = 10; // min_v stored = 10 → 10 + 255 = 265 Hz
        base[0x3C] = 20; // max_v stored = 20 → 20 + 255 = 275 Hz
        base[0x3D] = 30; // min_h stored = 30 → no offset = 30 kHz
        base[0x3E] = 40; // max_h stored = 40 → 40 + 255 = 295 kHz
        base[0x3F] = 60; // max pixel clock = 600 MHz
        base[0x40] = 0x01; // byte 10: RangeLimitsOnly

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert_eq!(caps.min_v_rate, Some(265));
        assert_eq!(caps.max_v_rate, Some(275));
        assert_eq!(caps.min_h_rate_khz, Some(30));
        assert_eq!(caps.max_h_rate_khz, Some(295));
        assert_eq!(caps.max_pixel_clock_mhz, Some(600));
        assert_eq!(caps.timing_formula, Some(TimingFormula::RangeLimitsOnly));
    }

    #[test]
    fn test_range_limits_secondary_gtf() {
        use crate::model::timing::{GtfSecondaryParams, TimingFormula};

        let mut base = [0u8; 128];
        base[0x36..0x3B].copy_from_slice(&[0x00, 0x00, 0x00, 0xFD, 0x00]);
        base[0x3B] = 48; // VMin
        base[0x3C] = 120; // VMax
        base[0x3D] = 30; // HMin
        base[0x3E] = 230; // HMax
        base[0x3F] = 60; // max pixel clock = 600 MHz
        base[0x40] = 0x02; // byte 10: SecondaryGtf
        base[0x41] = 0x00; // byte 11: reserved
        base[0x42] = 55; // byte 12: start_freq = 55 * 2 = 110 kHz
        base[0x43] = 68; // byte 13: C = 68 / 2 = 34
        base[0x44] = 0x58; // byte 14: M LSB (600 = 0x0258)
        base[0x45] = 0x02; // byte 15: M MSB
        base[0x46] = 128; // byte 16: K = 128
        base[0x47] = 40; // byte 17: J = 40 / 2 = 20

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert_eq!(
            caps.timing_formula,
            Some(TimingFormula::SecondaryGtf(GtfSecondaryParams {
                start_freq_khz: 110,
                c: 34,
                m: 600,
                k: 128,
                j: 20,
            }))
        );
    }

    #[test]
    fn test_range_limits_cvt() {
        use crate::model::timing::{
            CvtAspectRatio, CvtAspectRatios, CvtScaling, CvtSupportParams, TimingFormula,
        };

        let mut base = [0u8; 128];
        base[0x36..0x3B].copy_from_slice(&[0x00, 0x00, 0x00, 0xFD, 0x00]);
        base[0x3B] = 48; // VMin
        base[0x3C] = 120; // VMax
        base[0x3D] = 30; // HMin
        base[0x3E] = 230; // HMax
        base[0x3F] = 60; // max pixel clock = 600 MHz
        base[0x40] = 0x04; // byte 10: CVT

        // byte 11: version 1.1
        base[0x41] = 0x11;
        // byte 12: pixel_clock_adjust = 4 (bits 7-2 = 0b000001_00 = 0x04),
        //          max_h_active_pixels MSBs = 0b01 → 1 × 256 = 256 contribution
        // bits 7-2 = 0b000001 → 0x04; bits 1-0 = 0b01 → byte value = 0x05
        base[0x42] = (4 << 2) | 0x01; // = 0x11
                                      // byte 13: max_h_active_pixels LSB = 0x80 = 128 → total = 128 + 256 = 384 → × 8 = 3072
        base[0x43] = 0x80;
        // byte 14: supported aspect ratios — 4:3, 16:9, 16:10
        base[0x44] = CvtAspectRatios::R4_3.bits()
            | CvtAspectRatios::R16_9.bits()
            | CvtAspectRatios::R16_10.bits();
        // byte 15: preferred = 16:9 (0b001 << 5 = 0x20), standard blanking (0x08), no reduced
        base[0x45] = 0x20 | 0x08;
        // byte 16: scaling — horizontal stretch + vertical shrink
        base[0x46] = CvtScaling::HORIZONTAL_STRETCH.bits() | CvtScaling::VERTICAL_SHRINK.bits();
        // byte 17: preferred vertical rate = 60 Hz
        base[0x47] = 60;

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert_eq!(
            caps.timing_formula,
            Some(TimingFormula::Cvt(CvtSupportParams {
                version: 0x11,
                pixel_clock_adjust: 4,
                max_h_active_pixels: Some(3072),
                supported_aspect_ratios: CvtAspectRatios::R4_3
                    | CvtAspectRatios::R16_9
                    | CvtAspectRatios::R16_10,
                preferred_aspect_ratio: Some(CvtAspectRatio::R16_9),
                standard_blanking: true,
                reduced_blanking: false,
                scaling: CvtScaling::HORIZONTAL_STRETCH | CvtScaling::VERTICAL_SHRINK,
                preferred_v_rate: Some(60),
            }))
        );
    }

    #[test]
    fn test_color_management_data() {
        use crate::model::color::{ColorManagementData, DcmChannel};

        let mut base = [0u8; 128];
        base[0x36..0x3C].copy_from_slice(&[0x00, 0x00, 0x00, 0xF9, 0x00, 0x03]);
        // Red:   a3 = 0x1234, a2 = 0x5678
        base[0x3C] = 0x34;
        base[0x3D] = 0x12; // red a3 LSB, MSB
        base[0x3E] = 0x78;
        base[0x3F] = 0x56; // red a2 LSB, MSB
                           // Green: a3 = 0xABCD, a2 = 0xEF01
        base[0x40] = 0xCD;
        base[0x41] = 0xAB; // green a3
        base[0x42] = 0x01;
        base[0x43] = 0xEF; // green a2
                           // Blue:  a3 = 0x0200, a2 = 0x0400
        base[0x44] = 0x00;
        base[0x45] = 0x02; // blue a3
        base[0x46] = 0x00;
        base[0x47] = 0x04; // blue a2

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert_eq!(
            caps.color_management,
            Some(ColorManagementData {
                red: DcmChannel {
                    a3: 0x1234,
                    a2: 0x5678
                },
                green: DcmChannel {
                    a3: 0xABCD,
                    a2: 0xEF01
                },
                blue: DcmChannel {
                    a3: 0x0200,
                    a2: 0x0400
                },
            })
        );
    }

    #[test]
    fn test_color_management_wrong_version_ignored() {
        let mut base = [0u8; 128];
        // Version byte = 0x02 (reserved) — should not decode
        base[0x36..0x3C].copy_from_slice(&[0x00, 0x00, 0x00, 0xF9, 0x00, 0x02]);
        base[0x3C] = 0xFF;
        base[0x3D] = 0xFF;

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert_eq!(caps.color_management, None);
    }

    #[test]
    fn test_cvt_3_byte_code_descriptor() {
        let mut base = [0u8; 128];

        // 0xF8 descriptor at 0x36, version byte 0x01
        base[0x36..0x3B].copy_from_slice(&[0x00, 0x00, 0x00, 0xF8, 0x00]);
        base[0x3B] = 0x01; // version

        // Entry 0: 1920x1080 @ 60 Hz preferred, 16:9
        // lines_raw = 1080/2 - 1 = 539 = 0x21B
        // b0 = 0x1B, b1 = (0x200>>4) | AR(16:9=0b01<<2) = 0x20|0x04 = 0x24
        // b2: preferred-60Hz bit = 0x08
        base[0x3C] = 0x1B;
        base[0x3D] = 0x24;
        base[0x3E] = 0x08;

        // Entry 1: 1280x720 @ 50 Hz, 16:9
        // lines_raw = 720/2 - 1 = 359 = 0x167
        // b0 = 0x67, b1 = (0x100>>4) | 0x04 = 0x10|0x04 = 0x14
        // b2: 50 Hz bit = 0x10
        base[0x3F] = 0x67;
        base[0x40] = 0x14;
        base[0x41] = 0x10;

        // Entries 2-3: unused (b0 = 0)

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            ..Default::default()
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1280,
            height: 720,
            refresh_rate: 50,
            ..Default::default()
        }));
        // 60 Hz RB (0x01 bit) deduplicates against preferred 60 Hz
        assert_eq!(
            caps.supported_modes
                .iter()
                .filter(|m| m.width == 1920 && m.height == 1080 && m.refresh_rate == 60)
                .count(),
            1
        );
    }
}
