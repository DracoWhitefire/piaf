use crate::model::capabilities::VideoMode;

/// Extended tag codes used in CEA Extended Tag Data Blocks (outer tag `0x07`).
/// The first byte of the block payload is the extended tag.
pub(super) const EXT_TAG_VSVDB: u8 = 0x01;
pub(super) const EXT_TAG_VESA_DDDB: u8 = 0x02;
pub(super) const EXT_TAG_VTB_EXT: u8 = 0x03;
pub(super) const EXT_TAG_VIDEO_CAPABILITY: u8 = 0x00;
pub(super) const EXT_TAG_VSADB: u8 = 0x11;
pub(super) const EXT_TAG_T7VTDB: u8 = 0x22;
pub(super) const EXT_TAG_T8VTDB: u8 = 0x23;
pub(super) const EXT_TAG_T10VTDB: u8 = 0x2A;
pub(super) const EXT_TAG_HF_EEODB: u8 = 0x78;
pub(super) const EXT_TAG_HF_SCDB: u8 = 0x79;
pub(super) const EXT_TAG_COLORIMETRY: u8 = 0x05;
pub(super) const EXT_TAG_HDR_STATIC_METADATA: u8 = 0x06;
pub(super) const EXT_TAG_HDR_DYNAMIC_METADATA: u8 = 0x07;
pub(super) const EXT_TAG_VIDEO_FORMAT_PREFERENCE: u8 = 0x0D;
pub(super) const EXT_TAG_Y420_VIDEO: u8 = 0x0E;
pub(super) const EXT_TAG_Y420_CAPABILITY_MAP: u8 = 0x0F;
pub(super) const EXT_TAG_HDMI_AUDIO: u8 = 0x12;
pub(super) const EXT_TAG_ROOM_CONFIGURATION: u8 = 0x13;
pub(super) const EXT_TAG_SPEAKER_LOCATION: u8 = 0x14;
pub(super) const EXT_TAG_INFOFRAME: u8 = 0x20;

/// All extended tag codes that are implemented by this library.
///
/// This list must be kept in sync with the match arms in `mod.rs`.
/// The `test_all_extended_tags_accounted_for` test uses it to verify that the
/// union of implemented and reserved tags covers every value 0x00–0xFF.
#[cfg(test)]
pub(super) const IMPLEMENTED_EXTENDED_TAGS: &[u8] = &[
    EXT_TAG_VIDEO_CAPABILITY,        // 0x00
    EXT_TAG_VSVDB,                   // 0x01
    EXT_TAG_VESA_DDDB,               // 0x02
    EXT_TAG_VTB_EXT,                 // 0x03
    EXT_TAG_COLORIMETRY,             // 0x05
    EXT_TAG_HDR_STATIC_METADATA,     // 0x06
    EXT_TAG_HDR_DYNAMIC_METADATA,    // 0x07
    EXT_TAG_VIDEO_FORMAT_PREFERENCE, // 0x0D
    EXT_TAG_Y420_VIDEO,              // 0x0E
    EXT_TAG_Y420_CAPABILITY_MAP,     // 0x0F
    EXT_TAG_VSADB,                   // 0x11
    EXT_TAG_HDMI_AUDIO,              // 0x12
    EXT_TAG_ROOM_CONFIGURATION,      // 0x13
    EXT_TAG_SPEAKER_LOCATION,        // 0x14
    EXT_TAG_INFOFRAME,               // 0x20
    EXT_TAG_T7VTDB,                  // 0x22
    EXT_TAG_T8VTDB,                  // 0x23
    EXT_TAG_T10VTDB,                 // 0x2A
    EXT_TAG_HF_EEODB,                // 0x78
    EXT_TAG_HF_SCDB,                 // 0x79
];

/// Extended tag ranges that are explicitly reserved or deferred.
///
/// Each entry is an inclusive `(first, last)` range. These are listed here so
/// that `test_all_extended_tags_accounted_for` can confirm there are no gaps.
/// When the CTA spec assigns a new tag, remove it from the appropriate range,
/// add an `EXT_TAG_*` constant, and implement the block (or add it to
/// `IMPLEMENTED_EXTENDED_TAGS` as a no-op stub).
#[cfg(test)]
pub(super) const RESERVED_EXTENDED_TAG_RANGES: &[(u8, u8)] = &[
    (0x04, 0x04), // HDMI Forum Video Data Block — no public structure without Forum membership
    (0x08, 0x0C), // Reserved for video-related blocks (CTA-861-H Table 62)
    (0x10, 0x10), // Reserved for CTA Miscellaneous Audio Fields (MAF)
    (0x15, 0x1F), // Reserved for audio-related blocks
    (0x21, 0x21), // Reserved
    (0x24, 0x29), // Reserved
    (0x2B, 0x77), // Reserved
    (0x7A, 0xFF), // Reserved (including HDMI Forum reserved range 0x7A–0x7F)
];

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

// ---------------------------------------------------------------------------
// Speaker Allocation Data Block (standard tag 0x04)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// VESA Display Transfer Characteristic Data Block (standard tag 0x05)
// ---------------------------------------------------------------------------

/// Point encoding precision for the VESA Display Transfer Characteristic Data Block.
///
/// Encoded in bits 7:6 of the first payload byte.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtcPointEncoding {
    /// 8 bits per luminance point.
    Bits8,
    /// 10 bits per luminance point (packed, 5 bytes per 4 points).
    Bits10,
    /// 12 bits per luminance point (packed, 3 bytes per 2 points).
    Bits12,
}

/// Decoded VESA Display Transfer Characteristic Data Block (standard tag `0x05`).
///
/// Encodes the display's luminance transfer function as a sequence of sample
/// points at evenly-spaced input levels from 0 (black) to 1 (white).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)] // no Eq: contains f32
pub struct VesaTransferCharacteristic {
    /// Bit depth used to encode each luminance point.
    pub encoding: DtcPointEncoding,
    /// Luminance values, normalized to [0.0, 1.0].
    pub points: Vec<f32>,
}

pub(super) fn parse_vesa_transfer_characteristic(
    block_data: &[u8],
) -> Option<VesaTransferCharacteristic> {
    let first = *block_data.first()?;
    let encoding = match (first >> 6) & 0x03 {
        0x00 => DtcPointEncoding::Bits8,
        0x01 => DtcPointEncoding::Bits10,
        0x02 => DtcPointEncoding::Bits12,
        _ => return None, // reserved
    };

    let data = &block_data[1..];
    let points = match encoding {
        DtcPointEncoding::Bits8 => data.iter().map(|&b| b as f32 / 255.0).collect(),
        DtcPointEncoding::Bits10 => {
            let mut pts = Vec::new();
            let mut i = 0;
            while i + 5 <= data.len() {
                // 5 bytes encode 4 × 10-bit values, packed MSB-first.
                let [b0, b1, b2, b3, b4] = [
                    data[i] as u16,
                    data[i + 1] as u16,
                    data[i + 2] as u16,
                    data[i + 3] as u16,
                    data[i + 4] as u16,
                ];
                pts.push(((b0 << 2) | (b1 >> 6)) as f32 / 1023.0);
                pts.push((((b1 & 0x3F) << 4) | (b2 >> 4)) as f32 / 1023.0);
                pts.push((((b2 & 0x0F) << 6) | (b3 >> 2)) as f32 / 1023.0);
                pts.push((((b3 & 0x03) << 8) | b4) as f32 / 1023.0);
                i += 5;
            }
            pts
        }
        DtcPointEncoding::Bits12 => {
            let mut pts = Vec::new();
            let mut i = 0;
            while i + 3 <= data.len() {
                // 3 bytes encode 2 × 12-bit values, packed MSB-first.
                let [b0, b1, b2] = [data[i] as u16, data[i + 1] as u16, data[i + 2] as u16];
                pts.push(((b0 << 4) | (b1 >> 4)) as f32 / 4095.0);
                pts.push((((b1 & 0x0F) << 8) | b2) as f32 / 4095.0);
                i += 3;
            }
            pts
        }
    };

    Some(VesaTransferCharacteristic { encoding, points })
}

bitflags::bitflags! {
    /// Speaker channel presence flags, byte 1 of the Speaker Allocation Data Block.
    ///
    /// | Bit | Mask   | Channels                        |
    /// |-----|--------|---------------------------------|
    /// | 7   | `0x80` | FLW/FRW (Front Left/Right Wide) |
    /// | 6   | `0x40` | RLC/RRC (Rear Left/Right Center)|
    /// | 5   | `0x20` | FLC/FRC (Front Left/Right Ctr)  |
    /// | 4   | `0x10` | BC (Back Center)                |
    /// | 3   | `0x08` | BL/BR (Back Left/Right)         |
    /// | 2   | `0x04` | FC (Front Center)               |
    /// | 1   | `0x02` | LFE1 (Low-Frequency Effects 1)  |
    /// | 0   | `0x01` | FL/FR (Front Left/Right)        |
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SpeakerAllocationFlags: u8 {
        /// Front Left / Front Right channels.
        const FL_FR   = 0x01;
        /// Low-Frequency Effects channel 1.
        const LFE1    = 0x02;
        /// Front Center channel.
        const FC      = 0x04;
        /// Back Left / Back Right channels.
        const BL_BR   = 0x08;
        /// Back Center channel.
        const BC      = 0x10;
        /// Front Left Center / Front Right Center channels.
        const FLC_FRC = 0x20;
        /// Rear Left Center / Rear Right Center channels.
        const RLC_RRC = 0x40;
        /// Front Left Wide / Front Right Wide channels.
        const FLW_FRW = 0x80;
    }
}

bitflags::bitflags! {
    /// Speaker channel presence flags, byte 2 of the Speaker Allocation Data Block.
    ///
    /// | Bit | Mask   | Channels                           |
    /// |-----|--------|------------------------------------|
    /// | 7   | `0x80` | TpSiL/TpSiR (Top Side Left/Right)  |
    /// | 6   | `0x40` | SiL/SiR (Side Left/Right)          |
    /// | 5   | `0x20` | TpBC (Top Back Center)             |
    /// | 4   | `0x10` | LFE2 (Low-Frequency Effects 2)     |
    /// | 3   | `0x08` | LS/RS (Left/Right Surround)        |
    /// | 2   | `0x04` | TpFC (Top Front Center)            |
    /// | 1   | `0x02` | TpC (Top Center)                   |
    /// | 0   | `0x01` | TpFL/TpFR (Top Front Left/Right)   |
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SpeakerAllocationFlags2: u8 {
        /// Top Front Left / Top Front Right channels.
        const TP_FL_FR      = 0x01;
        /// Top Center channel.
        const TP_C          = 0x02;
        /// Top Front Center channel.
        const TP_FC         = 0x04;
        /// Left Surround / Right Surround channels.
        const LS_RS         = 0x08;
        /// Low-Frequency Effects channel 2.
        const LFE2          = 0x10;
        /// Top Back Center channel.
        const TP_BC         = 0x20;
        /// Side Left / Side Right channels.
        const SI_L_SI_R     = 0x40;
        /// Top Side Left / Top Side Right channels.
        const TP_SI_L_TP_SI_R = 0x80;
    }
}

bitflags::bitflags! {
    /// Speaker channel presence flags, byte 3 of the Speaker Allocation Data Block.
    ///
    /// | Bit | Mask   | Channels                              |
    /// |-----|--------|---------------------------------------|
    /// | 3   | `0x08` | TpLS/TpRS (Top Left/Right Surround)   |
    /// | 2   | `0x04` | BtFL/BtFR (Bottom Front Left/Right)   |
    /// | 1   | `0x02` | BtFC (Bottom Front Center)            |
    /// | 0   | `0x01` | TpBL/TpBR (Top Back Left/Right)       |
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SpeakerAllocationFlags3: u8 {
        /// Top Back Left / Top Back Right channels.
        const TP_BL_TP_BR  = 0x01;
        /// Bottom Front Center channel.
        const BT_FC        = 0x02;
        /// Bottom Front Left / Bottom Front Right channels.
        const BT_FL_BT_FR  = 0x04;
        /// Top Left Surround / Top Right Surround channels.
        const TP_LS_TP_RS  = 0x08;
    }
}

/// Decoded Speaker Allocation Data Block (standard tag `0x04`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeakerAllocation {
    /// Channels from byte 1 (core speaker channels).
    pub channels: SpeakerAllocationFlags,
    /// Channels from byte 2 (extended — top/surround/LFE2).
    pub channels_2: SpeakerAllocationFlags2,
    /// Channels from byte 3 (extended — top-back/bottom-front).
    pub channels_3: SpeakerAllocationFlags3,
}

pub(super) fn parse_speaker_allocation(block_data: &[u8]) -> Option<SpeakerAllocation> {
    let channels = SpeakerAllocationFlags::from_bits_truncate(*block_data.first()?);
    let channels_2 =
        SpeakerAllocationFlags2::from_bits_truncate(block_data.get(1).copied().unwrap_or(0));
    let channels_3 =
        SpeakerAllocationFlags3::from_bits_truncate(block_data.get(2).copied().unwrap_or(0));
    Some(SpeakerAllocation {
        channels,
        channels_2,
        channels_3,
    })
}

// ---------------------------------------------------------------------------
// HDR Dynamic Metadata Data Block (extended tag 0x07)
// ---------------------------------------------------------------------------

/// One entry from an HDR Dynamic Metadata Data Block (extended tag `0x07`).
///
/// Each descriptor identifies the HDR dynamic metadata technology supported
/// (e.g. HDR10+ / SMPTE ST 2094, or Dolby Vision).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HdrDynamicMetadataDescriptor {
    /// Application type identifier (bits 5–0 of the descriptor byte).
    ///
    /// `1` = SMPTE ST 2094 (HDR10+); `2` = Dolby Vision.
    pub application_type: u8,
    /// Application metadata version (bits 7–6 of the descriptor byte).
    pub application_version: u8,
}

