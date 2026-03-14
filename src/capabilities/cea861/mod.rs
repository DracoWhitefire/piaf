#[cfg(any(feature = "alloc", feature = "std"))]
mod audio;
#[cfg(any(feature = "alloc", feature = "std"))]
mod extended_blocks;
#[cfg(any(feature = "alloc", feature = "std"))]
mod hdmi_vsdb;
mod vic_table;

#[cfg(any(feature = "alloc", feature = "std"))]
pub use audio::{AudioFormat, AudioFormatInfo, AudioSampleRates, ShortAudioDescriptor};
#[cfg(any(feature = "alloc", feature = "std"))]
pub use extended_blocks::{
    ColorimetryBlock, ColorimetryFlags, HdrDynamicMetadataDescriptor, HdrEotf, HdrStaticMetadata,
    SpeakerAllocation, SpeakerAllocationFlags, SpeakerAllocationFlags2, SpeakerAllocationFlags3,
    VideoCapability, VideoCapabilityFlags,
};
#[cfg(any(feature = "alloc", feature = "std"))]
pub use hdmi_vsdb::{HdmiVsdb, HdmiVsdbFlags};

#[cfg(any(feature = "alloc", feature = "std"))]
use crate::capabilities::base::timings::decode_dtd_slot;
use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionHandler;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::Vec;
#[cfg(any(feature = "alloc", feature = "std"))]
use audio::parse_audio_data_block;
#[cfg(any(feature = "alloc", feature = "std"))]
use extended_blocks::{
    parse_colorimetry, parse_hdr_dynamic_metadata, parse_hdr_static_metadata,
    parse_speaker_allocation, parse_video_capability, parse_video_format_preferences,
    parse_y420_capability_map, parse_y420_vdb, EXT_TAG_COLORIMETRY, EXT_TAG_HDR_DYNAMIC_METADATA,
    EXT_TAG_HDR_STATIC_METADATA, EXT_TAG_VIDEO_CAPABILITY, EXT_TAG_VIDEO_FORMAT_PREFERENCE,
    EXT_TAG_Y420_CAPABILITY_MAP, EXT_TAG_Y420_VIDEO,
};
#[cfg(any(feature = "alloc", feature = "std"))]
use hdmi_vsdb::parse_hdmi_vsdb;
#[cfg(any(feature = "alloc", feature = "std"))]
use vic_table::vic_to_mode;

bitflags::bitflags! {
    /// Capability flags from byte 3 of a CEA-861 extension block.
    ///
    /// | Bit | Mask   | Meaning                  |
    /// |-----|--------|--------------------------|
    /// | 7   | `0x80` | Underscan support        |
    /// | 6   | `0x40` | Basic audio support      |
    /// | 5   | `0x20` | YCbCr 4:4:4 support      |
    /// | 4   | `0x10` | YCbCr 4:2:0 support      |
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Cea861Flags: u8 {
        /// The display supports underscan.
        const UNDERSCAN   = 0x80;
        /// The display supports basic audio.
        const BASIC_AUDIO = 0x40;
        /// The display supports YCbCr 4:4:4 color encoding.
        const YCBCR_444   = 0x20;
        /// The display supports YCbCr 4:2:0 color encoding.
        const YCBCR_422   = 0x10;
    }
}

