/// IEEE OUI for HDMI Licensing, LLC: `0x000C03` (stored little-endian as `[0x03, 0x0C, 0x00]`).
pub(super) const HDMI_OUI: [u8; 3] = [0x03, 0x0C, 0x00];

bitflags::bitflags! {
    /// Capability flags from byte 5 of the HDMI VSDB payload (byte 8 of the CEA block,
    /// after the 3-byte header and 3-byte OUI).
    ///
    /// | Bit | Mask   | Meaning                                  |
    /// |-----|--------|------------------------------------------|
    /// | 7   | `0x80` | Supports ACP / ISRC packets (`SUPPORTS_AI`) |
    /// | 6   | `0x40` | 48-bit deep color                        |
    /// | 5   | `0x20` | 36-bit deep color                        |
    /// | 4   | `0x10` | 30-bit deep color                        |
    /// | 3   | `0x08` | YCbCr 4:4:4 in deep color modes          |
    /// | 0   | `0x01` | DVI dual-link                            |
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct HdmiVsdbFlags: u8 {
        /// Sink supports ACP, ISRC1, and ISRC2 packets.
        const SUPPORTS_AI = 0x80;
        /// Sink supports 48-bit (16 bpc) deep color.
        const DC_48BIT    = 0x40;
        /// Sink supports 36-bit (12 bpc) deep color.
        const DC_36BIT    = 0x20;
        /// Sink supports 30-bit (10 bpc) deep color.
        const DC_30BIT    = 0x10;
        /// Sink supports YCbCr 4:4:4 in deep color modes.
        const DC_Y444     = 0x08;
        /// Source supports DVI dual-link.
        const DVI_DUAL    = 0x01;
    }
}

/// Decoded HDMI 1.x Vendor-Specific Data Block (OUI `0x000C03`).
///
/// Stored in [`super::Cea861Capabilities::hdmi_vsdb`] when the CEA extension block
/// contains an HDMI VSDB.
///
/// Field presence depends on the block length:
/// - `source_physical_address` is always present (minimum valid VSDB is 5 bytes after OUI).
/// - `flags` and `max_tmds_clock_mhz` require at least 2 and 3 payload bytes respectively.
/// - Latency fields require the corresponding presence bits in the misc byte.
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HdmiVsdb {
    /// Source physical address encoded as four 4-bit nibbles (A.B.C.D) in a `u16`.
    ///
    /// Nibble layout: `AABBCCDD` where AA = bits 15–12, BB = 11–8, CC = 7–4, DD = 3–0.
    pub source_physical_address: u16,
    /// Deep color and audio capability flags (byte 5 of the VSDB payload).
    ///
    /// `HdmiVsdbFlags::empty()` when the byte is absent (short VSDB).
    pub flags: HdmiVsdbFlags,
    /// Maximum TMDS clock supported by the sink in MHz.
    ///
    /// Decoded as raw byte × 5. `None` when the byte is absent.
    pub max_tmds_clock_mhz: Option<u16>,
    /// Progressive video latency in milliseconds, or `None` if absent or unknown.
    pub video_latency_ms: Option<u16>,
    /// Progressive audio latency in milliseconds, or `None` if absent or unknown.
    pub audio_latency_ms: Option<u16>,
    /// Interlaced video latency in milliseconds, or `None` if absent or unknown.
    pub interlaced_video_latency_ms: Option<u16>,
    /// Interlaced audio latency in milliseconds, or `None` if absent or unknown.
    pub interlaced_audio_latency_ms: Option<u16>,
}

/// Decodes a latency byte per the HDMI 1.x spec:
/// - `0` or `255` → `None` (not defined / unknown)
/// - `1..=251` → `Some((value - 1) * 2)` milliseconds
/// - `252..=254` → `None` (reserved)
fn decode_latency(raw: u8) -> Option<u16> {
    match raw {
        1..=251 => Some((raw as u16 - 1) * 2),
        _ => None,
    }
}

