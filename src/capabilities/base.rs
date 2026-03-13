use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::capabilities::VideoMode;
use crate::model::color::{
    AnalogColorType, Chromaticity, ChromaticityPoint, ColorBitDepth, DigitalColorEncoding,
    DisplayGamma, WhitePoint,
};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;
use crate::model::edid::EdidVersion;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionHandler;
use crate::model::features::DisplayFeatureFlags;
use crate::model::input::{AnalogSyncLevel, VideoInputFlags, VideoInterface};
use crate::model::manufacture::ManufactureDate;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::{String, Vec};
use crate::model::screen::ScreenSize;

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
        _warnings: &mut Vec<EdidWarning>,
    ) {
        decode_header_fields(base, caps);
        decode_descriptors(base, caps);
        decode_established_timings(base, caps);
        decode_standard_timings(base, caps);
        decode_detailed_timings(base, caps);
    }
}

/// Decodes fixed-position header fields: manufacturer, dates, version, product code,
/// serial number, video input definition, and physical dimensions.
#[cfg(any(feature = "alloc", feature = "std"))]
fn decode_header_fields(base: &[u8; 128], caps: &mut DisplayCapabilities) {
    // Manufacturer ID (offsets 0x08-0x09)
    // 2 bytes, 3 characters, 5 bits per character (00001=A, ..., 11010=Z)
    let id_raw = ((base[0x08] as u16) << 8) | (base[0x09] as u16);
    let char1 = ((id_raw >> 10) & 0x1F) as u8;
    let char2 = ((id_raw >> 5) & 0x1F) as u8;
    let char3 = (id_raw & 0x1F) as u8;
    if char1 > 0 && char2 > 0 && char3 > 0 {
        let mut mfg = String::new();
        mfg.push((char1 + b'A' - 1) as char);
        mfg.push((char2 + b'A' - 1) as char);
        mfg.push((char3 + b'A' - 1) as char);
        caps.manufacturer = Some(mfg);
    }

    // Product code (offsets 0x0A-0x0B, little-endian)
    let product_code = ((base[0x0B] as u16) << 8) | (base[0x0A] as u16);
    if product_code != 0 {
        caps.product_code = Some(product_code);
    }

    // Serial number (offsets 0x0C-0x0F, little-endian)
    let serial = ((base[0x0F] as u32) << 24)
        | ((base[0x0E] as u32) << 16)
        | ((base[0x0D] as u32) << 8)
        | (base[0x0C] as u32);
    if serial != 0 {
        caps.serial_number = Some(serial);
    }

    // Manufacture date (bytes 16-17)
    caps.manufacture_date = Some(ManufactureDate::from_edid_bytes(base[16], base[17]));

    // EDID version and revision (bytes 18-19)
    caps.edid_version = Some(EdidVersion {
        version: base[18],
        revision: base[19],
    });

    // Chromaticity coordinates (bytes 0x19-0x22)
    caps.chromaticity = Chromaticity::from_edid_bytes(base);

    // Display gamma (byte 0x17); 0xFF means undefined
    caps.gamma = DisplayGamma::from_edid_byte(base[0x17]);

    // Display feature support (byte 0x18)
    caps.display_features = Some(DisplayFeatureFlags::from_bits_truncate(base[0x18]));

    // Color type / encoding (byte 0x18 bits 4–3); meaning differs by input type and EDID version.
    // Digital encoding is only defined for EDID 1.4+; analog color type applies to any version.
    let is_digital = base[0x14] & 0x80 != 0;
    let edid_revision = base[19];
    if is_digital && edid_revision >= 4 {
        caps.digital_color_encoding = Some(DigitalColorEncoding::from_edid_bits(base[0x18]));
    } else if !is_digital {
        caps.analog_color_type = AnalogColorType::from_edid_bits(base[0x18]);
    }

    // Video input definition (byte 0x14)
    let video_input = VideoInputFlags::from_bits_truncate(base[0x14]);
    caps.digital = video_input.contains(VideoInputFlags::DIGITAL);
    if caps.digital {
        caps.color_bit_depth = ColorBitDepth::from_edid_bits(base[0x14] >> 4);
        caps.video_interface = VideoInterface::from_edid_bits(base[0x14]);
    } else {
        caps.analog_sync_level = Some(AnalogSyncLevel::from_edid_bits(base[0x14]));
    }

    // Screen size or aspect ratio (bytes 0x15-0x16)
    caps.screen_size = ScreenSize::from_edid_bytes(base[0x15], base[0x16]);
}

