//! This crate is a single-pass compiler to WebAssembly for the language chasm.
//!
//! chasm is a very simple language created by Colin Eberhardt, to introduce the basic building
//! blocks of compilers, and reveal some of the inner workings of WebAssembly. This is a
//! implementation of the compiler in Rust.
#![deny(missing_docs)]
use std::io::Write;

mod wasm_macro;
use wasm_macro::wasm;

pub(crate) mod compiler;
pub use compiler::{Error, ErrorKind};

#[cfg(test)]
mod run_wasm;
#[cfg(test)]
mod test;

fn write_section(w: &mut Vec<u8>, section_type: u8, f: impl Fn(&mut Vec<u8>)) {
    // section type
    w.write_all(&[section_type]).unwrap();
    let section_start = w.len();

    f(w);

    // write the length of the section at the start

    // write the length
    let section_len = w.len() - section_start;
    let len = leb128::write::unsigned(w, section_len as u64).unwrap();
    // move it to the start
    w[section_start..].rotate_right(len);
}

/// Compile the given chasm source code in a WebAssembly module.
///
/// The created module imports the function `"env" "print"` that received a f32 and return nothing,
/// and a memory `"env" "memory"` with a minimal size of 1, and exports the function `"main"`, that
/// has no argument or return, which is the code entry point.
///
/// At the end of the execution of the created module, the rendered 100x100 output will be in the
/// linear memory, in the range 0..10000.
///
/// # Example
/// ```
/// let source = "
///     var y = 0
///     while (y < 100)
///         var x = 0
///         while (x < 100)
///             c = ((y/100)*255)
///             setpixel (x, y, c)
///             x = (x + 1)
///         endwhile
///         y = (y + 1)
///     endwhile";
///
/// let wasm = chasm_rs::compile(source);
///
/// assert!(wasm.is_ok());
/// ```
pub fn compile<'s>(source: &'s str) -> Result<Vec<u8>, Error<'s>> {
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
            w.write_all(&f.code).unwrap();
        }
    });

    Ok(binary)
}
