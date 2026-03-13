use std::fs;
use std::path::Path;
use piaf::{parse_edid, capabilities_from_edid, ExtensionLibrary};

fn main() {
    println!("--- Searching for connected displays (EDID) ---");

    let drm_path = Path::new("/sys/class/drm");
    if !drm_path.exists() {
        eprintln!("Error: /sys/class/drm not found. This script only works on Linux.");
        return;
    }

    let library = ExtensionLibrary::new();
    let registry = library.export_tags();
    let mut found = 0;

    if let Ok(entries) = fs::read_dir(drm_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Check for edid file in the connector directory
            let edid_path = path.join("edid");
            if edid_path.exists() {
                if let Ok(bytes) = fs::read(&edid_path) {
                    if bytes.is_empty() {
                        continue;
                    }

                    // A common "null" EDID is all zeros
                    if bytes.iter().all(|&b| b == 0) {
                        continue;
                    }

                    println!("\nFound connector: {}", name);
                    println!("EDID file: {}", edid_path.display());
                    println!("Data size: {} bytes", bytes.len());

                    match parse_edid(&bytes, &registry) {
                        Ok(parsed) => {
                            let caps = capabilities_from_edid(&parsed);
                            
                            println!("  Manufacturer: {:?}", caps.manufacturer.as_deref().unwrap_or("Unknown"));
                            println!("  Display Name: {:?}", caps.display_name.as_deref().unwrap_or("Unknown"));
                            println!("  Product Code: {:?}", caps.product_code);
                            println!("  Serial:       {:?}", caps.serial_number);
                            println!("  Dimensions:   {:?}x{:?} cm", caps.width_cm, caps.height_cm);
                            println!("  Input type:   {}", if caps.digital { "Digital" } else { "Analog" });
                            println!("  Audio support: {}", if caps.has_audio { "Yes" } else { "No" });

                            if let (Some(min_v), Some(max_v)) = (caps.min_v_rate, caps.max_v_rate) {
                                println!("  V-Range:      {} - {} Hz", min_v, max_v);
                            }
                            if let (Some(min_h), Some(max_h)) = (caps.min_h_rate_khz, caps.max_h_rate_khz) {
                                println!("  H-Range:      {} - {} kHz", min_h, max_h);
                            }
                            if let Some(clock) = caps.max_pixel_clock_mhz {
                                println!("  Max Clock:    {} MHz", clock);
                            }
                            
                            if !caps.warnings.is_empty() {
                                println!("  Warnings ({}):", caps.warnings.len());
                                for warning in &caps.warnings {
                                    println!("    - {:?}", warning);
                                }
                            }
                            
                            if !caps.supported_modes.is_empty() {
                                println!("  Supported Modes ({}):", caps.supported_modes.len());
                                for mode in caps.supported_modes.iter().take(5) {
                                    println!("    - {}x{}@{}Hz", mode.width, mode.height, mode.refresh_rate);
                                }
                                if caps.supported_modes.len() > 5 {
                                    println!("    ... and {} more", caps.supported_modes.len() - 5);
                                }
                            }

                            if !parsed.warnings.is_empty() {
                                println!("  Warnings: {:?}", parsed.warnings);
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
