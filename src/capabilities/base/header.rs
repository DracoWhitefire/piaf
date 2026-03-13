use crate::model::capabilities::DisplayCapabilities;
use crate::model::color::{
    AnalogColorType, Chromaticity, ColorBitDepth, DigitalColorEncoding, DisplayGamma,
};
use crate::model::edid::EdidVersion;
use crate::model::features::DisplayFeatureFlags;
use crate::model::input::{AnalogSyncLevel, VideoInputFlags, VideoInterface};
use crate::model::manufacture::ManufactureDate;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::String;
use crate::model::screen::ScreenSize;

/// Decodes fixed-position header fields: manufacturer, dates, version, product code,
/// serial number, video input definition, and physical dimensions.
#[cfg(any(feature = "alloc", feature = "std"))]
pub(super) fn decode_header_fields(base: &[u8; 128], caps: &mut DisplayCapabilities) {
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

#[cfg(test)]
#[cfg(any(feature = "alloc", feature = "std"))]
mod tests {
    use crate::capabilities::base::BaseBlockHandler;
    use crate::model::capabilities::DisplayCapabilities;
    use crate::model::color::{
        AnalogColorType, Chromaticity, ColorBitDepth, DigitalColorEncoding, DisplayGamma,
    };
    use crate::model::edid::EdidVersion;
    use crate::model::extension::ExtensionHandler;
    use crate::model::features::DisplayFeatureFlags;
    use crate::model::input::{AnalogSyncLevel, VideoInterface};
    use crate::model::manufacture::ManufactureDate;
    use crate::model::prelude::Vec;
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
}
