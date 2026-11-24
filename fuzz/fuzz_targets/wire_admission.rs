#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let _ = paignia_io::solve_static_json(bytes);
    let _ = paignia_io::solve_profile_json(bytes);
});