/// Decodes the four 18-byte monitor descriptor slots (offsets 0x36, 0x48, 0x5A, 0x6C).
/// Handles monitor name (0xFC) and range limits (0xFD) descriptor types.
#[cfg(any(feature = "alloc", feature = "std"))]
fn decode_descriptors(base: &[u8; 128], caps: &mut DisplayCapabilities) {
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
                if let Some(mode) =
                    decode_standard_timing_entry(descriptor[base_off], descriptor[base_off + 1])
                {
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
        if descriptor[0..4] == [0x00, 0x00, 0x00, 0xFD] {
            caps.min_v_rate = Some(descriptor[5]);
            caps.max_v_rate = Some(descriptor[6]);
            caps.min_h_rate_khz = Some(descriptor[7]);
            caps.max_h_rate_khz = Some(descriptor[8]);
            caps.max_pixel_clock_mhz = Some((descriptor[9] as u16) * 10);
        }
    }
}

/// Decodes the established timings bitmap (bytes 0x23–0x25, 17 predefined modes).
///
/// Each bit maps to a fixed resolution and refresh rate. Byte 0x25 bits 6–0 are
/// manufacturer-specific and are not decoded here.
#[cfg(any(feature = "alloc", feature = "std"))]
fn decode_established_timings(base: &[u8; 128], caps: &mut DisplayCapabilities) {
    // (byte offset, bit mask, width, height, refresh_rate)
    // Note: 1024x768@87 is interlaced in the EDID spec; stored as-is since VideoMode
    // has no interlace field.
    const TIMINGS: &[(usize, u8, u16, u16, u8)] = &[
        (0x23, 0x80, 720, 400, 70),
        (0x23, 0x40, 720, 400, 88),
        (0x23, 0x20, 640, 480, 60),
        (0x23, 0x10, 640, 480, 67),
        (0x23, 0x08, 640, 480, 72),
        (0x23, 0x04, 640, 480, 75),
        (0x23, 0x02, 800, 600, 56),
        (0x23, 0x01, 800, 600, 60),
        (0x24, 0x80, 800, 600, 72),
        (0x24, 0x40, 800, 600, 75),
        (0x24, 0x20, 832, 624, 75),
        (0x24, 0x10, 1024, 768, 87),
        (0x24, 0x08, 1024, 768, 60),
        (0x24, 0x04, 1024, 768, 70),
        (0x24, 0x02, 1024, 768, 75),
        (0x24, 0x01, 1280, 1024, 75),
        (0x25, 0x80, 1152, 870, 75), // Apple Macintosh II
    ];

    for &(byte_off, mask, w, h, rate) in TIMINGS {
        if base[byte_off] & mask != 0 {
            let mode = VideoMode {
                width: w,
                height: h,
                refresh_rate: rate,
            };
            if !caps.supported_modes.contains(&mode) {
                caps.supported_modes.push(mode);
            }
        }
    }
}

/// Decodes a single 2-byte standard timing entry into a [`VideoMode`].
///
/// Returns `None` for unused entries (`0x01 0x01` or a zero first byte).
#[cfg(any(feature = "alloc", feature = "std"))]
fn decode_standard_timing_entry(b1: u8, b2: u8) -> Option<VideoMode> {
    if (b1 == 0x01 && b2 == 0x01) || b1 == 0x00 {
        return None;
    }
    let w = (b1 as u16 + 31) * 8;
    let h = match (b2 >> 6) & 0x03 {
        0x00 => (w * 10) / 16, // 16:10
        0x01 => (w * 3) / 4,   // 4:3
        0x02 => (w * 4) / 5,   // 5:4
        _ => (w * 9) / 16,     // 16:9
    };
    Some(VideoMode {
        width: w,
        height: h,
        refresh_rate: (b2 & 0x3F) + 60,
    })
}

/// Decodes the eight standard timing descriptors (offsets 0x26–0x35, 2 bytes each).
#[cfg(any(feature = "alloc", feature = "std"))]
fn decode_standard_timings(base: &[u8; 128], caps: &mut DisplayCapabilities) {
    for i in 0..8 {
        let offset = 0x26 + (i * 2);
        if let Some(mode) = decode_standard_timing_entry(base[offset], base[offset + 1]) {
            caps.supported_modes.push(mode);
        }
    }
}

