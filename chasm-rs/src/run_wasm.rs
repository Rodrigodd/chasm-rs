use std::fmt::Write;
use std::sync::{Arc, Mutex};

use wasmer::{Function, Instance, Memory, MemoryType, Module, Store, imports};

pub fn dump_hex(data: &[u8]) {
    let mut bytes = data;
    let mut offset = 0;
    loop {
        const D: usize = 16;
        print!("{:04x}: ", offset);
        for i in 0..D {
            if i < bytes.len() {
                print!("{:02x} ", bytes[i]);
            } else {
                print!("   ");
            }
        }
        print!(": ");
        for &b in bytes.iter().take(D) {
            let c = b as char;
            print!(
                "{}",
                if c.is_ascii_graphic() || c == ' ' {
                    c
                } else {
                    '.'
                }
            );
        }
        println!();
        if bytes.len() < D {
            return;
        }
        bytes = &bytes[D..];
        offset += D;
    }
}
use wasmer::WasmerEnv;

pub fn run_binary<W: Write + Send + 'static>(
    binary: &[u8],
    out: Arc<Mutex<W>>,
) -> anyhow::Result<Vec<u8>> {
    dump_hex(&binary);

    struct Writer<W: Send> {
        w: Arc<Mutex<W>>,
    }
    impl<W: Send> WasmerEnv for Writer<W> {}
    impl<W: Send> Clone for Writer<W> {
        fn clone(&self) -> Self {
            Self { w: self.w.clone() }
        }
    }

    let writer = Writer { w: out };

    let store = Store::default();
    let module = Module::new(&store, &binary)?;
    let memory = Memory::new(&store, MemoryType::new(1, None, false)).unwrap();
    let import_object = imports! {
        "env" => {
            "print" => Function::new_native_with_env(&store, writer, |out: &Writer<W>, x: f32| writeln!(&mut *out.w.lock().unwrap(), "{}", x)),
            "memory" => memory.clone(),
        }
    };
    let instance = Instance::new(&module, &import_object)?;
    let main = instance.exports.get_function("main")?;
    main.call(&[])?;
    let mut data = unsafe { memory.data_unchecked() }.to_owned();
    data.resize(100*100, 0);
    Ok(data)
}
