use std::io::{self, Write};

mod run_wasm;

// From gimli-rs/leb128

const CONTINUATION_BIT: u8 = 1 << 7;

#[doc(hidden)]
#[inline]
pub fn low_bits_of_byte(byte: u8) -> u8 {
    byte & !CONTINUATION_BIT
}

#[doc(hidden)]
#[inline]
pub fn low_bits_of_u64(val: u64) -> u8 {
    let byte = val & (std::u8::MAX as u64);
    low_bits_of_byte(byte as u8)
}

/// Write `val` to the `std::io::Write` stream `w` as an unsigned LEB128 value.
///
/// On success, return the number of bytes written to `w`.
pub fn uleb128<W>(w: &mut W, mut val: u64) -> Result<usize, io::Error>
where
    W: ?Sized + Write,
{
    let mut bytes_written = 0;
    loop {
        let mut byte = low_bits_of_u64(val);
        val >>= 7;
        if val != 0 {
            // More bytes to come, so set the continuation bit.
            byte |= CONTINUATION_BIT;
        }

        let buf = [byte];
        w.write_all(&buf)?;
        bytes_written += 1;

        if val == 0 {
            return Ok(bytes_written);
        }
    }
}

#[repr(u8)]
enum Section {
    Type = 1,
    Func = 3,
    Export = 7,
    Code = 0xa,
}

// https://webassembly.github.io/spec/core/binary/modules.html#sections
// sections are encoded by their type followed by their vector contents
fn create_section(section_type: Section, data: &[u8]) -> Vec<u8> {
    let mut section = Vec::new();
    section.push(section_type as u8);
    uleb128(&mut section, data.len() as u64).unwrap();
    section.write(data).unwrap();
    section
}

macro_rules! wasm {
    (new  $($e:tt)*) => {
        {
            let mut vec = Vec::new();
            wasm!(&mut vec, $($e)*);
            vec
        }
    };
    ($w:expr, $( ( $($e:tt)* ) )*) => {
        $( wasm!( $w, $($e)* ); )*
    };
	($w:expr, get_local $e:literal) => {
		($w).write(&[0x20]).unwrap();
        uleb128($w, $e).unwrap();
	};
	($w:expr, str $e:literal) => {
        {
            let data = ($e).as_bytes();
            uleb128(&mut $w, data.len() as u64).unwrap();
            ($w).write(data).unwrap();
        }
	};
	($w:expr, $e:literal) => {
        uleb128($w, $e).unwrap();
	};
    ($w:expr, data $e:expr) => {
        uleb128($w, ($e).len() as u64).unwrap();
        ($w).write($e).unwrap();
    };
	($w:expr, i32.add) => {
		($w).write(&[0x6a]).unwrap();
	};
    ($w:expr, vec $($e:tt)*) => {
        {
            let mut vector = Vec::<u8>::new();
            let mut n = 0;
            $(
                wasm!( &mut vector, $e );
                n += 1; 
            )*
            uleb128($w, n).unwrap();
            ($w).write(&vector).unwrap();
        }
    };
	($w:expr, end) => {
		($w).write(&[0x0b]).unwrap();
	};
	($w:expr, functype) => {
		($w).write(&[0x60]).unwrap();
	};
	($w:expr, i32) => {
		($w).write(&[0x7f]).unwrap();
	};
	($w:expr, f32) => {
		($w).write(&[0x7d]).unwrap();
	};
	($w:expr, exporttypefunc) => {
		($w).write(&[0x00]).unwrap();
	};

}

/// Compile the given chasm code in a wasm binary.
pub fn compile(code: &str) -> anyhow::Result<Vec<u8>> {
    // (section type (vec (func (vec i32 i32) (vec i32)))
    // (section func (vec 0))
    // (section export (vec ((str "main") func 0x0)))
    // (section code (vec (size func)))

    let types = wasm!(new (vec ((functype) (vec i32 i32) (vec i32))));
    let type_section = create_section(Section::Type, &types);

    let func_section = create_section(Section::Func, &wasm!(new (vec 0x0)));

    let export_section = create_section(
        Section::Export,
        &wasm!(new (vec ((str "main") (exporttypefunc) (0x00)))),
    );

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
    wasm!(&mut functions, (vec (data &function)));
    let code_section = create_section(Section::Code, &functions);

    let mut binary = Vec::new();
    // magic module header
    binary.write(b"\0asm").unwrap();
    // module version
    binary.write(&[1, 0, 0, 0]).unwrap();
    binary.write(&type_section).unwrap();
    binary.write(&func_section).unwrap();
    binary.write(&export_section).unwrap();
    binary.write(&code_section).unwrap();

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
