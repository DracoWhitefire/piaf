mod coded;
mod detailed;
mod short;

use super::{
    TAG_CTA_VIDEO_TIMING, TAG_TYPE_I_TIMING, TAG_TYPE_II_TIMING, TAG_TYPE_III_TIMING,
    TAG_TYPE_IV_TIMING, TAG_TYPE_V_TIMING, TAG_TYPE_VI_TIMING, TAG_VESA_VIDEO_TIMING,
    for_each_data_block,
};
use crate::model::capabilities::ModeSink;
use coded::{decode_cta_video_timing_block, decode_type_iv_block, decode_vesa_video_timing_block};
use detailed::{decode_type_i_descriptor, decode_type_ii_descriptor, decode_type_vi_descriptor};
use short::{decode_type_iii_descriptor, decode_type_v_descriptor};

/// Iterates DisplayID 1.x data blocks within a fragment's payload region and pushes
/// decoded modes to `sink`.
///
/// `payload` must be the data-block region: bytes `block[4..4+section_byte_count]`,
/// clamped to `block[4..127]` to exclude the checksum byte.
pub(super) fn process_data_blocks(payload: &[u8], sink: &mut dyn ModeSink) {
    for_each_data_block(payload, |tag, revision, block_payload| {
        if tag == TAG_TYPE_I_TIMING {
            let mut i = 0;
            while i + 20 <= block_payload.len() {
                let descriptor: &[u8; 20] = block_payload[i..i + 20].try_into().expect("slice length guaranteed by loop condition");
                decode_type_i_descriptor(descriptor, sink);
                i += 20;
            }
        } else if tag == TAG_TYPE_II_TIMING {
            let mut i = 0;
            while i + 11 <= block_payload.len() {
                let descriptor: &[u8; 11] = block_payload[i..i + 11].try_into().expect("slice length guaranteed by loop condition");
                decode_type_ii_descriptor(descriptor, sink);
                i += 11;
            }
        } else if tag == TAG_TYPE_III_TIMING {
            let mut i = 0;
            while i + 3 <= block_payload.len() {
                let descriptor: &[u8; 3] = block_payload[i..i + 3].try_into().expect("slice length guaranteed by loop condition");
                decode_type_iii_descriptor(descriptor, sink);
                i += 3;
            }
        } else if tag == TAG_TYPE_IV_TIMING {
            // Revision byte bits 7:6 select the code space (0=DMT, 1=VIC, 2=HDMI VIC).
            let code_type = (revision >> 6) & 0x03;
            decode_type_iv_block(block_payload, code_type, sink);
        } else if tag == TAG_VESA_VIDEO_TIMING {
            decode_vesa_video_timing_block(block_payload, sink);
        } else if tag == TAG_CTA_VIDEO_TIMING {
            decode_cta_video_timing_block(block_payload, sink);
        } else if tag == TAG_TYPE_V_TIMING {
            let mut i = 0;
            while i + 7 <= block_payload.len() {
                let descriptor: &[u8; 7] = block_payload[i..i + 7].try_into().expect("slice length guaranteed by loop condition");
                decode_type_v_descriptor(descriptor, sink);
                i += 7;
            }
        } else if tag == TAG_TYPE_VI_TIMING {
            let mut i = 0;
            while i + 14 <= block_payload.len() {
                let consumed = decode_type_vi_descriptor(&block_payload[i..], sink);
                if consumed == 0 {
                    break;
                }
                i += consumed;
            }
        }
        // Unknown block tags are silently skipped.
    });
}

#[cfg(test)]
#[cfg(any(feature = "alloc", feature = "std"))]
mod tests {
    use super::*;
    use crate::model::capabilities::{
        DisplayCapabilities, StaticContext, StaticDisplayCapabilities,
    };

    fn make_type_i_descriptor(
        pixel_clock_10khz: u16,
        h_active: u16,
        h_blank: u16,
        h_fp: u16,
        h_sw: u16,
        v_active: u16,
        v_blank: u16,
        v_fp: u16,
        v_sw: u16,
        flags: u8,
    ) -> [u8; 20] {
        let mut d = [0u8; 20];
        d[1..3].copy_from_slice(&pixel_clock_10khz.to_le_bytes());
        d[3..5].copy_from_slice(&h_active.to_le_bytes());
        d[5..7].copy_from_slice(&h_blank.to_le_bytes());
        d[7..9].copy_from_slice(&h_fp.to_le_bytes());
        d[9..11].copy_from_slice(&h_sw.to_le_bytes());
        d[11..13].copy_from_slice(&v_active.to_le_bytes());
        d[13..15].copy_from_slice(&v_blank.to_le_bytes());
        d[15..17].copy_from_slice(&v_fp.to_le_bytes());
        d[17..19].copy_from_slice(&v_sw.to_le_bytes());
        d[19] = flags;
        d
    }

