use crate::model::capabilities::{StereoMode, SyncDefinition, VideoMode};

/// Returns the [`VideoMode`] for the given CEA-861-E Video Identification Code (VIC),
/// or `None` if the VIC is not in the table (valid range: 1–64).
///
/// Timing parameters are sourced from CEA-861-E Table 1. Vertical timings for
/// interlaced modes are given per-field, matching the convention used in EDID
/// detailed timing descriptors.
pub(super) fn vic_to_mode(vic: u8) -> Option<VideoMode> {
    // Columns: (width, height, refresh_hz, interlaced,
    //           h_front_porch, h_sync_width,
    //           v_front_porch, v_sync_width,
    //           v_sync_positive, h_sync_positive)
    let (w, h, r, interlaced, hfp, hsw, vfp, vsw, vpos, hpos): (
        u16, u16, u8, bool, u16, u16, u16, u16, bool, bool,
    ) = match vic {
        // --- NTSC-derived 60 Hz family ---
        1  => (640,  480,  60,  false,   16,  96,  10, 2, false, false),
        2  => (720,  480,  60,  false,   16,  62,   9, 6, false, false),
        3  => (720,  480,  60,  false,   16,  62,   9, 6, false, false),
        4  => (1280, 720,  60,  false,  110,  40,   5, 5, true,  true ),
        5  => (1920, 1080, 60,  true,    88,  44,   2, 5, true,  true ),
        6  => (1440, 480,  60,  true,    38, 124,   4, 3, false, false),
        7  => (1440, 480,  60,  true,    38, 124,   4, 3, false, false),
        8  => (1440, 240,  60,  false,   38, 124,   4, 3, false, false),
        9  => (1440, 240,  60,  false,   38, 124,   4, 3, false, false),
        10 => (2880, 480,  60,  true,    76, 248,   4, 3, false, false),
        11 => (2880, 480,  60,  true,    76, 248,   4, 3, false, false),
        12 => (2880, 240,  60,  false,   76, 248,   4, 3, false, false),
        13 => (2880, 240,  60,  false,   76, 248,   4, 3, false, false),
        14 => (1440, 480,  60,  false,   32, 124,   9, 6, false, false),
        15 => (1440, 480,  60,  false,   32, 124,   9, 6, false, false),
        16 => (1920, 1080, 60,  false,   88,  44,   4, 5, true,  true ),
        // --- PAL-derived 50 Hz family ---
        17 => (720,  576,  50,  false,   12,  64,   5, 5, false, false),
        18 => (720,  576,  50,  false,   12,  64,   5, 5, false, false),
        19 => (1280, 720,  50,  false,  440,  40,   5, 5, true,  true ),
        20 => (1920, 1080, 50,  true,   528,  44,   2, 5, true,  true ),
        21 => (1440, 576,  50,  true,    24, 126,   2, 3, false, false),
        22 => (1440, 576,  50,  true,    24, 126,   2, 3, false, false),
        23 => (1440, 288,  50,  false,   24, 126,   2, 3, false, false),
        24 => (1440, 288,  50,  false,   24, 126,   2, 3, false, false),
        25 => (2880, 576,  50,  true,    48, 252,   2, 3, false, false),
        26 => (2880, 576,  50,  true,    48, 252,   2, 3, false, false),
        27 => (2880, 288,  50,  false,   48, 252,   2, 3, false, false),
        28 => (2880, 288,  50,  false,   48, 252,   2, 3, false, false),
        29 => (1440, 576,  50,  false,   24, 128,   5, 5, false, false),
        30 => (1440, 576,  50,  false,   24, 128,   5, 5, false, false),
        31 => (1920, 1080, 50,  false,  528,  44,   4, 5, true,  true ),
        // --- 1080p low-rate variants ---
        32 => (1920, 1080, 24,  false,  638,  44,   4, 5, true,  true ),
        33 => (1920, 1080, 25,  false,  528,  44,   4, 5, true,  true ),
        34 => (1920, 1080, 30,  false,   88,  44,   4, 5, true,  true ),
        // --- 2880-wide pixel-quadrupled formats ---
        35 => (2880, 480,  60,  false,   64, 248,   9, 6, false, false),
        36 => (2880, 480,  60,  false,   64, 248,   9, 6, false, false),
        37 => (2880, 576,  50,  false,   48, 256,   5, 5, false, false),
        38 => (2880, 576,  50,  false,   48, 256,   5, 5, false, false),
        // --- SMPTE 295M 1080i50 (1250-line system) ---
        39 => (1920, 1080, 50,  true,    32, 168,  23, 5, false, true ),
        // --- 100 Hz family ---
        40 => (1920, 1080, 100, true,   528,  44,   2, 5, true,  true ),
        41 => (1280, 720,  100, false,  440,  40,   5, 5, true,  true ),
        42 => (720,  576,  100, false,   12,  64,   5, 5, false, false),
        43 => (720,  576,  100, false,   12,  64,   5, 5, false, false),
        44 => (1440, 576,  100, true,    24, 126,   2, 3, false, false),
        45 => (1440, 576,  100, true,    24, 126,   2, 3, false, false),
        // --- 120 Hz family ---
        46 => (1920, 1080, 120, true,    88,  44,   2, 5, true,  true ),
        47 => (1280, 720,  120, false,  110,  40,   5, 5, true,  true ),
        48 => (720,  480,  120, false,   16,  62,   9, 6, false, false),
        49 => (720,  480,  120, false,   16,  62,   9, 6, false, false),
        50 => (1440, 480,  120, true,    38, 124,   4, 3, false, false),
        51 => (1440, 480,  120, true,    38, 124,   4, 3, false, false),
        // --- 200 Hz family ---
        52 => (720,  576,  200, false,   12,  64,   5, 5, false, false),
        53 => (720,  576,  200, false,   12,  64,   5, 5, false, false),
        54 => (1440, 576,  200, true,    24, 126,   2, 3, false, false),
        55 => (1440, 576,  200, true,    24, 126,   2, 3, false, false),
        // --- 240 Hz family ---
        56 => (720,  480,  240, false,   16,  62,   9, 6, false, false),
        57 => (720,  480,  240, false,   16,  62,   9, 6, false, false),
        58 => (1440, 480,  240, true,    38, 124,   4, 3, false, false),
        59 => (1440, 480,  240, true,    38, 124,   4, 3, false, false),
        // --- 720p low-rate variants ---
        // Htotal=3300 for 24/30 Hz; Htotal=3960 for 25 Hz (same blanking, different pixel clock)
        60 => (1280, 720,  24,  false, 1760,  40,   5, 5, true,  true ),
        61 => (1280, 720,  25,  false, 2420,  40,   5, 5, true,  true ),
        62 => (1280, 720,  30,  false, 1760,  40,   5, 5, true,  true ),
        // --- 1080p high-rate ---
        63 => (1920, 1080, 120, false,   88,  44,   4, 5, true,  true ),
        64 => (1920, 1080, 100, false,  528,  44,   4, 5, true,  true ),
        _  => return None,
    };

    Some(VideoMode {
        width: w,
        height: h,
        refresh_rate: r,
        interlaced,
        h_front_porch: hfp,
        h_sync_width: hsw,
        v_front_porch: vfp,
        v_sync_width: vsw,
        h_border: 0,
        v_border: 0,
        stereo: StereoMode::None,
        sync: Some(SyncDefinition::DigitalSeparate {
            v_sync_positive: vpos,
            h_sync_positive: hpos,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vic1_640x480p60() {
        let mode = vic_to_mode(1).unwrap();
        assert_eq!(mode.width, 640);
        assert_eq!(mode.height, 480);
        assert_eq!(mode.refresh_rate, 60);
        assert!(!mode.interlaced);
        assert_eq!(mode.h_front_porch, 16);
        assert_eq!(mode.h_sync_width, 96);
        assert_eq!(mode.v_front_porch, 10);
        assert_eq!(mode.v_sync_width, 2);
    }

    #[test]
    fn test_vic16_1080p60() {
        let mode = vic_to_mode(16).unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
        assert!(!mode.interlaced);
        assert!(matches!(
            mode.sync,
            Some(SyncDefinition::DigitalSeparate {
                v_sync_positive: true,
                h_sync_positive: true
            })
        ));
    }

    #[test]
    fn test_vic5_1080i60_interlaced() {
        let mode = vic_to_mode(5).unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
        assert!(mode.interlaced);
    }

    #[test]
    fn test_vic63_1080p120() {
        let mode = vic_to_mode(63).unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 120);
        assert!(!mode.interlaced);
    }

    #[test]
    fn test_vic0_is_none() {
        assert!(vic_to_mode(0).is_none());
    }

    #[test]
    fn test_vic65_is_none() {
        assert!(vic_to_mode(65).is_none());
    }

    #[test]
    fn test_all_vics_1_to_64_are_some() {
        for vic in 1..=64 {
            assert!(vic_to_mode(vic).is_some(), "VIC {vic} returned None");
        }
    }
}
