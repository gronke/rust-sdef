#![no_main]

use libfuzzer_sys::fuzz_target;
use std::str::FromStr;

use sdef::Dictionary;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    // Lenient-mode parse must never panic regardless of input shape.
    let _ = Dictionary::from_str(s);
});
