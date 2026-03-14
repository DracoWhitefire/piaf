mod vic_table;

use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionHandler;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::Vec;
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cea861Capabilities {
    /// Capability flags from byte 3 of the CEA-861 header.
    pub flags: Cea861Flags,
    /// Short Video Descriptors from the CEA Video Data Block.
    ///
    /// Each entry is `(vic_number, is_native)`. VICs beyond the range of the
    /// built-in lookup table are included here but do not produce an entry in
    /// [`DisplayCapabilities::supported_modes`].
    pub vics: Vec<(u8, bool)>,
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

                if tag == 0x02 {
                    // Video Data Block: each byte is a Short Video Descriptor.
                    for &svd in block_data {
                        let native = (svd & 0x80) != 0;
                        let vic = svd & 0x7F;

                        if vic == 0 {
                            continue; // VIC 0 is reserved
                        }

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
                }
                // Other block types (audio, speaker allocation, vendor-specific, …)
                // will be handled in future iterations.

                i += length;
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
}
