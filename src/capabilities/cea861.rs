use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionHandler;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::Vec;

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
        caps.set_extension_data(0x02, Cea861Capabilities { flags });
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
}
