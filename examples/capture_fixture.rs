//! Captures EDID data from connected displays and generates fixture test stubs.
//!
//! Run from the repository root:
//!
//! ```text
//! cargo run --features std --example capture_fixture
//! ```
//!
//! For each connected display with a non-empty EDID:
//!   1. Saves the raw bytes to `testdata/valid/<name>.bin`.
//!   2. Prints a ready-to-paste Rust test block to **stdout**.
//!
//! Status messages go to **stderr** so the test code can be redirected cleanly:
//!
//! ```text
//! cargo run --features std --example capture_fixture >> tests/fixtures.rs
//! ```
//!
//! Files that already exist in `testdata/valid/` are skipped — re-running is safe.

use piaf::{
    ColorBitDepth, DisplayIdCapabilities, ExtensionLibrary, ManufactureDate, ScreenSize,
    VideoInterface, capabilities_from_edid, parse_edid,
};
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Formats an `Option<RefreshRate>` as a Rust expression string for use in generated test code.
fn refresh_rate_expr(r: Option<piaf::RefreshRate>) -> String {
    match r {
        Some(r) if r.denom() == 1 => format!("Some(RefreshRate::integral({}))", r.numer()),
        Some(r) => format!(
            "Some(RefreshRate::fractional({}, {}))",
            r.numer(),
            r.denom()
        ),
        None => "None".to_string(),
    }
}

/// Converts an arbitrary string into a valid snake_case Rust identifier segment.
fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn fn_header(fn_name: &str, bin_path: &str) {
    println!("#[test]");
    println!("fn {fn_name}() {{");
    println!("    let bytes = load(\"{bin_path}\");");
    println!("    let library = ExtensionLibrary::with_standard_handlers();");
}

fn parse_lines() {
    println!("    let parsed = parse_edid(&bytes, &library).unwrap();");
    println!("    let caps = capabilities_from_edid(&parsed, &library);");
}

// ---------------------------------------------------------------------------
// Test-block generators
// ---------------------------------------------------------------------------

fn gen_parses_without_error(mod_name: &str, bin_path: &str) {
    fn_header(&format!("{mod_name}_parses_without_error"), bin_path);
    println!("    assert!(parse_edid(&bytes, &library).is_ok());");
    println!("}}");
    println!();
}

fn gen_identification(mod_name: &str, bin_path: &str, caps: &piaf::DisplayCapabilities) {
    fn_header(&format!("{mod_name}_identification"), bin_path);
    parse_lines();
    println!();

    if let Some(m) = caps.manufacturer.as_ref() {
        println!(
            "    assert_eq!(caps.manufacturer.as_ref().map(|m| m.as_str()), Some(\"{}\"));",
            m.as_str()
        );
    }
    if let Some(name) = caps.display_name.as_deref() {
        println!("    assert_eq!(caps.display_name.as_deref(), Some(\"{name}\"));");
    }
    match caps.manufacture_date {
        Some(ManufactureDate::Manufactured {
            week: Some(w),
            year,
        }) => {
            println!(
                "    assert_eq!(caps.manufacture_date, \
                 Some(ManufactureDate::Manufactured {{ week: Some({w}), year: {year} }}));"
            );
        }
        Some(ManufactureDate::Manufactured { week: None, year }) => {
            println!(
                "    assert_eq!(caps.manufacture_date, \
                 Some(ManufactureDate::Manufactured {{ week: None, year: {year} }}));"
            );
        }
        Some(ManufactureDate::ModelYear(y)) => {
            println!(
                "    assert_eq!(caps.manufacture_date, Some(ManufactureDate::ModelYear({y})));"
            );
        }
        Some(_) | None => {}
    }
    if let Some(v) = caps.edid_version {
        println!(
            "    assert_eq!(caps.edid_version, \
             Some(EdidVersion {{ version: {}, revision: {} }}));",
            v.version, v.revision
        );
    }
    println!("    assert_eq!(caps.digital, {});", caps.digital);
    if let Some(depth) = caps.color_bit_depth {
        let variant = match depth {
            ColorBitDepth::Depth6 => "Depth6",
            ColorBitDepth::Depth8 => "Depth8",
            ColorBitDepth::Depth10 => "Depth10",
            ColorBitDepth::Depth12 => "Depth12",
            ColorBitDepth::Depth14 => "Depth14",
            ColorBitDepth::Depth16 => "Depth16",
            _ => "unknown",
        };
        println!("    assert_eq!(caps.color_bit_depth, Some(ColorBitDepth::{variant}));");
    }
    if let Some(iface) = caps.video_interface {
        let variant = match iface {
            VideoInterface::Dvi => "Dvi",
            VideoInterface::HdmiA => "HdmiA",
            VideoInterface::HdmiB => "HdmiB",
            VideoInterface::Mddi => "Mddi",
            VideoInterface::DisplayPort => "DisplayPort",
            _ => "unknown",
        };
        println!("    assert_eq!(caps.video_interface, Some(VideoInterface::{variant}));");
    }
    if let Some(ScreenSize::Physical {
        width_cm,
        height_cm,
    }) = caps.screen_size
    {
        println!(
            "    assert_eq!(caps.screen_size, \
             Some(ScreenSize::Physical {{ width_cm: {width_cm}, height_cm: {height_cm} }}));"
        );
    }
    if let Some((w, h)) = caps.preferred_image_size_mm {
        println!("    assert_eq!(caps.preferred_image_size_mm, Some(({w}, {h})));");
    }
    if let (Some(min_v), Some(max_v)) = (caps.min_v_rate, caps.max_v_rate) {
        println!("    assert_eq!(caps.min_v_rate, Some({min_v}));");
        println!("    assert_eq!(caps.max_v_rate, Some({max_v}));");
    }

    println!("}}");
    println!();
}

