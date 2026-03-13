use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::extension::ExtensionHandler;

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug)]
pub struct Cea861Handler;

#[cfg(any(feature = "alloc", feature = "std"))]
impl ExtensionHandler for Cea861Handler {
    fn process(&self, ext: &[u8; 128], caps: &mut DisplayCapabilities) {
        // CEA-861 Extension Block
        // Offset 2: Offset of DTDs
        // Bit 7 of byte 3: 1=Supports basic audio
        if (ext[3] & 0x40) != 0 {
            caps.has_audio = true;
        }

        // We could also parse Video Data Blocks (VICs) here to get more modes
    }
}
