/// Extended tag codes used in CEA Extended Tag Data Blocks (outer tag `0x07`).
/// The first byte of the block payload is the extended tag.
pub(super) const EXT_TAG_VIDEO_CAPABILITY: u8 = 0x00;
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
}