pub(super) fn parse_hdr_dynamic_metadata(block_data: &[u8]) -> Vec<HdrDynamicMetadataDescriptor> {
    // block_data[0] = extended tag; descriptors start at [1].
    // Each descriptor is one byte (type + version); type-specific trailing
    // bytes are not parsed — we advance one byte at a time.
    let mut out = Vec::new();
    // Each descriptor may have additional application-specific bytes.
    // Without type-specific knowledge of their length we cannot skip them,
    // so we parse only the first descriptor rather than risk misalignment.
    if let Some(&b) = block_data.get(1) {
        out.push(HdrDynamicMetadataDescriptor {
            application_type: b & 0x3F,
            application_version: (b >> 6) & 0x03,
        });
    }
    out
}

// ---------------------------------------------------------------------------
// Video Format Preference Data Block (extended tag 0x0D)
// ---------------------------------------------------------------------------

/// Returns the raw Short Video References (SVRs) from a Video Format Preference
/// Data Block (extended tag `0x0D`).
///
/// Each byte encodes a preferred video format:
/// - `1`–`127`: references a VIC in the Video Data Block.
/// - `129`–`144`: references a Detailed Timing Descriptor (`DTD[n-128]`).
/// - `145`–`160`: references an entry in the YCbCr 4:2:0 Video Data Block.
/// - All other values are reserved.
pub(super) fn parse_video_format_preferences(block_data: &[u8]) -> Vec<u8> {
    // block_data[0] = extended tag; SVRs start at [1].
    block_data[1..].to_vec()
}

// ---------------------------------------------------------------------------
// YCbCr 4:2:0 Video Data Block (extended tag 0x0E)
// ---------------------------------------------------------------------------

/// Returns VIC numbers from a YCbCr 4:2:0 Video Data Block (extended tag `0x0E`).
///
/// These VICs are **only** supported in the YCbCr 4:2:0 colour format.
/// Uses the same Short Video Descriptor encoding as the standard Video Data Block,
/// including the CTA-861-G extended SVD format for VICs 128–255 (a byte with
/// bits 6:0 == 0 followed by a byte with the full VIC number). VIC 0 is reserved
/// and is excluded from the returned list.
pub(super) fn parse_y420_vdb(block_data: &[u8]) -> Vec<u8> {
    // block_data[0] = extended tag; SVDs start at [1].
    let svds = &block_data[1..];
    let mut out = Vec::new();
    let mut j = 0;
    while j < svds.len() {
        let vic_low = svds[j] & 0x7F;
        let vic = if vic_low == 0 {
            j += 1;
            match svds.get(j).copied() {
                Some(0) | None => {
                    j += 1;
                    continue;
                }
                Some(v) => v,
            }
        } else {
            vic_low
        };
        j += 1;
        out.push(vic);
    }
    out
}

// ---------------------------------------------------------------------------
// YCbCr 4:2:0 Capability Map Data Block (extended tag 0x0F)
// ---------------------------------------------------------------------------

/// Raw bitmap from a YCbCr 4:2:0 Capability Map Data Block (extended tag `0x0F`).
///
/// Bit `n` (0-indexed across bytes, LSB-first within each byte) corresponds to
/// the `(n+1)`-th Short Video Descriptor in the standard Video Data Block.
/// A set bit means that mode **also** supports YCbCr 4:2:0.
///
/// An empty `Vec` means all modes in the Video Data Block support 4:2:0
/// (per the CTA-861 spec when this block is absent).
pub(super) fn parse_y420_capability_map(block_data: &[u8]) -> Vec<u8> {
    // block_data[0] = extended tag; bitmap bytes start at [1].
    block_data[1..].to_vec()
}

// ---------------------------------------------------------------------------
// Room Configuration Data Block (extended tag 0x13)
// ---------------------------------------------------------------------------

/// Decoded Room Configuration Data Block (extended tag `0x13`).
///
/// Describes the number of loudspeakers in the listening room and whether
/// individual speaker locations are provided in an accompanying
/// Speaker Location Data Block (extended tag `0x14`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomConfigurationBlock {
    /// Number of loudspeakers in the room (bits 4:0).  `0` means not specified.
    pub speaker_count: u8,
    /// If `true`, individual speaker location entries are present in an
    /// accompanying Speaker Location Data Block (extended tag `0x14`).
    pub has_speaker_locations: bool,
}

pub(super) fn parse_room_configuration(block_data: &[u8]) -> Option<RoomConfigurationBlock> {
    // block_data[0] = extended tag; payload byte at [1].
    let b = *block_data.get(1)?;
    Some(RoomConfigurationBlock {
        speaker_count: b & 0x1F,
        has_speaker_locations: b & 0x40 != 0,
    })
}

// ---------------------------------------------------------------------------
// Speaker Location Data Block (extended tag 0x14)
// ---------------------------------------------------------------------------

/// A single speaker location entry from the Speaker Location Data Block
/// (extended tag `0x14`).
///
/// Each entry is two bytes: a channel assignment and a normalized distance
/// from the listener (0 = closest, 255 = furthest).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpeakerLocationEntry {
    /// Channel assignment code (speaker role).
    ///
    /// Values follow the CTA-861-I Table 49 channel assignment codes
    /// (e.g. `0x00` = FL/FR, `0x01` = LFE1, `0x02` = FC, etc.).
    pub channel_assignment: u8,
    /// Normalized distance from the listener position.
    ///
    /// `0` = at or very close to the listener; `255` = furthest.
    /// The absolute distance is not encoded.
    pub distance: u8,
}

pub(super) fn parse_speaker_location(block_data: &[u8]) -> Vec<SpeakerLocationEntry> {
    // block_data[0] = extended tag; entries start at [1], each 2 bytes.
    block_data[1..]
        .chunks_exact(2)
        .map(|c| SpeakerLocationEntry {
            channel_assignment: c[0],
            distance: c[1],
        })
        .collect()
}

// ---------------------------------------------------------------------------
// InfoFrame Data Block (extended tag 0x20)
// ---------------------------------------------------------------------------

/// Well-known InfoFrame type codes from the InfoFrame Data Block (extended tag `0x20`).
///
/// These correspond to the InfoFrame types defined in HDMI and CTA-861.
pub mod infoframe_type {
    /// Vendor-Specific InfoFrame (VSI). The associated OUI identifies the vendor.
    pub const VENDOR_SPECIFIC: u8 = 0x01;
    /// AVI InfoFrame — active video format, colorimetry, aspect ratio.
    pub const AVI: u8 = 0x02;
    /// Source Product Descriptor InfoFrame.
    pub const SOURCE_PRODUCT_DESCRIPTOR: u8 = 0x03;
    /// Audio InfoFrame.
    pub const AUDIO: u8 = 0x04;
    /// MPEG Source InfoFrame.
    pub const MPEG_SOURCE: u8 = 0x05;
    /// NTSC VBI InfoFrame.
    pub const NTSC_VBI: u8 = 0x06;
    /// Dynamic Range and Mastering InfoFrame (HDR10 static metadata).
    pub const DYNAMIC_RANGE_MASTERING: u8 = 0x07;
}

/// One entry from an InfoFrame Data Block (extended tag `0x20`).
///
/// Each descriptor identifies an InfoFrame type that the sink is capable of
/// receiving. For Vendor-Specific InfoFrames (`type_code == 0x01`) the IEEE OUI
/// of the vendor is also present.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InfoFrameDescriptor {
    /// InfoFrame type code (bits 4:0 of the SID byte).
    ///
    /// See the constants in [`infoframe_type`] for the well-known values.
    pub type_code: u8,
    /// IEEE OUI for Vendor-Specific InfoFrames (`type_code == 0x01`).
    ///
    /// Stored as `(byte0 << 16) | (byte1 << 8) | byte2` following the byte
    /// order used in CTA-861.  `None` for all other types.
    pub vendor_oui: Option<u32>,
}

pub(super) fn parse_infoframe_db(block_data: &[u8]) -> Vec<InfoFrameDescriptor> {
    // block_data[0] = extended tag; SIDs start at [1].
    // Each SID: bits 7:5 = additional bytes after this byte, bits 4:0 = type.
    let mut out = Vec::new();
    let payload = &block_data[1..];
    let mut i = 0;

    while i < payload.len() {
        let b = payload[i];
        let type_code = b & 0x1F;
        let extra = ((b >> 5) & 0x07) as usize;
        i += 1;

        let vendor_oui = if type_code == infoframe_type::VENDOR_SPECIFIC && extra >= 3 {
            let oui = ((payload.get(i).copied().unwrap_or(0) as u32) << 16)
                | ((payload.get(i + 1).copied().unwrap_or(0) as u32) << 8)
                | (payload.get(i + 2).copied().unwrap_or(0) as u32);
            Some(oui)
        } else {
            None
        };

        out.push(InfoFrameDescriptor {
            type_code,
            vendor_oui,
        });

        // Advance past the extra bytes, clamping to remaining payload.
        i += extra.min(payload.len().saturating_sub(i));
    }

    out
}

// ---------------------------------------------------------------------------
// Vendor-Specific Video Data Block (extended tag 0x01)
// Vendor-Specific Audio Data Block (extended tag 0x11)
// ---------------------------------------------------------------------------

/// A decoded Vendor-Specific Video Data Block (VSVDB, extended tag `0x01`) or
/// Vendor-Specific Audio Data Block (VSADB, extended tag `0x11`).
///
/// Both block types share the same structure: a 3-byte IEEE OUI followed by an
/// opaque vendor-defined payload (CTA-861 Tables 56–57). The payload is stored
/// verbatim for consumers that recognise the OUI.
///
/// Well-known video OUIs include Dolby Vision (`0x00D046`) and HDR10+ (`0x90848B`).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VendorSpecificBlock {
    /// 24-bit IEEE OUI in canonical (MSB-first) form.
    ///
    /// Assembled from the three OUI bytes as `(byte2 << 16) | (byte1 << 8) | byte0`,
    /// where byte0 is the least-significant byte as stored on the wire.
    pub oui: u32,
    /// Vendor-defined payload bytes following the OUI.
    pub payload: Vec<u8>,
}

/// Parse a VSVDB or VSADB payload (`block_data` starts after the extended tag byte).
///
/// Returns `None` if the payload is shorter than the 3-byte OUI minimum.
pub(super) fn parse_vendor_specific_block(block_data: &[u8]) -> Option<VendorSpecificBlock> {
    if block_data.len() < 3 {
        return None;
    }
    let oui =
        ((block_data[2] as u32) << 16) | ((block_data[1] as u32) << 8) | (block_data[0] as u32);
    let payload = block_data[3..].to_vec();
    Some(VendorSpecificBlock { oui, payload })
}

// ---------------------------------------------------------------------------
// DisplayID Type VII Video Timing Data Block (extended tag 0x22)
// ---------------------------------------------------------------------------

/// A decoded DisplayID Type VII Video Timing Data Block (T7VTDB, extended tag `0x22`).
///
/// Each CTA T7VTDB carries exactly one 20-byte DisplayID-style timing descriptor.
/// Unlike an 18-byte EDID DTD, the pixel clock is expressed in kHz (not 10 kHz units)
/// and all horizontal/vertical fields are 16-bit rather than packed.
///
/// Multiple T7VTDBs are permitted per CTA extension block (one timing per block).
/// Per CTA-861, `interlaced` shall always be `false`; `y420` reflects the T7Y420 flag.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct T7VtdbBlock {
    /// Block revision (`Block_Rev` field, bits 2:0 of the descriptor header byte).
    /// CTA-861 expects revision `0x02`.
    pub version: u8,
    /// Decoded video timing for this descriptor.
    pub mode: VideoMode,
    /// When `true`, this timing also supports YCbCr 4:2:0 sampling (T7Y420 flag).
    pub y420: bool,
}

/// Parse a T7VTDB payload (`block_data` starts after the extended tag byte, at the descriptor
/// header). Returns `None` for payloads shorter than 22 bytes, zero pixel clock, or geometry
/// that would produce a refresh rate outside the range of `u8`.
pub(super) fn parse_t7vtdb(block_data: &[u8]) -> Option<T7VtdbBlock> {
    // Layout (offsets after the ext tag byte, i.e. starting at the descriptor header):
    //   [0]      T7_M[7:5] | DSC_PT[4] | reserved[3] | Block_Rev[2:0]
    //   [1]      F37[7]=0  | T7Y420[6] | T7HSP[5] | T7VSP[4] | reserved[3:0]
    //   [2..4]   Pixel clock in kHz (24-bit LE)
    //   [5]      3D_Support[7:6] | reserved[5] | T7IL[4] | T7_Aspect_Ratio[3:0]
    //   [6..7]   H Active (16-bit LE)
    //   [8..9]   H Blank  (16-bit LE)
    //   [10..11] H Offset / Front Porch (15-bit: low byte + high byte bits[6:0])
    //   [12..13] H Sync Width (16-bit LE)
    //   [14..15] V Active (16-bit LE)
    //   [16..17] V Blank  (16-bit LE)
    //   [18..19] V Offset / Front Porch (15-bit)
    //   [20..21] V Sync Width (16-bit LE)
    if block_data.len() < 22 {
        return None;
    }

    let version = block_data[0] & 0x07;
    let y420 = (block_data[1] >> 6) & 1 != 0;

    let pixel_clock_khz =
        (block_data[2] as u32) | ((block_data[3] as u32) << 8) | ((block_data[4] as u32) << 16);
    if pixel_clock_khz == 0 {
        return None;
    }

    let interlaced = (block_data[5] >> 4) & 1 != 0;

    let hactive = u16::from_le_bytes([block_data[6], block_data[7]]);
    let hblank = u16::from_le_bytes([block_data[8], block_data[9]]);
    let vactive = u16::from_le_bytes([block_data[14], block_data[15]]);
    let vblank = u16::from_le_bytes([block_data[16], block_data[17]]);

    if hactive == 0 || vactive == 0 || hblank == 0 || vblank == 0 {
        return None;
    }

    let h_total = hactive as u64 + hblank as u64;
    let v_total = vactive as u64 + vblank as u64;
    let refresh_hz = (pixel_clock_khz as u64 * 1000) / (h_total * v_total);
    let refresh_rate = u8::try_from(refresh_hz).ok()?;

    let mode = VideoMode {
        width: hactive,
        height: vactive,
        refresh_rate,
        interlaced,
        ..Default::default()
    };

    Some(T7VtdbBlock {
        version,
        mode,
        y420,
    })
}

