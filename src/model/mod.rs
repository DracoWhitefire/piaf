pub mod prelude;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use prelude::prelude::{String, Vec, Box};

pub mod extension;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use extension::ExtensionHandler;
pub use extension::{ExtensionLibrary, ExtensionMetadata, ExtensionTagRegistry};

pub mod diagnostics;
pub use diagnostics::{EdidError, EdidWarning};

pub mod edid;
pub use edid::ParsedEdid;

pub mod capabilities;
#[cfg(feature = "std")]
pub use capabilities::ExtensionData;
pub use capabilities::{DisplayCapabilities, VideoMode};
