#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[cfg(any(feature = "alloc", feature = "std"))]
mod prelude {
    #[cfg(feature = "std")]
    pub use std::vec::Vec;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::vec::Vec;
}

#[cfg(any(feature = "alloc", feature = "std"))]
use prelude::Vec;