fn gen_displayid(
    mod_name: &str,
    bin_path: &str,
    caps: &piaf::DisplayCapabilities,
    did: &DisplayIdCapabilities,
) {
    fn_header(&format!("{mod_name}_displayid"), bin_path);
    parse_lines();
    println!();
    println!("    let did = caps");
    println!("        .get_extension_data::<DisplayIdCapabilities>(0x70)");
    println!("        .expect(\"expected DisplayID section\");");
    println!("    assert_eq!(did.version, 0x{:02X});", did.version);
    println!("    assert_eq!(did.product_type, {});", did.product_type);
    if let Some((w, h)) = caps.preferred_image_size_mm {
        println!("    assert_eq!(caps.preferred_image_size_mm, Some(({w}, {h})));");
    }
    if let Some(m) = caps
        .supported_modes
        .iter()
        .max_by_key(|m| (m.width as u32 * m.height as u32, m.refresh_rate))
    {
        let (w, h) = (m.width, m.height);
        let r_expr = refresh_rate_expr(m.refresh_rate);
        let r_display = m.refresh_rate.map(|r| r.as_f64()).unwrap_or(0.0);
        println!("    assert!(");
        println!(
            "        caps.supported_modes.iter().any(\
             |m| m.width == {w} && m.height == {h} && m.refresh_rate == {r_expr}),"
        );
        println!("        \"expected {w}x{h}@{r_display:.3}Hz mode\"");
        println!("    );");
    }
    println!("}}");
    println!();
}

