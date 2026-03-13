//! PIAF — a Rust library for reading and interpreting EDID display capability data.
//!
//! The core pipeline is:
//!
//! 1. [`parse_edid`] — validate and decode raw bytes into [`ParsedEdid`].
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

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

/// Types for the EDID data model.
pub mod model;
#[cfg(feature = "std")]
pub use model::ExtensionData;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use model::ExtensionHandler;
pub use model::{
    ColorBitDepth, DisplayCapabilities, DisplayFeatureFlags, DisplayGamma, EdidError, EdidVersion,
    EdidWarning, ExtensionLibrary, ExtensionMetadata, ExtensionTagRegistry, KnownExtensions,
    ManufactureDate, ParsedEdid, VideoInputFlags, VideoInterface, VideoMode,
};

/// EDID byte-level parser.
pub mod parser;
pub use parser::parse_edid;

/// Capability extraction from a [`ParsedEdid`].
pub mod capabilities;
pub use capabilities::{capabilities_from_edid, Cea861Flags};