// ---------------------------------------------------------------------------
// DisplayID Type VIII Video Timing Data Block (extended tag 0x23)
// ---------------------------------------------------------------------------

/// A decoded DisplayID Type VIII Video Timing Data Block (T8VTDB, extended tag `0x23`).
///
/// Contains a list of VESA DMT (Display Monitor Timings) ID codes referencing
/// standardised monitor timings. Only `Code_Type = 0x00` (DMT) is defined by
/// CTA-861; other code types are returned as `None` by the parser.
///
/// Codes whose DMT IDs are not in the standard table are stored in `codes` but
/// omitted from `timings`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct T8VtdbBlock {
    /// Block revision (`Block_Rev` field, bits 2:0).
    pub version: u8,
    /// When `true`, all timings also support YCbCr 4:2:0 sampling (T8Y420 flag).
    pub y420: bool,
    /// Raw DMT timing codes as they appear in the block (1-byte or 2-byte each).
    pub codes: Vec<u16>,
    /// `VideoMode` values resolved from the DMT codes. Entries for unrecognised
    /// or reserved DMT IDs are omitted.
    pub timings: Vec<VideoMode>,
}

/// Look up a VESA DMT ID and return the corresponding `VideoMode`, or `None`
/// if the ID is not in the standard table.
///
/// Source: VESA DMT v1.13, Table 2-1 (cross-referenced against Linux kernel
/// `drivers/gpu/drm/drm_edid.c`).
///
/// When two DMT IDs share the same resolution and refresh rate (one Reduced
/// Blanking variant, one standard), both map to the same `VideoMode`.
/// The interlaced 1024×768@43 Hz entry (0x0F) is included; 0x58 (4096×2160 @
/// 59.94 Hz) is stored as 60 Hz because `VideoMode` uses integer refresh rates.
pub(super) fn dmt_to_mode(id: u16) -> Option<VideoMode> {
    let (width, height, refresh_rate, interlaced): (u16, u16, u8, bool) = match id {
        0x01 => (640, 350, 85, false),
        0x02 => (640, 400, 85, false),
        0x03 => (720, 400, 85, false),
        0x04 => (640, 480, 60, false),
        0x05 => (640, 480, 72, false),
        0x06 => (640, 480, 75, false),
        0x07 => (640, 480, 85, false),
        0x08 => (800, 600, 56, false),
        0x09 => (800, 600, 60, false),
        0x0A => (800, 600, 72, false),
        0x0B => (800, 600, 75, false),
        0x0C => (800, 600, 85, false),
        0x0D => (800, 600, 120, false),
        0x0E => (848, 480, 60, false),
        0x0F => (1024, 768, 43, true), // interlaced (1024×768i @ 43 Hz)
        0x10 => (1024, 768, 60, false),
        0x11 => (1024, 768, 70, false),
        0x12 => (1024, 768, 75, false),
        0x13 => (1024, 768, 85, false),
        0x14 => (1024, 768, 120, false),
        0x15 => (1152, 864, 75, false),
        0x16 => (1280, 768, 60, false), // reduced blanking
        0x17 => (1280, 768, 60, false),
        0x18 => (1280, 768, 75, false),
        0x19 => (1280, 768, 85, false),
        0x1A => (1280, 768, 120, false),
        0x1B => (1280, 800, 60, false), // reduced blanking
        0x1C => (1280, 800, 60, false),
        0x1D => (1280, 800, 75, false),
        0x1E => (1280, 800, 85, false),
        0x1F => (1280, 800, 120, false),
        0x20 => (1280, 960, 60, false),
        0x21 => (1280, 960, 85, false),
        0x22 => (1280, 960, 120, false),
        0x23 => (1280, 1024, 60, false),
        0x24 => (1280, 1024, 75, false),
        0x25 => (1280, 1024, 85, false),
        0x26 => (1280, 1024, 120, false),
        0x27 => (1360, 768, 60, false),
        0x28 => (1360, 768, 120, false),
        0x29 => (1400, 1050, 60, false), // reduced blanking
        0x2A => (1400, 1050, 60, false),
        0x2B => (1400, 1050, 75, false),
        0x2C => (1400, 1050, 85, false),
        0x2D => (1400, 1050, 120, false),
        0x2E => (1440, 900, 60, false), // reduced blanking
        0x2F => (1440, 900, 60, false),
        0x30 => (1440, 900, 75, false),
        0x31 => (1440, 900, 85, false),
        0x32 => (1440, 900, 120, false),
        0x33 => (1600, 1200, 60, false),
        0x34 => (1600, 1200, 65, false),
        0x35 => (1600, 1200, 70, false),
        0x36 => (1600, 1200, 75, false),
        0x37 => (1600, 1200, 85, false),
        0x38 => (1600, 1200, 120, false),
        0x39 => (1680, 1050, 60, false), // reduced blanking
        0x3A => (1680, 1050, 60, false),
        0x3B => (1680, 1050, 75, false),
        0x3C => (1680, 1050, 85, false),
        0x3D => (1680, 1050, 120, false),
        0x3E => (1792, 1344, 60, false),
        0x3F => (1792, 1344, 75, false),
        0x40 => (1792, 1344, 120, false),
        0x41 => (1856, 1392, 60, false),
        0x42 => (1856, 1392, 75, false),
        0x43 => (1856, 1392, 120, false),
        0x44 => (1920, 1200, 60, false), // reduced blanking
        0x45 => (1920, 1200, 60, false),
        0x46 => (1920, 1200, 75, false),
        0x47 => (1920, 1200, 85, false),
        0x48 => (1920, 1200, 120, false),
        0x49 => (1920, 1440, 60, false),
        0x4A => (1920, 1440, 75, false),
        0x4B => (1920, 1440, 120, false),
        0x4C => (2560, 1600, 60, false), // reduced blanking
        0x4D => (2560, 1600, 60, false),
        0x4E => (2560, 1600, 75, false),
        0x4F => (2560, 1600, 85, false),
        0x50 => (2560, 1600, 120, false),
        0x51 => (1366, 768, 60, false),
        0x52 => (1920, 1080, 60, false),
        0x53 => (1600, 900, 60, false),
        0x54 => (2048, 1152, 60, false),
        0x55 => (1280, 720, 60, false),
        0x56 => (1366, 768, 60, false),
        0x57 => (4096, 2160, 60, false),
        0x58 => (4096, 2160, 60, false), // 59.94 Hz, stored as 60 (integer field)
        _ => return None,
    };
    Some(VideoMode {
        width,
        height,
        refresh_rate,
        interlaced,
        ..Default::default()
    })
}

/// Parse a T8VTDB payload (`block_data` starts after the extended tag byte).
///
/// Returns `None` for an empty payload or a non-DMT `Code_Type` (CTA-861 only
/// defines `Code_Type = 0x00` for DMT).
pub(super) fn parse_t8vtdb(block_data: &[u8]) -> Option<T8VtdbBlock> {
    // block_data[0]: Code_Type[7:6] | T8Y420[5] | F34[4]=0 | TCS[3] | Block_Rev[2:0]
    let header = *block_data.first()?;
    let code_type = (header >> 6) & 0x03;
    if code_type != 0x00 {
        return None; // only DMT is defined for CTA-861
    }
    let y420 = (header >> 5) & 1 != 0;
    let tcs = (header >> 3) & 1 != 0; // false = 1-byte codes, true = 2-byte codes
    let version = header & 0x07;

    let payload = &block_data[1..];
    let mut codes: Vec<u16> = Vec::new();
    let mut timings: Vec<VideoMode> = Vec::new();

    if tcs {
        // 2-byte codes stored LSB-first; trailing incomplete pairs are ignored.
        let mut i = 0;
        while i + 2 <= payload.len() {
            let code = u16::from_le_bytes([payload[i], payload[i + 1]]);
            codes.push(code);
            if let Some(mode) = dmt_to_mode(code) {
                timings.push(mode);
            }
            i += 2;
        }
    } else {
        for &byte in payload {
            let code = byte as u16;
            codes.push(code);
            if let Some(mode) = dmt_to_mode(code) {
                timings.push(mode);
            }
        }
    }

    Some(T8VtdbBlock {
        version,
        y420,
        codes,
        timings,
    })
}

// ---------------------------------------------------------------------------
// DisplayID Type X Video Timing Data Block (extended tag 0x2A)
// ---------------------------------------------------------------------------

/// A single timing entry from a T10VTDB block.
///
/// Type X timings use a CVT formula to derive the full signal, but only the
/// display-facing parameters (resolution and refresh rate) are exposed here.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct T10VtdbEntry {
    /// Horizontal active pixels.
    pub width: u16,
    /// Vertical active lines.
    pub height: u16,
    /// Vertical refresh rate in Hz (1–1024).
    ///
    /// Values above 255 are only possible when the block uses 7- or 8-byte
    /// descriptors (M ≥ 1 in the `rev` byte).
    pub refresh_hz: u16,
    /// When `true`, this timing also supports YCbCr 4:2:0 sampling (YCC420 flag).
    pub y420: bool,
}

/// A decoded DisplayID Type X Video Timing Data Block (T10VTDB, extended tag `0x2A`).
///
/// Type X timings are CVT formula-based: each descriptor encodes the active
/// resolution and refresh rate directly, with blanking derived by the display.
/// A block may contain 1–4 descriptors (limited by the 30-byte CTA extended
/// block payload cap).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct T10VtdbBlock {
    /// Decoded timing entries. Each entry corresponds to one descriptor in the block.
    pub entries: Vec<T10VtdbEntry>,
}

/// Parse a T10VTDB payload (`block_data` starts after the extended tag byte).
///
/// Returns `None` for an empty payload or an invalid descriptor-size extension
/// (`M > 2`). Descriptors that do not divide evenly into the payload are ignored.
///
/// Layout (CTA-861-H Table 109–110 / DisplayID 2.x Type X):
/// - `block_data[0]`: `rev` byte — bits[6:4] = M (0→6 B, 1→7 B, 2→8 B per descriptor)
/// - `block_data[1..]`: array of N descriptors of `6 + M` bytes each
///
/// Per-descriptor layout:
/// - `[0]`: flags — `YCC420[7]`, `Stereo[6:5]`, `VR_HB[4]`, `EVS[3]`, `Formula[2:0]`
/// - `[1..2]`: H active − 1 (LE u16)
/// - `[3..4]`: V active − 1 (LE u16)
/// - `[5]`: refresh rate LSB (stored − 1, i.e. actual = `x[5] + 1`)
/// - `[6]` (M≥1): bits[1:0] = refresh rate high 2 bits
pub(super) fn parse_t10vtdb(block_data: &[u8]) -> Option<T10VtdbBlock> {
    let rev = *block_data.first()?;
    let m = (rev >> 4) & 0x07;
    if m > 2 {
        return None; // invalid per spec
    }
    let sz = 6 + m as usize;

    let payload = &block_data[1..];
    let mut entries = Vec::new();
    let mut i = 0;

    while i + sz <= payload.len() {
        let d = &payload[i..i + sz];

        let flags = d[0];
        let y420 = (flags >> 7) & 1 != 0;

        let width = u16::from_le_bytes([d[1], d[2]]).saturating_add(1);
        let height = u16::from_le_bytes([d[3], d[4]]).saturating_add(1);

        let refresh_lsb = d[5] as u16;
        let refresh_hz = if m >= 1 {
            let msb = (d[6] & 0x03) as u16;
            (refresh_lsb | (msb << 8)) + 1
        } else {
            refresh_lsb + 1
        };

        entries.push(T10VtdbEntry {
            width,
            height,
            refresh_hz,
            y420,
        });

        i += sz;
    }

    Some(T10VtdbBlock { entries })
}

// ---------------------------------------------------------------------------
// HDMI Forum EDID Extension Override Data Block (extended tag 0x78)
// ---------------------------------------------------------------------------

/// Parse the HF-EEODB payload (`block_data` starts after the extended tag byte).
///
/// Returns the EDID extension block count override, or `None` if the payload
/// is empty or the count is zero (meaningless).
///
/// The HF-EEODB is defined in HDMI 2.1 section 10.3.6. It must be the first
/// data block of Block 1 and overrides the extension count in the base EDID
/// header for HDMI 2.1 sinks whose full E-EDID exceeds what the 1-byte base
/// count can represent.
pub(super) fn parse_hf_eeodb(block_data: &[u8]) -> Option<u8> {
    let count = *block_data.first()?;
    if count == 0 {
        return None;
    }
    Some(count)
}

// ---------------------------------------------------------------------------
// HDMI Forum Sink Capability Data Block (extended tag 0x79)
// ---------------------------------------------------------------------------

