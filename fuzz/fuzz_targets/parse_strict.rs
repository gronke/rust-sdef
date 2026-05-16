#![no_main]

use libfuzzer_sys::fuzz_target;
use sdef::Dictionary;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    // Strict-mode parse must never panic regardless of input shape.
    // Errors are fine — we're verifying robustness, not validity.
    let _ = Dictionary::from_str_strict(s);
});
