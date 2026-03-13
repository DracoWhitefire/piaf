#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::Vec;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedEdid {
    pub base_block: [u8; 128],
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub extensions: Vec<[u8; 128]>,
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub warnings: Vec<EdidWarning>,
}