/// Decoded capabilities from a CEA-861 extension block.
///
/// Stored in [`DisplayCapabilities::extension_data`] under tag `0x02` by [`Cea861Handler`].
/// Retrieve it with `caps.get_extension_data::<Cea861Capabilities>(0x02)`.
#[cfg(any(feature = "alloc", feature = "std"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)] // no Eq: HdrStaticMetadata contains f32
pub struct Cea861Capabilities {
    /// Capability flags from byte 3 of the CEA-861 header.
    pub flags: Cea861Flags,
    /// Short Video Descriptors from the CEA Video Data Block (tag `0x02`).
    ///
    /// Each entry is `(vic_number, is_native)`. VICs beyond the range of the
    /// built-in lookup table are included here but do not produce an entry in
    /// [`DisplayCapabilities::supported_modes`].
    pub vics: Vec<(u8, bool)>,
    /// Short Audio Descriptors from the CEA Audio Data Block (tag `0x01`).
    pub audio_descriptors: Vec<ShortAudioDescriptor>,
    /// Decoded HDMI 1.x Vendor-Specific Data Block (OUI `0x000C03`), if present.
    pub hdmi_vsdb: Option<HdmiVsdb>,
    /// Decoded Video Capability Data Block (extended tag `0x00`), if present.
    pub video_capability: Option<VideoCapability>,
    /// Decoded Colorimetry Data Block (extended tag `0x05`), if present.
    pub colorimetry: Option<ColorimetryBlock>,
    /// Decoded HDR Static Metadata Data Block (extended tag `0x06`), if present.
    pub hdr_static_metadata: Option<HdrStaticMetadata>,
    /// Decoded Speaker Allocation Data Block (standard tag `0x04`), if present.
    pub speaker_allocation: Option<SpeakerAllocation>,
    /// HDR Dynamic Metadata application descriptors (extended tag `0x07`).
    ///
    /// One entry per application type found (e.g. HDR10+, Dolby Vision).
    pub hdr_dynamic_metadata: Vec<HdrDynamicMetadataDescriptor>,
    /// Raw Short Video References from the Video Format Preference Data Block
    /// (extended tag `0x0D`), if present.
    ///
    /// Each byte encodes a preferred format: `1`–`127` = VIC reference,
    /// `129`–`144` = DTD reference, `145`–`160` = Y420 VDB reference.
    pub video_format_preferences: Vec<u8>,
    /// VICs from the YCbCr 4:2:0 Video Data Block (extended tag `0x0E`).
    ///
    /// These modes are **only** supported in 4:2:0 colour format.
    /// The corresponding [`VideoMode`][crate::VideoMode] entries are also added to
    /// [`DisplayCapabilities::supported_modes`] when the VIC is in the lookup table.
    pub y420_vics: Vec<u8>,
    /// Raw capability bitmap from the YCbCr 4:2:0 Capability Map Data Block
    /// (extended tag `0x0F`).
    ///
    /// Bit `n` (0-indexed, LSB-first across bytes) corresponds to the `(n+1)`-th
    /// Short Video Descriptor in the standard Video Data Block. A set bit means
    /// that mode also supports YCbCr 4:2:0.
    pub y420_capability_map: Vec<u8>,
}

/// Processes a CEA-861 extension block (tag `0x02`).
#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug)]
pub struct Cea861Handler;