/// Decodes the four detailed timing descriptor (DTD) slots (offsets 0x36, 0x48, 0x5A, 0x6C).
/// Slots with a zero pixel clock are monitor descriptors and are skipped here.
#[cfg(any(feature = "alloc", feature = "std"))]
fn decode_detailed_timings(base: &[u8; 128], caps: &mut DisplayCapabilities) {
    for i in 0..4 {
        let offset = 0x36 + (i * 18);
        let dtd = &base[offset..offset + 18];

        if dtd[0] == 0x00 && dtd[1] == 0x00 {
            continue;
        }

        let pixel_clock = ((dtd[1] as u32) << 8) | (dtd[0] as u32);
        if pixel_clock == 0 {
            continue;
        }

        let hactive = (((dtd[4] as u16) & 0xF0) << 4) | (dtd[2] as u16);
        let hblank = (((dtd[4] as u16) & 0x0F) << 8) | (dtd[3] as u16);
        let vactive = (((dtd[7] as u16) & 0xF0) << 4) | (dtd[5] as u16);
        let vblank = (((dtd[7] as u16) & 0x0F) << 8) | (dtd[6] as u16);

        if hactive == 0 || vactive == 0 || hblank == 0 || vblank == 0 {
            continue;
        }

        let total_pixels = (hactive + hblank) as u32 * (vactive + vblank) as u32;
        if total_pixels == 0 {
            continue;
        }

        let rate = (pixel_clock * 10_000) / total_pixels;
        let Some(refresh_rate) = u8::try_from(rate).ok() else {
            continue;
        };

        let mode = VideoMode {
            width: hactive,
            height: vactive,
            refresh_rate,
        };
        if !caps.supported_modes.contains(&mode) {
            caps.supported_modes.push(mode);
        }
    }
}

#[cfg(test)]
#[cfg(any(feature = "alloc", feature = "std"))]
mod tests {
    use super::*;
    use crate::model::capabilities::{DisplayCapabilities, VideoMode};
    use crate::model::color::{
        AnalogColorType, Chromaticity, ChromaticityPoint, ColorBitDepth, DigitalColorEncoding,
        DisplayGamma,
    };
    use crate::model::edid::EdidVersion;
    use crate::model::features::DisplayFeatureFlags;
    use crate::model::input::{AnalogSyncLevel, VideoInterface};
    use crate::model::manufacture::ManufactureDate;
    use crate::model::screen::ScreenSize;

    #[test]
    fn test_gamma() {
        let mut base = [0u8; 128];

        base[0x17] = 120; // (120 + 100) / 100 = 2.20
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.gamma, Some(DisplayGamma::from_edid_byte(120).unwrap()));
        assert!((caps.gamma.unwrap().value() - 2.20).abs() < 0.001);

