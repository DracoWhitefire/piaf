#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

pub mod model;
pub use model::{
    DisplayCapabilities, EdidError, EdidWarning, ExtensionLibrary, ExtensionMetadata,
    ExtensionTagRegistry, ParsedEdid, VideoMode,
};

pub mod parser;
pub use parser::parse_edid;

pub mod capabilities;
pub use capabilities::capabilities_from_edid;
