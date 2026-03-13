mod base;
mod cea861;

#[cfg(any(feature = "alloc", feature = "std"))]
pub use base::BaseBlockHandler;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use cea861::Cea861Handler;

use crate::model::capabilities::DisplayCapabilities;
use crate::model::ParsedEdid;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::prelude::Box;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionLibrary;
#[cfg(not(any(feature = "alloc", feature = "std")))]
use crate::model::extension::ExtensionLibrary;

#[cfg(any(feature = "alloc", feature = "std"))]
impl ExtensionLibrary {
    pub fn with_standard_handlers() -> Self {
        let mut lib = Self::with_standard_extensions();
        lib.base_handler = Some(Box::new(BaseBlockHandler));
        if let Some(cea) = lib.extensions.iter_mut().find(|e| e.tag == 0x02) {
            cea.handler = Some(Box::new(Cea861Handler));
        }
        lib
    }
}

pub fn capabilities_from_edid(edid: &ParsedEdid, library: &ExtensionLibrary) -> DisplayCapabilities {
    #[cfg(any(feature = "alloc", feature = "std"))]
    let mut caps = DisplayCapabilities::default();
    #[cfg(not(any(feature = "alloc", feature = "std")))]
    let caps = DisplayCapabilities::default();

    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        // 1. Process Base Block via Base Handler (if present)
        if let Some(handler) = &library.base_handler {
            handler.process(&edid.base_block, &mut caps);
        }

        // 2. Process Extension Blocks via registered handlers
        for ext in &edid.extensions {
            let tag = ext[0];
            if let Some(metadata) = library.extensions.iter().find(|e| e.tag == tag) {
                if let Some(handler) = &metadata.handler {
                    handler.process(ext, &mut caps);
                }
            }
        }
    }

    #[cfg(not(any(feature = "alloc", feature = "std")))]
    {
        let _ = edid;
        let _ = library;
    }

    caps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::extension::ExtensionTagRegistry;
    use crate::parser::parse_edid;

    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
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
        let library = ExtensionLibrary::with_standard_handlers();
        let caps = capabilities_from_edid(&parsed, &library);

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
        let library = ExtensionLibrary::with_standard_handlers();
        let caps = capabilities_from_edid(&parsed, &library);

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
        let library = ExtensionLibrary::with_standard_handlers();
        let caps = capabilities_from_edid(&parsed, &library);

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
