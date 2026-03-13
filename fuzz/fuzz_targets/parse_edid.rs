#![no_main]
use libfuzzer_sys::fuzz_target;
use piaf::{capabilities_from_edid, parse_edid, ExtensionLibrary, ExtensionTagRegistry};

fuzz_target!(|data: &[u8]| {
    let registry = ExtensionTagRegistry::new();
    if let Ok(parsed) = parse_edid(data, &registry) {
        let library = ExtensionLibrary::with_standard_handlers();
        let _ = capabilities_from_edid(&parsed, &library);
    }
});
