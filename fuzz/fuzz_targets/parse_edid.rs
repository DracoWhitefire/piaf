#![no_main]
use libfuzzer_sys::fuzz_target;
use piaf::{capabilities_from_edid, parse_edid, ExtensionLibrary};

fuzz_target!(|data: &[u8]| {
    let library = ExtensionLibrary::with_standard_handlers();
    if let Ok(parsed) = parse_edid(data, &library) {
        let _ = capabilities_from_edid(&parsed, &library);
    }
});
