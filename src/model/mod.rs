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

pub mod extension;
pub use extension::{ExtensionLibrary, ExtensionMetadata, ExtensionTagRegistry};

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
