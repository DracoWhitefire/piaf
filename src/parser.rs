use crate::model::{EdidError, ParsedEdid};
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::Vec;


pub fn parse_edid(_bytes: &[u8]) -> Result<ParsedEdid, EdidError> {
    unimplemented!("EDID parsing is not yet implemented");
}
