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

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionTag {
    Cea861 = 0x02,
    DisplayId = 0x70,
    Unknown(u8),
}

impl From<u8> for ExtensionTag {
    fn from(tag: u8) -> Self {
        match tag {
            0x02 => ExtensionTag::Cea861,
            0x70 => ExtensionTag::DisplayId,
            _ => ExtensionTag::Unknown(tag),
        }
    }
}

pub struct ExtensionRegistry {
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub custom_tags: Vec<u8>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self {
            #[cfg(any(feature = "alloc", feature = "std"))]
            custom_tags: Vec::new(),
        }
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn register(&mut self, tag: u8) {
        if !self.custom_tags.contains(&tag) {
            self.custom_tags.push(tag);
        }
    }

    pub fn is_known(&self, tag: u8) -> bool {
        if !matches!(ExtensionTag::from(tag), ExtensionTag::Unknown(_)) {
            return true;
        }

        #[cfg(any(feature = "alloc", feature = "std"))]
        {
            return self.custom_tags.contains(&tag);
        }

        #[cfg(not(any(feature = "alloc", feature = "std")))]
        false
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
