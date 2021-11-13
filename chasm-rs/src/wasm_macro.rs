/// A macro for writing webassembly in binary format.
macro_rules! wasm {
    // create a new Vector<u8>
    (new $($e:tt)*) => {
        {
            let mut vec = Vec::new();
            wasm!(&mut vec, $($e)*);
            vec
        }
    };
    ($w:expr, $( ( $($e:tt)* ) )*) => {
        $( wasm!( $w, $($e)* ); )*
    };
    // write the b"\0asm" magic header, and the version 0x0100_0000
    ($w:expr, magic version) => {
        ($w).write_all(b"\0asm").unwrap();
        ($w).write_all(&[0x01, 0x00, 0x00, 0x00]).unwrap();
    };
    ($w:expr, str $e:literal) => {
        {
            let data = ($e).as_bytes();
            leb128::write::unsigned($w, data.len() as u64).unwrap();
            ($w).write_all(data).unwrap();
        }
    };
    ($w:expr, $e:literal) => {
        leb128::write::unsigned($w, $e).unwrap();
    };
    // write a u8 slice, but prepend its lenght first
    ($w:expr, data $e:expr) => {
        leb128::write::unsigned($w, ($e).len() as u64).unwrap();
        ($w).write_all($e).unwrap();
    };
    // write each element, but prepend the number of elements first
    ($w:expr, vec $($e:tt)*) => {
        {
            let mut vector = Vec::<u8>::new();
            let mut n = 0;
            $(
                wasm!( &mut vector, $e );
                n += 1;
            )*
            leb128::write::unsigned($w, n).unwrap();
            ($w).write_all(&vector).unwrap();
            drop(&mut vector);
            drop(&mut n);
        }
    };
    ($w:expr, end) => {
        ($w).write_all(&[0x0b]).unwrap();
    };
    ($w:expr, functype) => {
        ($w).write_all(&[0x60]).unwrap();
    };
    ($w:expr, i32) => {
        ($w).write_all(&[0x7f]).unwrap();
    };
    ($w:expr, f32) => {
        ($w).write_all(&[0x7d]).unwrap();
    };
    ($w:expr, exporttypefunc) => {
        ($w).write_all(&[0x00]).unwrap();
    };

    // call instruction
    ($w:expr, call $funcidx:expr) => {
        ($w).write_all(&[0x10]).unwrap();
        leb128::write::unsigned($w, ($funcidx) as u64).unwrap();
    };
    // f32.const instruction
    ($w:expr, f32.const $z:expr) => {
        ($w).write_all(&[0x43]).unwrap();
        ($w).write_all(&(($z) as f32).to_le_bytes()).unwrap();
    };
    // local.get instruction
    ($w:expr, local.get $e:expr) => {
        ($w).write_all(&[0x20]).unwrap();
        leb128::write::unsigned($w, ($e) as u64).unwrap();
    };
    // local.set instruction
    ($w:expr, local.set $e:expr) => {
        ($w).write_all(&[0x21]).unwrap();
        leb128::write::unsigned($w, ($e) as u64).unwrap();
    };
    // i32.add instruction
    ($w:expr, i32.add) => {
        { ($w).write_all(&[0x6a]).unwrap(); }
    };
    ($w:expr, i32.eqz) => {
        { ($w).write_all(&[0x45]).unwrap(); }
    };
    ($w:expr, i32.store8 $aling:literal $offset:literal) => {
        {
            ($w).write_all(&[0x3a]).unwrap();
            leb128::write::unsigned($w, ($aling) as u64).unwrap();
            leb128::write::unsigned($w, ($offset) as u64).unwrap();
        }
    };
    ($w:expr, i32.trunc_f32_s) => {
        { ($w).write_all(&[0xa8]).unwrap(); }
    };
    ($w:expr, f32.add) => {
        { ($w).write_all(&[0x92]).unwrap(); }
    };
    ($w:expr, f32.sub) => {
        { ($w).write_all(&[0x93]).unwrap(); }
    };
    ($w:expr, f32.mul) => {
        { ($w).write_all(&[0x94]).unwrap(); }
    };
    ($w:expr, f32.div) => {
        { ($w).write_all(&[0x95]).unwrap(); }
    };
    ($w:expr, f32.eq ) => {
        { ($w).write_all(&[0x5b]).unwrap(); }
    };
    ($w:expr, f32.lt ) => {
        { ($w).write_all(&[0x5d]).unwrap(); }
    };
    ($w:expr, f32.gt ) => {
        { ($w).write_all(&[0x5e]).unwrap(); }
    };
    ($w:expr, i32.and) => {
        { ($w).write_all(&[0x71]).unwrap(); }
    };


    ($w:expr, br $label:expr) => {
        {
            ($w).write_all(&[0x0C]).unwrap();
            leb128::write::unsigned($w, ($label) as u64).unwrap();
        }
    };
    ($w:expr, br_if $label:expr) => {
        {
            ($w).write_all(&[0x0D]).unwrap();
            leb128::write::unsigned($w, ($label) as u64).unwrap();
        }
    };

    // A block with a empty return type
    ($w:expr, block) => {
        ($w).write_all(&[0x02, 0x40]).unwrap();
    };
    // A if block with a empty return type
    ($w:expr, if) => {
        ($w).write_all(&[0x04, 0x40]).unwrap();
    };
    // A else block
    ($w:expr, else) => {
        ($w).write_all(&[0x05]).unwrap();
    };
    // A loop with a empty return type
    ($w:expr, loop) => {
        ($w).write_all(&[0x03, 0x40]).unwrap();
    };


    // creates a section
    // https://webassembly.github.io/spec/core/binary/modules.html#binary-section
    ($w:expr, section $id:tt $e:tt) => {
        ($w).write_all(&[wasm!(section_type $id)]).unwrap();
        let mut section = Vec::new();
        wasm!(&mut section, $e);
        leb128::write::unsigned($w, section.len() as u64).unwrap();
        ($w).write_all(&section).unwrap();
    };
    // create a functype, is used in the type section
    // https://webassembly.github.io/spec/core/binary/types.html#binary-functype
    ($w:expr, functype $param:tt $result:tt) => {
        ($w).write_all(&[0x60]).unwrap();
        wasm!($w, $param);
        wasm!($w, $result);
    };

    // creates a export, in used in the export section
    // https://webassembly.github.io/spec/core/binary/modules.html#binary-export
    ($w:expr, export $name:literal $id:tt $idx:tt) => {
        {
            let name = ($name).as_bytes();
            leb128::write::unsigned($w, name.len() as u64).unwrap();
            ($w).write_all(name).unwrap();
        }
        ($w).write_all(&[wasm!(export_type $id)]).unwrap();
        leb128::write::unsigned($w, $idx as u64).unwrap();
    };

    // creates a import, in used in the import section
    // https://webassembly.github.io/spec/core/binary/modules.html#binary-import
    ($w:expr, import $mod:literal $name:literal $desc:tt) => {
        {
            let module = ($mod).as_bytes();
            leb128::write::unsigned($w, module.len() as u64).unwrap();
            ($w).write_all(module).unwrap();
        }
        {
            let name = ($name).as_bytes();
            leb128::write::unsigned($w, name.len() as u64).unwrap();
            ($w).write_all(name).unwrap();
        }
        wasm!($w, import_desc $desc)
    };
    // import description of a function
    // https://webassembly.github.io/spec/core/binary/modules.html#binary-importdesc
    ($w:expr, import_desc (function $idx:expr)) => {
        ($w).write_all(&[0x00]).unwrap();
        leb128::write::unsigned($w, $idx as u64).unwrap();
    };
    // import description of a memory
    // https://webassembly.github.io/spec/core/binary/modules.html#binary-importdesc
    ($w:expr, import_desc (memory $min:literal $max:literal)) => {
        ($w).write_all(&[0x02]).unwrap();
        ($w).write_all(&[0x01]).unwrap();
        leb128::write::unsigned($w, $min as u64).unwrap();
        leb128::write::unsigned($w, $max as u64).unwrap();
    };
    ($w:expr, import_desc (memory $min:literal)) => {
        ($w).write_all(&[0x02]).unwrap();
        ($w).write_all(&[0x00]).unwrap();
        leb128::write::unsigned($w, $min as u64).unwrap();
    };

    (section_type type) => { 1 };
    (section_type import) => { 2 };
    (section_type function) => { 3 };
    (section_type export) => { 7 };
    (section_type code) => { 10 };

    (export_type function) => { 0x00 };
    (export_type table) => { 0x01 };
    (export_type memory) => { 0x02 };
    (export_type global) => { 0x03 };
}

pub(crate) use wasm;
