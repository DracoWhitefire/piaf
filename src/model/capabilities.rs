#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::prelude::{String, Vec};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VideoMode {
    pub width: u16,
    pub height: u16,
    pub refresh_rate: u8,
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
    pub min_v_rate: Option<u8>,
    pub max_v_rate: Option<u8>,
    pub min_h_rate_khz: Option<u8>,
    pub max_h_rate_khz: Option<u8>,
    pub max_pixel_clock_mhz: Option<u16>,
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub supported_modes: Vec<VideoMode>,
    pub has_audio: bool,
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub warnings: Vec<EdidWarning>,
}
