mod base;
mod cea861;

#[cfg(any(feature = "alloc", feature = "std"))]
pub use base::BaseBlockHandler;
pub use base::VideoInputFlags;
pub use cea861::Cea861Flags;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use cea861::Cea861Handler;

use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionLibrary;
#[cfg(not(any(feature = "alloc", feature = "std")))]
use crate::model::extension::ExtensionLibrary;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::Box;
use crate::model::ParsedEdid;

#[cfg(any(feature = "alloc", feature = "std"))]
impl ExtensionLibrary {
    /// Creates a library pre-loaded with the built-in [`BaseBlockHandler`] and [`Cea861Handler`].
    ///
    /// This is the recommended starting point for most consumers. Additional handlers can be
    /// added after construction via [`add_base_handler`][ExtensionLibrary::add_base_handler]
    /// and [`register`][ExtensionLibrary::register].
    pub fn with_standard_handlers() -> Self {
        let mut lib = Self::with_standard_extensions();
        lib.add_base_handler(BaseBlockHandler);
        if let Some(cea) = lib.extensions.iter_mut().find(|e| e.tag == 0x02) {
            cea.handler = Some(Box::new(Cea861Handler));
        }
        lib
    }
}

/// Derives [`DisplayCapabilities`] from a [`ParsedEdid`] by running all registered handlers.
///
/// Base handlers are called first (in registration order), then extension block handlers are
/// called for each extension block whose tag matches a registered entry in `library`.
/// Warnings from all handlers are collected into [`DisplayCapabilities::warnings`].
pub fn capabilities_from_edid(
    edid: &ParsedEdid,
    library: &ExtensionLibrary,
) -> DisplayCapabilities {
    #[cfg(any(feature = "alloc", feature = "std"))]
    let mut caps = DisplayCapabilities::default();
    #[cfg(not(any(feature = "alloc", feature = "std")))]
    let caps = DisplayCapabilities::default();

    #[cfg(any(feature = "alloc", feature = "std"))]
    {
        let mut warnings = Vec::new();

        // 1. Process Base Block through all registered base handlers, in order
        for handler in &library.base_handlers {
            handler.process(&edid.base_block, &mut caps, &mut warnings);
        }

        // 2. Process Extension Blocks via registered handlers
        for ext in &edid.extensions {
            let tag = ext[0];
            if let Some(metadata) = library.extensions.iter().find(|e| e.tag == tag) {
                if let Some(handler) = &metadata.handler {
                    handler.process(ext, &mut caps, &mut warnings);
                }
            }
        }

        caps.warnings.extend(warnings);
    }

    #[cfg(not(any(feature = "alloc", feature = "std")))]
    {
        let _ = edid;
        let _ = library;
    }

    caps
}

#[cfg(test)]
#[cfg(any(feature = "alloc", feature = "std"))]
mod tests {
    use super::*;
    use crate::model::extension::ExtensionTagRegistry;
    use crate::parser::parse_edid;

    // Verifies that with_standard_handlers() wires BaseBlockHandler into the pipeline.
    // Handler-level assertions live in base.rs and cea861.rs.
    #[test]
    fn test_standard_handlers_are_wired() {
        let mut bytes = [0u8; 128];
        bytes[0..8].copy_from_slice(&crate::parser::EDID_HEADER);
        bytes[0x14] = 0x80; // Digital input flag

        let mut sum = 0u8;
        for i in 0..127 {
            sum = sum.wrapping_add(bytes[i]);
        }
        bytes[127] = 0u8.wrapping_sub(sum);

        let registry = ExtensionTagRegistry::new();
        let parsed = parse_edid(&bytes, &registry).unwrap();
        let caps = capabilities_from_edid(&parsed, &ExtensionLibrary::with_standard_handlers());

        assert!(caps.digital);
    }
}
