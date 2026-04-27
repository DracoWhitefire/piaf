use piaf::{AudioFormat, AudioSampleRates, Cea861Capabilities, Cea861Flags, HdmiVsdbFlags};
use piaf::{
    ChromaticityPoint, ColorBitDepth, DigitalColorEncoding, DisplayFeatureFlags, DisplayGamma,
    EdidVersion, ExtensionLibrary, ExtensionTagRegistry, ManufactureDate, RefreshRate, ScreenSize,
    VideoInterface, capabilities_from_edid, parse_edid,
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

    assert_eq!(caps.manufacturer.as_ref().map(|m| m.as_str()), Some("GSM"));
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
    assert_eq!(
        caps.chromaticity.red,
        ChromaticityPoint {
            x_raw: 702,
            y_raw: 316
        }
    );
    assert_eq!(
        caps.chromaticity.green,
        ChromaticityPoint {
            x_raw: 271,
            y_raw: 684
        }
    );
    assert_eq!(
        caps.chromaticity.blue,
        ChromaticityPoint {
            x_raw: 154,
            y_raw: 59
        }
    );
    assert_eq!(
        caps.chromaticity.white,
        ChromaticityPoint {
            x_raw: 321,
            y_raw: 337
        }
    );
    assert_eq!(
        caps.screen_size,
        Some(ScreenSize::Physical {
            width_cm: 60,
            height_cm: 34
        })
    );
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

    let cea = caps.get_extension_data::<Cea861Capabilities>(0x02).unwrap();
    assert!(cea.flags.contains(Cea861Flags::BASIC_AUDIO));
}