/// Maximum Fixed Rate Link (FRL) bandwidth supported by a HDMI 2.1 sink.
///
/// Source: HDMI 2.1a, table of `Max_FRL_Rate` values.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdmiForumFrl {
    /// FRL not supported; TMDS only.
    NotSupported,
    /// Up to 3 Gbps/lane on 3 lanes (≈ 9 Gbps total).
    Rate3Gbps3Lanes,
    /// Up to 6 Gbps/lane on 3 lanes (≈ 18 Gbps).
    Rate6Gbps3Lanes,
    /// Up to 6 Gbps/lane on 3 lanes and 4 lanes (≈ 24 Gbps).
    Rate6Gbps4Lanes,
    /// Up to 8 Gbps/lane on 4 lanes (≈ 32 Gbps).
    Rate8Gbps4Lanes,
    /// Up to 10 Gbps/lane on 4 lanes (≈ 40 Gbps).
    Rate10Gbps4Lanes,
    /// Up to 12 Gbps/lane on 4 lanes (≈ 48 Gbps).
    Rate12Gbps4Lanes,
}

impl HdmiForumFrl {
    fn from_raw(raw: u8) -> Self {
        match raw {
            0 => Self::NotSupported,
            1 => Self::Rate3Gbps3Lanes,
            2 => Self::Rate6Gbps3Lanes,
            3 => Self::Rate6Gbps4Lanes,
            4 => Self::Rate8Gbps4Lanes,
            5 => Self::Rate10Gbps4Lanes,
            6 => Self::Rate12Gbps4Lanes,
            _ => Self::NotSupported, // reserved values
        }
    }
}

/// Maximum number of horizontal DSC slices supported by a HDMI 2.1 sink.
///
/// Source: HDMI 2.1a `DSC_MaxSlices` field.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdmiDscMaxSlices {
    /// DSC not supported.
    NotSupported,
    /// Up to 1 slice; ≤ 340 MHz pixel clock per slice.
    Slices1,
    /// Up to 2 slices; ≤ 340 MHz per slice.
    Slices2,
    /// Up to 4 slices; ≤ 340 MHz per slice.
    Slices4,
    /// Up to 8 slices; ≤ 340 MHz per slice.
    Slices8At340Mhz,
    /// Up to 8 slices; ≤ 400 MHz per slice.
    Slices8At400Mhz,
    /// Up to 12 slices; ≤ 400 MHz per slice.
    Slices12,
    /// Up to 16 slices; ≤ 400 MHz per slice.
    Slices16,
}

impl HdmiDscMaxSlices {
    fn from_raw(raw: u8) -> Self {
        match raw {
            0 => Self::NotSupported,
            1 => Self::Slices1,
            2 => Self::Slices2,
            3 => Self::Slices4,
            4 => Self::Slices8At340Mhz,
            5 => Self::Slices8At400Mhz,
            6 => Self::Slices12,
            7 => Self::Slices16,
            _ => Self::NotSupported, // reserved values
        }
    }
}

/// Display Stream Compression (DSC) capabilities from the HF-SCDB.
///
/// Present only when the block carries the optional DSC section (≥ 10 bytes
/// of SCDS payload).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HdmiForumDsc {
    /// Supports VESA DSC 1.2a compressed video transport.
    pub dsc_1p2: bool,
    /// Supports DSC for YCbCr 4:2:0 pixel encoding (`DSC_Native_420`).
    pub native_420: bool,
    /// QMS TFR_max: highest QMS frame rate equals `vrr_max_hz` (otherwise 60 Hz).
    pub qms_tfr_max: bool,
    /// QMS TFR_min: lowest QMS frame rate equals `vrr_min_hz` (otherwise 24/1.001 Hz).
    pub qms_tfr_min: bool,
    /// Supports DSC at any valid 1/16th-bit bits-per-pixel value (`DSC_All_BPC`).
    pub all_bpc: bool,
    /// Supports 12 bpc DSC compressed video transport.
    pub bpc12: bool,
    /// Supports 10 bpc DSC compressed video transport.
    pub bpc10: bool,
    /// Maximum FRL rate for DSC transport.
    pub max_frl_rate: HdmiForumFrl,
    /// Maximum number of horizontal DSC slices.
    pub max_slices: HdmiDscMaxSlices,
    /// Maximum total bytes per line of chunks: `1024 × (1 + raw_field)`. `0` = not reported.
    pub max_chunk_bytes: u32,
}

/// Decoded HDMI Forum Sink Capability Data Block (HF-SCDB, extended tag `0x79`).
///
/// Defined in HDMI 2.1a section 10.3.6. Structure reconstructed from the Linux
/// kernel `drm_edid.c` and the edid-decode reference implementation.
///
/// The block contains two reserved bytes after the extended tag, followed by the
/// HDMI Sink Capability Data Structure (SCDS). Optional extended fields (VRR
/// range, DSC capabilities) are present only when the block is long enough.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HdmiForumSinkCap {
    /// SCDS version field; expected to be `1`.
    pub version: u8,
    /// Maximum TMDS character rate in MHz (`raw_byte × 5`). `0` means ≤ 340 MHz.
    pub max_tmds_rate_mhz: u16,

    // ---- SCDC and 3D flags (byte 2 of SCDS) ----
    /// Sink contains a Status and Control Data Channel (SCDC).
    pub scdc_present: bool,
    /// SCDC read request capable (`RR_Capable`).
    pub rr_capable: bool,
    /// Supports cable-status indication via SCDC.
    pub cable_status: bool,
    /// Supports Color Content Bits Per Component Indication (`CCBPCI`).
    pub ccbpci: bool,
    /// Supports scrambling at TMDS rates ≤ 340 Mcsc (`LTE_340Mcsc_Scramble`).
    pub lte_340mcsc_scramble: bool,
    /// Supports 3D Independent View signaling (`Independent_View`).
    pub independent_view_3d: bool,
    /// Supports 3D Dual View signaling (`Dual_View`).
    pub dual_view_3d: bool,
    /// Supports 3D OSD Disparity signaling (`OSD_Disparity`).
    pub osd_disparity_3d: bool,

    // ---- FRL and Deep Color flags (byte 3 of SCDS) ----
    /// Maximum Fixed Rate Link bandwidth.
    pub max_frl_rate: HdmiForumFrl,
    /// Supports UHD VIC indication in AVI InfoFrame (`UHD_VIC`).
    pub uhd_vic: bool,
    /// Supports 16 bpc (48-bit) Deep Color in YCbCr 4:2:0 (`DC_48bit_420`).
    pub dc_48bit_420: bool,
    /// Supports 12 bpc (36-bit) Deep Color in YCbCr 4:2:0 (`DC_36bit_420`).
    pub dc_36bit_420: bool,
    /// Supports 10 bpc (30-bit) Deep Color in YCbCr 4:2:0 (`DC_30bit_420`).
    pub dc_30bit_420: bool,

    // ---- Optional extended feature flags (byte 4 of SCDS; false if absent) ----
    /// Supports FAPA end extended to the Vactive upper bound of 360 µs.
    pub fapa_end_extended: bool,
    /// Supports QMS VRR (Quality Media Synchronization Variable Refresh Rate).
    pub qms: bool,
    /// Has a limit on rate-of-change variations in `M_VRR` values (`M_Delta`).
    pub m_delta: bool,
    /// Supports Fast VActive (`FVA`).
    pub fva: bool,
    /// Supports Auto Low-Latency Mode (`ALLM`).
    pub allm: bool,
    /// FAPA starts on first horizontal blank after first active video pixel.
    pub fapa_start_location: bool,
    /// Supports negative `M_VRR` values (`Neg_MVRR`).
    pub neg_mvrr: bool,

    // ---- Optional VRR range (bytes 5–6 of SCDS; None if absent) ----
    /// Minimum VRR frame rate in Hz. `Some(0)` means VRR is not supported.
    /// `None` means this section was not present in the block.
    pub vrr_min_hz: Option<u8>,
    /// Maximum VRR frame rate in Hz (10-bit). `Some(0)` means VRR is not supported.
    /// `None` means this section was not present in the block.
    pub vrr_max_hz: Option<u16>,

    // ---- Optional DSC capabilities (bytes 7–9 of SCDS; None if absent) ----
    /// Display Stream Compression capabilities. `None` if the DSC section is absent.
    pub dsc: Option<HdmiForumDsc>,
}

/// Parse an HF-SCDB payload (`block_data` starts after the extended tag byte).
///
/// Returns `None` if the block is too short (fewer than 6 bytes: 2 reserved +
/// 4 mandatory SCDS bytes).
///
/// Layout (offsets from start of `block_data`):
/// - `[0..1]`: Reserved (0x00)
/// - `[2]`: SCDS Version (expected 1)
/// - `[3]`: Max_TMDS_Character_Rate (× 5 = MHz; 0 → ≤ 340 MHz)
/// - `[4]`: SCDC/3D flags
/// - `[5]`: Max_FRL_Rate[7:4] | UHD_VIC[3] | DC_420 flags[2:0]
/// - `[6]`: optional — FAPA/QMS/ALLM/VRR feature flags
/// - `[7]`: optional — VRRmax[9:8] | VRRmin[5:0]
/// - `[8]`: optional — VRRmax[7:0]
/// - `[9]`: optional — DSC capability flags
/// - `[10]`: optional — DSC_Max_FRL_Rate[7:4] | DSC_MaxSlices[3:0]
/// - `[11]`: optional — DSC_MaxTotalChunkBytes[5:0]
pub(super) fn parse_hf_scdb(block_data: &[u8]) -> Option<HdmiForumSinkCap> {
    if block_data.len() < 6 {
        return None;
    }

    let version = block_data[2];
    let max_tmds_rate_mhz = (block_data[3] as u16) * 5;

    let scdc = block_data[4];
    let scdc_present = (scdc >> 7) & 1 != 0;
    let rr_capable = (scdc >> 6) & 1 != 0;
    let cable_status = (scdc >> 5) & 1 != 0;
    let ccbpci = (scdc >> 4) & 1 != 0;
    let lte_340mcsc_scramble = (scdc >> 3) & 1 != 0;
    let independent_view_3d = (scdc >> 2) & 1 != 0;
    let dual_view_3d = (scdc >> 1) & 1 != 0;
    let osd_disparity_3d = scdc & 1 != 0;

    let frl_dc = block_data[5];
    let max_frl_rate = HdmiForumFrl::from_raw((frl_dc >> 4) & 0x0F);
    let uhd_vic = (frl_dc >> 3) & 1 != 0;
    let dc_48bit_420 = (frl_dc >> 2) & 1 != 0;
    let dc_36bit_420 = (frl_dc >> 1) & 1 != 0;
    let dc_30bit_420 = frl_dc & 1 != 0;

    // Optional extended section: feature flags (byte 6) + VRR range (bytes 7–8).
    let (
        fapa_end_extended,
        qms,
        m_delta,
        fva,
        allm,
        fapa_start_location,
        neg_mvrr,
        vrr_min_hz,
        vrr_max_hz,
    ) = if let Some(&ext) = block_data.get(6) {
        let fapa_end_extended = (ext >> 7) & 1 != 0;
        let qms = (ext >> 6) & 1 != 0;
        let m_delta = (ext >> 5) & 1 != 0;
        // bit 4 = CinemaVRR (deprecated, ignored)
        let neg_mvrr = (ext >> 3) & 1 != 0;
        let fva = (ext >> 2) & 1 != 0;
        let allm = (ext >> 1) & 1 != 0;
        let fapa_start_location = ext & 1 != 0;

        let vrr = block_data.get(7).and_then(|&b7| {
            block_data.get(8).map(|&b8| {
                let vrr_min = b7 & 0x3F;
                let vrr_max = ((b7 >> 6) as u16) << 8 | b8 as u16;
                (vrr_min, vrr_max)
            })
        });
        let (vrr_min_hz, vrr_max_hz) = match vrr {
            Some((min, max)) => (Some(min), Some(max)),
            None => (None, None),
        };
        (
            fapa_end_extended,
            qms,
            m_delta,
            fva,
            allm,
            fapa_start_location,
            neg_mvrr,
            vrr_min_hz,
            vrr_max_hz,
        )
    } else {
        (false, false, false, false, false, false, false, None, None)
    };

    // Optional DSC section (bytes 9–11).
    let dsc = block_data.get(9).and_then(|&dsc_flags| {
        let dsc_frl_slices = *block_data.get(10)?;
        let chunk_raw = block_data.get(11).map_or(0, |&b| b & 0x3F);

        let max_chunk_bytes = if chunk_raw == 0 {
            0
        } else {
            1024 * (1 + chunk_raw as u32)
        };

        Some(HdmiForumDsc {
            dsc_1p2: (dsc_flags >> 7) & 1 != 0,
            native_420: (dsc_flags >> 6) & 1 != 0,
            qms_tfr_max: (dsc_flags >> 5) & 1 != 0,
            qms_tfr_min: (dsc_flags >> 4) & 1 != 0,
            all_bpc: (dsc_flags >> 3) & 1 != 0,
            // bit 2 is reserved in SCDB (valid in HF-VSDB only)
            bpc12: (dsc_flags >> 1) & 1 != 0,
            bpc10: dsc_flags & 1 != 0,
            max_frl_rate: HdmiForumFrl::from_raw((dsc_frl_slices >> 4) & 0x0F),
            max_slices: HdmiDscMaxSlices::from_raw(dsc_frl_slices & 0x0F),
            max_chunk_bytes,
        })
    });

    Some(HdmiForumSinkCap {
        version,
        max_tmds_rate_mhz,
        scdc_present,
        rr_capable,
        cable_status,
        ccbpci,
        lte_340mcsc_scramble,
        independent_view_3d,
        dual_view_3d,
        osd_disparity_3d,
        max_frl_rate,
        uhd_vic,
        dc_48bit_420,
        dc_36bit_420,
        dc_30bit_420,
        fapa_end_extended,
        qms,
        m_delta,
        fva,
        allm,
        fapa_start_location,
        neg_mvrr,
        vrr_min_hz,
        vrr_max_hz,
        dsc,
    })
}

// ---------------------------------------------------------------------------
// VESA Display Device Data Block (extended tag 0x02)
// ---------------------------------------------------------------------------

