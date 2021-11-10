#![no_main]
use libfuzzer_sys::fuzz_target;
use wasmi::Module;

fuzz_target!(|source: &str| {
    // `chasm_rs::compile` should never panic
    if let Ok(binary) = chasm_rs::compile(source) {
        // if the source compiled, then it should be a valid wasm.
        let _ = Module::from_buffer(&binary).unwrap();
    }
});