/// Attempts to parse `block_data` (the full VSDB payload, starting with the 3-byte OUI)
/// as an HDMI 1.x VSDB.
///
/// Returns `None` if the OUI does not match or the block is too short to contain a
/// valid source physical address.
pub(super) fn parse_hdmi_vsdb(block_data: &[u8]) -> Option<HdmiVsdb> {
    // Minimum: 3-byte OUI + 2-byte source physical address = 5 bytes.
    if block_data.len() < 5 {
        return None;
    }
    if block_data[0..3] != HDMI_OUI {
        return None;
    }

    let source_physical_address = ((block_data[3] as u16) << 8) | (block_data[4] as u16);

    let flags = block_data
        .get(5)
        .map(|&b| HdmiVsdbFlags::from_bits_truncate(b))
        .unwrap_or_else(HdmiVsdbFlags::empty);

    let max_tmds_clock_mhz = block_data.get(6).map(|&b| (b as u16) * 5);

    // Byte 7: misc flags.
    let latency_present = block_data.get(7).is_some_and(|&b| b & 0x80 != 0);
    let i_latency_present = block_data.get(7).is_some_and(|&b| b & 0x40 != 0);

    // Latency fields start at byte 8, present only when the flag is set.
    let (video_latency_ms, audio_latency_ms) = if latency_present && block_data.len() >= 10 {
        (decode_latency(block_data[8]), decode_latency(block_data[9]))
    } else {
        (None, None)
    };

    let latency_offset = if latency_present { 2 } else { 0 };
    let (interlaced_video_latency_ms, interlaced_audio_latency_ms) =
        if i_latency_present && block_data.len() >= 10 + latency_offset {
            (
                decode_latency(block_data[8 + latency_offset]),
                decode_latency(block_data[9 + latency_offset]),
            )
        } else {
            (None, None)
        };

    Some(HdmiVsdb {
        source_physical_address,
        flags,
        max_tmds_clock_mhz,
        video_latency_ms,
        audio_latency_ms,
        interlaced_video_latency_ms,
        interlaced_audio_latency_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_vsdb() {
        // 3-byte OUI + 2-byte source physical address, no optional bytes.
        let data = [0x03, 0x0C, 0x00, 0x10, 0x00];
        let vsdb = parse_hdmi_vsdb(&data).unwrap();
        assert_eq!(vsdb.source_physical_address, 0x1000);
        assert_eq!(vsdb.flags, HdmiVsdbFlags::empty());
        assert_eq!(vsdb.max_tmds_clock_mhz, None);
        assert_eq!(vsdb.video_latency_ms, None);
    }

    #[test]
    fn test_full_vsdb_with_deep_color() {
        // OUI + SPA + flags (DC_36bit | DC_30bit | DC_Y444) + max_tmds (74 × 5 = 370 MHz)
        let data = [0x03, 0x0C, 0x00, 0x10, 0x00, 0x38, 74, 0x00];
        let vsdb = parse_hdmi_vsdb(&data).unwrap();
        assert!(vsdb.flags.contains(HdmiVsdbFlags::DC_36BIT));
        assert!(vsdb.flags.contains(HdmiVsdbFlags::DC_30BIT));
        assert!(vsdb.flags.contains(HdmiVsdbFlags::DC_Y444));
        assert!(!vsdb.flags.contains(HdmiVsdbFlags::DC_48BIT));
        assert_eq!(vsdb.max_tmds_clock_mhz, Some(370));
    }

    #[test]
    fn test_vsdb_with_latency() {
        // Byte 7: Latency_Fields_Present (0x80); video=51→100ms, audio=1→0ms.
        // latency byte → ms: (byte - 1) * 2. So 51 → 100 ms, 1 → 0 ms.
        let data = [0x03, 0x0C, 0x00, 0x10, 0x00, 0x00, 0x00, 0x80, 51, 1];
        let vsdb = parse_hdmi_vsdb(&data).unwrap();
        assert_eq!(vsdb.video_latency_ms, Some(100));
        assert_eq!(vsdb.audio_latency_ms, Some(0));
        assert_eq!(vsdb.interlaced_video_latency_ms, None);
    }

    #[test]
    fn test_vsdb_with_interlaced_latency() {
        // Byte 7: both Latency_Fields_Present (0x80) and I_Latency_Fields_Present (0x40).
        // Progressive: video=11→20ms, audio=6→10ms.
        // Interlaced: video=21→40ms, audio=16→30ms.
        let data = [
            0x03, 0x0C, 0x00, 0x10, 0x00, // OUI + SPA
            0x00, 0x00, // flags, max_tmds
            0xC0, // both latency flags set
            11, 6, // progressive latency
            21, 16, // interlaced latency
        ];
        let vsdb = parse_hdmi_vsdb(&data).unwrap();
        assert_eq!(vsdb.video_latency_ms, Some(20));
        assert_eq!(vsdb.audio_latency_ms, Some(10));
        assert_eq!(vsdb.interlaced_video_latency_ms, Some(40));
        assert_eq!(vsdb.interlaced_audio_latency_ms, Some(30));
    }

    #[test]
    fn test_wrong_oui_returns_none() {
        let data = [0x01, 0x02, 0x03, 0x10, 0x00];
        assert!(parse_hdmi_vsdb(&data).is_none());
    }

    #[test]
    fn test_too_short_returns_none() {
        let data = [0x03, 0x0C, 0x00, 0x10]; // missing one byte of SPA
        assert!(parse_hdmi_vsdb(&data).is_none());
    }

    #[test]
    fn test_latency_decode() {
        assert_eq!(decode_latency(0), None);
        assert_eq!(decode_latency(255), None);
        assert_eq!(decode_latency(1), Some(0));
        assert_eq!(decode_latency(2), Some(2));
        assert_eq!(decode_latency(251), Some(500));
    }
}
