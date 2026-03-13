use piaf::{
    capabilities_from_edid, parse_edid, AnalogColorType, ChromaticityPoint, ColorBitDepth,
    DigitalColorEncoding, DisplayFeatureFlags, DisplayGamma, EdidVersion, ExtensionLibrary,
    ExtensionTagRegistry, ManufactureDate, VideoInterface,
};

fn load(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("Failed to read fixture {path}: {e}"))
}

// ---------------------------------------------------------------------------
// LG UltraGear (GSM) — HDMI with CEA-861 extension block
// ---------------------------------------------------------------------------

#[test]
fn lg_ultragear_parses_without_error() {
    let bytes = load("testdata/valid/lg_ultragear_gsm.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    assert!(parse_edid(&bytes, &library).is_ok());
}

#[test]
fn lg_ultragear_identification() {
    let bytes = load("testdata/valid/lg_ultragear_gsm.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    assert_eq!(caps.manufacturer.as_deref(), Some("GSM"));
    assert_eq!(caps.display_name.as_deref(), Some("LG ULTRAGEAR"));
    assert_eq!(
        caps.manufacture_date,
        Some(ManufactureDate::Manufactured {
            week: Some(3),
            year: 2021
        })
    );
    assert_eq!(
        caps.edid_version,
        Some(EdidVersion {
            version: 1,
            revision: 3
        })
    );
    assert_eq!(caps.gamma, DisplayGamma::from_edid_byte(0x78));
    assert!(caps.digital);
    assert_eq!(caps.color_bit_depth, None); // undefined in base block
    assert_eq!(caps.video_interface, None); // undefined in base block

    // 0xEA = DPMS_STANDBY | DPMS_SUSPEND | DPMS_ACTIVE_OFF | PREFERRED_TIMING
    let features = caps.display_features.unwrap();
    assert!(features.contains(DisplayFeatureFlags::DPMS_STANDBY));
    assert!(features.contains(DisplayFeatureFlags::DPMS_SUSPEND));
    assert!(features.contains(DisplayFeatureFlags::DPMS_ACTIVE_OFF));
    assert!(features.contains(DisplayFeatureFlags::PREFERRED_TIMING));
    assert!(!features.contains(DisplayFeatureFlags::CONTINUOUS_TIMINGS));
    // EDID 1.3 digital — color encoding field not decoded
    assert_eq!(caps.digital_color_encoding, None);
    // Chromaticity — wide-gamut red, D65 white point
    assert_eq!(caps.chromaticity.red,   ChromaticityPoint { x_raw: 702, y_raw: 316 });
    assert_eq!(caps.chromaticity.green, ChromaticityPoint { x_raw: 271, y_raw: 684 });
    assert_eq!(caps.chromaticity.blue,  ChromaticityPoint { x_raw: 154, y_raw:  59 });
    assert_eq!(caps.chromaticity.white, ChromaticityPoint { x_raw: 321, y_raw: 337 });
    assert_eq!(caps.width_cm, Some(60));
    assert_eq!(caps.height_cm, Some(34));
}

#[test]
fn lg_ultragear_range_limits() {
    let bytes = load("testdata/valid/lg_ultragear_gsm.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    assert_eq!(caps.min_v_rate, Some(48));
    assert_eq!(caps.max_v_rate, Some(120));
    assert_eq!(caps.min_h_rate_khz, Some(30));
    assert_eq!(caps.max_h_rate_khz, Some(230));
    assert_eq!(caps.max_pixel_clock_mhz, Some(600));
}

#[test]
fn lg_ultragear_has_audio() {
    let bytes = load("testdata/valid/lg_ultragear_gsm.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    assert!(caps.has_audio);
}

#[test]
fn lg_ultragear_has_cea_extension() {
    let bytes = load("testdata/valid/lg_ultragear_gsm.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();

    assert_eq!(parsed.extensions.len(), 1);
    assert_eq!(parsed.extensions[0][0], 0x02); // CEA-861 tag
}

// ---------------------------------------------------------------------------
// AUO eDP laptop panel — base block only, no extensions
// ---------------------------------------------------------------------------

#[test]
fn auo_edp_parses_without_error() {
    let bytes = load("testdata/valid/auo_edp_laptop.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    assert!(parse_edid(&bytes, &library).is_ok());
}

#[test]
fn auo_edp_identification() {
    let bytes = load("testdata/valid/auo_edp_laptop.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    assert_eq!(caps.manufacturer.as_deref(), Some("AUO"));
    assert_eq!(
        caps.manufacture_date,
        Some(ManufactureDate::Manufactured {
            week: Some(3),
            year: 2020
        })
    );
    assert_eq!(
        caps.edid_version,
        Some(EdidVersion {
            version: 1,
            revision: 4
        })
    );
    assert_eq!(caps.gamma, DisplayGamma::from_edid_byte(0x78));
    assert!(caps.digital);
    assert_eq!(caps.color_bit_depth, Some(ColorBitDepth::Depth8));
    assert_eq!(caps.video_interface, Some(VideoInterface::DisplayPort));
    // 0x03 = PREFERRED_TIMING | CONTINUOUS_TIMINGS
    let features = caps.display_features.unwrap();
    assert!(features.contains(DisplayFeatureFlags::PREFERRED_TIMING));
    assert!(features.contains(DisplayFeatureFlags::CONTINUOUS_TIMINGS));
    assert!(!features.contains(DisplayFeatureFlags::DPMS_STANDBY));
    // EDID 1.4 digital, byte 0x18 bits 4-3 = 0b00 → Rgb444
    assert_eq!(
        caps.digital_color_encoding,
        Some(DigitalColorEncoding::Rgb444)
    );
    assert_eq!(caps.analog_color_type, None);
    // Chromaticity — sRGB-ish primaries, D65 white point
    assert_eq!(caps.chromaticity.red,   ChromaticityPoint { x_raw: 589, y_raw: 355 });
    assert_eq!(caps.chromaticity.green, ChromaticityPoint { x_raw: 360, y_raw: 592 });
    assert_eq!(caps.chromaticity.blue,  ChromaticityPoint { x_raw: 165, y_raw: 131 });
    assert_eq!(caps.chromaticity.white, ChromaticityPoint { x_raw: 321, y_raw: 337 });
    assert_eq!(caps.width_cm, Some(38));
    assert_eq!(caps.height_cm, Some(22));
}

#[test]
fn auo_edp_range_limits() {
    let bytes = load("testdata/valid/auo_edp_laptop.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    assert_eq!(caps.min_v_rate, Some(60));
    assert_eq!(caps.max_v_rate, Some(144));
    assert_eq!(caps.max_pixel_clock_mhz, Some(370));
}

#[test]
fn auo_edp_no_extensions() {
    let bytes = load("testdata/valid/auo_edp_laptop.bin");
    let registry = ExtensionTagRegistry::new();
    let parsed = parse_edid(&bytes, &registry).unwrap();

    assert_eq!(parsed.extensions.len(), 0);
    assert!(caps_have_no_audio(&bytes));
}

fn caps_have_no_audio(bytes: &[u8]) -> bool {
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);
    !caps.has_audio
}
