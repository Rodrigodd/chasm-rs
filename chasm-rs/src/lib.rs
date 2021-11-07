use std::io::Write;

mod wasm_macro;
use wasm_macro::wasm;

mod compiler;
pub use compiler::Error;

#[cfg(test)]
mod test;
#[cfg(test)]
mod run_wasm;

fn write_section(w: &mut Vec<u8>, section_type: u8, f: impl Fn(&mut Vec<u8>)) {
    // section type
    w.write(&[section_type]).unwrap();
    let section_start = w.len();

    f(w);

    // write the length of the section at the start

    // write the length
    let section_len = w.len() - section_start;
    let len = leb128::write::unsigned(w, section_len as u64).unwrap();
    // move it to the start
    w[section_start..].rotate_right(len);
}

/// Compile the given chasm source code in a wasm binary.
pub fn compile(source: &str) -> anyhow::Result<Vec<u8>> {
    let functions = compiler::Parser::parse(source)?;

    let mut binary = wasm!( new
        (magic version)
    );

    // section type
    write_section(&mut binary, wasm!(section_type type), |mut w| {
        // number of types
        leb128::write::unsigned(&mut w, 1 + functions.len() as u64).unwrap();
        // print function type
        wasm!(&mut w, (functype (vec f32) (vec)));
        for f in &functions {
            wasm!(&mut w, functype);
            leb128::write::unsigned(&mut w, f.num_param as u64).unwrap();
            for _ in 0..f.num_param {
                wasm!(&mut w, f32);
            }
            wasm!(&mut w, (vec));
        }
    });

    wasm!(&mut binary, 
        (section import (vec
            (import "env" "print" (function 0x0))
            (import "env" "memory" (memory 1))))
    );

    // (section function (vec 1))
    write_section(&mut binary, wasm!(section_type function), |mut w| {
        // number of functions
        leb128::write::unsigned(&mut w, functions.len() as u64).unwrap();
        // print function type
        for f in &functions {
            leb128::write::unsigned(&mut w, f.idx as u64).unwrap();
        }
    });

    wasm!(&mut binary, (section export (vec (export "main" function 0x1))));

    // section code
    write_section(&mut binary, wasm!(section_type code), |mut w| {
        // number of functions
        leb128::write::unsigned(&mut w, functions.len() as u64).unwrap();
        // print function type
        for f in &functions {
            leb128::write::unsigned(&mut w, f.code.len() as u64).unwrap();
            w.write(&f.code).unwrap();
        }
    });

    Ok(binary)
}

