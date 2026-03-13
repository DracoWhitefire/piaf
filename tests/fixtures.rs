use piaf::{
    capabilities_from_edid, parse_edid, ColorBitDepth, DisplayGamma, EdidVersion, ExtensionLibrary,
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