fn gen_supported_modes(mod_name: &str, bin_path: &str, caps: &piaf::DisplayCapabilities) {
    if caps.supported_modes.is_empty() {
        return;
    }
    fn_header(&format!("{mod_name}_supported_modes"), bin_path);
    parse_lines();
    println!();
    println!("    assert!(!caps.supported_modes.is_empty());");

    // Assert up to 3 distinct resolutions: highest first, then a couple of others.
    let mut modes: Vec<_> = caps.supported_modes.iter().collect();
    modes.sort_by_key(|m| std::cmp::Reverse((m.width as u32 * m.height as u32, m.refresh_rate)));
    modes.dedup_by_key(|m| (m.width, m.height));
    for m in modes.iter().take(3) {
        let (w, h) = (m.width, m.height);
        let r_expr = refresh_rate_expr(m.refresh_rate);
        println!(
            "    assert!(caps.supported_modes.iter().any(\
             |m| m.width == {w} && m.height == {h} && m.refresh_rate == {r_expr}));"
        );
    }
    println!("}}");
    println!();
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let drm_path = Path::new("/sys/class/drm");
    if !drm_path.exists() {
        eprintln!("Error: /sys/class/drm not found. This tool only works on Linux.");
        std::process::exit(1);
    }

    fs::create_dir_all("testdata/valid").expect("failed to create testdata/valid");

    let library = ExtensionLibrary::with_standard_handlers();
    let mut captured = 0;
    let mut skipped = 0;

    let mut entries: Vec<_> = fs::read_dir(drm_path)
        .expect("failed to read /sys/class/drm")
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let edid_path = path.join("edid");
        if !edid_path.exists() {
            continue;
        }
        let Ok(bytes) = fs::read(&edid_path) else {
            continue;
        };
        if bytes.is_empty() || bytes.iter().all(|&b| b == 0) {
            continue;
        }

        let connector = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let caps = match parse_edid(&bytes, &library) {
            Ok(parsed) => capabilities_from_edid(&parsed, &library),
            Err(e) => {
                eprintln!("  {connector}: parse error — {e:?}, skipping");
                continue;
            }
        };

        let mfr_str = caps
            .manufacturer
            .as_ref()
            .map(|m| m.as_str().to_ascii_lowercase())
            .unwrap_or_else(|| "unk".into());
        // Prefer display_name; fall back to "<mfr>_<product_code_hex>" so the
        // name is stable and independent of the connector path.
        let label = if let Some(name) = caps.display_name.as_deref() {
            name.to_string()
        } else if let Some(code) = caps.product_code {
            format!("{mfr_str}_{code:04x}")
        } else {
            mfr_str.clone()
        };
        let file_stem = format!("{}_{}", sanitize(&label), sanitize(&mfr_str));
        let bin_path = format!("testdata/valid/{file_stem}.bin");

        if Path::new(&bin_path).exists() {
            eprintln!("  {connector}: {bin_path} already exists, skipping");
            skipped += 1;
            continue;
        }

        fs::write(&bin_path, &bytes).unwrap_or_else(|e| panic!("failed to write {bin_path}: {e}"));

        let has_displayid = caps
            .get_extension_data::<DisplayIdCapabilities>(0x70)
            .is_some();
        eprintln!(
            "  {connector} → {bin_path} ({} byte(s){})",
            bytes.len(),
            if has_displayid { ", DisplayID" } else { "" },
        );
        captured += 1;

        // Emit the test block header comment.
        let mfr_display = caps
            .manufacturer
            .as_ref()
            .map(|m| m.as_str())
            .unwrap_or("???");
        let name_display = caps.display_name.as_deref().unwrap_or("Unknown");
        println!("// {}", "-".repeat(75));
        println!(
            "// {name_display} ({mfr_display}){}",
            if has_displayid { " — DisplayID" } else { "" }
        );
        println!("// {}", "-".repeat(75));
        println!();

        let mod_name = file_stem.replace('-', "_");

        gen_parses_without_error(&mod_name, &bin_path);
        gen_identification(&mod_name, &bin_path, &caps);
        if let Some(did) = caps.get_extension_data::<DisplayIdCapabilities>(0x70) {
            gen_displayid(&mod_name, &bin_path, &caps, did);
        }
        gen_supported_modes(&mod_name, &bin_path, &caps);
    }

    eprintln!();
    if captured == 0 && skipped == 0 {
        eprintln!("No active displays with EDID data found.");
    } else if captured == 0 {
        eprintln!("All displays already captured ({skipped} already existed).");
    } else {
        eprintln!(
            "Captured {captured} display(s){}.",
            if skipped > 0 {
                format!(" ({skipped} already existed, skipped)")
            } else {
                String::new()
            }
        );
        eprintln!("Paste the test blocks printed above into tests/fixtures.rs.");

        if !fs::read_to_string("tests/fixtures.rs")
            .map(|s| s.contains("DisplayIdCapabilities"))
            .unwrap_or(false)
        {
            eprintln!();
            eprintln!(
                "  Note: if any display has a DisplayID section, add to the imports \
                 at the top of tests/fixtures.rs:"
            );
            eprintln!("    use piaf::DisplayIdCapabilities;");
        }
    }
}