/// Decoded VESA Display Device Data Block (extended tag `0x02`).
///
/// A fixed 30-byte payload describing the physical and electrical characteristics
/// of the display, per the VESA Display Device Data Block (DDDB) Standard, Version 1.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VesaDisplayDeviceBlock {
    /// Interface type code (bits 7:4). 0=Analog, 1=LVDS, 3=DVI-D, 6=HDMI-A, 9=DP, etc.
    pub interface_type: u8,
    /// Number of lanes/channels (bits 3:0). For analog interfaces this is a subtype code.
    pub num_links: u8,
    /// Interface standard version (bits 7:4 of byte 0x03).
    pub interface_version: u8,
    /// Interface standard release (bits 3:0 of byte 0x03).
    pub interface_release: u8,
    /// Content protection code (byte 0x04). 0=none, 1=HDCP, 2=DTCP, 3=DPCP.
    pub content_protection: u8,
    /// Minimum supported clock frequency per link in MHz (6-bit, range 0–63).
    pub min_clock_mhz: u8,
    /// Maximum supported clock frequency per link in MHz (10-bit, range 0–1023).
    pub max_clock_mhz: u16,
    /// Native horizontal pixel count, or `None` if the display has no fixed format.
    pub native_width: Option<u16>,
    /// Native vertical pixel count, or `None` if the display has no fixed format.
    pub native_height: Option<u16>,
    /// Aspect ratio raw byte. Physical AR = (raw / 100.0) + 1.0 (long-axis / short-axis).
    pub aspect_ratio_raw: u8,
    /// Default orientation: 0=landscape, 1=portrait, 2=not_fixed, 3=undefined.
    pub default_orientation: u8,
    /// Rotation capability: 0=none, 1=90°CW, 2=90°CCW, 3=both.
    pub rotation_capability: u8,
    /// Zero pixel (scan origin) location: 0=upper-left, 1=upper-right, 2=lower-left, 3=lower-right.
    pub zero_pixel_location: u8,
    /// Scan direction: 0=undefined, 1=long-axis-fast, 2=short-axis-fast, 3=reserved.
    pub scan_direction: u8,
    /// Subpixel layout code (byte 0x0D). 0=undefined, 1=RGB-V, 2=RGB-H, etc.
    pub subpixel_layout: u8,
    /// Horizontal pixel pitch in 0.01 mm increments (range 0.00–2.55 mm).
    pub h_pitch_hundredths_mm: u8,
    /// Vertical pixel pitch in 0.01 mm increments (range 0.00–2.55 mm).
    pub v_pitch_hundredths_mm: u8,
    /// Dithering type: 0=none, 1=spatial, 2=temporal, 3=spatial+temporal.
    pub dithering: u8,
    /// Display is direct-drive — no internal scaling/de-interlacing/FRC.
    pub direct_drive: bool,
    /// Video source should not apply overdrive for this display.
    pub overdrive_not_recommended: bool,
    /// Display can de-interlace interlaced input to progressive scan.
    pub deinterlacing: bool,
    /// Audio is supported on the video interface.
    pub audio_on_video_interface: bool,
    /// Separate audio inputs are provided independently of the video interface.
    pub separate_audio_inputs: bool,
    /// Audio received on the video interface automatically overrides other audio inputs.
    pub audio_input_override: bool,
    /// Signed audio delay in milliseconds (positive = audio after video, negative = before).
    /// `None` means no delay information is provided (raw delay byte was 0x00).
    pub audio_delay_ms: Option<i16>,
    /// Frame-rate conversion capability: 0=none, 1=single-buffer, 2=double-buffer, 3=interpolation.
    pub frame_rate_conversion: u8,
    /// Maximum excursion (±FPS) from the nominal frame rate (6 bits, 0–63).
    pub frame_rate_range: u8,
    /// Native or nominal display frame rate in frames/second.
    pub native_frame_rate: u8,
    /// Color bit depth per primary on the video interface (1–16).
    pub interface_color_depth: u8,
    /// Color bit depth per primary at the display panel without temporal dithering (1–16).
    pub display_color_depth: u8,
    /// Raw bytes 0x16–0x1D: chromaticity data for up to three additional primaries.
    pub additional_chromaticities: [u8; 8],
    /// Response time in milliseconds (0 = < 1 ms, 127 = > 126 ms).
    pub response_time_ms: u8,
    /// `true` if the response time is white-to-black; `false` if black-to-white.
    pub response_time_white_to_black: bool,
    /// Percentage of active image outside the visible screen area, horizontally (0–15%).
    pub h_overscan_pct: u8,
    /// Percentage of active image outside the visible screen area, vertically (0–15%).
    pub v_overscan_pct: u8,
}

pub(super) fn parse_vesa_display_device(block_data: &[u8]) -> Option<VesaDisplayDeviceBlock> {
    // Fixed 30-byte payload after the extended tag byte.
    if block_data.len() < 30 {
        return None;
    }

    let interface_type = block_data[0] >> 4;
    let num_links = block_data[0] & 0x0F;
    let interface_version = block_data[1] >> 4;
    let interface_release = block_data[1] & 0x0F;
    let content_protection = block_data[2];

    // Clock frequency:
    //   Byte 0x05: [m5 m4 m3 m2 m1 m0 M9 M8]  → min = bits 7:2, max hi = bits 1:0
    //   Byte 0x06: [M7 M6 M5 M4 M3 M2 M1 M0]  → max lo
    let min_clock_mhz = block_data[3] >> 2;
    let max_clock_mhz = ((block_data[3] as u16 & 0x03) << 8) | block_data[4] as u16;

    // Native pixel format: 16-bit little-endian pairs, stored as (count - 1).
    // All-zero means no fixed pixel format.
    let h_raw = u16::from_le_bytes([block_data[5], block_data[6]]);
    let v_raw = u16::from_le_bytes([block_data[7], block_data[8]]);
    let (native_width, native_height) = if h_raw == 0 && v_raw == 0 {
        (None, None)
    } else {
        (Some(h_raw.saturating_add(1)), Some(v_raw.saturating_add(1)))
    };

    let aspect_ratio_raw = block_data[9];
    let default_orientation = (block_data[10] >> 6) & 0x03;
    let rotation_capability = (block_data[10] >> 4) & 0x03;
    let zero_pixel_location = (block_data[10] >> 2) & 0x03;
    let scan_direction = block_data[10] & 0x03;

    let subpixel_layout = block_data[11];
    let h_pitch_hundredths_mm = block_data[12];
    let v_pitch_hundredths_mm = block_data[13];

    let misc = block_data[14];
    let dithering = (misc >> 6) & 0x03;
    let direct_drive = misc & 0x20 != 0;
    let overdrive_not_recommended = misc & 0x10 != 0;
    let deinterlacing = misc & 0x08 != 0;

    let audio = block_data[15];
    let audio_on_video_interface = audio & 0x80 != 0;
    let separate_audio_inputs = audio & 0x40 != 0;
    let audio_input_override = audio & 0x20 != 0;

    // Audio delay: bit 7 = sign (1 = audio late / positive), bits 6:0 * 2 = ms.
    // Value 0x00 = no delay information provided.
    let delay_raw = block_data[16];
    let audio_delay_ms = if delay_raw == 0 {
        None
    } else {
        let mag = (delay_raw & 0x7F) as i16 * 2;
        Some(if delay_raw & 0x80 != 0 { mag } else { -mag })
    };

    let frame_rate_byte = block_data[17];
    let frame_rate_conversion = (frame_rate_byte >> 6) & 0x03;
    let frame_rate_range = frame_rate_byte & 0x3F;
    let native_frame_rate = block_data[18];

    // Color bit depth: bits 7:4 = interface (value + 1), bits 3:0 = display (value + 1).
    let cbd = block_data[19];
    let interface_color_depth = (cbd >> 4) + 1;
    let display_color_depth = (cbd & 0x0F) + 1;

    let mut additional_chromaticities = [0u8; 8];
    additional_chromaticities.copy_from_slice(&block_data[20..28]);

    let rt = block_data[28];
    let response_time_ms = rt & 0x7F;
    let response_time_white_to_black = rt & 0x80 != 0;

    let overscan = block_data[29];
    let h_overscan_pct = (overscan >> 4) & 0x0F;
    let v_overscan_pct = overscan & 0x0F;

    Some(VesaDisplayDeviceBlock {
        interface_type,
        num_links,
        interface_version,
        interface_release,
        content_protection,
        min_clock_mhz,
        max_clock_mhz,
        native_width,
        native_height,
        aspect_ratio_raw,
        default_orientation,
        rotation_capability,
        zero_pixel_location,
        scan_direction,
        subpixel_layout,
        h_pitch_hundredths_mm,
        v_pitch_hundredths_mm,
        dithering,
        direct_drive,
        overdrive_not_recommended,
        deinterlacing,
        audio_on_video_interface,
        separate_audio_inputs,
        audio_input_override,
        audio_delay_ms,
        frame_rate_conversion,
        frame_rate_range,
        native_frame_rate,
        interface_color_depth,
        display_color_depth,
        additional_chromaticities,
        response_time_ms,
        response_time_white_to_black,
        h_overscan_pct,
        v_overscan_pct,
    })
}

// ---------------------------------------------------------------------------
// VESA Video Timing Block Extension (extended tag 0x03)
// ---------------------------------------------------------------------------

/// Decoded VESA Video Timing Block Extension (extended tag `0x03`).
///
/// Carries additional video timing modes beyond what fits in the base EDID block.
/// Each block may contain Detailed Timing Descriptors (DTBs), Coordinated Video
/// Timings (CVTs), and Standard Timing (ST) entries, per the VESA VTB-EXT Standard,
/// Release A.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct VtbExtBlock {
    /// VTB-EXT version byte (expected `0x01`).
    pub version: u8,
    /// All video modes decoded from this block (DTBs, CVTs, and STs combined).
    pub timings: Vec<VideoMode>,
}

/// Decodes a single 18-byte DTD slice into a [`VideoMode`], without side effects.
/// Returns `None` for non-timing descriptors (zero pixel clock) or invalid geometry.
fn decode_dtd_to_mode(dtd: &[u8]) -> Option<VideoMode> {
    if dtd.len() < 18 || (dtd[0] == 0 && dtd[1] == 0) {
        return None;
    }
    let pixel_clock = ((dtd[1] as u32) << 8) | (dtd[0] as u32);
    if pixel_clock == 0 {
        return None;
    }
    let hactive = (((dtd[4] as u16) & 0xF0) << 4) | (dtd[2] as u16);
    let hblank = (((dtd[4] as u16) & 0x0F) << 8) | (dtd[3] as u16);
    let vactive = (((dtd[7] as u16) & 0xF0) << 4) | (dtd[5] as u16);
    let vblank = (((dtd[7] as u16) & 0x0F) << 8) | (dtd[6] as u16);
    if hactive == 0 || vactive == 0 || hblank == 0 || vblank == 0 {
        return None;
    }
    let total = (hactive + hblank) as u32 * (vactive + vblank) as u32;
    let rate = (pixel_clock * 10_000) / total;
    let refresh_rate = u8::try_from(rate).ok()?;
    Some(VideoMode {
        width: hactive,
        height: vactive,
        refresh_rate,
        interlaced: dtd[17] & 0x80 != 0,
        ..Default::default()
    })
}

