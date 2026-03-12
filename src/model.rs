#[cfg(any(feature = "alloc", feature = "std"))]
mod prelude {
    #[cfg(feature = "std")]
    pub use std::string::String;
    #[cfg(feature = "std")]
    pub use std::vec::Vec;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::string::String;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::vec::Vec;
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub use prelude::{String, Vec};

#[derive(Debug, Clone, PartialEq)]
pub struct Extension {
    pub tag: u8,
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub display_name: String,
}

pub struct ExtensionRegistry {
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub known_extensions: Vec<Extension>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        #[cfg(any(feature = "alloc", feature = "std"))]
        let mut known_extensions = Vec::new();
        #[cfg(any(feature = "alloc", feature = "std"))]
        {
            known_extensions.push(Extension {
                tag: 0x02,
                display_name: String::from("CEA-861"),
            });
            known_extensions.push(Extension {
                tag: 0x70,
                display_name: String::from("DisplayID"),
            });
        }

        Self {
            #[cfg(any(feature = "alloc", feature = "std"))]
            known_extensions,
        }
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn register(&mut self, extension: Extension) {
        if !self.known_extensions.iter().any(|ext| ext.tag == extension.tag) {
            self.known_extensions.push(extension);
        }
    }

    pub fn is_known(&self, tag: u8) -> bool {
        #[cfg(any(feature = "alloc", feature = "std"))]
        {
            return self.known_extensions.iter().any(|ext| ext.tag == tag);
        }

        #[cfg(not(any(feature = "alloc", feature = "std")))]
        {
            // In a truly no_alloc/no_std environment, we might have to hardcode standard tags
            // or provide a different mechanism. For now, let's keep it simple.
            tag == 0x02 || tag == 0x70
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdidWarning {
    UnknownExtension(u8),
    DescriptorParseFailed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdidError {
    InvalidLength,
    InvalidHeader,
    ChecksumMismatch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedEdid {
    pub base_block: [u8; 128],
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub extensions: Vec<[u8; 128]>,
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub warnings: Vec<EdidWarning>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DisplayCapabilities {
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub manufacturer: Option<String>,
    pub product_code: Option<u16>,
    pub serial_number: Option<u32>,
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub display_name: Option<String>,
    pub digital: bool,
    pub width_cm: Option<u16>,
    pub height_cm: Option<u16>,
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub warnings: Vec<EdidWarning>,
}
