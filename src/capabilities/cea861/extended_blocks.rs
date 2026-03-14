/// Extended tag codes used in CEA Extended Tag Data Blocks (outer tag `0x07`).
/// The first byte of the block payload is the extended tag.
pub(super) const EXT_TAG_VIDEO_CAPABILITY: u8 = 0x00;
pub(super) const EXT_TAG_COLORIMETRY: u8 = 0x05;
pub(super) const EXT_TAG_HDR_STATIC_METADATA: u8 = 0x06;

// ---------------------------------------------------------------------------
// Video Capability Data Block (extended tag 0x00)
// ---------------------------------------------------------------------------

bitflags::bitflags! {
    /// Flags from the Video Capability Data Block (extended tag `0x00`).
    ///
    /// Describes over-/underscan behaviour and quantization range support.
    ///
    /// | Bit | Mask   | Meaning                                          |
    /// |-----|--------|--------------------------------------------------|
    /// | 7   | `0x80` | QY: quantization range selectable (YCC)          |
    /// | 6   | `0x40` | QS: quantization range selectable (RGB)          |
    /// | 5–4 | `0x30` | PT: preferred PT behavior (2-bit field)          |
    /// | 3–2 | `0x0C` | IT: IT content behavior (2-bit field)            |
    /// | 1–0 | `0x03` | CE: CE content behavior (2-bit field)            |
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct VideoCapabilityFlags: u8 {
        /// YCC quantization range is selectable (QY).
        const QY = 0x80;
        /// RGB quantization range is selectable (QS).
        const QS = 0x40;
    }
}

/// Decoded Video Capability Data Block (extended tag `0x00`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoCapability {
    /// Quantization range and overscan flags.
    pub flags: VideoCapabilityFlags,
    /// Preferred timing overscan/underscan behaviour (bits 5–4, 2-bit field).
    pub pt_behaviour: u8,
    /// IT content overscan/underscan behaviour (bits 3–2, 2-bit field).
    pub it_behaviour: u8,
    /// CE content overscan/underscan behaviour (bits 1–0, 2-bit field).
    pub ce_behaviour: u8,
}

pub(super) fn parse_video_capability(block_data: &[u8]) -> Option<VideoCapability> {
    // block_data[0] = extended tag; payload starts at [1].
    let b = *block_data.get(1)?;
    Some(VideoCapability {
        flags: VideoCapabilityFlags::from_bits_truncate(b & 0xC0),
        pt_behaviour: (b >> 4) & 0x03,
        it_behaviour: (b >> 2) & 0x03,
        ce_behaviour: b & 0x03,
    })
}

// ---------------------------------------------------------------------------
// Colorimetry Data Block (extended tag 0x05)
// ---------------------------------------------------------------------------

bitflags::bitflags! {
    /// Colorimetry standards supported by the display, from the Colorimetry
    /// Data Block (extended tag `0x05`).
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ColorimetryFlags: u8 {
        /// xvYCC 601.
        const XVYCC601   = 0x01;
        /// xvYCC 709.
        const XVYCC709   = 0x02;
        /// sYCC 601.
        const SYCC601    = 0x04;
        /// opYCC 601.
        const OPYCC601   = 0x08;
        /// opRGB.
        const OPRGB      = 0x10;
        /// BT.2020 cYCC.
        const BT2020CYCC = 0x20;
        /// BT.2020 YCC.
        const BT2020YCC  = 0x40;
        /// BT.2020 RGB.
        const BT2020RGB  = 0x80;
    }
}

/// Decoded Colorimetry Data Block (extended tag `0x05`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorimetryBlock {
    /// Supported colorimetry standards.
    pub colorimetry: ColorimetryFlags,
    /// Gamut metadata profile support bitmap (bits 3–0 of byte 2).
    pub metadata_profiles: u8,
}

pub(super) fn parse_colorimetry(block_data: &[u8]) -> Option<ColorimetryBlock> {
    // block_data[0] = extended tag; payload is [1] and optionally [2].
    let colorimetry = ColorimetryFlags::from_bits_truncate(*block_data.get(1)?);
    let metadata_profiles = block_data.get(2).map_or(0, |&b| b & 0x0F);
    Some(ColorimetryBlock {
        colorimetry,
        metadata_profiles,
    })
}

// ---------------------------------------------------------------------------
// HDR Static Metadata Data Block (extended tag 0x06)
// ---------------------------------------------------------------------------

bitflags::bitflags! {
    /// Electro-Optical Transfer Functions (EOTFs) supported by the display,
    /// from the HDR Static Metadata Data Block (extended tag `0x06`).
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct HdrEotf: u8 {
        /// Traditional gamma — SDR luminance range.
        const SDR    = 0x01;
        /// Traditional gamma — HDR luminance range.
        const HDR    = 0x02;
        /// SMPTE ST 2084 (PQ / HDR10).
        const ST2084 = 0x04;
        /// Hybrid Log-Gamma (HLG).
        const HLG    = 0x08;
    }
}