#[test]
fn lg_ultragear_audio_descriptors() {
    let bytes = load("testdata/valid/lg_ultragear_gsm.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    let cea = caps.get_extension_data::<Cea861Capabilities>(0x02).unwrap();

    // The LG UltraGear advertises at least one LPCM descriptor (stereo at 32/44.1/48 kHz).
    let lpcm = cea
        .audio_descriptors
        .iter()
        .find(|s| s.format == AudioFormat::Lpcm);
    assert!(lpcm.is_some(), "expected at least one LPCM SAD");
    let lpcm = lpcm.unwrap();
    assert!(lpcm.sample_rates.contains(AudioSampleRates::HZ_48000));
}

#[test]
fn lg_ultragear_hdmi_vsdb() {
    let bytes = load("testdata/valid/lg_ultragear_gsm.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    let cea = caps.get_extension_data::<Cea861Capabilities>(0x02).unwrap();
    let vsdb = cea
        .hdmi_vsdb
        .as_ref()
        .expect("LG UltraGear should have an HDMI VSDB");

    // LG UltraGear is an HDMI display — expect deep color and a reasonable TMDS clock.
    assert!(
        vsdb.flags.contains(HdmiVsdbFlags::DC_36BIT)
            || vsdb.flags.contains(HdmiVsdbFlags::DC_30BIT),
        "expected at least one deep color flag"
    );
    assert!(
        vsdb.max_tmds_clock_mhz.is_some_and(|c| c >= 300),
        "expected max TMDS clock >= 300 MHz"
    );
}

#[test]
fn lg_ultragear_cea_vics() {
    let bytes = load("testdata/valid/lg_ultragear_gsm.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    let cea = caps.get_extension_data::<Cea861Capabilities>(0x02).unwrap();

    // VICs present in the LG UltraGear Video Data Block
    let vic_nums: Vec<u8> = cea.vics.iter().map(|(v, _)| *v).collect();
    for expected in [1u8, 3, 4, 16, 18, 19, 31, 63] {
        assert!(vic_nums.contains(&expected), "VIC {expected} missing");
    }

    // Each in-table VIC should have a corresponding entry in supported_modes
    assert!(caps.supported_modes.iter().any(|m| m.width == 1920
        && m.height == 1080
        && m.refresh_rate == Some(RefreshRate::integral(60))
        && !m.interlaced)); // VIC 16
    assert!(caps.supported_modes.iter().any(|m| m.width == 1280
        && m.height == 720
        && m.refresh_rate == Some(RefreshRate::integral(60)))); // VIC 4
    assert!(caps.supported_modes.iter().any(|m| m.width == 1920
        && m.height == 1080
        && m.refresh_rate == Some(RefreshRate::integral(120))
        && !m.interlaced)); // VIC 63

    // 4K UHD VICs are now in the extended table (VICs 93–97)
    assert!(caps.supported_modes.iter().any(|m| m.width == 3840
        && m.height == 2160
        && m.refresh_rate == Some(RefreshRate::integral(24)))); // VIC 93
    assert!(caps.supported_modes.iter().any(|m| m.width == 3840
        && m.height == 2160
        && m.refresh_rate == Some(RefreshRate::integral(30)))); // VIC 95
    assert!(caps.supported_modes.iter().any(|m| m.width == 3840
        && m.height == 2160
        && m.refresh_rate == Some(RefreshRate::integral(60)))); // VIC 97
}

#[test]
fn lg_ultragear_has_cea_extension() {
    let bytes = load("testdata/valid/lg_ultragear_gsm.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();

    assert_eq!(parsed.num_extensions, 1);
    assert_eq!(parsed.extension_block(0).unwrap()[0], 0x02); // CEA-861 tag
}

// ---------------------------------------------------------------------------
// AUO eDP laptop panel — base block only, no extensions
// ---------------------------------------------------------------------------

#[test]
fn auo_edp_parses_without_error() {
    let bytes = load("testdata/valid/auo_978f_auo.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    assert!(parse_edid(&bytes, &library).is_ok());
}

#[test]
fn auo_edp_identification() {
    let bytes = load("testdata/valid/auo_978f_auo.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    assert_eq!(caps.manufacturer.as_ref().map(|m| m.as_str()), Some("AUO"));
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
    assert_eq!(
        caps.chromaticity.red,
        ChromaticityPoint {
            x_raw: 589,
            y_raw: 355
        }
    );
    assert_eq!(
        caps.chromaticity.green,
        ChromaticityPoint {
            x_raw: 360,
            y_raw: 592
        }
    );
    assert_eq!(
        caps.chromaticity.blue,
        ChromaticityPoint {
            x_raw: 165,
            y_raw: 131
        }
    );
    assert_eq!(
        caps.chromaticity.white,
        ChromaticityPoint {
            x_raw: 321,
            y_raw: 337
        }
    );
    assert_eq!(
        caps.screen_size,
        Some(ScreenSize::Physical {
            width_cm: 38,
            height_cm: 22
        })
    );
    assert_eq!(caps.preferred_image_size_mm, Some((382, 215)));
}

#[test]
fn auo_edp_range_limits() {
    let bytes = load("testdata/valid/auo_978f_auo.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    assert_eq!(caps.min_v_rate, Some(60));
    assert_eq!(caps.max_v_rate, Some(144));
    assert_eq!(caps.max_pixel_clock_mhz, Some(370));
}

#[test]
fn auo_edp_supported_modes() {
    let bytes = load("testdata/valid/auo_978f_auo.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    assert!(!caps.supported_modes.is_empty());
    assert!(caps.supported_modes.iter().any(|m| m.width == 1920
        && m.height == 1080
        && m.refresh_rate == Some(RefreshRate::integral(144))));
}

#[test]
fn auo_edp_no_extensions() {
    let bytes = load("testdata/valid/auo_978f_auo.bin");
    let registry = ExtensionTagRegistry::new();
    let parsed = parse_edid(&bytes, &registry).unwrap();

    assert_eq!(parsed.num_extensions, 0);
    assert!(caps_have_no_audio(&bytes));
}

fn caps_have_no_audio(bytes: &[u8]) -> bool {
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);
    caps.get_extension_data::<Cea861Capabilities>(0x02)
        .is_none_or(|cea| !cea.flags.contains(Cea861Flags::BASIC_AUDIO))
}
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// PHILIPS FTV (PHL)
// ---------------------------------------------------------------------------

#[test]
fn philips_ftv_phl_parses_without_error() {
    let bytes = load("testdata/valid/philips_ftv_phl.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    assert!(parse_edid(&bytes, &library).is_ok());
}

#[test]
fn philips_ftv_phl_identification() {
    let bytes = load("testdata/valid/philips_ftv_phl.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    assert_eq!(caps.manufacturer.as_ref().map(|m| m.as_str()), Some("PHL"));
    assert_eq!(caps.display_name.as_deref(), Some("PHILIPS FTV"));
    assert_eq!(
        caps.manufacture_date,
        Some(ManufactureDate::Manufactured {
            week: Some(30),
            year: 2016
        })
    );
    assert_eq!(
        caps.edid_version,
        Some(EdidVersion {
            version: 1,
            revision: 3
        })
    );
    assert!(caps.digital);
    assert_eq!(
        caps.screen_size,
        Some(ScreenSize::Physical {
            width_cm: 64,
            height_cm: 36
        })
    );
    assert_eq!(caps.preferred_image_size_mm, Some((640, 360)));
    assert_eq!(caps.min_v_rate, Some(56));
    assert_eq!(caps.max_v_rate, Some(76));
}

#[test]
fn philips_ftv_phl_supported_modes() {
    let bytes = load("testdata/valid/philips_ftv_phl.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    assert!(!caps.supported_modes.is_empty());
    assert!(caps.supported_modes.iter().any(|m| m.width == 1920
        && m.height == 1080
        && m.refresh_rate == Some(RefreshRate::integral(60))));
    assert!(caps.supported_modes.iter().any(|m| m.width == 1280
        && m.height == 1024
        && m.refresh_rate == Some(RefreshRate::integral(60))));
    assert!(caps.supported_modes.iter().any(|m| m.width == 1360
        && m.height == 768
        && m.refresh_rate == Some(RefreshRate::integral(59))));
}

// ---------------------------------------------------------------------------
// PHL 275E1 (PHL)
// ---------------------------------------------------------------------------

#[test]
fn phl_275e1_phl_parses_without_error() {
    let bytes = load("testdata/valid/phl_275e1_phl.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    assert!(parse_edid(&bytes, &library).is_ok());
}

#[test]
fn phl_275e1_phl_identification() {
    let bytes = load("testdata/valid/phl_275e1_phl.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    assert_eq!(caps.manufacturer.as_ref().map(|m| m.as_str()), Some("PHL"));
    assert_eq!(caps.display_name.as_deref(), Some("PHL 275E1"));
    assert_eq!(
        caps.manufacture_date,
        Some(ManufactureDate::Manufactured {
            week: Some(13),
            year: 2022
        })
    );
    assert_eq!(
        caps.edid_version,
        Some(EdidVersion {
            version: 1,
            revision: 3
        })
    );
    assert!(caps.digital);
    assert_eq!(
        caps.screen_size,
        Some(ScreenSize::Physical {
            width_cm: 60,
            height_cm: 34
        })
    );
    assert_eq!(caps.preferred_image_size_mm, Some((597, 336)));
    assert_eq!(caps.min_v_rate, Some(48));
    assert_eq!(caps.max_v_rate, Some(75));
}

#[test]
fn phl_275e1_phl_supported_modes() {
    let bytes = load("testdata/valid/phl_275e1_phl.bin");
    let library = ExtensionLibrary::with_standard_handlers();
    let parsed = parse_edid(&bytes, &library).unwrap();
    let caps = capabilities_from_edid(&parsed, &library);

    assert!(!caps.supported_modes.is_empty());
    assert!(caps.supported_modes.iter().any(|m| m.width == 2560
        && m.height == 1440
        && m.refresh_rate == Some(RefreshRate::integral(74))));
    assert!(caps.supported_modes.iter().any(|m| m.width == 1920
        && m.height == 1080
        && m.refresh_rate == Some(RefreshRate::integral(60))));
    assert!(caps.supported_modes.iter().any(|m| m.width == 1280
        && m.height == 1440
        && m.refresh_rate == Some(RefreshRate::integral(59))));
}
