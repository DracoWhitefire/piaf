#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::prelude::{String, Vec, Box};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;

#[cfg(any(feature = "alloc", feature = "std"))]
pub trait ExtensionHandler: core::fmt::Debug {
    fn process(&self, block: &[u8; 128], caps: &mut DisplayCapabilities, warnings: &mut Vec<EdidWarning>);
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub struct ExtensionMetadata {
    pub tag: u8,
    pub display_name: String,
    pub handler: Option<Box<dyn ExtensionHandler>>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl core::fmt::Debug for ExtensionMetadata {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ExtensionMetadata")
            .field("tag", &self.tag)
            .field("display_name", &self.display_name)
            .field("has_handler", &self.handler.is_some())
            .finish()
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionMetadata {
    pub tag: u8,
}

/// Implemented by types that can tell the parser which extension tags are known.
/// Implemented for both [`ExtensionTagRegistry`] and [`ExtensionLibrary`], so either
/// can be passed directly to [`parse_edid`][crate::parse_edid].
pub trait KnownExtensions {
    fn is_known(&self, tag: u8) -> bool;
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub struct ExtensionTagRegistry {
    pub known_tags: Vec<u8>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl ExtensionTagRegistry {
    pub fn new() -> Self {
        Self { known_tags: Vec::new() }
    }

    pub fn register(&mut self, tag: u8) {
        if !self.known_tags.contains(&tag) {
            self.known_tags.push(tag);
        }
    }

    pub fn is_known(&self, tag: u8) -> bool {
        self.known_tags.contains(&tag)
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
pub struct ExtensionTagRegistry {
    tags: [u8; 16],
    len: usize,
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl ExtensionTagRegistry {
    pub fn new() -> Self {
        Self { tags: [0u8; 16], len: 0 }
    }

    pub fn register(&mut self, tag: u8) {
        if self.len < 16 && !self.is_known(tag) {
            self.tags[self.len] = tag;
            self.len += 1;
        }
    }

    pub fn is_known(&self, tag: u8) -> bool {
        self.tags[..self.len].contains(&tag)
    }
}

impl KnownExtensions for ExtensionTagRegistry {
    fn is_known(&self, tag: u8) -> bool {
        ExtensionTagRegistry::is_known(self, tag)
    }
}

pub struct ExtensionLibrary {
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub base_handlers: Vec<Box<dyn ExtensionHandler>>,
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub extensions: Vec<ExtensionMetadata>,
}

impl ExtensionLibrary {
    pub fn new() -> Self {
        Self {
            #[cfg(any(feature = "alloc", feature = "std"))]
            base_handlers: Vec::new(),
            #[cfg(any(feature = "alloc", feature = "std"))]
            extensions: Vec::new(),
        }
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn add_base_handler<H: ExtensionHandler + 'static>(&mut self, handler: H) {
        self.base_handlers.push(Box::new(handler));
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn with_standard_extensions() -> Self {
        let mut lib = Self::new();
        lib.register(ExtensionMetadata {
            tag: 0x02,
            display_name: String::from("CEA-861"),
            handler: None, // Will be set by caller or library user
        });
        lib.register(ExtensionMetadata {
            tag: 0x70,
            display_name: String::from("DisplayID"),
            handler: None,
        });
        lib
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn register(&mut self, metadata: ExtensionMetadata) {
        if !self.extensions.iter().any(|ext| ext.tag == metadata.tag) {
            self.extensions.push(metadata);
        }
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn export_tags(&self) -> ExtensionTagRegistry {
        let mut known_tags = Vec::new();
        for ext in &self.extensions {
            known_tags.push(ext.tag);
        }
        ExtensionTagRegistry { known_tags }
    }

    #[cfg(not(any(feature = "alloc", feature = "std")))]
    pub fn export_tags(&self) -> ExtensionTagRegistry {
        ExtensionTagRegistry::new()
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl KnownExtensions for ExtensionLibrary {
    fn is_known(&self, tag: u8) -> bool {
        self.extensions.iter().any(|e| e.tag == tag)
    }
}
