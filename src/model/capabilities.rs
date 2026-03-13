#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::prelude::{String, Vec};
#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::sync::Arc;
#[cfg(feature = "std")]
use core::any::Any;

/// Trait for typed data stored in [`DisplayCapabilities::extension_data`] by custom handlers.
///
/// A blanket implementation covers any type that is `Any + Debug + Send + Sync`, so consumers
/// do not need to implement this trait manually — `#[derive(Debug)]` on a `Send + Sync` type
/// is sufficient.
#[cfg(feature = "std")]
pub trait ExtensionData: Any + core::fmt::Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

#[cfg(feature = "std")]
impl<T: Any + core::fmt::Debug + Send + Sync> ExtensionData for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VideoMode {
    pub width: u16,
    pub height: u16,
    pub refresh_rate: u8,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
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
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub extension_data: HashMap<u8, Arc<dyn ExtensionData>>,
}

#[cfg(feature = "std")]
impl DisplayCapabilities {
    /// Store typed data from a custom handler, keyed by an extension tag.
    pub fn set_extension_data<T: ExtensionData>(&mut self, tag: u8, data: T) {
        self.extension_data.insert(tag, Arc::new(data));
    }

    /// Retrieve typed data previously stored by a handler for the given tag.
    /// Returns `None` if no data is stored for the tag or the type does not match.
    pub fn get_extension_data<T: Any>(&self, tag: u8) -> Option<&T> {
        self.extension_data.get(&tag)?.as_any().downcast_ref::<T>()
    }
}
