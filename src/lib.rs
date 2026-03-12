#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

pub mod model;
pub use model::{DisplayCapabilities, EdidError, EdidWarning, ParsedEdid, ExtensionTagRegistry, ExtensionLibrary, ExtensionMetadata};

pub mod parser;
pub use parser::parse_edid;


pub fn capabilities_from_edid(edid: &ParsedEdid) -> DisplayCapabilities {
    let mut caps = DisplayCapabilities::default();
    let base = &edid.base_block;

    // 1. Manufacturer ID (offsets 0x08-0x09)
    // 2 bytes, 3 characters, 5 bits per character (00001=A, ..., 11010=Z)
    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        let id_raw = ((base[0x08] as u16) << 8) | (base[0x09] as u16);
        let char1 = ((id_raw >> 10) & 0x1F) as u8;
        let char2 = ((id_raw >> 5) & 0x1F) as u8;
        let char3 = (id_raw & 0x1F) as u8;

        if char1 > 0 && char2 > 0 && char3 > 0 {
            let mut mfg = String::new();
            mfg.push((char1 + b'A' - 1) as char);
            mfg.push((char2 + b'A' - 1) as char);
            mfg.push((char3 + b'A' - 1) as char);
            caps.manufacturer = Some(mfg);
        }
    }

    caps
}
