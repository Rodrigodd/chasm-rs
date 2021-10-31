/// A macro for writing webassembly in binary format.
macro_rules! wasm {
    // create a new Vector<u8>
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
    // write the b"\0asm" magic header, and the version 0x0100_0000
    ($w:expr, magic version) => {
        ($w).write(b"\0asm").unwrap();
        ($w).write(&[0x01, 0x00, 0x00, 0x00]).unwrap();
    };
    ($w:expr, str $e:literal) => {
        {
            let data = ($e).as_bytes();
            leb128::write::unsigned($w, data.len() as u64).unwrap();
            ($w).write(data).unwrap();
        }
    };
    ($w:expr, $e:literal) => {
        leb128::write::unsigned($w, $e).unwrap();
    };
    // write a u8 slice, but prepend its lenght first
    ($w:expr, data $e:expr) => {
        leb128::write::unsigned($w, ($e).len() as u64).unwrap();
        ($w).write($e).unwrap();
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
            ($w).write(&vector).unwrap();
            drop(&mut vector);
            drop(&mut n);
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

    // call instruction
    ($w:expr, call $funcidx:expr) => {
        ($w).write(&[0x10]).unwrap();
        leb128::write::unsigned($w, $funcidx).unwrap();
    };
    // f32.const instruction
    ($w:expr, f32.const $z:expr) => {
        ($w).write(&[0x43]).unwrap();
        ($w).write(&($z).to_le_bytes()).unwrap();
    };
    // get_local instruction
    ($w:expr, get_local $e:literal) => {
        ($w).write(&[0x20]).unwrap();
        leb128::write::unsigned($w, $e).unwrap();
    };
    // i32.add instruction
    ($w:expr, i32.add) => {
        { ($w).write(&[0x6a]).unwrap(); }
    };
    ($w:expr, f32.add) => {
        { ($w).write(&[0x92]).unwrap(); }
    };
    ($w:expr, f32.sub) => {
        { ($w).write(&[0x93]).unwrap(); }
    };
    ($w:expr, f32.mul) => {
        { ($w).write(&[0x94]).unwrap(); }
    };
    ($w:expr, f32.div) => {
        { ($w).write(&[0x95]).unwrap(); }
    };
    ($w:expr, f32.eq ) => {
        { ($w).write(&[0x5b]).unwrap(); }
    };
    ($w:expr, f32.lt ) => {
        { ($w).write(&[0x5d]).unwrap(); }
    };
    ($w:expr, f32.gt ) => {
        { ($w).write(&[0x5e]).unwrap(); }
    };
    ($w:expr, i32.and) => {
        { ($w).write(&[0x71]).unwrap(); }
    };
    // creates a section
    // https://webassembly.github.io/spec/core/binary/modules.html#binary-section
    ($w:expr, section $id:tt $e:tt) => {
        ($w).write(&[wasm!(section_type $id)]).unwrap();
        let mut section = Vec::new();
        wasm!(&mut section, $e);
        leb128::write::unsigned($w, section.len() as u64).unwrap();
        ($w).write(&section).unwrap();
    };
    // create a functype, is used in the type section
    // https://webassembly.github.io/spec/core/binary/types.html#binary-functype
    ($w:expr, functype $param:tt $result:tt) => {
        ($w).write(&[0x60]).unwrap();
        wasm!($w, $param);
        wasm!($w, $result);
    };

    // creates a export, in used in the export section
    // https://webassembly.github.io/spec/core/binary/modules.html#binary-export
    ($w:expr, export $name:literal $id:tt $idx:tt) => {
        {
            let name = ($name).as_bytes();
            leb128::write::unsigned($w, name.len() as u64).unwrap();
            ($w).write(name).unwrap();
        }
        ($w).write(&[wasm!(export_type $id)]).unwrap();
        leb128::write::unsigned($w, $idx as u64).unwrap();
    };

    // creates a import, in used in the import section
    // https://webassembly.github.io/spec/core/binary/modules.html#binary-import
    ($w:expr, import $mod:literal $name:literal $id:tt $idx:tt) => {
        {
            let module = ($mod).as_bytes();
            leb128::write::unsigned($w, module.len() as u64).unwrap();
            ($w).write(module).unwrap();
        }
        {
            let name = ($name).as_bytes();
            leb128::write::unsigned($w, name.len() as u64).unwrap();
            ($w).write(name).unwrap();
        }
        ($w).write(&[wasm!(export_type $id)]).unwrap();
        leb128::write::unsigned($w, $idx as u64).unwrap();
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
