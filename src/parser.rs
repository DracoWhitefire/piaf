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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_invalid_length() {
        let bytes = [0u8; 10];
        assert_eq!(parse_edid(&bytes), Err(EdidError::InvalidLength));
    }

    #[test]
    fn test_parse_invalid_header() {
        let mut bytes = [0u8; 128];
        bytes[0] = 0x01; // Corrupt header
        assert_eq!(parse_edid(&bytes), Err(EdidError::InvalidHeader));
    }

    #[test]
    fn test_parse_checksum_mismatch() {
        let mut bytes = [0u8; 128];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[127] = 0x01; // Wrong checksum (should be 6 for all-zeros block with header)
        assert_eq!(parse_edid(&bytes), Err(EdidError::ChecksumMismatch));
    }

    #[test]
    fn test_parse_valid_minimal() {
        let mut bytes = [0u8; 128];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[127] = 6; // Correct checksum for header + zeros
        let result = parse_edid(&bytes);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.base_block[0..8], EDID_HEADER);
    }
}