#[cfg(any(feature = "alloc", feature = "std"))]
impl ExtensionHandler for Cea861Handler {
    fn process(
        &self,
        ext: &[u8; 128],
        caps: &mut DisplayCapabilities,
        _warnings: &mut Vec<EdidWarning>,
    ) {
        let flags = Cea861Flags::from_bits_truncate(ext[3]);
        let dtd_offset = ext[2] as usize;

        let mut cea_caps = Cea861Capabilities {
            flags,
            vics: Vec::new(),
            audio_descriptors: Vec::new(),
            hdmi_vsdb: None,
            video_capability: None,
            colorimetry: None,
            hdr_static_metadata: None,
            speaker_allocation: None,
            hdr_dynamic_metadata: Vec::new(),
            video_format_preferences: Vec::new(),
            y420_vics: Vec::new(),
            y420_capability_map: Vec::new(),
        };

        // Parse the data block collection: bytes 4 through dtd_offset-1.
        // When dtd_offset == 0 the spec says no DTDs are present, so data blocks
        // may fill the rest of the block (bytes 4–127).
        let collection_end = if dtd_offset == 0 {
            128
        } else {
            dtd_offset.min(128)
        };

        if collection_end > 4 {
            let collection = &ext[4..collection_end];
            let mut i = 0;

            while i < collection.len() {
                let header = collection[i];

                // A zero header byte is padding — stop scanning.
                if header == 0 {
                    break;
                }

                let tag = (header >> 5) & 0x07;
                let length = (header & 0x1F) as usize;
                i += 1;

                if i + length > collection.len() {
                    // Malformed block extends past end of collection.
                    break;
                }

                let block_data = &collection[i..i + length];

                if tag == 0x01 {
                    // Audio Data Block: 3-byte Short Audio Descriptors.
                    cea_caps
                        .audio_descriptors
                        .extend(parse_audio_data_block(block_data));
                } else if tag == 0x02 {
                    // Video Data Block: Short Video Descriptors.
                    // Standard SVD: 1 byte; bits 6:0 = VIC (1–127), bit 7 = native.
                    // Extended SVD (CTA-861-G+): if bits 6:0 == 0, the *next* byte
                    // is the full VIC number (128–255); bit 7 of the first byte = native.
                    let mut j = 0;
                    while j < block_data.len() {
                        let b = block_data[j];
                        let native = (b & 0x80) != 0;
                        let vic_low = b & 0x7F;

                        let vic = if vic_low == 0 {
                            // Extended SVD — consume one more byte.
                            j += 1;
                            match block_data.get(j).copied() {
                                Some(0) | None => {
                                    j += 1;
                                    continue; // VIC 0 reserved
                                }
                                Some(v) => v,
                            }
                        } else {
                            vic_low
                        };
                        j += 1;

                        cea_caps.vics.push((vic, native));

                        if let Some(mode) = vic_to_mode(vic) {
                            let already_present = caps.supported_modes.iter().any(|m| {
                                m.width == mode.width
                                    && m.height == mode.height
                                    && m.refresh_rate == mode.refresh_rate
                                    && m.interlaced == mode.interlaced
                            });
                            if !already_present {
                                caps.supported_modes.push(mode);
                            }
                        }
                    }
                } else if tag == 0x03 {
                    // Vendor-Specific Data Block: check for HDMI OUI.
                    if cea_caps.hdmi_vsdb.is_none() {
                        cea_caps.hdmi_vsdb = parse_hdmi_vsdb(block_data);
                    }
                } else if tag == 0x04 {
                    // Speaker Allocation Data Block.
                    if cea_caps.speaker_allocation.is_none() {
                        cea_caps.speaker_allocation = parse_speaker_allocation(block_data);
                    }
                } else if tag == 0x07 {
                    // Extended Tag Data Block: first payload byte is the extended tag.
                    match block_data.first().copied() {
                        Some(EXT_TAG_VIDEO_CAPABILITY) if cea_caps.video_capability.is_none() => {
                            cea_caps.video_capability = parse_video_capability(block_data);
                        }
                        Some(EXT_TAG_COLORIMETRY) if cea_caps.colorimetry.is_none() => {
                            cea_caps.colorimetry = parse_colorimetry(block_data);
                        }
                        Some(EXT_TAG_HDR_STATIC_METADATA)
                            if cea_caps.hdr_static_metadata.is_none() =>
                        {
                            cea_caps.hdr_static_metadata = parse_hdr_static_metadata(block_data);
                        }
                        Some(EXT_TAG_HDR_DYNAMIC_METADATA) => {
                            cea_caps
                                .hdr_dynamic_metadata
                                .extend(parse_hdr_dynamic_metadata(block_data));
                        }
                        Some(EXT_TAG_VIDEO_FORMAT_PREFERENCE)
                            if cea_caps.video_format_preferences.is_empty() =>
                        {
                            cea_caps.video_format_preferences =
                                parse_video_format_preferences(block_data);
                        }
                        Some(EXT_TAG_Y420_VIDEO) => {
                            let vics = parse_y420_vdb(block_data);
                            for &vic in &vics {
                                if let Some(mode) = vic_to_mode(vic) {
                                    let already_present = caps.supported_modes.iter().any(|m| {
                                        m.width == mode.width
                                            && m.height == mode.height
                                            && m.refresh_rate == mode.refresh_rate
                                            && m.interlaced == mode.interlaced
                                    });
                                    if !already_present {
                                        caps.supported_modes.push(mode);
                                    }
                                }
                            }
                            cea_caps.y420_vics.extend(vics);
                        }
                        Some(EXT_TAG_Y420_CAPABILITY_MAP)
                            if cea_caps.y420_capability_map.is_empty() =>
                        {
                            cea_caps.y420_capability_map = parse_y420_capability_map(block_data);
                        }
                        _ => {}
                    }
                }

                i += length;
            }
        }

        // Parse Detailed Timing Descriptors from byte dtd_offset through byte 126.
        // When dtd_offset == 0 there are no DTDs (the spec says no timing data present).
        if (4..=110).contains(&dtd_offset) {
            let mut offset = dtd_offset;
            while offset + 18 <= 127 {
                decode_dtd_slot(&ext[offset..offset + 18], caps);
                offset += 18;
            }
        }

        caps.set_extension_data(0x02, cea_caps);
    }
}

