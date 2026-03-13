#[cfg(feature = "std")]
pub use std::boxed::Box;
#[cfg(feature = "std")]
pub use std::string::String;
#[cfg(feature = "std")]
pub use std::sync::Arc;
#[cfg(feature = "std")]
pub use std::vec::Vec;

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc::boxed::Box;
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc::string::String;
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc::sync::Arc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc::vec::Vec;
