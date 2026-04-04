#![no_main]

use std::path::Path;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);
    let _ = deslop::validate_source(Path::new("fuzz_input.py"), source.as_ref());
});
