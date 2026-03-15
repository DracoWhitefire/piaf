use crate::model::diagnostics::EdidError;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::{EdidWarning, ParseWarning};
use crate::model::extension::KnownExtensions;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::Arc;
use crate::model::ParsedEdid;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::Vec;

/// The fixed 8-byte magic header at offset 0 of every valid EDID block.
pub const EDID_HEADER: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];

/// Maximum number of extension blocks the parser will store and process.
///
/// The EDID spec allows up to 255 extension blocks, but real displays have at most a handful.
/// Inputs claiming more than this limit are most likely malformed or hostile; the parser
/// emits [`EdidWarning::ExtensionBlockLimitReached`] and ignores the excess blocks.
pub const MAX_EXTENSION_BLOCKS: usize = 64;

/// Validates and decodes a raw EDID byte slice into a [`ParsedEdid`].
///
/// Performs length validation, header verification, and checksum verification for the base block
/// and all declared extension blocks. Extension blocks with tags not known to `tags` are stored
/// but generate an [`EdidWarning::UnknownExtension`] warning.
///
/// Pass an [`ExtensionLibrary`][crate::ExtensionLibrary], an
/// [`ExtensionTagRegistry`][crate::ExtensionTagRegistry], or a
/// `&[&dyn StaticExtensionHandler]` slice such as [`STANDARD_HANDLERS`][crate::STANDARD_HANDLERS]
/// as `tags`; all implement [`KnownExtensions`].
///
/// # Errors
///
/// Returns [`EdidError`] if the byte slice fails any structural check.
pub fn parse_edid<T: KnownExtensions + ?Sized>(
    bytes: &[u8],
    tags: &T,
) -> Result<ParsedEdid, EdidError> {
    let _ = tags; // Suppress unused warning in no_alloc
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
    let mut warnings: Vec<ParseWarning> = Vec::new();

    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        let extension_count = base_block[126] as usize;
        let total_required = 128 * (1 + extension_count);

        if bytes.len() < total_required {
            return Err(EdidError::InvalidLength);
        }

        if bytes.len() > total_required {
            warnings.push(Arc::new(EdidWarning::SizeMismatch {
                expected: total_required,
                actual: bytes.len(),
            }));
        }

        let blocks_to_parse = if extension_count > MAX_EXTENSION_BLOCKS {
            warnings.push(Arc::new(EdidWarning::ExtensionBlockLimitReached {
                declared: extension_count,
                limit: MAX_EXTENSION_BLOCKS,
            }));
            MAX_EXTENSION_BLOCKS
        } else {
            extension_count
        };

        for i in 1..=blocks_to_parse {
            let start = i * 128;
            let mut ext_block = [0u8; 128];
            ext_block.copy_from_slice(&bytes[start..start + 128]);

            let ext_sum: u8 = ext_block.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
            if ext_sum != 0 {
                return Err(EdidError::ChecksumMismatch);
            }

            let tag = ext_block[0];
            if !tags.is_known(tag) {
                warnings.push(Arc::new(EdidWarning::UnknownExtension(tag)));
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
    use crate::model::extension::ExtensionTagRegistry;

    #[test]
    fn test_parse_invalid_length() {
        let bytes = [0u8; 10];
        let registry = ExtensionTagRegistry::new();
        assert_eq!(
            parse_edid(&bytes, &registry).unwrap_err(),
            EdidError::InvalidLength
        );
    }

    #[test]
    fn test_parse_invalid_header() {
        let mut bytes = [0u8; 128];
        bytes[0] = 0x01; // Corrupt header
        let registry = ExtensionTagRegistry::new();
        assert_eq!(
            parse_edid(&bytes, &registry).unwrap_err(),
            EdidError::InvalidHeader
        );
    }

    #[test]
    fn test_parse_checksum_mismatch() {
        let mut bytes = [0u8; 128];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[127] = 0x01; // Wrong checksum (should be 6 for all-zeros block with header)
        let registry = ExtensionTagRegistry::new();
        assert_eq!(
            parse_edid(&bytes, &registry).unwrap_err(),
            EdidError::ChecksumMismatch
        );
    }

    #[test]
    fn test_parse_valid_minimal() {
        let mut bytes = [0u8; 128];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[127] = 6; // Correct checksum for header + zeros
        let registry = ExtensionTagRegistry::new();
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

        let registry = ExtensionTagRegistry::new();
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
        let registry = ExtensionTagRegistry::new();
        assert_eq!(
            parse_edid(&bytes, &registry).unwrap_err(),
            EdidError::ChecksumMismatch
        );
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

        let registry = ExtensionTagRegistry::new();
        let result = parse_edid(&bytes, &registry);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.warnings.len(), 1);
        assert_eq!(
            (*parsed.warnings[0]).downcast_ref::<EdidWarning>(),
            Some(&EdidWarning::UnknownExtension(0xEE))
        );
    }
    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn test_parse_trailing_bytes_warns() {
        // 256-byte buffer but extension_count = 0 → expected 128, actual 256
        let mut bytes = [0u8; 256];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[127] = 6; // Correct checksum for header + zeros (no extensions)
        let registry = ExtensionTagRegistry::new();
        let result = parse_edid(&bytes, &registry);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.warnings.len(), 1);
        assert_eq!(
            (*parsed.warnings[0]).downcast_ref::<EdidWarning>(),
            Some(&EdidWarning::SizeMismatch {
                expected: 128,
                actual: 256
            })
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

        let registry =
            crate::model::extension::ExtensionLibrary::with_standard_extensions().export_tags();
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

        let mut registry = ExtensionTagRegistry::new();
        registry.register(custom_tag);

        let result = parse_edid(&bytes, &registry);
        assert!(result.is_ok());
        let parsed = result.unwrap();

        // Should NOT have a warning because it was registered
        assert_eq!(parsed.warnings.len(), 0);
        assert_eq!(parsed.extensions.len(), 1);
        assert_eq!(parsed.extensions[0][0], custom_tag);
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn test_extension_block_limit() {
        // Build an EDID that declares MAX_EXTENSION_BLOCKS + 1 extension blocks.
        let declared = MAX_EXTENSION_BLOCKS + 1;
        let total_bytes = 128 * (1 + declared);
        let mut bytes = vec![0u8; total_bytes];
        bytes[0..8].copy_from_slice(&EDID_HEADER);
        bytes[126] = declared as u8;

        // Fix up base block checksum.
        let base_sum: u8 = bytes[0..127]
            .iter()
            .fold(0u8, |acc, &x| acc.wrapping_add(x));
        bytes[127] = 0u8.wrapping_sub(base_sum);

        // Each extension block needs a valid checksum (all-zero payload → checksum 0).
        // They are already zero-initialised, so no further fixup needed.

        let registry = ExtensionTagRegistry::new();
        let parsed = parse_edid(&bytes, &registry).unwrap();

        assert_eq!(parsed.extensions.len(), MAX_EXTENSION_BLOCKS);
        assert!(parsed.warnings.iter().any(|w| {
            (*w).downcast_ref::<EdidWarning>()
                == Some(&EdidWarning::ExtensionBlockLimitReached {
                    declared,
                    limit: MAX_EXTENSION_BLOCKS,
                })
        }));
    }
}