        base[0x17] = 0xFF; // undefined
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.gamma, None);
    }

    #[test]
    fn test_edid_version() {
        let mut base = [0u8; 128];
        base[18] = 1;
        base[19] = 4;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(
            caps.edid_version,
            Some(EdidVersion {
                version: 1,
                revision: 4
            })
        );
    }

    #[test]
    fn test_manufacture_date() {
        let mut base = [0u8; 128];

        // Week + year
        base[16] = 12;
        base[17] = 30; // 1990 + 30 = 2020
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(
            caps.manufacture_date,
            Some(ManufactureDate::Manufactured {
                week: Some(12),
                year: 2020
            })
        );

        // Week unspecified
        base[16] = 0x00;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(
            caps.manufacture_date,
            Some(ManufactureDate::Manufactured {
                week: None,
                year: 2020
            })
        );

        // Model year
        base[16] = 0xFF;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(
            caps.manufacture_date,
            Some(ManufactureDate::ModelYear(2020))
        );
    }

    #[test]
    fn test_screen_size() {
        let mut base = [0u8; 128];

        // Physical dimensions
        base[0x15] = 60;
        base[0x16] = 34;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(
            caps.screen_size,
            Some(ScreenSize::Physical {
                width_cm: 60,
                height_cm: 34
            })
        );

        // Landscape aspect ratio: byte 0x16 = 0, byte 0x15 = 196 → (196+99)/100 = 2.95
        base[0x15] = 196;
        base[0x16] = 0;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.screen_size, Some(ScreenSize::Landscape(196)));
        let ratio = caps.screen_size.unwrap().landscape_ratio().unwrap();
        assert!((ratio - 2.95).abs() < 0.001);

        // Portrait aspect ratio: byte 0x15 = 0, byte 0x16 = 101 → 100/(101+99) = 0.5
        base[0x15] = 0;
        base[0x16] = 101;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.screen_size, Some(ScreenSize::Portrait(101)));
        let ratio = caps.screen_size.unwrap().portrait_ratio().unwrap();
        assert!((ratio - 0.5).abs() < 0.001);

        // Both zero → undefined
        base[0x15] = 0;
        base[0x16] = 0;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.screen_size, None);
    }

    #[test]
    fn test_analog_sync_level() {
        let mut base = [0u8; 128];

        // Analog (bit 7 = 0), bits 6-5 = 0b00 → V700_300
        base[0x14] = 0x00;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.analog_sync_level, Some(AnalogSyncLevel::V700_300));
        assert_eq!(caps.color_bit_depth, None);
        assert_eq!(caps.video_interface, None);

        // Analog, bits 6-5 = 0b01 → V714_286
        base[0x14] = 0x20;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.analog_sync_level, Some(AnalogSyncLevel::V714_286));

        // Analog, bits 6-5 = 0b10 → V1000_400
        base[0x14] = 0x40;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.analog_sync_level, Some(AnalogSyncLevel::V1000_400));

        // Analog, bits 6-5 = 0b11 → V700_0
        base[0x14] = 0x60;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.analog_sync_level, Some(AnalogSyncLevel::V700_0));

        // Digital — sync level not populated
        base[0x14] = 0x80;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.analog_sync_level, None);
    }

    #[test]
    fn test_color_bit_depth_and_video_interface() {
        let mut base = [0u8; 128];

        // Digital, 8 bpc (bits 6-4 = 0b010), DisplayPort (bits 3-0 = 0x5)
        base[0x14] = 0x80 | 0x25;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert!(caps.digital);
        assert_eq!(caps.color_bit_depth, Some(ColorBitDepth::Depth8));
        assert_eq!(caps.video_interface, Some(VideoInterface::DisplayPort));

        // Digital, undefined depth and interface
        base[0x14] = 0x80;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.color_bit_depth, None);
        assert_eq!(caps.video_interface, None);

        // Analog — neither field populated
        base[0x14] = 0x00;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert!(!caps.digital);
        assert_eq!(caps.color_bit_depth, None);
        assert_eq!(caps.video_interface, None);
    }

    #[test]
    fn test_chromaticity() {
        let mut base = [0u8; 128];

        // Encode R=(0.640, 0.330), G=(0.300, 0.600), B=(0.150, 0.060), W=(0.3127, 0.3290)
        // as 10-bit raw values: multiply by 1024 and round
        // R: x=655 (0x28F), y=338 (0x152)
        // G: x=307 (0x133), y=614 (0x266)
        // B: x=154 (0x09A), y=61  (0x03D)
        // W: x=320 (0x140), y=337 (0x151)
        base[0x1B] = (655u16 >> 2) as u8; // R x MSB
        base[0x1C] = (338u16 >> 2) as u8; // R y MSB
        base[0x1D] = (307u16 >> 2) as u8; // G x MSB
        base[0x1E] = (614u16 >> 2) as u8; // G y MSB
        base[0x1F] = (154u16 >> 2) as u8; // B x MSB
        base[0x20] = (61u16 >> 2) as u8; // B y MSB
        base[0x21] = (320u16 >> 2) as u8; // W x MSB
        base[0x22] = (337u16 >> 2) as u8; // W y MSB
                                          // LSB byte 0x19: Rx[1:0] | Ry[1:0] | Gx[1:0] | Gy[1:0]
        base[0x19] =
            (((655u16 & 3) << 6) | ((338u16 & 3) << 4) | ((307u16 & 3) << 2) | (614u16 & 3)) as u8;
        // LSB byte 0x1A: Bx[1:0] | By[1:0] | Wx[1:0] | Wy[1:0]
        base[0x1A] =
            (((154u16 & 3) << 6) | ((61u16 & 3) << 4) | ((320u16 & 3) << 2) | (337u16 & 3)) as u8;

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert_eq!(caps.chromaticity, Chromaticity::from_edid_bytes(&base));
        assert_eq!(caps.chromaticity.red.x_raw, 655);
        assert_eq!(caps.chromaticity.red.y_raw, 338);
        assert_eq!(caps.chromaticity.green.x_raw, 307);
        assert_eq!(caps.chromaticity.white.x_raw, 320);
        assert!((caps.chromaticity.red.x() - 0.640).abs() < 0.002);
        assert!((caps.chromaticity.white.x() - 0.3125).abs() < 0.002);
    }

    #[test]
    fn test_color_type() {
        let mut base = [0u8; 128];

        // Digital, EDID 1.4 (revision = 4), bits 4-3 = 0b11 → Rgb444YCbCr444YCbCr422
        base[0x14] = 0x80; // digital
        base[19] = 4; // revision 4
        base[0x18] = 0x18; // bits 4-3 = 0b11
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(
            caps.digital_color_encoding,
            Some(DigitalColorEncoding::Rgb444YCbCr444YCbCr422)
        );
        assert_eq!(caps.analog_color_type, None);

        // Digital, EDID 1.3 — encoding field not decoded
        base[19] = 3;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.digital_color_encoding, None);
        assert_eq!(caps.analog_color_type, None);

        // Analog, bits 4-3 = 0b01 → Rgb
        base[0x14] = 0x00; // analog
        base[0x18] = 0x08; // bits 4-3 = 0b01
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.digital_color_encoding, None);
        assert_eq!(caps.analog_color_type, Some(AnalogColorType::Rgb));

        // Analog, bits 4-3 = 0b11 → undefined (None)
        base[0x18] = 0x18;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        assert_eq!(caps.analog_color_type, None);
    }

    #[test]
    fn test_display_features() {
        let mut base = [0u8; 128];

        // DPMS standby + suspend + active-off + preferred timing = 0xE2
        base[0x18] = 0xE2;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        let flags = caps.display_features.unwrap();
        assert!(flags.contains(DisplayFeatureFlags::DPMS_STANDBY));
        assert!(flags.contains(DisplayFeatureFlags::DPMS_SUSPEND));
        assert!(flags.contains(DisplayFeatureFlags::DPMS_ACTIVE_OFF));
        assert!(flags.contains(DisplayFeatureFlags::PREFERRED_TIMING));
        assert!(!flags.contains(DisplayFeatureFlags::SRGB));
        assert!(!flags.contains(DisplayFeatureFlags::CONTINUOUS_TIMINGS));

        // sRGB + preferred timing + continuous timings = 0x07
        base[0x18] = 0x07;
        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());
        let flags = caps.display_features.unwrap();
        assert!(flags.contains(DisplayFeatureFlags::SRGB));
        assert!(flags.contains(DisplayFeatureFlags::PREFERRED_TIMING));
        assert!(flags.contains(DisplayFeatureFlags::CONTINUOUS_TIMINGS));
    }

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
    fn test_identification() {
        let mut base = [0u8; 128];

        // Manufacturer "SAM": S=19 (10011), A=1 (00001), M=13 (01101)
        // 0 10011 00001 01101 => 0x4C 0x2D
        base[0x08] = 0x4C;
        base[0x09] = 0x2D;

        // Product Code: 0x1234 (little-endian)
        base[0x0A] = 0x34;
        base[0x0B] = 0x12;

        // Serial Number: 0x12345678 (little-endian)
        base[0x0C] = 0x78;
        base[0x0D] = 0x56;
        base[0x0E] = 0x34;
        base[0x0F] = 0x12;

        // Video Input: Digital
        base[0x14] = 0x80;

        // Physical Dimensions: 51cm x 29cm
        base[0x15] = 51;
        base[0x16] = 29;

        // Monitor Name Descriptor at 0x36: "PIAF"
        base[0x36..0x3B].copy_from_slice(&[0x00, 0x00, 0x00, 0xFC, 0x00]);
        base[0x3B..0x3F].copy_from_slice(b"PIAF");
        base[0x3F] = 0x0A;
        for i in 0x40..0x48 {
            base[i] = 0x20;
        }

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert_eq!(caps.manufacturer, Some("SAM".to_string()));
        assert_eq!(caps.product_code, Some(0x1234));
        assert_eq!(caps.serial_number, Some(0x12345678));
        assert!(caps.digital);
        assert_eq!(
            caps.screen_size,
            Some(ScreenSize::Physical {
                width_cm: 51,
                height_cm: 29
            })
        );
        assert_eq!(caps.display_name, Some("PIAF".to_string()));
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
            refresh_rate: 85
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1152,
            height: 864,
            refresh_rate: 75
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1280,
            height: 1024,
            refresh_rate: 60
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1600,
            height: 1200,
            refresh_rate: 60
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
            refresh_rate: 60
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1280,
            height: 720,
            refresh_rate: 60
        }));
    }

    #[test]
    fn test_established_timings() {
        let mut base = [0u8; 128];

        // Set 640x480@60, 800x600@60, 1024x768@60, 1280x1024@75
        base[0x23] = 0x20; // 640x480@60
        base[0x23] |= 0x01; // 800x600@60
        base[0x24] = 0x08; // 1024x768@60
        base[0x24] |= 0x01; // 1280x1024@75

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert_eq!(caps.supported_modes.len(), 4);
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 640,
            height: 480,
            refresh_rate: 60
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 800,
            height: 600,
            refresh_rate: 60
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1024,
            height: 768,
            refresh_rate: 60
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1280,
            height: 1024,
            refresh_rate: 75
        }));
    }

    #[test]
    fn test_standard_timings() {
        let mut base = [0u8; 128];

        // 1920x1080 @ 60Hz: width byte = 1920/8 - 31 = 209, flags = 16:9 (3<<6) | 0Hz offset
        base[0x26] = 209;
        base[0x27] = 0xC0;

        // 1280x1024 @ 75Hz: width byte = 1280/8 - 31 = 129, flags = 5:4 (2<<6) | 15Hz offset
        base[0x28] = 129;
        base[0x29] = 0x8F;

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert_eq!(caps.supported_modes.len(), 2);
        assert_eq!(caps.supported_modes[0].width, 1920);
        assert_eq!(caps.supported_modes[0].height, 1080);
        assert_eq!(caps.supported_modes[0].refresh_rate, 60);
        assert_eq!(caps.supported_modes[1].width, 1280);
        assert_eq!(caps.supported_modes[1].height, 1024);
        assert_eq!(caps.supported_modes[1].refresh_rate, 75);
    }

    #[test]
    fn test_detailed_timing_and_range_limits() {
        let mut base = [0u8; 128];

        // DTD at 0x36: 1920x1080 @ 60Hz
        // Pixel clock: 14850 (units of 10kHz = 148.50 MHz)
        base[0x36] = 0x02;
        base[0x37] = 0x3A;
        // HActive=1920 (0x780), HBlank=280 (0x118): high nibbles packed into byte 4
        base[0x38] = 0x80; // HActive LSB
        base[0x39] = 0x18; // HBlank LSB
        base[0x3A] = 0x71; // HActive high (0x7) | HBlank high (0x1)
                           // VActive=1080 (0x438), VBlank=45 (0x02D): high nibbles packed into byte 7
        base[0x3B] = 0x38; // VActive LSB
        base[0x3C] = 0x2D; // VBlank LSB
        base[0x3D] = 0x40; // VActive high (0x4) | VBlank high (0x0)

        // Monitor Range Limits at 0x48
        base[0x48..0x4D].copy_from_slice(&[0x00, 0x00, 0x00, 0xFD, 0x00]);
        base[0x4D] = 48; // VMin
        base[0x4E] = 75; // VMax
        base[0x4F] = 30; // HMin (kHz)
        base[0x50] = 83; // HMax (kHz)
        base[0x51] = 17; // Max pixel clock (170 MHz)

        let mut caps = DisplayCapabilities::default();
        BaseBlockHandler.process(&base, &mut caps, &mut Vec::new());

        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 1920);
        assert_eq!(caps.supported_modes[0].height, 1080);
        assert_eq!(caps.supported_modes[0].refresh_rate, 60);
        assert_eq!(caps.min_v_rate, Some(48));
        assert_eq!(caps.max_v_rate, Some(75));
        assert_eq!(caps.min_h_rate_khz, Some(30));
        assert_eq!(caps.max_h_rate_khz, Some(83));
        assert_eq!(caps.max_pixel_clock_mhz, Some(170));
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
            refresh_rate: 60
        }));
        assert!(caps.supported_modes.contains(&VideoMode {
            width: 1280,
            height: 720,
            refresh_rate: 50
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
