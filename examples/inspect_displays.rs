use piaf::{
    capabilities_from_edid, parse_edid, Cea861Capabilities, Cea861Flags, Cea861Handler,
    DisplayCapabilities, EdidWarning, ExtensionHandler, ExtensionLibrary,
};
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Custom extension data example
//
// This handler supersedes the built-in Cea861Handler for tag 0x02. It stores
// typed extension data that consumers can retrieve via `caps.get_extension_data`.
//
// Any type that is Debug + Send + Sync can be stored — no manual trait impl
// required beyond #[derive(Debug)].
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct CeaDetails {
    version: u8,
    dtd_offset: u8,
}

#[derive(Debug)]
struct CeaDetailsHandler;

impl ExtensionHandler for CeaDetailsHandler {
    fn process(
        &self,
        ext: &[u8; 128],
        caps: &mut DisplayCapabilities,
        warnings: &mut Vec<EdidWarning>,
    ) {
        // Run the built-in handler first so VICs and modes are populated normally.
        Cea861Handler.process(ext, caps, warnings);

        // Then overlay additional typed data under a custom key.
        caps.set_extension_data(
            0xF2,
            CeaDetails {
                version: ext[1],
                dtd_offset: ext[2],
            },
        );
    }
}

fn main() {
    println!("--- Searching for connected displays (EDID) ---");

    let drm_path = Path::new("/sys/class/drm");
    if !drm_path.exists() {
        eprintln!("Error: /sys/class/drm not found. This script only works on Linux.");
        return;
    }

    // Start from the standard handler set, then replace the CEA-861 handler
    // with our custom one to capture additional typed data.
    let mut library = ExtensionLibrary::with_standard_handlers();
    if let Some(cea) = library.extensions.iter_mut().find(|e| e.tag == 0x02) {
        cea.handler = Some(Box::new(CeaDetailsHandler));
    }

    let mut found = 0;

    if let Ok(entries) = fs::read_dir(drm_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            let edid_path = path.join("edid");
            if edid_path.exists() {
                if let Ok(bytes) = fs::read(&edid_path) {
                    if bytes.is_empty() || bytes.iter().all(|&b| b == 0) {
                        continue;
                    }

                    println!("\nFound connector: {}", name);
                    println!("EDID file: {}", edid_path.display());
                    println!("Data size: {} bytes", bytes.len());

                    match parse_edid(&bytes, &library) {
                        Ok(parsed) => {
                            let caps = capabilities_from_edid(&parsed, &library);

                            if let Some(v) = caps.edid_version {
                                println!("  EDID version: {}", v);
                            }
                            println!(
                                "  Manufacturer: {:?}",
                                caps.manufacturer.as_deref().unwrap_or("Unknown")
                            );
                            match caps.manufacture_date {
                                Some(piaf::ManufactureDate::Manufactured {
                                    week: Some(w),
                                    year,
                                }) => println!("  Manufactured: week {}, {}", w, year),
                                Some(piaf::ManufactureDate::Manufactured { week: None, year }) => {
                                    println!("  Manufactured: {}", year)
                                }
                                Some(piaf::ManufactureDate::ModelYear(year)) => {
                                    println!("  Model year:   {}", year)
                                }
                                None => {}
                            }
                            println!(
                                "  Display Name: {:?}",
                                caps.display_name.as_deref().unwrap_or("Unknown")
                            );
                            for text in &caps.unspecified_text {
                                println!("  Info:         {}", text);
                            }
                            println!("  Product Code: {:?}", caps.product_code);
                            println!("  Serial:       {:?}", caps.serial_number);
                            if let Some(s) = caps.serial_number_string.as_deref() {
                                println!("  Serial string: {}", s);
                            }
                            match caps.screen_size {
                                Some(piaf::ScreenSize::Physical {
                                    width_cm,
                                    height_cm,
                                }) => println!("  Dimensions:   {}x{} cm", width_cm, height_cm),
                                Some(piaf::ScreenSize::Landscape(v)) => println!(
                                    "  Aspect ratio: {:.2}:1 (landscape)",
                                    (v as f32 + 99.0) / 100.0
                                ),
                                Some(piaf::ScreenSize::Portrait(v)) => println!(
                                    "  Aspect ratio: 1:{:.2} (portrait)",
                                    (v as f32 + 99.0) / 100.0
                                ),
                                None => {}
                            }
                            println!(
                                "  Input type:   {}",
                                if caps.digital { "Digital" } else { "Analog" }
                            );
                            if let Some(level) = caps.analog_sync_level {
                                println!("  Sync level:   {:?}", level);
                            }
                            if let Some(depth) = caps.color_bit_depth {
                                println!("  Color depth:  {} bpc", depth.bits_per_primary());
                            }
                            if let Some(gamma) = caps.gamma {
                                println!("  Gamma:        {:.2}", gamma.value());
                            }
                            let c = &caps.chromaticity;
                            println!("  Chromaticity: R({:.4},{:.4}) G({:.4},{:.4}) B({:.4},{:.4}) W({:.4},{:.4})",
                                c.red.x(),   c.red.y(),
                                c.green.x(), c.green.y(),
                                c.blue.x(),  c.blue.y(),
                                c.white.x(), c.white.y(),
                            );
                            if let Some(iface) = caps.video_interface {
                                println!("  Interface:    {:?}", iface);
                            }
                            if let Some(f) = caps.display_features {
                                println!("  Features:     {:?}", f);
                            }
                            if let Some(enc) = caps.digital_color_encoding {
                                println!("  Color enc:    {:?}", enc);
                            }
                            if let Some(ct) = caps.analog_color_type {
                                println!("  Color type:   {:?}", ct);
                            }
                            if let Some(cea) = caps.get_extension_data::<Cea861Capabilities>(0x02) {
                                println!(
                                    "  Audio support: {}",
                                    if cea.flags.contains(Cea861Flags::BASIC_AUDIO) {
                                        "Yes"
                                    } else {
                                        "No"
                                    }
                                );
                                if !cea.vics.is_empty() {
                                    let vic_strs: Vec<String> = cea
                                        .vics
                                        .iter()
                                        .map(|(v, native)| {
                                            if *native {
                                                format!("{}*", v)
                                            } else {
                                                v.to_string()
                                            }
                                        })
                                        .collect();
                                    println!(
                                        "  CEA VICs ({}): {}",
                                        cea.vics.len(),
                                        vic_strs.join(", ")
                                    );
                                }
                            }

                            if let (Some(min_v), Some(max_v)) = (caps.min_v_rate, caps.max_v_rate) {
                                println!("  V-Range:      {} - {} Hz", min_v, max_v);
                            }
                            if let (Some(min_h), Some(max_h)) =
                                (caps.min_h_rate_khz, caps.max_h_rate_khz)
                            {
                                println!("  H-Range:      {} - {} kHz", min_h, max_h);
                            }
                            if let Some(clock) = caps.max_pixel_clock_mhz {
                                println!("  Max Clock:    {} MHz", clock);
                            }

                            // Read back additional typed data stored by CeaDetailsHandler
                            if let Some(cea) = caps.get_extension_data::<CeaDetails>(0xF2) {
                                println!("  CEA Version:   {}", cea.version);
                                println!("  CEA DTD Offset: {}", cea.dtd_offset);
                            }

                            if !caps.white_points.is_empty() {
                                println!("  White points ({}):", caps.white_points.len());
                                for wp in &caps.white_points {
                                    let g = wp.gamma.map(|g| g.value());
                                    println!(
                                        "    [{}] ({:.4},{:.4}) gamma={:?}",
                                        wp.index,
                                        wp.chromaticity.x(),
                                        wp.chromaticity.y(),
                                        g,
                                    );
                                }
                            }
                            if !caps.warnings.is_empty() {
                                println!("  Warnings ({}):", caps.warnings.len());
                                for warning in &caps.warnings {
                                    println!("    - {:?}", warning);
                                }
                            }

                            if !caps.supported_modes.is_empty() {
                                println!("  Supported Modes ({}):", caps.supported_modes.len());
                                for mode in caps.supported_modes.iter() {
                                    println!(
                                        "    - {}x{}@{}Hz",
                                        mode.width, mode.height, mode.refresh_rate
                                    );
                                }
                            }

                            if !parsed.warnings.is_empty() {
                                println!("  Parse warnings: {:?}", parsed.warnings);
                            }
                        }
                        Err(e) => {
                            println!("  Error parsing EDID: {:?}", e);
                        }
                    }
                    found += 1;
                }
            }
        }
    }

    if found == 0 {
        println!("No active displays with EDID data were found.");
    } else {
        println!("\n--- Finished. Found {} display(s). ---", found);
    }
}
