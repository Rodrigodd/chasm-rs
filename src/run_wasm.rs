use wasmer::{imports, Function, Instance, Module, Store, Value};

fn dump_hex(data: &[u8]) {
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

pub fn run_binary(binary: &[u8]) -> anyhow::Result<()> {
    dump_hex(&binary);
    let store = Store::default();
    let module = Module::new(&store, &binary)?;
    let import_object = imports! {
        "env" => {
            "print" => Function::new_native(&store, |x: f32| println!("{}", x))
        }
    };
    let instance = Instance::new(&module, &import_object)?;
    let main = instance.exports.get_function("main")?;
    println!("{:?}", main.call(&[Value::I32(8), Value::I32(9)])?);
    Ok(())
}