    // -----------------------------------------------------------------------
    // Dispatcher behaviour
    // -----------------------------------------------------------------------

    #[test]
    fn test_multiple_descriptors_in_block() {
        let desc1 = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let desc2 = make_type_i_descriptor(22118, 2560, 440, 80, 32, 1440, 41, 4, 5, 0x00);
        let mut payload = vec![TAG_TYPE_I_TIMING, 0x00, 40];
        payload.extend_from_slice(&desc1);
        payload.extend_from_slice(&desc2);
        let mut caps = DisplayCapabilities::default();
        process_data_blocks(&payload, &mut caps);
        assert_eq!(caps.supported_modes.len(), 2);
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 1920 && m.height == 1080)
        );
        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 2560 && m.height == 1440)
        );
    }

    #[test]
    fn test_malformed_data_block_stops_iteration() {
        // A data block that claims a length extending past the payload boundary.
        let payload = [TAG_TYPE_I_TIMING, 0x00, 50]; // claims 50 bytes; 0 remain
        let mut caps = DisplayCapabilities::default();
        process_data_blocks(&payload, &mut caps);
        assert!(caps.supported_modes.is_empty());
    }

    #[test]
    fn test_eos_sentinel_stops_iteration() {
        // A valid Type I descriptor followed by an EOS sentinel; a second descriptor must not decode.
        let desc = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let desc2 = make_type_i_descriptor(22118, 2560, 440, 80, 32, 1440, 41, 4, 5, 0x00);
        let mut payload = vec![TAG_TYPE_I_TIMING, 0x00, 20];
        payload.extend_from_slice(&desc);
        payload.extend_from_slice(&[0x00, 0x00, 0x00]); // EOS sentinel
        payload.extend_from_slice(&[TAG_TYPE_I_TIMING, 0x00, 20]);
        payload.extend_from_slice(&desc2);
        let mut caps = DisplayCapabilities::default();
        process_data_blocks(&payload, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 1920);
    }

    #[test]
    fn test_unknown_data_block_tag_skipped() {
        // An unknown data block (tag 0xFF, 10-byte payload) followed by a valid Type I block.
        let desc = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let mut payload = vec![0xFFu8, 0x00, 10];
        payload.extend_from_slice(&[0u8; 10]);
        payload.extend_from_slice(&[TAG_TYPE_I_TIMING, 0x00, 20]);
        payload.extend_from_slice(&desc);
        let mut caps = DisplayCapabilities::default();
        process_data_blocks(&payload, &mut caps);
        assert_eq!(caps.supported_modes.len(), 1);
        assert_eq!(caps.supported_modes[0].width, 1920);
    }

    #[test]
    fn test_partial_descriptor_ignored() {
        // A Type II block with only 10 bytes of payload — one short of a full descriptor.
        let mut payload = vec![TAG_TYPE_II_TIMING, 0x00, 10];
        payload.extend_from_slice(&[0u8; 10]);
        let mut caps = DisplayCapabilities::default();
        process_data_blocks(&payload, &mut caps);
        assert!(caps.supported_modes.is_empty());
    }

    // -----------------------------------------------------------------------
    // Static pipeline (process_data_blocks with StaticContext)
    // -----------------------------------------------------------------------

    #[test]
    fn test_type_i_static_pipeline() {
        let d = make_type_i_descriptor(14850, 1920, 280, 88, 44, 1080, 45, 4, 5, 0x00);
        let mut payload = vec![TAG_TYPE_I_TIMING, 0x00, 20];
        payload.extend_from_slice(&d);
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
    }

    #[test]
    fn test_type_ii_static_pipeline() {
        // ha_raw=239→h=1920, va_raw=1079→v=1080, pixel_clock raw=15153→actual=15154×10kHz
        let mut payload = vec![TAG_TYPE_II_TIMING, 0x00, 11];
        let mut d = [0u8; 11];
        d[0] = 15153u32 as u8;
        d[1] = (15153u32 >> 8) as u8;
        d[3] = 0x0C; // H+, V+ sync
        d[4] = (239u16 & 0xFF) as u8;
        d[5] = (((239u16 >> 8) & 0x01) as u8) | ((34u8 & 0x7F) << 1);
        d[6] = ((10u8 & 0x0F) << 4) | (5u8 & 0x0F);
        d[7] = (1079u16 & 0xFF) as u8;
        d[8] = ((1079u16 >> 8) & 0x0F) as u8;
        d[9] = 0x43;
        payload.extend_from_slice(&d);
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
    }

    #[test]
    fn test_type_iii_static_pipeline() {
        // 1920×1080@60: aspect=16:9(0x04), h_raw=239→h=1920, refresh_raw=59→60 Hz
        let byte0 = 0x04u8; // aspect 16:9
        let payload = vec![TAG_TYPE_III_TIMING, 0x00, 3, byte0, 239, 59];
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
    }

    #[test]
    fn test_type_iv_static_pipeline() {
        // DMT 0x09 = 800×600@60 Hz; code_type=0 (DMT) → revision bits 7:6 = 0b00
        let payload = vec![TAG_TYPE_IV_TIMING, 0x00, 1, 0x09];
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 800);
        assert_eq!(mode.height, 600);
        assert_eq!(mode.refresh_rate, 60);
    }

    #[test]
    fn test_vesa_timing_static_pipeline() {
        // Byte 1 bit 0 = DMT 0x09 = 800×600@60 Hz.
        let payload = vec![TAG_VESA_VIDEO_TIMING, 0x00, 2, 0x00, 0x01];
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 800);
        assert_eq!(mode.height, 600);
        assert_eq!(mode.refresh_rate, 60);
    }

    #[test]
    fn test_cta_timing_static_pipeline() {
        // Byte 0 bit 0 = VIC 1 = 640×480@60 Hz.
        let payload = vec![TAG_CTA_VIDEO_TIMING, 0x00, 1, 0x01];
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        assert_eq!(caps.supported_modes[0].as_ref().unwrap().width, 640);
    }

    #[test]
    fn test_type_v_static_pipeline() {
        // 1920×1080@60 Hz: h=1920 (LE), v=1080 (LE), refresh_raw=59→60 Hz
        let mut payload = vec![TAG_TYPE_V_TIMING, 0x00, 7];
        payload.extend_from_slice(&[0x00]); // options
        payload.extend_from_slice(&1920u16.to_le_bytes()); // h_active
        payload.extend_from_slice(&1080u16.to_le_bytes()); // v_active
        payload.push(59); // refresh_raw
        payload.push(0x00); // reserved
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
    }

    #[test]
    fn test_type_vi_static_pipeline() {
        // 1920×1080@60 Hz: pixel clock 148_500 kHz, h_total=2200, v_total=1125
        let pc: u32 = 148_500;
        let h_word: u16 = 1920 | (1 << 15); // H sync +
        let v_word: u16 = 1080 | (1 << 15); // V sync +
        let h_blank: u16 = 280;
        let v_blank: u8 = 45;
        let mut descriptor = [0u8; 14];
        descriptor[0] = (pc & 0xFF) as u8;
        descriptor[1] = ((pc >> 8) & 0xFF) as u8;
        descriptor[2] = ((pc >> 16) & 0x3F) as u8;
        descriptor[3..5].copy_from_slice(&h_word.to_le_bytes());
        descriptor[5..7].copy_from_slice(&v_word.to_le_bytes());
        descriptor[7] = (h_blank & 0xFF) as u8;
        descriptor[9] = ((h_blank >> 8) & 0x0F) as u8;
        descriptor[11] = v_blank;
        let mut payload = vec![TAG_TYPE_VI_TIMING, 0x00, 14];
        payload.extend_from_slice(&descriptor);
        let mut caps = StaticDisplayCapabilities::<16>::default();
        let mut ctx = StaticContext::new(&mut caps);
        process_data_blocks(&payload, &mut ctx);
        assert_eq!(caps.num_modes, 1);
        let mode = caps.supported_modes[0].as_ref().unwrap();
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert_eq!(mode.refresh_rate, 60);
    }
}
