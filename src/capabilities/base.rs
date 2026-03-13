use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::capabilities::VideoMode;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionHandler;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::prelude::String;

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug)]
pub struct BaseBlockHandler;

#[cfg(any(feature = "alloc", feature = "std"))]
impl ExtensionHandler for BaseBlockHandler {
    fn process(&self, base: &[u8; 128], caps: &mut DisplayCapabilities) {
        // 1. Manufacturer ID (offsets 0x08-0x09)
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
        let width = base[0x15] as u16;
        let height = base[0x16] as u16;
        if width > 0 && height > 0 {
            caps.width_cm = Some(width);
            caps.height_cm = Some(height);
        }

        // 6. 18-byte Descriptors (offsets 0x36, 0x48, 0x5A, 0x6C)
        for i in 0..4 {
            let offset = 0x36 + (i * 18);
            let descriptor = &base[offset..offset + 18];

            // Monitor Name Descriptor: Header 00 00 00 FC 00
            if descriptor[0..4] == [0x00, 0x00, 0x00, 0xFC] {
                let name_bytes = &descriptor[5..18];
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

        // 7. Standard Timings (offsets 0x26-0x35, 8 descriptors, 2 bytes each)
        for i in 0..8 {
            let offset = 0x26 + (i * 2);
            let b1 = base[offset];
            let b2 = base[offset + 1];

            if b1 == 0x01 && b2 == 0x01 || b1 == 0x00 {
                continue; // Unused
            }

            let w = (b1 as u16 + 31) * 8;
            let ratio_bits = (b2 >> 6) & 0x03;
            let refresh_rate = (b2 & 0x3F) + 60;

            let h = match ratio_bits {
                0x00 => (w * 10) / 16, // 16:10
                0x01 => (w * 3) / 4,   // 4:3
                0x02 => (w * 4) / 5,   // 5:4
                0x03 => (w * 9) / 16,  // 16:9
                _ => unreachable!(),
            };

            caps.supported_modes.push(VideoMode {
                width: w,
                height: h,
                refresh_rate,
            });
        }

        // 8. Detailed Timing Descriptors (DTD) (offsets 0x36, 0x48, 0x5A, 0x6C)
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
}