#[cfg(test)]
#[cfg(any(feature = "alloc", feature = "std"))]
mod tests {
    use super::*;
    use crate::model::capabilities::DisplayCapabilities;

    #[test]
    fn test_audio_flag() {
        let mut ext = [0u8; 128];
        ext[3] = Cea861Flags::BASIC_AUDIO.bits();

        let mut caps = DisplayCapabilities::default();
        Cea861Handler.process(&ext, &mut caps, &mut Vec::new());

        let cea = caps.get_extension_data::<Cea861Capabilities>(0x02).unwrap();
        assert!(cea.flags.contains(Cea861Flags::BASIC_AUDIO));
    }

    #[test]
    fn test_no_audio_flag() {
        let ext = [0u8; 128];

        let mut caps = DisplayCapabilities::default();
        Cea861Handler.process(&ext, &mut caps, &mut Vec::new());

        let cea = caps.get_extension_data::<Cea861Capabilities>(0x02).unwrap();
        assert!(!cea.flags.contains(Cea861Flags::BASIC_AUDIO));
    }

    #[test]
    fn test_flags_parsing() {
        let flags = Cea861Flags::from_bits_truncate(0xF0);
        assert!(flags.contains(Cea861Flags::UNDERSCAN));
        assert!(flags.contains(Cea861Flags::BASIC_AUDIO));
        assert!(flags.contains(Cea861Flags::YCBCR_444));
        assert!(flags.contains(Cea861Flags::YCBCR_422));
    }

    #[test]
    fn test_svd_parsing() {
        let mut ext = [0u8; 128];
        // dtd_offset = 8: data block collection occupies bytes 4–7
        ext[2] = 8;
        ext[3] = Cea861Flags::BASIC_AUDIO.bits();
        // Video Data Block: tag=2 (010), length=3  →  header = (2 << 5) | 3 = 0x43
        ext[4] = (2 << 5) | 3;
        ext[5] = 0x90; // VIC 16, native flag set  (0x80 | 16)
        ext[6] = 0x04; // VIC 4
        ext[7] = 0x01; // VIC 1

        let mut caps = DisplayCapabilities::default();
        Cea861Handler.process(&ext, &mut caps, &mut Vec::new());

        let cea = caps.get_extension_data::<Cea861Capabilities>(0x02).unwrap();
        assert_eq!(cea.vics, vec![(16, true), (4, false), (1, false)]);

        // All three VICs should produce entries in supported_modes
        assert_eq!(caps.supported_modes.len(), 3);
        assert!(caps
            .supported_modes
            .iter()
            .any(|m| m.width == 1920 && m.height == 1080 && m.refresh_rate == 60 && !m.interlaced));
        assert!(caps
            .supported_modes
            .iter()
            .any(|m| m.width == 1280 && m.height == 720 && m.refresh_rate == 60));
        assert!(caps
            .supported_modes
            .iter()
            .any(|m| m.width == 640 && m.height == 480 && m.refresh_rate == 60));
    }

