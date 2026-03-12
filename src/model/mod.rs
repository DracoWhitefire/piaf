pub mod prelude;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use prelude::prelude::{String, Vec};

pub mod extension;
pub use extension::{ExtensionLibrary, ExtensionMetadata, ExtensionTagRegistry};

pub mod diagnostics;
pub use diagnostics::{EdidError, EdidWarning};

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
