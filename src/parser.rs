use crate::model::{EdidError, ParsedEdid};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::Vec;

pub const EDID_HEADER: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];

pub fn parse_edid(bytes: &[u8]) -> Result<ParsedEdid, EdidError> {
    if bytes.len() < 128 {
        return Err(EdidError::InvalidLength);
    }

    if bytes[0..8] != EDID_HEADER {
        return Err(EdidError::InvalidHeader);
    }

    let mut base_block = [0u8; 128];
    base_block.copy_from_slice(&bytes[0..128]);

    let checksum: u8 = base_block.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
    if checksum != 0 {
        return Err(EdidError::ChecksumMismatch);
    }

    Ok(ParsedEdid {
        base_block,
        #[cfg(any(feature = "alloc", feature = "std"))]
        extensions: Vec::new(),
        #[cfg(any(feature = "alloc", feature = "std"))]
        warnings: Vec::new(),
    })
}
