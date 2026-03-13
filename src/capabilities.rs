use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::capabilities::VideoMode;
use crate::model::ParsedEdid;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::prelude::String;

pub fn capabilities_from_edid(edid: &ParsedEdid) -> DisplayCapabilities {
    let mut caps = DisplayCapabilities::default();
    let base = &edid.base_block;

    // 1. Manufacturer ID (offsets 0x08-0x09)
    // 2 bytes, 3 characters, 5 bits per character (00001=A, ..., 11010=Z)
    #[cfg(any(feature = "alloc", feature = "std"))]
    {
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
    }

    // 2. Product Code (offsets 0x0A-0x0B, little-endian)
    let product_code = ((base[0x0B] as u16) << 8) | (base[0x0A] as u16);
    if product_code != 0 {
        caps.product_code = Some(product_code);
    }

    // 3. Serial Number (offsets 0x0C-0x0F, little-endian)
    let serial = ((base[0x0F] as u32) << 24)
        | ((base[0x0E] as u32) << 16)
        | ((base[0x0D] as u32) << 8)
        | (base[0x0C] as u32);
    if serial != 0 {
        caps.serial_number = Some(serial);
    }

    // 4. Video Input Definition (offset 0x14)
    // Bit 7: 1=Digital, 0=Analog
    caps.digital = (base[0x14] & 0x80) != 0;

    // 5. Physical Dimensions (offsets 0x15-0x16, width and height in cm)
    // 0, 0 means undefined
    let width = base[0x15] as u16;
    let height = base[0x16] as u16;
    if width > 0 && height > 0 {
        caps.width_cm = Some(width);
        caps.height_cm = Some(height);
    }

    // 6. 18-byte Descriptors (offsets 0x36, 0x48, 0x5A, 0x6C)
    // We'll look for the Display Name descriptor (Tag 0xFC)
    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        for i in 0..4 {
            let offset = 0x36 + (i * 18);
            let descriptor = &base[offset..offset + 18];

            // Monitor Name Descriptor: Header 00 00 00 FC 00
            if descriptor[0..4] == [0x00, 0x00, 0x00, 0xFC] {
                let name_bytes = &descriptor[5..18];
                // Strip trailing newline (0x0A) or padding (0x20)
                let name = String::from_utf8_lossy(name_bytes);
                let trimmed = name.trim().to_string();
                if !trimmed.is_empty() {
                    caps.display_name = Some(trimmed);
                }
            }

            // Monitor Range Limits: Header 00 00 00 FD 00
            if descriptor[0..4] == [0x00, 0x00, 0x00, 0xFD] {
                caps.min_v_rate = Some(descriptor[5]);
                caps.max_v_rate = Some(descriptor[6]);
                caps.min_h_rate_khz = Some(descriptor[7]);
                caps.max_h_rate_khz = Some(descriptor[8]);
                caps.max_pixel_clock_mhz = Some((descriptor[9] as u16) * 10);
            }
        }
    }

    // 7. Standard Timings (offsets 0x26-0x35, 8 descriptors, 2 bytes each)
    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        for i in 0..8 {
            let offset = 0x26 + (i * 2);
            let b1 = base[offset];
            let b2 = base[offset + 1];

            if b1 == 0x01 && b2 == 0x01 || b1 == 0x00 {
                continue; // Unused
            }

            let width = (b1 as u16 + 31) * 8;
            let ratio_bits = (b2 >> 6) & 0x03;
            let refresh_rate = (b2 & 0x3F) + 60;

            let height = match ratio_bits {
                0x00 => (width * 10) / 16, // 16:10
                0x01 => (width * 3) / 4,   // 4:3
                0x02 => (width * 4) / 5,   // 5:4
                0x03 => (width * 9) / 16,  // 16:9
                _ => unreachable!(),
            };

            caps.supported_modes.push(VideoMode {
                width,
                height,
                refresh_rate,
            });
        }
    }

    // 8. Detailed Timing Descriptors (DTD) (offsets 0x36, 0x48, 0x5A, 0x6C)
    // First one is mandatory, others can be Monitor Descriptors
    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        for i in 0..4 {
            let offset = 0x36 + (i * 18);
            let dtd = &base[offset..offset + 18];

            // If first two bytes are 0, it's NOT a DTD (it's a monitor descriptor)
            if dtd[0] == 0x00 && dtd[1] == 0x00 {
                continue;
            }

            // Simple DTD extraction (pixel clock != 0)
            let pixel_clock = ((dtd[1] as u32) << 8) | (dtd[0] as u32);
            if pixel_clock == 0 {
                continue;
            }

            let hactive = (((dtd[4] as u16) & 0xF0) << 4) | (dtd[2] as u16);
            let hblank = (((dtd[4] as u16) & 0x0F) << 8) | (dtd[3] as u16);
            let vactive = (((dtd[7] as u16) & 0xF0) << 4) | (dtd[5] as u16);
            let vblank = (((dtd[7] as u16) & 0x0F) << 8) | (dtd[6] as u16);

            // Refresh rate calculation: PixelClock / (HActive+HBlank * VActive+VBlank)
            // Pixel clock is in 10kHz units.
            let refresh_rate = if hactive > 0 && vactive > 0 && hblank > 0 && vblank > 0 {
                let total_pixels = (hactive + hblank) as u32 * (vactive + vblank) as u32;
                if total_pixels > 0 {
                    let rate = (pixel_clock * 10_000) / total_pixels;
                    rate as u8
                } else {
                    60
                }
            } else {
                60
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

    // 9. Process Extensions (CEA-861, etc.)
    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        for ext in &edid.extensions {
            let tag = ext[0];
            if tag == 0x02 {
                // CEA-861 Extension Block
                // Offset 2: Offset of DTDs
                // Bit 7 of byte 3: 1=Supports basic audio
                if (ext[3] & 0x40) != 0 {
                    caps.has_audio = true;
                }
                
                // We could also parse Video Data Blocks (VICs) here to get more modes
            }
        }
    }

    caps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::extension::ExtensionTagRegistry;
    use crate::parser::parse_edid;

    #[test]
    fn test_capabilities_identification() {
        let mut bytes = [0u8; 128];
        bytes[0..8].copy_from_slice(&crate::parser::EDID_HEADER);

        // Manufacturer "SAM"
        // S = 19 (10011), A = 1 (00001), M = 13 (01101)
        // 0 10011 00 001 01101 => 01001100 00101101 => 0x4C, 0x2D
        bytes[0x08] = 0x4C;
        bytes[0x09] = 0x2D;

        // Product Code: 0x1234
        bytes[0x0A] = 0x34;
        bytes[0x0B] = 0x12;

        // Serial Number: 0x12345678
        bytes[0x0C] = 0x78;
        bytes[0x0D] = 0x56;
        bytes[0x0E] = 0x34;
        bytes[0x0F] = 0x12;

        // Video Input: Digital (0x80)
        bytes[0x14] = 0x80;

        // Dimensions: 51cm x 29cm
        bytes[0x15] = 51;
        bytes[0x16] = 29;

        // Monitor Name descriptor at offset 0x36
        // 00 00 00 FC 00 'P' 'I' 'A' 'F' 0A 20 20 20 20 20 20 20 20
        bytes[0x36..0x3B].copy_from_slice(&[0x00, 0x00, 0x00, 0xFC, 0x00]);
        bytes[0x3B..0x3F].copy_from_slice(b"PIAF");
        bytes[0x3F] = 0x0A; // Newline
        for i in 0x40..0x48 {
            bytes[i] = 0x20;
        } // Padding

        // Calculate checksum for header + IDs
        let mut sum = 0u8;
        for i in 0..127 {
            sum = sum.wrapping_add(bytes[i]);
        }
        bytes[127] = 0u8.wrapping_sub(sum);

        let registry = ExtensionTagRegistry::new();
        let parsed = parse_edid(&bytes, &registry).unwrap();
        let caps = capabilities_from_edid(&parsed);

        #[cfg(any(feature = "alloc", feature = "std"))]
        assert_eq!(caps.manufacturer, Some("SAM".to_string()));
        assert_eq!(caps.product_code, Some(0x1234));
        assert_eq!(caps.serial_number, Some(0x12345678));
        assert!(caps.digital);
        assert_eq!(caps.width_cm, Some(51));
        assert_eq!(caps.height_cm, Some(29));
        #[cfg(any(feature = "alloc", feature = "std"))]
        assert_eq!(caps.display_name, Some("PIAF".to_string()));
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn test_standard_timings_decoding() {
        let mut bytes = [0u8; 128];
        bytes[0..8].copy_from_slice(&crate::parser::EDID_HEADER);

        // 1920x1080 @ 60Hz (Standard Timing)
        // Width: (239 + 31) * 8 = 2160? No.
        // For 1920: 1920/8 = 240. 240 - 31 = 209 (0xD1)
        // Ratio 16:9 is 11 (3). 60Hz is 0.
        // Byte 2: (3 << 6) | 0 = 0xC0
        bytes[0x26] = 209;
        bytes[0x26 + 1] = 0xC0;

        // 1280x1024 @ 75Hz
        // Width: 1280/8 = 160. 160 - 31 = 129 (0x81)
        // Ratio 5:4 is 10 (2). 75Hz is 15 (0x0F)
        // Byte 2: (2 << 6) | 15 = 0x80 | 0x0F = 0x8F
        bytes[0x28] = 129;
        bytes[0x28 + 1] = 0x8F;

        // Checksum
        let mut sum = 0u8;
        for i in 0..127 {
            sum = sum.wrapping_add(bytes[i]);
        }
        bytes[127] = 0u8.wrapping_sub(sum);

        let registry = ExtensionTagRegistry::new();
        let parsed = parse_edid(&bytes, &registry).unwrap();
        let caps = capabilities_from_edid(&parsed);

        assert_eq!(caps.supported_modes.len(), 2);

        // Mode 1: 1920x1080 @ 60Hz
        assert_eq!(caps.supported_modes[0].width, 1920);
        assert_eq!(caps.supported_modes[0].height, 1080);
        assert_eq!(caps.supported_modes[0].refresh_rate, 60);

        // Mode 2: 1280x1024 @ 75Hz
        assert_eq!(caps.supported_modes[1].width, 1280);
        assert_eq!(caps.supported_modes[1].height, 1024);
        assert_eq!(caps.supported_modes[1].refresh_rate, 75);
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn test_detailed_timing_decoding() {
        let mut bytes = [0u8; 128];
        bytes[0..8].copy_from_slice(&crate::parser::EDID_HEADER);

        // 1920x1080 @ 60Hz DTD (common example)
        // Pixel clock: 148.50 MHz = 14850 = 0x3A02
        bytes[0x36] = 0x02;
        bytes[0x36 + 1] = 0x3A;
        
        // HActive: 1920 = 0x780. 
        // LSB: 0x80. Bits 4-7 of offset 4: 0x7
        bytes[0x36 + 2] = 0x80;
        bytes[0x36 + 4] = 0x70;

        // HBlank: 280 = 0x118.
        // LSB: 0x18. Bits 0-3 of offset 4: 0x1
        bytes[0x36 + 3] = 0x18;
        bytes[0x36 + 4] |= 0x01;

        // VActive: 1080 = 0x438.
        // LSB: 0x38. Bits 4-7 of offset 7: 0x4
        bytes[0x36 + 5] = 0x38;
        bytes[0x36 + 7] = 0x40;

        // VBlank: 45 = 0x02D.
        // LSB: 0x2D. Bits 0-3 of offset 7: 0x0
        bytes[0x36 + 6] = 0x2D;
        bytes[0x36 + 7] |= 0x00;

        // Monitor Range Limits descriptor at offset 0x48
        // 00 00 00 FD 00 VMin VMax HMin HMax Clock
        bytes[0x48..0x4D].copy_from_slice(&[0x00, 0x00, 0x00, 0xFD, 0x00]);
        bytes[0x4D] = 48; // VMin
        bytes[0x4E] = 75; // VMax
        bytes[0x4F] = 30; // HMin
        bytes[0x50] = 83; // HMax
        bytes[0x51] = 17; // Max clock: 170MHz

        // Checksum
        let mut sum = 0u8;
        for i in 0..127 {
            sum = sum.wrapping_add(bytes[i]);
        }
        bytes[127] = 0u8.wrapping_sub(sum);

        let registry = ExtensionTagRegistry::new();
        let parsed = parse_edid(&bytes, &registry).unwrap();
        let caps = capabilities_from_edid(&parsed);

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
}