    #[test]
    fn test_svd_dedup() {
        // If a mode already appears in supported_modes it should not be added again.
        let mut ext = [0u8; 128];
        // dtd_offset = 7: collection = ext[4..7] = 3 bytes (1 header + 2 SVDs)
        ext[2] = 7;
        // Video Data Block with two SVDs for the same VIC
        ext[4] = (2 << 5) | 2; // tag=2, length=2
        ext[5] = 0x10; // VIC 16
        ext[6] = 0x90; // VIC 16, native — duplicate

        let mut caps = DisplayCapabilities::default();
        Cea861Handler.process(&ext, &mut caps, &mut Vec::new());

        let cea = caps.get_extension_data::<Cea861Capabilities>(0x02).unwrap();
        // Both SVD bytes are recorded in vics…
        assert_eq!(cea.vics.len(), 2);
        // …but supported_modes has only one entry for 1920×1080p60
        assert_eq!(
            caps.supported_modes
                .iter()
                .filter(|m| m.width == 1920 && m.height == 1080 && m.refresh_rate == 60)
                .count(),
            1
        );
    }

    #[test]
    fn test_svd_byte_0x80_is_skipped() {
        // A standard SVD byte uses bit 7 as the native flag and bits 6-0 as the
        // VIC number. Byte 0x80 → native=true, vic=0. VIC 0 is reserved and must
        // be skipped: it should not appear in vics and must not produce a mode.
        let mut ext = [0u8; 128];
        ext[2] = 6;
        ext[4] = (2 << 5) | 1; // tag=2, length=1
        ext[5] = 0x80; // native flag set, vic=0

        let mut caps = DisplayCapabilities::default();
        Cea861Handler.process(&ext, &mut caps, &mut Vec::new());

        let cea = caps.get_extension_data::<Cea861Capabilities>(0x02).unwrap();
        assert!(cea.vics.is_empty());
        assert!(caps.supported_modes.is_empty());
    }

    /// Builds a minimal valid 18-byte DTD for 2560×1440@144Hz into `ext` starting at `offset`.
    ///
    /// Pixel clock = 583.92 MHz → stored as 58392 (little-endian units of 10 kHz).
    /// Htotal = 2720 (active 2560 + blank 160), Vtotal = 1481 (active 1440 + blank 41).
    /// 58392 * 10000 / (2720 * 1481) = 144.8 → rounds to 144.
    fn write_dtd_2560x1440_144(ext: &mut [u8; 128], offset: usize) {
        let clk: u16 = 58392; // 10 kHz units
        ext[offset] = (clk & 0xFF) as u8;
        ext[offset + 1] = (clk >> 8) as u8;
        // hactive=2560, hblank=160
        ext[offset + 2] = (2560 & 0xFF) as u8; // hactive low
        ext[offset + 3] = (160 & 0xFF) as u8; // hblank low
        ext[offset + 4] = ((2560 >> 4) & 0xF0) as u8; // hblank high nibble = 0 (160 < 256)
                                                      // vactive=1440, vblank=41
        ext[offset + 5] = (1440 & 0xFF) as u8; // vactive low
        ext[offset + 6] = (41 & 0xFF) as u8; // vblank low
        ext[offset + 7] = ((1440 >> 4) & 0xF0) as u8; // vblank high nibble = 0 (41 < 256)
                                                      // bytes 8-16: sync/image (leave zero — minimal)
                                                      // byte 17: digital separate sync, both polarities positive
        ext[offset + 17] = 0x1E; // bit4=digital, bit3=separate, bit2=Vpos, bit1=Hpos
    }

