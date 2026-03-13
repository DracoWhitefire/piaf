#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::prelude::{String, Vec, Box};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::capabilities::DisplayCapabilities;

#[cfg(any(feature = "alloc", feature = "std"))]
pub trait ExtensionHandler: core::fmt::Debug {
    fn process(&self, block: &[u8; 128], caps: &mut DisplayCapabilities);
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

pub struct ExtensionTagRegistry {
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub known_tags: Vec<u8>,
}

impl ExtensionTagRegistry {
    pub fn new() -> Self {
        #[cfg(any(feature = "alloc", feature = "std"))]
        let known_tags = Vec::new();

        Self {
            #[cfg(any(feature = "alloc", feature = "std"))]
            known_tags,
        }
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn register(&mut self, tag: u8) {
        if !self.known_tags.contains(&tag) {
            self.known_tags.push(tag);
        }
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn is_known(&self, tag: u8) -> bool {
        self.known_tags.contains(&tag)
    }

    #[cfg(not(any(feature = "alloc", feature = "std")))]
    pub fn is_known(&self, tag: u8) -> bool {
        tag == 0x02 || tag == 0x70
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
