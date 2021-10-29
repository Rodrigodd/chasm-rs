use std::io::Write;
use leb128::write::unsigned as uleb128;

mod run_wasm;
mod wasm_macro;
use wasm_macro::wasm;

/// Compile the given chasm code in a wasm binary.
pub fn compile(_code: &str) -> anyhow::Result<Vec<u8>> {

    let mut function = Vec::new();
    // locals
    wasm!(&mut function, (vec));
    // code
    wasm! {&mut function,
        (get_local 0)
        (get_local 1)
        (i32.add)
        (end)
    }

    let mut functions = Vec::<u8>::new();
    wasm!(&mut functions, (vec(data & function)));

    let mut binary = Vec::new();
    // magic module header
    binary.write(b"\0asm").unwrap();
    // module version
    binary.write(&[1, 0, 0, 0]).unwrap();
    wasm!( &mut binary,
           (section type (vec (functype (vec i32 i32) (vec i32))))
           (section function (vec 0))
           (section export (vec (export "main" function 0x0)))
           (section code (vec (data &function)))
    );

    Ok(binary)
}

#[test]
fn run_add() -> anyhow::Result<()> {
    let binary = compile("")?;
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