    #[test]
    fn test_cea_dtd_parsed() {
        let mut ext = [0u8; 128];
        // No data block collection; DTDs start at offset 4.
        ext[2] = 4; // dtd_offset = 4
        write_dtd_2560x1440_144(&mut ext, 4);

        let mut caps = DisplayCapabilities::default();
        Cea861Handler.process(&ext, &mut caps, &mut Vec::new());

        assert!(
            caps.supported_modes
                .iter()
                .any(|m| m.width == 2560 && m.height == 1440 && m.refresh_rate == 144),
            "2560×1440@144 not found in supported_modes"
        );
    }

    #[test]
    fn test_cea_dtd_zero_clock_skipped() {
        // A DTD slot whose first two bytes are 0x00 is a monitor descriptor and must be skipped.
        let mut ext = [0u8; 128];
        ext[2] = 4; // dtd_offset = 4 — no data blocks, DTDs start immediately
                    // ext[4..22] stays all-zero → zero pixel clock → skipped

        let mut caps = DisplayCapabilities::default();
        Cea861Handler.process(&ext, &mut caps, &mut Vec::new());

        assert!(caps.supported_modes.is_empty());
    }

    #[test]
    fn test_speaker_allocation_parsed() {
        let mut ext = [0u8; 128];
        // Speaker Allocation Data Block: tag=4, length=3 → header = (4 << 5) | 3 = 0x83
        ext[2] = 8;
        ext[4] = (4 << 5) | 3;
        ext[5] = 0x07; // FL_FR | LFE1 | FC
        ext[6] = 0x00;
        ext[7] = 0x00;

        let mut caps = DisplayCapabilities::default();
        Cea861Handler.process(&ext, &mut caps, &mut Vec::new());

        let cea = caps.get_extension_data::<Cea861Capabilities>(0x02).unwrap();
        let sa = cea.speaker_allocation.unwrap();
        assert!(sa.channels.contains(SpeakerAllocationFlags::FL_FR));
        assert!(sa.channels.contains(SpeakerAllocationFlags::LFE1));
        assert!(sa.channels.contains(SpeakerAllocationFlags::FC));
    }

    #[test]
    fn test_y420_vdb_adds_modes_and_vics() {
        let mut ext = [0u8; 128];
        // Y420 VDB extended block: outer tag=7, length=2 → header = (7 << 5) | 2 = 0xE2
        // payload: extended tag 0x0E, VIC 96 (2160p50)
        ext[2] = 8;
        ext[4] = (7 << 5) | 2;
        ext[5] = 0x0E; // EXT_TAG_Y420_VIDEO
        ext[6] = 96;   // VIC 96

        let mut caps = DisplayCapabilities::default();
        Cea861Handler.process(&ext, &mut caps, &mut Vec::new());

        let cea = caps.get_extension_data::<Cea861Capabilities>(0x02).unwrap();
        assert_eq!(cea.y420_vics, vec![96]);
        // VIC 96 = 3840×2160@50Hz — should appear in supported_modes
        assert!(caps.supported_modes.iter().any(|m| m.width == 3840 && m.height == 2160 && m.refresh_rate == 50));
    }

    #[test]
    fn test_y420_capability_map_stored() {
        let mut ext = [0u8; 128];
        // Y420 Capability Map: outer tag=7, length=2 → 0xE2; ext tag 0x0F, bitmap 0b00000101
        ext[2] = 8;
        ext[4] = (7 << 5) | 2;
        ext[5] = 0x0F; // EXT_TAG_Y420_CAPABILITY_MAP
        ext[6] = 0b0000_0101;

        let mut caps = DisplayCapabilities::default();
        Cea861Handler.process(&ext, &mut caps, &mut Vec::new());

        let cea = caps.get_extension_data::<Cea861Capabilities>(0x02).unwrap();
        assert_eq!(cea.y420_capability_map, vec![0b0000_0101]);
    }
}
