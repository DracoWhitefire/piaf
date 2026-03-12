#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::prelude::{String, Vec};

#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionMetadata {
    pub tag: u8,
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub display_name: String,
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
    pub extensions: Vec<ExtensionMetadata>,
}

impl ExtensionLibrary {
    pub fn new() -> Self {
        #[cfg(any(feature = "alloc", feature = "std"))]
        let mut extensions = Vec::new();
        #[cfg(any(feature = "alloc", feature = "std"))]
        {
            extensions.push(ExtensionMetadata {
                tag: 0x02,
                display_name: String::from("CEA-861"),
            });
            extensions.push(ExtensionMetadata {
                tag: 0x70,
                display_name: String::from("DisplayID"),
            });
        }

        Self {
            #[cfg(any(feature = "alloc", feature = "std"))]
            extensions,
        }
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
}
