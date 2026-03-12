#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

pub mod model;
pub use model::{DisplayCapabilities, EdidError, EdidWarning, ParsedEdid};


pub fn parse_edid(_bytes: &[u8]) -> Result<ParsedEdid, EdidError> {
    unimplemented!("EDID parsing is not yet implemented");
}

pub fn capabilities_from_edid(_edid: &ParsedEdid) -> DisplayCapabilities {
    unimplemented!("Capability derivation is not yet implemented")
}
