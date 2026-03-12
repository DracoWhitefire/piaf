use crate::model::{EdidError, ParsedEdid, ExtensionRegistry};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::Vec;

pub const EDID_HEADER: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];

pub fn parse_edid(bytes: &[u8], registry: &ExtensionRegistry) -> Result<ParsedEdid, EdidError> {
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

    #[cfg(any(feature = "alloc", feature = "std"))]
    let mut extensions = Vec::new();
    #[cfg(any(feature = "alloc", feature = "std"))]
    let mut warnings = Vec::new();

    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        let extension_count = base_block[126] as usize;
        let total_required = 128 * (1 + extension_count);

        if bytes.len() < total_required {
            return Err(EdidError::InvalidLength);
        }

        for i in 1..=extension_count {
            let start = i * 128;
            let mut ext_block = [0u8; 128];
            ext_block.copy_from_slice(&bytes[start..start + 128]);

            let ext_sum: u8 = ext_block.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
            if ext_sum != 0 {
                return Err(EdidError::ChecksumMismatch);
            }

            let tag = ext_block[0];
            if !registry.is_known(tag) {
                warnings.push(crate::model::EdidWarning::UnknownExtension(tag));
            }

            extensions.push(ext_block);
        }
    }

    Ok(ParsedEdid {
        base_block,
        #[cfg(any(feature = "alloc", feature = "std"))]
        extensions,
        #[cfg(any(feature = "alloc", feature = "std"))]
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_invalid_length() {
        let bytes = [0u8; 10];
        let registry = ExtensionRegistry::new();
        assert_eq!(parse_edid(&bytes, &registry), Err(EdidError::InvalidLength));
    }

    #[test]
    fn test_parse_invalid_header() {
        let mut bytes = [0u8; 128];
        bytes[0] = 0x01; // Corrupt header
        let registry = ExtensionRegistry::new();
        assert_eq!(parse_edid(&bytes, &registry), Err(EdidError::InvalidHeader));
    }

    #[test]
    fn test_parse_checksum_mismatch() {
        let mut bytes = [0u8; 128];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[127] = 0x01; // Wrong checksum (should be 6 for all-zeros block with header)
        let registry = ExtensionRegistry::new();
        assert_eq!(parse_edid(&bytes, &registry), Err(EdidError::ChecksumMismatch));
    }

    #[test]
    fn test_parse_valid_minimal() {
        let mut bytes = [0u8; 128];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[127] = 6; // Correct checksum for header + zeros
        let registry = ExtensionRegistry::new();
        let result = parse_edid(&bytes, &registry);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.base_block[0..8], EDID_HEADER);
        #[cfg(any(feature = "alloc", feature = "std"))]
        assert_eq!(parsed.extensions.len(), 0);
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn test_parse_with_extensions() {
        let mut bytes = [0u8; 256];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[126] = 1; // 1 extension
        bytes[127] = 5; // Checksum for header + extension_count=1

        // Extension block
        bytes[128] = 0x02; // Some tag
        bytes[255] = 254; // Checksum: 256 - 2 = 254 (0xFE)

        let registry = ExtensionRegistry::new();
        let result = parse_edid(&bytes, &registry);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.extensions.len(), 1);
        assert_eq!(parsed.extensions[0][0], 0x02);
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn test_parse_extension_checksum_mismatch() {
        let mut bytes = [0u8; 256];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[126] = 1;
        bytes[127] = 5;
        bytes[128] = 0x01;
        bytes[255] = 0x00; // Wrong checksum
        let registry = ExtensionRegistry::new();
        assert_eq!(parse_edid(&bytes, &registry), Err(EdidError::ChecksumMismatch));
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn test_parse_unknown_extension_warning() {
        let mut bytes = [0u8; 256];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[126] = 1;
        bytes[127] = 5;

        // Extension block with tag 0xEE (Unknown)
        bytes[128] = 0xEE;
        bytes[255] = 256u16.wrapping_sub(0xEE) as u8; // Correct checksum for 0xEE

        let registry = ExtensionRegistry::new();
        let result = parse_edid(&bytes, &registry);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.warnings.len(), 1);
        assert_eq!(
            parsed.warnings[0],
            crate::model::EdidWarning::UnknownExtension(0xEE)
        );
    }
    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn test_parse_known_extension_displayid() {
        let mut bytes = [0u8; 256];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[126] = 1;
        bytes[127] = 5;

        // Extension block with tag 0x70 (DisplayID)
        bytes[128] = 0x70;
        bytes[255] = 256u16.wrapping_sub(0x70) as u8;

        let registry = ExtensionRegistry::new();
        let result = parse_edid(&bytes, &registry);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.warnings.len(), 0); // Should be known
        assert_eq!(parsed.extensions.len(), 1);
        assert_eq!(parsed.extensions[0][0], 0x70);
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn test_custom_extension_registration() {
        let mut bytes = [0u8; 256];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[126] = 1;
        bytes[127] = 5;

        // Custom tag 0xEE
        let custom_tag = 0xEE;
        bytes[128] = custom_tag;
        bytes[255] = 256u16.wrapping_sub(custom_tag as u16) as u8;

        let mut registry = ExtensionRegistry::new();
        registry.register(custom_tag);

        let result = parse_edid(&bytes, &registry);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        
        // Should NOT have a warning because it was registered
        assert_eq!(parsed.warnings.len(), 0);
        assert_eq!(parsed.extensions.len(), 1);
        assert_eq!(parsed.extensions[0][0], custom_tag);
    }
}