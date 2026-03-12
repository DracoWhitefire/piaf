pub mod prelude;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use prelude::prelude::{String, Vec};

pub mod extension;
pub use extension::{ExtensionLibrary, ExtensionMetadata, ExtensionTagRegistry};

pub mod diagnostics;
pub use diagnostics::{EdidError, EdidWarning};

pub mod edid;
pub use edid::ParsedEdid;

pub mod capabilities;
pub use capabilities::DisplayCapabilities;
