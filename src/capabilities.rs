use crate::model::capabilities::DisplayCapabilities;
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
                break; // Found the name
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
}
