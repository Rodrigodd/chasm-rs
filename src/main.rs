use leb128::write::unsigned as uleb128;
use std::io::Write;

mod run_wasm;
mod wasm_macro;
use wasm_macro::wasm;

mod compiler;

/// Compile the given chasm source code in a wasm binary.
pub fn compile(source: &str) -> anyhow::Result<Vec<u8>> {
    let mut main_function = wasm! {new
        // locals
        (vec)
    };

    // code
    compiler::Parser::parse(source, &mut main_function)?;

    wasm!(&mut main_function, (end));

    let binary = wasm!( new
        (magic version)
        (section type (vec
            (functype (vec f32) (vec))
            (functype (vec i32 i32) (vec))
        ))
        (section import (vec (import "env" "print" function 0x0)))
        (section function (vec 1))
        (section export (vec (export "main" function 0x1)))
        (section code (vec (data &main_function)))
    );

    Ok(binary)
}

#[test]
fn print() -> anyhow::Result<()> {
    let binary = compile("print 12")?;
    run_wasm::run_binary(&binary)
}

fn main() -> anyhow::Result<()> {
    let module_wat = r#"
        (module
         (import "env" "print" (func $print (param i32)))
         (func $fibo (param i32) (result i32)
          (i32.lt_s (get_local 0) (i32.const 2))
          (if (result i32)
           (then (i32.const 1))
           (else 
             get_local 0
             i32.const -1
             i32.add
             call $fibo
             get_local 0
             i32.const -2
             i32.add
             call $fibo
             i32.add
           ))
         )
         (func $main (export "main") (param $p0 i32)
          get_local $p0
          call $fibo
          call $print
        ))
        "#;

    let binary = wat::parse_str(module_wat)?;
    run_wasm::run_binary(&binary)?;

    Ok(())
}
