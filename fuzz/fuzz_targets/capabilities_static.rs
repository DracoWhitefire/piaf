#![no_main]
use libfuzzer_sys::fuzz_target;
use piaf::{capabilities_from_edid_static, parse_edid, CEA861_HANDLER, DISPLAYID_HANDLER, STANDARD_HANDLERS};

fuzz_target!(|data: &[u8]| {
    let handlers: &[&dyn piaf::StaticExtensionHandler] = &[CEA861_HANDLER, DISPLAYID_HANDLER];
    if let Ok(parsed) = parse_edid(data, STANDARD_HANDLERS) {
        let _: piaf::StaticDisplayCapabilities<64> =
            capabilities_from_edid_static(&parsed, handlers);
    }
});
