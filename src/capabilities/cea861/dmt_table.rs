use crate::model::capabilities::{StereoMode, SyncDefinition, VideoMode};

/// Look up a VESA DMT ID and return the corresponding `VideoMode`, or `None`
/// if the ID is not in the standard table.
///
/// Source: VESA DMT v1.13, Table 2-1 (cross-referenced against Linux kernel
/// `drivers/gpu/drm/drm_edid.c`).
///
/// When two DMT IDs share the same resolution and refresh rate (one Reduced
/// Blanking variant, one standard), both map to distinct `VideoMode` values with
/// different timing parameters. The interlaced 1024×768@43 Hz entry (0x0F) is
/// included; 0x58 (4096×2160 @ 59.94 Hz) is stored as 60 Hz because `VideoMode`
/// uses integer refresh rates.
pub(crate) fn dmt_to_mode(id: u16) -> Option<VideoMode> {
    // Columns: width, height, refresh_rate, interlaced,
    //          h_front_porch, h_sync_width, v_front_porch, v_sync_width,
    //          h_sync_positive, v_sync_positive
    macro_rules! e {
        ($w:expr, $h:expr, $rr:expr, $i:expr,
         $hfp:expr, $hsw:expr, $vfp:expr, $vsw:expr,
         $hp:expr, $vp:expr) => {
            VideoMode {
                width: $w,
                height: $h,
                refresh_rate: $rr,
                interlaced: $i,
                h_front_porch: $hfp,
                h_sync_width: $hsw,
                v_front_porch: $vfp,
                v_sync_width: $vsw,
                h_border: 0,
                v_border: 0,
                stereo: StereoMode::None,
                sync: Some(SyncDefinition::DigitalSeparate {
                    h_sync_positive: $hp,
                    v_sync_positive: $vp,
                }),
            }
        };
    }

    Some(match id {
        // 640×350
        0x01 => e!(640, 350, 85, false, 32, 64, 32, 3, true, false),
        // 640×400
        0x02 => e!(640, 400, 85, false, 32, 64, 1, 3, false, true),
        // 720×400
        0x03 => e!(720, 400, 85, false, 36, 72, 1, 3, false, true),
        // 640×480
        0x04 => e!(640, 480, 60, false, 16, 96, 10, 2, false, false),
        0x05 => e!(640, 480, 72, false, 24, 40, 9, 3, false, false),
        0x06 => e!(640, 480, 75, false, 16, 64, 1, 3, false, false),
        0x07 => e!(640, 480, 85, false, 56, 56, 1, 3, false, false),
        // 800×600
        0x08 => e!(800, 600, 56, false, 24, 72, 1, 2, true, true),
        0x09 => e!(800, 600, 60, false, 40, 128, 1, 4, true, true),
        0x0A => e!(800, 600, 72, false, 56, 120, 37, 6, true, true),
        0x0B => e!(800, 600, 75, false, 16, 80, 1, 3, true, true),
        0x0C => e!(800, 600, 85, false, 32, 64, 1, 3, true, true),
        0x0D => e!(800, 600, 120, false, 48, 32, 3, 4, true, false), // RB
        // 848×480
        0x0E => e!(848, 480, 60, false, 16, 112, 6, 8, true, true),
        // 1024×768i (interlaced)
        0x0F => e!(1024, 768, 43, true, 8, 176, 0, 4, false, false),
        // 1024×768
        0x10 => e!(1024, 768, 60, false, 24, 136, 3, 6, false, false),
        0x11 => e!(1024, 768, 70, false, 24, 136, 3, 6, false, false),
        0x12 => e!(1024, 768, 75, false, 16, 96, 1, 3, true, true),
        0x13 => e!(1024, 768, 85, false, 48, 96, 1, 3, true, true),
        0x14 => e!(1024, 768, 120, false, 48, 32, 3, 4, true, false), // RB
        // 1152×864
        0x15 => e!(1152, 864, 75, false, 64, 128, 1, 3, true, true),
        // 1280×768
        0x16 => e!(1280, 768, 60, false, 48, 32, 3, 7, true, false), // RB
        0x17 => e!(1280, 768, 60, false, 64, 128, 3, 7, false, true),
        0x18 => e!(1280, 768, 75, false, 80, 128, 3, 7, false, true),
        0x19 => e!(1280, 768, 85, false, 80, 136, 3, 7, false, true),
        0x1A => e!(1280, 768, 120, false, 48, 32, 3, 7, true, false), // RB
        // 1280×800
        0x1B => e!(1280, 800, 60, false, 48, 32, 3, 6, true, false), // RB
        0x1C => e!(1280, 800, 60, false, 72, 128, 3, 6, false, true),
        0x1D => e!(1280, 800, 75, false, 80, 128, 3, 6, false, true),
        0x1E => e!(1280, 800, 85, false, 80, 136, 3, 6, false, true),
        0x1F => e!(1280, 800, 120, false, 48, 32, 3, 6, true, false), // RB
        // 1280×960
        0x20 => e!(1280, 960, 60, false, 96, 112, 1, 3, true, true),
        0x21 => e!(1280, 960, 85, false, 64, 160, 1, 3, true, true),
        0x22 => e!(1280, 960, 120, false, 48, 32, 3, 4, true, false), // RB
        // 1280×1024
        0x23 => e!(1280, 1024, 60, false, 48, 112, 1, 3, true, true),
        0x24 => e!(1280, 1024, 75, false, 16, 144, 1, 3, true, true),
        0x25 => e!(1280, 1024, 85, false, 64, 160, 1, 3, true, true),
        0x26 => e!(1280, 1024, 120, false, 48, 32, 3, 7, true, false), // RB
        // 1360×768
        0x27 => e!(1360, 768, 60, false, 64, 112, 3, 6, true, true),
        0x28 => e!(1360, 768, 120, false, 48, 32, 3, 5, true, false), // RB
        // 1400×1050
        0x29 => e!(1400, 1050, 60, false, 48, 32, 3, 4, true, false), // RB
        0x2A => e!(1400, 1050, 60, false, 88, 144, 3, 4, false, true),
        0x2B => e!(1400, 1050, 75, false, 104, 144, 3, 4, false, true),
        0x2C => e!(1400, 1050, 85, false, 104, 152, 3, 4, false, true),
        0x2D => e!(1400, 1050, 120, false, 48, 32, 3, 4, true, false), // RB
        // 1440×900
        0x2E => e!(1440, 900, 60, false, 48, 32, 3, 6, true, false), // RB
        0x2F => e!(1440, 900, 60, false, 80, 152, 3, 6, false, true),
        0x30 => e!(1440, 900, 75, false, 96, 152, 3, 6, false, true),
        0x31 => e!(1440, 900, 85, false, 104, 152, 3, 6, false, true),
        0x32 => e!(1440, 900, 120, false, 48, 32, 3, 6, true, false), // RB
        // 1600×1200
        0x33 => e!(1600, 1200, 60, false, 64, 192, 1, 3, true, true),
        0x34 => e!(1600, 1200, 65, false, 64, 192, 1, 3, true, true),
        0x35 => e!(1600, 1200, 70, false, 64, 192, 1, 3, true, true),
        0x36 => e!(1600, 1200, 75, false, 64, 192, 1, 3, true, true),
        0x37 => e!(1600, 1200, 85, false, 64, 192, 1, 3, true, true),
        0x38 => e!(1600, 1200, 120, false, 48, 32, 3, 4, true, false), // RB
        // 1680×1050
        0x39 => e!(1680, 1050, 60, false, 48, 32, 3, 6, true, false), // RB
        0x3A => e!(1680, 1050, 60, false, 104, 176, 3, 6, false, true),
        0x3B => e!(1680, 1050, 75, false, 120, 176, 3, 6, false, true),
        0x3C => e!(1680, 1050, 85, false, 128, 176, 3, 6, false, true),
        0x3D => e!(1680, 1050, 120, false, 48, 32, 3, 6, true, false), // RB
        // 1792×1344
        0x3E => e!(1792, 1344, 60, false, 128, 200, 1, 3, false, true),
        0x3F => e!(1792, 1344, 75, false, 96, 216, 1, 3, false, true),
        0x40 => e!(1792, 1344, 120, false, 48, 32, 3, 4, true, false), // RB
        // 1856×1392
        0x41 => e!(1856, 1392, 60, false, 96, 224, 1, 3, false, true),
        0x42 => e!(1856, 1392, 75, false, 128, 224, 1, 3, false, true),
        0x43 => e!(1856, 1392, 120, false, 48, 32, 3, 4, true, false), // RB
        // 1920×1200
        0x44 => e!(1920, 1200, 60, false, 48, 32, 3, 6, true, false), // RB
        0x45 => e!(1920, 1200, 60, false, 136, 200, 3, 6, false, true),
        0x46 => e!(1920, 1200, 75, false, 136, 208, 3, 6, false, true),
        0x47 => e!(1920, 1200, 85, false, 144, 208, 3, 6, false, true),
        0x48 => e!(1920, 1200, 120, false, 48, 32, 3, 6, true, false), // RB
        // 1920×1440
        0x49 => e!(1920, 1440, 60, false, 128, 208, 1, 3, false, true),
        0x4A => e!(1920, 1440, 75, false, 144, 224, 1, 3, false, true),
        0x4B => e!(1920, 1440, 120, false, 48, 32, 3, 4, true, false), // RB
        // 2560×1600
        0x4C => e!(2560, 1600, 60, false, 48, 32, 3, 6, true, false), // RB
        0x4D => e!(2560, 1600, 60, false, 192, 280, 3, 6, false, true),
        0x4E => e!(2560, 1600, 75, false, 208, 280, 3, 6, false, true),
        0x4F => e!(2560, 1600, 85, false, 208, 280, 3, 6, false, true),
        0x50 => e!(2560, 1600, 120, false, 48, 32, 3, 6, true, false), // RB
        // Additional VESA DMT 1.13 entries (codes 0x51–0x58)
        0x51 => e!(1366, 768, 60, false, 70, 143, 3, 3, true, true),
        0x52 => e!(1920, 1080, 60, false, 88, 44, 4, 5, true, true),
        0x53 => e!(1600, 900, 60, false, 48, 32, 3, 5, true, false), // RB
        0x54 => e!(2048, 1152, 60, false, 48, 32, 3, 4, true, false), // RB
        0x55 => e!(1280, 720, 60, false, 110, 40, 5, 5, true, true),
        0x56 => e!(1366, 768, 60, false, 14, 56, 1, 3, true, true),
        0x57 => e!(4096, 2160, 60, false, 8, 32, 48, 8, true, true),
        0x58 => e!(4096, 2160, 60, false, 8, 32, 48, 8, true, true), // 59.94 Hz, stored as 60
        _ => return None,
    })
}
