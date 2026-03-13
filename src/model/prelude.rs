#[cfg(any(feature = "alloc", feature = "std"))]
pub mod prelude {
    #[cfg(feature = "std")]
    pub use std::string::String;
    #[cfg(feature = "std")]
    pub use std::vec::Vec;
    #[cfg(feature = "std")]
    pub use std::boxed::Box;
    
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::string::String;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::vec::Vec;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub use alloc::boxed::Box;
}
