//! PIAF — a Rust library for reading and interpreting EDID display capability data.
//!
//! The core pipeline is:
//!
//! 1. [`parse_edid`] — validate and decode raw bytes into [`ParsedEdidRef`] (zero-copy).
//! 2. [`capabilities_from_edid`] — run extension handlers to produce [`DisplayCapabilities`].
//!
//! # Quick start
//!
//! ```no_run
//! use piaf::{parse_edid, capabilities_from_edid, ExtensionLibrary};
//!
//! let bytes = std::fs::read("/sys/class/drm/card0-HDMI-A-1/edid").unwrap();
//! let library = ExtensionLibrary::with_standard_handlers();
//! let parsed = parse_edid(&bytes, &library).unwrap();
//! let caps = capabilities_from_edid(&parsed, &library);
//! println!("{:?}", caps.display_name);
//! ```
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
// In a bare no_std build (no alloc, no std) the handler layer is absent, so the
// pub(crate) decode functions on model types appear unused. They are intentionally
// kept available for consumers who want to call them directly without the handler
// pipeline, so we suppress the warning rather than gating the items away.
#![cfg_attr(
    not(any(feature = "alloc", feature = "std")),
    allow(dead_code, unused_imports)
)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

/// Types for the EDID data model.
pub mod model;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use model::ExtensionData;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use model::ExtensionHandler;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use model::ParseWarning;
pub use model::{
    AnalogColorType, AnalogSyncLevel, Chromaticity, ChromaticityPoint, ColorBitDepth,
    ColorManagementData, CvtAspectRatio, CvtAspectRatios, CvtScaling, CvtSupportParams, DcmChannel,
    DigitalColorEncoding, DisplayCapabilities, DisplayFeatureFlags, DisplayGamma, EdidError,
    EdidSource, EdidVersion, EdidWarning, ExtensionLibrary, ExtensionMetadata,
    ExtensionTagRegistry, GtfSecondaryParams, KnownExtensions, ManufactureDate, ManufacturerId,
    ModeSink, MonitorString, ParsedEdid, ParsedEdidRef, ScreenSize, StaticDisplayCapabilities,
    StaticExtensionHandler, StereoMode, SyncDefinition, TimingFormula, VideoInputFlags,
    VideoInterface, VideoMode, WhitePoint,
};

/// EDID byte-level parser.
pub mod parser;
pub use parser::{parse_edid, parse_edid_owned};

/// Capability extraction from a [`ParsedEdid`].
pub mod capabilities;
pub use capabilities::capabilities_from_edid;
pub use capabilities::capabilities_from_edid_static;
pub use capabilities::Cea861Flags;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use capabilities::{
    infoframe_type, AudioFormat, AudioFormatInfo, AudioSampleRates, Cea861Capabilities,
    ColorimetryBlock, ColorimetryFlags, DtcPointEncoding, HdmiAudioBlock, HdmiDscMaxSlices,
    HdmiForumDsc, HdmiForumFrl, HdmiForumSinkCap, HdmiVsdb, HdmiVsdbFlags,
    HdrDynamicMetadataDescriptor, HdrEotf, HdrStaticMetadata, InfoFrameDescriptor,
    RoomConfigurationBlock, ShortAudioDescriptor, SpeakerAllocation, SpeakerAllocationFlags,
    SpeakerAllocationFlags2, SpeakerAllocationFlags3, SpeakerLocationEntry, T10VtdbBlock,
    T10VtdbEntry, T7VtdbBlock, T8VtdbBlock, VendorSpecificBlock, VesaDisplayDeviceBlock,
    VesaTransferCharacteristic, VideoCapability, VideoCapabilityFlags, VtbExtBlock,
};
pub use capabilities::{Cea861Handler, CEA861_HANDLER, STANDARD_HANDLERS};