/// Decoded HDR Static Metadata Data Block (extended tag `0x06`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)] // no Eq: contains f32
pub struct HdrStaticMetadata {
    /// Supported EOTFs (tone-mapping curves).
    pub eotf: HdrEotf,
    /// Static Metadata Descriptor type bitmap (usually bit 0 = Type 1 / MaxCLL/MaxFALL).
    pub static_metadata_descriptors: u8,
    /// Desired content maximum luminance in cd/m², decoded from byte 3.
    ///
    /// Encoded as `50 × 2^(raw / 32)`. `None` when the byte is absent.
    pub max_luminance: Option<f32>,
    /// Desired content maximum frame-average light level (MaxFALL) in cd/m².
    ///
    /// Same encoding as `max_luminance`. `None` when absent.
    pub max_fall: Option<f32>,
    /// Desired content minimum luminance in cd/m², decoded from byte 5.
    ///
    /// Encoded as `max_luminance × (raw / 255)² / 100`. `None` when absent or when
    /// `max_luminance` is not present.
    pub min_luminance: Option<f32>,
}

/// Decodes the `50 × 2^(raw/32)` luminance encoding used in the HDR block.
fn decode_luminance(raw: u8) -> f32 {
    50.0 * 2f32.powf(raw as f32 / 32.0)
}

pub(super) fn parse_hdr_static_metadata(block_data: &[u8]) -> Option<HdrStaticMetadata> {
    // block_data[0] = extended tag; EOTF at [1], SMD at [2], luminance at [3-5].
    let eotf = HdrEotf::from_bits_truncate(*block_data.get(1)?);
    let static_metadata_descriptors = *block_data.get(2).unwrap_or(&0);

    let max_luminance = block_data.get(3).map(|&b| decode_luminance(b));
    let max_fall = block_data.get(4).map(|&b| decode_luminance(b));
    let min_luminance = block_data
        .get(5)
        .and_then(|&b| max_luminance.map(|max| max * (b as f32 / 255.0).powi(2) / 100.0));

    Some(HdrStaticMetadata {
        eotf,
        static_metadata_descriptors,
        max_luminance,
        max_fall,
        min_luminance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_capability() {
        // QS set, PT=1, IT=0, CE=3
        let data = [EXT_TAG_VIDEO_CAPABILITY, 0b0101_0011];
        let vc = parse_video_capability(&data).unwrap();
        assert!(vc.flags.contains(VideoCapabilityFlags::QS));
        assert!(!vc.flags.contains(VideoCapabilityFlags::QY));
        assert_eq!(vc.pt_behaviour, 1);
        assert_eq!(vc.it_behaviour, 0);
        assert_eq!(vc.ce_behaviour, 3);
    }

    #[test]
    fn test_colorimetry_bt2020() {
        // BT2020RGB | BT2020YCC, metadata profile 0x05
        let data = [EXT_TAG_COLORIMETRY, 0xC0, 0x05];
        let cb = parse_colorimetry(&data).unwrap();
        assert!(cb.colorimetry.contains(ColorimetryFlags::BT2020RGB));
        assert!(cb.colorimetry.contains(ColorimetryFlags::BT2020YCC));
        assert!(!cb.colorimetry.contains(ColorimetryFlags::XVYCC601));
        assert_eq!(cb.metadata_profiles, 5);
    }

    #[test]
    fn test_colorimetry_no_metadata_byte() {
        let data = [EXT_TAG_COLORIMETRY, 0x03];
        let cb = parse_colorimetry(&data).unwrap();
        assert_eq!(cb.metadata_profiles, 0);
    }

    #[test]
    fn test_hdr_metadata_basic() {
        // SDR + ST2084, SMD type 1, no luminance bytes
        let data = [EXT_TAG_HDR_STATIC_METADATA, 0x05, 0x01];
        let hdr = parse_hdr_static_metadata(&data).unwrap();
        assert!(hdr.eotf.contains(HdrEotf::SDR));
        assert!(hdr.eotf.contains(HdrEotf::ST2084));
        assert!(!hdr.eotf.contains(HdrEotf::HLG));
        assert_eq!(hdr.static_metadata_descriptors, 1);
        assert!(hdr.max_luminance.is_none());
        assert!(hdr.min_luminance.is_none());
    }

    #[test]
    fn test_hdr_metadata_luminance() {
        // raw=96 → 50 * 2^(96/32) = 50 * 8 = 400 cd/m²
        // min raw=128 → 400 * (128/255)^2 / 100 ≈ 1.006 cd/m²
        let data = [EXT_TAG_HDR_STATIC_METADATA, 0x04, 0x01, 96, 96, 128];
        let hdr = parse_hdr_static_metadata(&data).unwrap();
        let max = hdr.max_luminance.unwrap();
        assert!((max - 400.0).abs() < 0.1, "max={max}");
        let min = hdr.min_luminance.unwrap();
        assert!(min > 0.9 && min < 1.1, "min={min}");
    }

    #[test]
    fn test_hdr_too_short_returns_none() {
        // Missing the EOTF byte
        let data = [EXT_TAG_HDR_STATIC_METADATA];
        assert!(parse_hdr_static_metadata(&data).is_none());
    }
}
