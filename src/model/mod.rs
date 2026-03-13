//! Data model types for parsed EDID data and display capabilities.

/// Re-exports of `alloc`/`std` collection types used across the crate.
pub mod prelude;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use prelude::{Box, String, Vec};

/// Extension handler traits and registries.
pub mod extension;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use extension::ExtensionHandler;
pub use extension::{ExtensionLibrary, ExtensionMetadata, ExtensionTagRegistry, KnownExtensions};

/// Error and warning types.
pub mod diagnostics;
pub use diagnostics::{EdidError, EdidWarning};

/// Parsed EDID intermediate representation.
pub mod edid;
pub use edid::ParsedEdid;

/// Consumer-facing capability types.
pub mod capabilities;
#[cfg(feature = "std")]
pub use capabilities::ExtensionData;
pub use capabilities::{DisplayCapabilities, VideoMode};