pub(super) fn parse_vtb_ext(block_data: &[u8]) -> Option<VtbExtBlock> {
    // Payload layout (after the extended tag byte):
    //   [0] version (expected 0x01)
    //   [1] w = number of 18-byte DTBs  (0–6)
    //   [2] y = number of 3-byte CVTs   (0–40)
    //   [3] z = number of 2-byte STs    (0–61)
    //   [4..] timing data
    if block_data.len() < 4 {
        return None;
    }
    let version = block_data[0];
    let w = block_data[1] as usize;
    let y = block_data[2] as usize;
    let z = block_data[3] as usize;
    let data = &block_data[4..];

    let required = w * 18 + y * 3 + z * 2;
    if data.len() < required {
        return None;
    }

    let mut timings = Vec::new();

    // Detailed Timing Blocks (18 bytes each)
    for i in 0..w {
        if let Some(mode) = decode_dtd_to_mode(&data[i * 18..(i + 1) * 18]) {
            if !timings.contains(&mode) {
                timings.push(mode);
            }
        }
    }

    // CVT 3-byte descriptors
    let cvt_base = w * 18;
    for i in 0..y {
        let off = cvt_base + i * 3;
        let (b0, b1, b2) = (data[off], data[off + 1], data[off + 2]);
        if b0 == 0 {
            continue; // unused slot
        }
        let lines_raw = (((b1 as u16) & 0xF0) << 4) | (b0 as u16);
        let v_add = (lines_raw + 1) * 2;
        let h_add = {
            let v = v_add as u32;
            let h = match (b1 >> 2) & 0x03 {
                0b00 => v * 4 / 3,
                0b01 => v * 16 / 9,
                0b10 => v * 16 / 10,
                _ => continue, // undefined aspect ratio
            };
            ((h / 8) * 8) as u16
        };
        for (mask, rate) in [
            (0x10u8, 50u8),
            (0x08, 60),
            (0x04, 75),
            (0x02, 85),
            (0x01, 60),
        ] {
            if b2 & mask != 0 {
                let mode = VideoMode {
                    width: h_add,
                    height: v_add,
                    refresh_rate: rate,
                    interlaced: false,
                    ..Default::default()
                };
                if !timings.contains(&mode) {
                    timings.push(mode);
                }
            }
        }
    }

    // Standard Timing 2-byte descriptors
    let st_base = w * 18 + y * 3;
    for i in 0..z {
        let off = st_base + i * 2;
        let (b1, b2) = (data[off], data[off + 1]);
        if (b1 == 0x01 && b2 == 0x01) || b1 == 0x00 {
            continue; // unused slot
        }
        let width = (b1 as u16 + 31) * 8;
        let height = match (b2 >> 6) & 0x03 {
            0x00 => (width * 10) / 16, // 16:10
            0x01 => (width * 3) / 4,   // 4:3
            0x02 => (width * 4) / 5,   // 5:4
            _ => (width * 9) / 16,     // 16:9
        };
        let mode = VideoMode {
            width,
            height,
            refresh_rate: (b2 & 0x3F) + 60,
            interlaced: false,
            ..Default::default()
        };
        if !timings.contains(&mode) {
            timings.push(mode);
        }
    }

    Some(VtbExtBlock { version, timings })
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

    #[test]
    fn test_speaker_allocation_basic() {
        // FL/FR + LFE1 + FC set; bytes 2 and 3 absent → zero
        let data = [0x07u8]; // FL_FR | LFE1 | FC
        let sa = parse_speaker_allocation(&data).unwrap();
        assert!(sa.channels.contains(SpeakerAllocationFlags::FL_FR));
        assert!(sa.channels.contains(SpeakerAllocationFlags::LFE1));
        assert!(sa.channels.contains(SpeakerAllocationFlags::FC));
        assert!(!sa.channels.contains(SpeakerAllocationFlags::BL_BR));
        assert_eq!(sa.channels_2, SpeakerAllocationFlags2::empty());
        assert_eq!(sa.channels_3, SpeakerAllocationFlags3::empty());
    }

    #[test]
    fn test_speaker_allocation_extended_bytes() {
        // All three bytes present: FL/FR, TpFL/TpFR, TpBL/TpBR
        let data = [0x01u8, 0x01u8, 0x01u8];
        let sa = parse_speaker_allocation(&data).unwrap();
        assert!(sa.channels.contains(SpeakerAllocationFlags::FL_FR));
        assert!(sa.channels_2.contains(SpeakerAllocationFlags2::TP_FL_FR));
        assert!(sa.channels_3.contains(SpeakerAllocationFlags3::TP_BL_TP_BR));
    }

    #[test]
    fn test_speaker_allocation_empty_returns_none() {
        assert!(parse_speaker_allocation(&[]).is_none());
    }

    #[test]
    fn test_hdr_dynamic_metadata_hdr10_plus() {
        // Extended tag + one descriptor: version=0, type=1 (HDR10+)
        let data = [EXT_TAG_HDR_DYNAMIC_METADATA, 0x01];
        let descs = parse_hdr_dynamic_metadata(&data);
        assert_eq!(descs.len(), 1);
        assert_eq!(descs[0].application_type, 1);
        assert_eq!(descs[0].application_version, 0);
    }

    #[test]
    fn test_hdr_dynamic_metadata_empty_block() {
        let data = [EXT_TAG_HDR_DYNAMIC_METADATA];
        let descs = parse_hdr_dynamic_metadata(&data);
        assert!(descs.is_empty());
    }

    #[test]
    fn test_video_format_preferences() {
        // Extended tag + three SVRs: VIC 16, DTD ref 129, Y420 ref 145
        let data = [EXT_TAG_VIDEO_FORMAT_PREFERENCE, 16, 129, 145];
        let prefs = parse_video_format_preferences(&data);
        assert_eq!(prefs, vec![16, 129, 145]);
    }

    #[test]
    fn test_y420_vdb_filters_vic0() {
        // Extended tag + VIC 93 (native, bit 7 set), VIC 0 reserved (skipped)
        let data = [EXT_TAG_Y420_VIDEO, 0x80 | 93, 0x80];
        let vics = parse_y420_vdb(&data);
        assert_eq!(vics, vec![93]);
    }

    #[test]
    fn test_y420_capability_map() {
        let data = [EXT_TAG_Y420_CAPABILITY_MAP, 0b0000_0101, 0xFF];
        let bitmap = parse_y420_capability_map(&data);
        assert_eq!(bitmap, vec![0b0000_0101, 0xFF]);
    }

    // --- VESA Display Transfer Characteristic tests ---

    #[test]
    fn test_vesa_dtc_8bit() {
        // 8-bit encoding: bits 7:6 = 0b00; payload = [0x00, 0x80, 0xFF] (black, mid, white)
        let data = [0x00u8, 0x00, 0x80, 0xFF];
        let dtc = parse_vesa_transfer_characteristic(&data).unwrap();
        assert_eq!(dtc.encoding, DtcPointEncoding::Bits8);
        assert_eq!(dtc.points.len(), 3);
        assert!((dtc.points[0] - 0.0).abs() < 0.001);
        assert!((dtc.points[1] - 0x80 as f32 / 255.0).abs() < 0.001);
        assert!((dtc.points[2] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vesa_dtc_10bit() {
        // 10-bit encoding: bits 7:6 = 0b01; encode 4 points packed in 5 bytes.
        // Point 0 = 0x3FF (max), Point 1 = 0x000, Point 2 = 0x200, Point 3 = 0x155
        // Pack: b0[9:2]=0xFF b1[9:8|7:4]=0xC0|0x00=0xC0 b2[3:0|9:6]=0x08 b3[5:0|9:8]=0x00|0x01 b4[7:0]=0x55
        // Simpler: use 0x000 and 0x3FF for easy verification.
        // Pack 0x3FF, 0x000, 0x000, 0x000:
        // bits: 11_1111_1111 00_0000_0000 00_0000_0000 00_0000_0000
        // byte0 = 1111_1111 = 0xFF
        // byte1 = 11_000000 = 0xC0
        // byte2 = 00_0000_00 = 0x00
        // byte3 = 00_000000 = 0x00
        // byte4 = 0000_0000 = 0x00
        let data = [0x40u8, 0xFF, 0xC0, 0x00, 0x00, 0x00]; // 0x40 = 01_000000 = 10-bit
        let dtc = parse_vesa_transfer_characteristic(&data).unwrap();
        assert_eq!(dtc.encoding, DtcPointEncoding::Bits10);
        assert_eq!(dtc.points.len(), 4);
        assert!(
            (dtc.points[0] - 1.0).abs() < 0.001,
            "point0={}",
            dtc.points[0]
        );
        assert!((dtc.points[1] - 0.0).abs() < 0.001);
        assert!((dtc.points[2] - 0.0).abs() < 0.001);
        assert!((dtc.points[3] - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_vesa_dtc_12bit() {
        // 12-bit encoding: bits 7:6 = 0b10; encode 2 points packed in 3 bytes.
        // Point 0 = 0xFFF, Point 1 = 0x000
        // byte0 = 0xFF, byte1 = 0xF0, byte2 = 0x00
        let data = [0x80u8, 0xFF, 0xF0, 0x00]; // 0x80 = 10_000000 = 12-bit
        let dtc = parse_vesa_transfer_characteristic(&data).unwrap();
        assert_eq!(dtc.encoding, DtcPointEncoding::Bits12);
        assert_eq!(dtc.points.len(), 2);
        assert!(
            (dtc.points[0] - 1.0).abs() < 0.001,
            "point0={}",
            dtc.points[0]
        );
        assert!((dtc.points[1] - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_vesa_dtc_reserved_encoding_returns_none() {
        // bits 7:6 = 0b11 → reserved → None
        let data = [0xC0u8, 0x00];
        assert!(parse_vesa_transfer_characteristic(&data).is_none());
    }

    #[test]
    fn test_vesa_dtc_empty_returns_none() {
        assert!(parse_vesa_transfer_characteristic(&[]).is_none());
    }

    // --- Room Configuration Data Block tests ---

    #[test]
    fn test_room_configuration_basic() {
        // 5 speakers, speaker location entries present
        let data = [EXT_TAG_ROOM_CONFIGURATION, 0x45]; // 0b0100_0101: bit6=1, bits4:0=5
        let rc = parse_room_configuration(&data).unwrap();
        assert_eq!(rc.speaker_count, 5);
        assert!(rc.has_speaker_locations);
    }

    #[test]
    fn test_room_configuration_no_locations() {
        // 7 speakers, no location data
        let data = [EXT_TAG_ROOM_CONFIGURATION, 0x07]; // bit6=0, count=7
        let rc = parse_room_configuration(&data).unwrap();
        assert_eq!(rc.speaker_count, 7);
        assert!(!rc.has_speaker_locations);
    }

    #[test]
    fn test_room_configuration_too_short_returns_none() {
        let data = [EXT_TAG_ROOM_CONFIGURATION];
        assert!(parse_room_configuration(&data).is_none());
    }

    // --- Speaker Location Data Block tests ---

    #[test]
    fn test_speaker_location_entries() {
        // Two entries: FL/FR at distance 100, LFE1 at distance 80
        let data = [EXT_TAG_SPEAKER_LOCATION, 0x00, 100, 0x01, 80];
        let entries = parse_speaker_location(&data);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].channel_assignment, 0x00);
        assert_eq!(entries[0].distance, 100);
        assert_eq!(entries[1].channel_assignment, 0x01);
        assert_eq!(entries[1].distance, 80);
    }

    #[test]
    fn test_speaker_location_odd_trailing_byte_ignored() {
        // Three bytes after tag = one complete entry + one orphan byte → 1 entry
        let data = [EXT_TAG_SPEAKER_LOCATION, 0x02, 50, 0xFF];
        let entries = parse_speaker_location(&data);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].channel_assignment, 0x02);
        assert_eq!(entries[0].distance, 50);
    }

    #[test]
    fn test_speaker_location_empty_block() {
        let data = [EXT_TAG_SPEAKER_LOCATION];
        assert!(parse_speaker_location(&data).is_empty());
    }

    // --- InfoFrame Data Block tests ---

    #[test]
    fn test_infoframe_avi_and_audio() {
        // Two standard SIDs (no extra bytes): AVI (0x02) and Audio (0x04).
        // SID byte format: bits 7:5 = extra byte count (0), bits 4:0 = type.
        let data = [EXT_TAG_INFOFRAME, 0x02, 0x04];
        let descs = parse_infoframe_db(&data);
        assert_eq!(descs.len(), 2);
        assert_eq!(descs[0].type_code, infoframe_type::AVI);
        assert!(descs[0].vendor_oui.is_none());
        assert_eq!(descs[1].type_code, infoframe_type::AUDIO);
        assert!(descs[1].vendor_oui.is_none());
    }

    #[test]
    fn test_infoframe_vendor_specific_with_oui() {
        // VSI SID: type=0x01, extra=3 → byte = (3 << 5) | 0x01 = 0x61; OUI = 0x000C03 (HDMI)
        let data = [EXT_TAG_INFOFRAME, 0x61, 0x00, 0x0C, 0x03];
        let descs = parse_infoframe_db(&data);
        assert_eq!(descs.len(), 1);
        assert_eq!(descs[0].type_code, infoframe_type::VENDOR_SPECIFIC);
        assert_eq!(descs[0].vendor_oui, Some(0x000C03));
    }

    #[test]
    fn test_infoframe_mixed_vsi_and_standard() {
        // VSI (3 extra bytes) followed by HDR (0 extra bytes).
        let data = [EXT_TAG_INFOFRAME, 0x61, 0x00, 0x0C, 0x03, 0x07];
        let descs = parse_infoframe_db(&data);
        assert_eq!(descs.len(), 2);
        assert_eq!(descs[0].type_code, infoframe_type::VENDOR_SPECIFIC);
        assert_eq!(descs[0].vendor_oui, Some(0x000C03));
        assert_eq!(descs[1].type_code, infoframe_type::DYNAMIC_RANGE_MASTERING);
        assert!(descs[1].vendor_oui.is_none());
    }

    #[test]
    fn test_infoframe_empty_block() {
        let data = [EXT_TAG_INFOFRAME];
        assert!(parse_infoframe_db(&data).is_empty());
    }

    // --- VESA Display Device Data Block tests ---

    fn minimal_dddb_payload() -> [u8; 30] {
        let mut d = [0u8; 30];
        // interface_type=0x0A (DisplayPort), num_links=2 → byte[0] = 0xA2
        d[0] = 0xA2;
        // interface_version=1, interface_release=0 → byte[1] = 0x10
        d[1] = 0x10;
        // content_protection = 0x01 (HDCP)
        d[2] = 0x01;
        // min_clock=40 MHz (40<<2=0xA0), max_clock hi bits=0 → byte[3] = 0xA0
        // max_clock lo = 0xF0 (240 dec, so max = 240 MHz) → byte[4] = 0xF0
        d[3] = 0xA0;
        d[4] = 0xF0;
        // native pixel: 1920x1080 → stored as 1919 and 1079 (LE)
        let w: u16 = 1919;
        let h: u16 = 1079;
        d[5] = (w & 0xFF) as u8;
        d[6] = (w >> 8) as u8;
        d[7] = (h & 0xFF) as u8;
        d[8] = (h >> 8) as u8;
        // aspect_ratio_raw = 0x09 (16:9)
        d[9] = 0x09;
        // orientation/rotation/zero_pixel/scan: default_orientation=0, rest 0
        d[10] = 0x00;
        // subpixel=1 (RGB), pitch defaults to 0
        d[11] = 0x01;
        // audio: separate audio inputs set → byte[15] bit6 = 0x40
        d[15] = 0x40;
        // audio_delay = 0x82 → sign bit set (audio late), mag = (0x02 * 2) = 4 ms
        d[16] = 0x82;
        // color bit depth: interface=8 (stored as 7), display=6 (stored as 5) → 0x75
        d[19] = 0x75;
        // response time: 5 ms, black-to-white (bit7=0) → 0x05
        d[28] = 0x05;
        // overscan: h=2%, v=3% → 0x23
        d[29] = 0x23;
        d
    }

    #[test]
    fn test_dddb_basic_fields() {
        let payload = minimal_dddb_payload();
        let block = parse_vesa_display_device(&payload).unwrap();
        assert_eq!(block.interface_type, 0x0A);
        assert_eq!(block.num_links, 2);
        assert_eq!(block.interface_version, 1);
        assert_eq!(block.interface_release, 0);
        assert_eq!(block.content_protection, 0x01);
    }

    #[test]
    fn test_dddb_clock_frequency() {
        let payload = minimal_dddb_payload();
        let block = parse_vesa_display_device(&payload).unwrap();
        assert_eq!(block.min_clock_mhz, 40);
        assert_eq!(block.max_clock_mhz, 0xF0);
    }

    #[test]
    fn test_dddb_native_resolution() {
        let payload = minimal_dddb_payload();
        let block = parse_vesa_display_device(&payload).unwrap();
        assert_eq!(block.native_width, Some(1920));
        assert_eq!(block.native_height, Some(1080));
    }

    #[test]
    fn test_dddb_native_resolution_none_when_zero() {
        let mut payload = minimal_dddb_payload();
        payload[5] = 0;
        payload[6] = 0;
        payload[7] = 0;
        payload[8] = 0;
        let block = parse_vesa_display_device(&payload).unwrap();
        assert_eq!(block.native_width, None);
        assert_eq!(block.native_height, None);
    }

    #[test]
    fn test_dddb_audio_and_delay() {
        let payload = minimal_dddb_payload();
        let block = parse_vesa_display_device(&payload).unwrap();
        assert!(!block.audio_on_video_interface);
        assert!(block.separate_audio_inputs);
        assert!(!block.audio_input_override);
        // delay_raw = 0x82: sign bit set → positive (audio late), mag = 2*2 = 4 ms
        assert_eq!(block.audio_delay_ms, Some(4));
    }

    #[test]
    fn test_dddb_audio_delay_none_when_zero() {
        let mut payload = minimal_dddb_payload();
        payload[16] = 0;
        let block = parse_vesa_display_device(&payload).unwrap();
        assert!(block.audio_delay_ms.is_none());
    }

    #[test]
    fn test_dddb_audio_delay_negative() {
        // delay_raw = 0x02: sign bit clear → negative, mag = 2*2 = 4 → -4 ms
        let mut payload = minimal_dddb_payload();
        payload[16] = 0x02;
        let block = parse_vesa_display_device(&payload).unwrap();
        assert_eq!(block.audio_delay_ms, Some(-4));
    }

    #[test]
    fn test_dddb_color_depth() {
        let payload = minimal_dddb_payload();
        let block = parse_vesa_display_device(&payload).unwrap();
        // 0x75 → interface = (7 + 1) = 8, display = (5 + 1) = 6
        assert_eq!(block.interface_color_depth, 8);
        assert_eq!(block.display_color_depth, 6);
    }

    #[test]
    fn test_dddb_response_time_and_overscan() {
        let payload = minimal_dddb_payload();
        let block = parse_vesa_display_device(&payload).unwrap();
        assert_eq!(block.response_time_ms, 5);
        assert!(!block.response_time_white_to_black);
        assert_eq!(block.h_overscan_pct, 2);
        assert_eq!(block.v_overscan_pct, 3);
    }

    #[test]
    fn test_dddb_too_short_returns_none() {
        let payload = [0u8; 29];
        assert!(parse_vesa_display_device(&payload).is_none());
    }

    // --- VTB-EXT tests ---

    #[test]
    fn test_vtb_ext_empty_version_only() {
        // version=1, 0 DTBs, 0 CVTs, 0 STs
        let data = [0x01u8, 0x00, 0x00, 0x00];
        let vtb = parse_vtb_ext(&data).unwrap();
        assert_eq!(vtb.version, 1);
        assert!(vtb.timings.is_empty());
    }

    #[test]
    fn test_vtb_ext_too_short_returns_none() {
        let data = [0x01u8, 0x00, 0x00];
        assert!(parse_vtb_ext(&data).is_none());
    }

    #[test]
    fn test_vtb_ext_standard_timings() {
        // version=1, 0 DTBs, 0 CVTs, 1 ST
        // width=1280: (b1+31)*8=1280 → b1=129=0x81; b2=0x00 → 16:10 ratio, rate=60+0=60
        let data = [0x01u8, 0x00, 0x00, 0x01, 0x81, 0x00];
        let vtb = parse_vtb_ext(&data).unwrap();
        assert_eq!(vtb.timings.len(), 1);
        assert_eq!(vtb.timings[0].width, 1280);
        assert_eq!(vtb.timings[0].height, 800); // 1280 * 10 / 16 = 800
        assert_eq!(vtb.timings[0].refresh_rate, 60);
    }

    #[test]
    fn test_vtb_ext_cvt_descriptor_16_9() {
        // version=1, 0 DTBs, 1 CVT, 0 STs
        // CVT encoding: 1080 lines → lines_raw = (1080/2 - 1) = 539 = 0x21B
        //   b0 = 0x1B, b1 = (0x20) | aspect_16_9=(0x01<<2) = 0x24
        //   b2: 60Hz bit set = 0x08
        let data = [0x01u8, 0x00, 0x01, 0x00, 0x1B, 0x24, 0x08];
        let vtb = parse_vtb_ext(&data).unwrap();
        assert!(!vtb.timings.is_empty());
        let t = vtb.timings.iter().find(|m| m.refresh_rate == 60).unwrap();
        assert_eq!(t.height, 1080);
        // 16:9: h = (1080 * 16 / 9) rounded down to multiple of 8 = 1920
        assert_eq!(t.width, 1920);
    }

    #[test]
    fn test_vtb_ext_insufficient_data_for_dtb_returns_none() {
        // version=1, 1 DTB, 0 CVTs, 0 STs — but only 4 bytes, no room for 18-byte DTB
        let data = [0x01u8, 0x01, 0x00, 0x00];
        assert!(parse_vtb_ext(&data).is_none());
    }

    // --- Vendor-Specific Video / Audio Data Block tests ---

    #[test]
    fn test_vsvdb_basic() {
        // OUI = 0x00D046 (Dolby), stored LSB-first: [0x46, 0xD0, 0x00], payload = [0xAB, 0xCD]
        let payload = [0x46u8, 0xD0, 0x00, 0xAB, 0xCD];
        let b = parse_vendor_specific_block(&payload).unwrap();
        assert_eq!(b.oui, 0x00D046);
        assert_eq!(b.payload, vec![0xAB, 0xCD]);
    }

    #[test]
    fn test_vsvdb_oui_only_no_payload() {
        let payload = [0x03u8, 0x0C, 0x00]; // OUI = 0x000C03 (HDMI), no payload
        let b = parse_vendor_specific_block(&payload).unwrap();
        assert_eq!(b.oui, 0x000C03);
        assert!(b.payload.is_empty());
    }

    #[test]
    fn test_vsvdb_too_short_returns_none() {
        assert!(parse_vendor_specific_block(&[0x46u8, 0xD0]).is_none());
    }

    #[test]
    fn test_vsadb_same_structure() {
        // VSADB has identical wire format to VSVDB; reuse the same parser.
        let payload = [0x8Bu8, 0x84, 0x90, 0x01]; // OUI = 0x90848B (HDR10+), payload = [0x01]
        let b = parse_vendor_specific_block(&payload).unwrap();
        assert_eq!(b.oui, 0x90848B);
        assert_eq!(b.payload, vec![0x01]);
    }

    // --- DisplayID Type VII Video Timing Data Block (T7VTDB) tests ---

    /// Build a minimal valid 22-byte T7VTDB descriptor payload (after ext tag).
    ///
    /// Encodes 1920x1080@60Hz:
    ///   pixel_clock = 148500 kHz = 0x024414 → LE bytes [0x14, 0x44, 0x02]
    ///   H active = 1920 = 0x0780 → [0x80, 0x07]
    ///   H blank  =  280 = 0x0118 → [0x18, 0x01]
    ///   V active = 1080 = 0x0438 → [0x38, 0x04]
    ///   V blank  =   45 = 0x002D → [0x2D, 0x00]
    ///   refresh  = 148500000 / (2200 × 1125) = 60 Hz
    fn t7_1080p60() -> [u8; 22] {
        let mut d = [0u8; 22];
        d[0] = 0x02; // Block_Rev = 010b
        d[1] = 0x00; // T7Y420=0, T7HSP=0, T7VSP=0
                     // Pixel clock: 148500 kHz
        d[2] = 0x14;
        d[3] = 0x44;
        d[4] = 0x02;
        d[5] = 0x00; // 3D_Support=00, T7IL=0, T7_Aspect_Ratio=0
                     // H Active: 1920
        d[6] = 0x80;
        d[7] = 0x07;
        // H Blank: 280
        d[8] = 0x18;
        d[9] = 0x01;
        // H Offset + H Sync: zeros (don't affect VideoMode)
        // V Active: 1080
        d[14] = 0x38;
        d[15] = 0x04;
        // V Blank: 45
        d[16] = 0x2D;
        d[17] = 0x00;
        d
    }

    #[test]
    fn test_t7vtdb_1080p60() {
        let d = t7_1080p60();
        let t7 = parse_t7vtdb(&d).unwrap();
        assert_eq!(t7.version, 2);
        assert_eq!(t7.mode.width, 1920);
        assert_eq!(t7.mode.height, 1080);
        assert_eq!(t7.mode.refresh_rate, 60);
        assert!(!t7.mode.interlaced);
        assert!(!t7.y420);
    }

    #[test]
    fn test_t7vtdb_y420_flag() {
        let mut d = t7_1080p60();
        d[1] = 0x40; // T7Y420 = bit 6
        let t7 = parse_t7vtdb(&d).unwrap();
        assert!(t7.y420);
    }

    #[test]
    fn test_t7vtdb_interlaced_flag() {
        let mut d = t7_1080p60();
        d[5] = 0x10; // T7IL = bit 4
        let t7 = parse_t7vtdb(&d).unwrap();
        assert!(t7.mode.interlaced);
    }

    #[test]
    fn test_t7vtdb_too_short_returns_none() {
        let d = [0u8; 21];
        assert!(parse_t7vtdb(&d).is_none());
    }

    #[test]
    fn test_t7vtdb_zero_pixel_clock_returns_none() {
        let mut d = t7_1080p60();
        d[2] = 0;
        d[3] = 0;
        d[4] = 0;
        assert!(parse_t7vtdb(&d).is_none());
    }

    #[test]
    fn test_t7vtdb_zero_hactive_returns_none() {
        let mut d = t7_1080p60();
        d[6] = 0;
        d[7] = 0;
        assert!(parse_t7vtdb(&d).is_none());
    }

    #[test]
    fn test_t7vtdb_720p60() {
        // 1280x720@60Hz: pixel_clock = 74250 kHz, H blank = 370, V blank = 30
        // h_total = 1650, v_total = 750
        // refresh = 74250000 / (1650 * 750) = 74250000 / 1237500 = 60 Hz
        // 74250 = 0x012 24A → LE [0x4A, 0x22, 0x01]
        // 1280 = 0x0500 → [0x00, 0x05]
        // 370  = 0x0172 → [0x72, 0x01]
        // 720  = 0x02D0 → [0xD0, 0x02]
        // 30   = 0x001E → [0x1E, 0x00]
        let mut d = [0u8; 22];
        d[0] = 0x02;
        d[2] = 0x4A;
        d[3] = 0x22;
        d[4] = 0x01;
        d[6] = 0x00;
        d[7] = 0x05;
        d[8] = 0x72;
        d[9] = 0x01;
        d[14] = 0xD0;
        d[15] = 0x02;
        d[16] = 0x1E;
        let t7 = parse_t7vtdb(&d).unwrap();
        assert_eq!(t7.mode.width, 1280);
        assert_eq!(t7.mode.height, 720);
        assert_eq!(t7.mode.refresh_rate, 60);
    }

    // T8VTDB tests

    #[test]
    fn test_t8vtdb_single_1080p60() {
        // header: Code_Type=0, y420=0, TCS=0, Rev=0 → 0x00; code 0x52 = 1920×1080@60
        let data = [0x00u8, 0x52];
        let t8 = parse_t8vtdb(&data).unwrap();
        assert_eq!(t8.version, 0);
        assert!(!t8.y420);
        assert_eq!(t8.codes, vec![0x52]);
        assert_eq!(t8.timings.len(), 1);
        assert_eq!(t8.timings[0].width, 1920);
        assert_eq!(t8.timings[0].height, 1080);
        assert_eq!(t8.timings[0].refresh_rate, 60);
        assert!(!t8.timings[0].interlaced);
    }

    #[test]
    fn test_t8vtdb_multiple_codes() {
        // codes 0x04 (640×480@60), 0x09 (800×600@60), 0x10 (1024×768@60)
        let data = [0x00u8, 0x04, 0x09, 0x10];
        let t8 = parse_t8vtdb(&data).unwrap();
        assert_eq!(t8.codes, vec![0x04, 0x09, 0x10]);
        assert_eq!(t8.timings.len(), 3);
        assert_eq!(t8.timings[0].width, 640);
        assert_eq!(t8.timings[1].width, 800);
        assert_eq!(t8.timings[2].width, 1024);
    }

    #[test]
    fn test_t8vtdb_interlaced_0x0f() {
        // 0x0F = 1024×768@43 Hz, interlaced
        let data = [0x00u8, 0x0F];
        let t8 = parse_t8vtdb(&data).unwrap();
        assert_eq!(t8.timings[0].width, 1024);
        assert_eq!(t8.timings[0].height, 768);
        assert_eq!(t8.timings[0].refresh_rate, 43);
        assert!(t8.timings[0].interlaced);
    }

    #[test]
    fn test_t8vtdb_two_byte_codes() {
        // TCS=1: header bit 3 set → 0x08; code 0x0052 = 1920×1080@60 stored LSB-first
        let data = [0x08u8, 0x52, 0x00];
        let t8 = parse_t8vtdb(&data).unwrap();
        assert_eq!(t8.codes, vec![0x0052]);
        assert_eq!(t8.timings[0].width, 1920);
        assert_eq!(t8.timings[0].refresh_rate, 60);
    }

    #[test]
    fn test_t8vtdb_two_byte_codes_odd_trailing_byte_ignored() {
        // TCS=1; 3 payload bytes → one complete 2-byte code plus one orphan byte
        let data = [0x08u8, 0x52, 0x00, 0x55];
        let t8 = parse_t8vtdb(&data).unwrap();
        assert_eq!(t8.codes, vec![0x0052]); // trailing 0x55 ignored
    }

    #[test]
    fn test_t8vtdb_unknown_code_in_codes_but_not_timings() {
        // 0xFF is not a defined DMT ID
        let data = [0x00u8, 0xFF];
        let t8 = parse_t8vtdb(&data).unwrap();
        assert_eq!(t8.codes, vec![0xFF]);
        assert!(t8.timings.is_empty());
    }

    #[test]
    fn test_t8vtdb_y420_flag() {
        // T8Y420 = bit 5 of header → 0x20
        let data = [0x20u8, 0x52];
        let t8 = parse_t8vtdb(&data).unwrap();
        assert!(t8.y420);
    }

    #[test]
    fn test_t8vtdb_non_dmt_returns_none() {
        // Code_Type = 0x01 (bits 7:6 = 01) → not DMT → None
        let data = [0x40u8, 0x52];
        assert!(parse_t8vtdb(&data).is_none());
    }

    #[test]
    fn test_t8vtdb_empty_returns_none() {
        assert!(parse_t8vtdb(&[]).is_none());
    }

    #[test]
    fn test_dmt_to_mode_spot_checks() {
        let m = dmt_to_mode(0x55).unwrap();
        assert_eq!(m.width, 1280);
        assert_eq!(m.height, 720);
        assert_eq!(m.refresh_rate, 60);
        assert!(!m.interlaced);

        let m = dmt_to_mode(0x0F).unwrap();
        assert!(m.interlaced);
        assert_eq!(m.refresh_rate, 43);

        assert!(dmt_to_mode(0x00).is_none());
        assert!(dmt_to_mode(0x59).is_none());
    }

    // T10VTDB tests

    /// Build a 6-byte T10 descriptor for the given width, height, refresh (M=0).
    fn t10_desc_6(width: u16, height: u16, refresh: u16, y420: bool) -> [u8; 6] {
        let flags = if y420 { 0x80 } else { 0x00 };
        let h = (width - 1).to_le_bytes();
        let v = (height - 1).to_le_bytes();
        [flags, h[0], h[1], v[0], v[1], (refresh - 1) as u8]
    }

    #[test]
    fn test_t10vtdb_6byte_single() {
        // M=0 (rev=0x00); 640×480@60
        let desc = t10_desc_6(640, 480, 60, false);
        let mut data = vec![0x00u8]; // rev byte (M=0)
        data.extend_from_slice(&desc);
        let t10 = parse_t10vtdb(&data).unwrap();
        assert_eq!(t10.entries.len(), 1);
        assert_eq!(t10.entries[0].width, 640);
        assert_eq!(t10.entries[0].height, 480);
        assert_eq!(t10.entries[0].refresh_hz, 60);
        assert!(!t10.entries[0].y420);
    }

    #[test]
    fn test_t10vtdb_7byte_high_refresh() {
        // M=1 (rev=0x10); 1920×1080@512 — refresh > 255 requires 2-byte field
        // refresh stored = 511 = 0x1FF → lsb=0xFF, msb_bits=0x01
        let flags = 0x00u8;
        let h = (1919u16).to_le_bytes(); // 1920 - 1
        let v = (1079u16).to_le_bytes(); // 1080 - 1
        let data = [
            0x10, // rev: M=1
            flags, h[0], h[1], v[0], v[1], 0xFF, // refresh lsb (511 & 0xFF)
            0x01, // refresh msb bits[1:0] = 0x01 → (0x01 << 8) | 0xFF = 511; +1 = 512
        ];
        let t10 = parse_t10vtdb(&data).unwrap();
        assert_eq!(t10.entries[0].width, 1920);
        assert_eq!(t10.entries[0].height, 1080);
        assert_eq!(t10.entries[0].refresh_hz, 512);
    }

    #[test]
    fn test_t10vtdb_multiple_descriptors() {
        // M=0; two descriptors back-to-back
        let d1 = t10_desc_6(1280, 720, 60, false);
        let d2 = t10_desc_6(1920, 1080, 60, false);
        let mut data = vec![0x00u8];
        data.extend_from_slice(&d1);
        data.extend_from_slice(&d2);
        let t10 = parse_t10vtdb(&data).unwrap();
        assert_eq!(t10.entries.len(), 2);
        assert_eq!(t10.entries[0].width, 1280);
        assert_eq!(t10.entries[1].width, 1920);
    }

    #[test]
    fn test_t10vtdb_y420_flag() {
        let desc = t10_desc_6(3840, 2160, 60, true);
        let mut data = vec![0x00u8];
        data.extend_from_slice(&desc);
        let t10 = parse_t10vtdb(&data).unwrap();
        assert!(t10.entries[0].y420);
    }

    #[test]
    fn test_t10vtdb_invalid_m_returns_none() {
        // M=3 (bits[6:4]=011 → rev=0x30) is invalid
        let data = [0x30u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(parse_t10vtdb(&data).is_none());
    }

    #[test]
    fn test_t10vtdb_empty_returns_none() {
        assert!(parse_t10vtdb(&[]).is_none());
    }

    #[test]
    fn test_t10vtdb_trailing_partial_descriptor_ignored() {
        // M=0 (6-byte descriptors); 7 payload bytes = 1 complete + 1 orphan
        let desc = t10_desc_6(800, 600, 60, false);
        let mut data = vec![0x00u8];
        data.extend_from_slice(&desc);
        data.push(0xFF); // orphan byte
        let t10 = parse_t10vtdb(&data).unwrap();
        assert_eq!(t10.entries.len(), 1); // only the complete descriptor
    }

    // HF-EEODB tests

    #[test]
    fn test_hf_eeodb_valid() {
        assert_eq!(parse_hf_eeodb(&[5]), Some(5));
        assert_eq!(parse_hf_eeodb(&[255]), Some(255));
        assert_eq!(parse_hf_eeodb(&[1]), Some(1));
    }

    #[test]
    fn test_hf_eeodb_zero_returns_none() {
        assert!(parse_hf_eeodb(&[0]).is_none());
    }

    #[test]
    fn test_hf_eeodb_empty_returns_none() {
        assert!(parse_hf_eeodb(&[]).is_none());
    }

    // HF-SCDB tests

    /// Build a minimal HF-SCDB block_data (2 reserved + 4 mandatory SCDS bytes).
    fn hf_scdb_min(version: u8, tmds_raw: u8, scdc: u8, frl_dc: u8) -> Vec<u8> {
        vec![0x00, 0x00, version, tmds_raw, scdc, frl_dc]
    }

    #[test]
    fn test_hf_scdb_basic_fields() {
        // version=1, max_tmds=600MHz (raw=120=0x78), scdc_present+rr_capable=0xC0,
        // max_frl_rate=6 (12Gbps×4), uhd_vic=1 → frl_dc = (6<<4)|0x08 = 0x68
        let data = hf_scdb_min(1, 0x78, 0xC0, 0x68);
        let cap = parse_hf_scdb(&data).unwrap();
        assert_eq!(cap.version, 1);
        assert_eq!(cap.max_tmds_rate_mhz, 600);
        assert!(cap.scdc_present);
        assert!(cap.rr_capable);
        assert!(!cap.cable_status);
        assert_eq!(cap.max_frl_rate, HdmiForumFrl::Rate12Gbps4Lanes);
        assert!(cap.uhd_vic);
        assert!(!cap.dc_30bit_420);
        assert!(cap.vrr_min_hz.is_none());
        assert!(cap.dsc.is_none());
    }

    #[test]
    fn test_hf_scdb_dc_flags() {
        // dc_48bit_420[2]=1, dc_36bit_420[1]=1, dc_30bit_420[0]=1 → frl_dc = 0x07
        let data = hf_scdb_min(1, 0x00, 0x00, 0x07);
        let cap = parse_hf_scdb(&data).unwrap();
        assert!(cap.dc_48bit_420);
        assert!(cap.dc_36bit_420);
        assert!(cap.dc_30bit_420);
    }

    #[test]
    fn test_hf_scdb_extended_flags_and_vrr() {
        // Extended section: ALLM[1]=1 → ext_byte = 0x02
        // VRRmin=48 (0x30), VRRmax=144 (0x0090 → b7=(0<<6)|0x30=0x30, b8=0x90)
        let mut data = hf_scdb_min(1, 0x78, 0xC0, 0x68);
        data.push(0x02); // ext_byte: ALLM=bit1
        data.push(0x30); // VRRmax[9:8]=0, VRRmin=48
        data.push(0x90); // VRRmax[7:0]=144
        let cap = parse_hf_scdb(&data).unwrap();
        assert!(cap.allm);
        assert!(!cap.fva);
        assert_eq!(cap.vrr_min_hz, Some(48));
        assert_eq!(cap.vrr_max_hz, Some(144));
    }

    #[test]
    fn test_hf_scdb_vrr_10bit_max() {
        // VRRmax = 0x1FF = 511 → b7 bits[7:6]=0b11 → 0xC0 | VRRmin=0 → 0xC0; b8=0xFF
        let mut data = hf_scdb_min(1, 0x00, 0x00, 0x00);
        data.push(0x00); // ext_byte
        data.push(0xC0); // VRRmax[9:8]=0b11, VRRmin=0
        data.push(0xFF); // VRRmax[7:0]=255
        let cap = parse_hf_scdb(&data).unwrap();
        assert_eq!(cap.vrr_max_hz, Some(0x03FF)); // 1023
    }

    #[test]
    fn test_hf_scdb_dsc_section() {
        // dsc_flags: DSC_1p2[7]=1, DSC_Native_420[6]=1 → 0xC0
        // dsc_frl_slices: DSC_Max_FRL_Rate=6 [7:4], DSC_MaxSlices=4 [3:0] → 0x64
        // chunk_raw=2 → max_chunk_bytes = 1024 * 3 = 3072
        let mut data = hf_scdb_min(1, 0x78, 0xC0, 0x68);
        data.extend_from_slice(&[0x00, 0x00, 0x00]); // ext + VRR (no VRR)
        data.extend_from_slice(&[0xC0, 0x64, 0x02]); // DSC bytes
        let cap = parse_hf_scdb(&data).unwrap();
        let dsc = cap.dsc.unwrap();
        assert!(dsc.dsc_1p2);
        assert!(dsc.native_420);
        assert!(!dsc.bpc12);
        assert_eq!(dsc.max_frl_rate, HdmiForumFrl::Rate12Gbps4Lanes);
        assert_eq!(dsc.max_slices, HdmiDscMaxSlices::Slices8At340Mhz);
        assert_eq!(dsc.max_chunk_bytes, 3072);
    }

    #[test]
    fn test_hf_scdb_too_short_returns_none() {
        // Only 5 bytes (need 6 minimum)
        assert!(parse_hf_scdb(&[0x00, 0x00, 0x01, 0x78, 0xC0]).is_none());
    }

    #[test]
    fn test_hf_scdb_empty_returns_none() {
        assert!(parse_hf_scdb(&[]).is_none());
    }

    #[test]
    fn test_hf_scdb_partial_vrr_gives_none() {
        // Extended byte present but VRR bytes incomplete (only 1 of 2)
        let mut data = hf_scdb_min(1, 0x00, 0x00, 0x00);
        data.push(0x02); // ext_byte (ALLM)
        data.push(0x30); // b7 (VRRmin), but b8 is missing
        let cap = parse_hf_scdb(&data).unwrap();
        assert!(cap.allm); // feature flag was parsed
        assert!(cap.vrr_min_hz.is_none()); // VRR range absent (need both bytes)
        assert!(cap.vrr_max_hz.is_none());
    }

    // Extended tag coverage

    #[test]
    fn test_all_extended_tags_accounted_for() {
        // Every value 0x00–0xFF must be either in IMPLEMENTED_EXTENDED_TAGS or
        // within one of RESERVED_EXTENDED_TAG_RANGES.  If this test fails after
        // a spec update, add an EXT_TAG_* constant, implement the block (or add
        // a stub), update IMPLEMENTED_EXTENDED_TAGS, and shrink the relevant
        // reserved range.
        for tag in 0u16..=255 {
            let tag = tag as u8;
            let implemented = IMPLEMENTED_EXTENDED_TAGS.contains(&tag);
            let reserved = RESERVED_EXTENDED_TAG_RANGES
                .iter()
                .any(|&(lo, hi)| tag >= lo && tag <= hi);
            assert!(
                implemented || reserved,
                "Extended tag 0x{:02X} is unaccounted for: \
                 add it to IMPLEMENTED_EXTENDED_TAGS or RESERVED_EXTENDED_TAG_RANGES",
                tag
            );
        }
    }

    #[test]
    fn test_implemented_and_reserved_are_disjoint() {
        // Sanity check: no tag should appear in both lists simultaneously.
        for &tag in IMPLEMENTED_EXTENDED_TAGS {
            let in_reserved = RESERVED_EXTENDED_TAG_RANGES
                .iter()
                .any(|&(lo, hi)| tag >= lo && tag <= hi);
            assert!(
                !in_reserved,
                "Extended tag 0x{:02X} appears in both IMPLEMENTED and RESERVED",
                tag
            );
        }
    }
}
