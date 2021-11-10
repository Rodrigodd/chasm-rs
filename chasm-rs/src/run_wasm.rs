use core::panic;
use std::fmt::Write;
use std::sync::{Arc, Mutex};

use wasmi::memory_units::Pages;
use wasmi::nan_preserving_float::F32;
use wasmi::{
    Error as InterpreterError, Externals, FuncInstance, ImportsBuilder, MemoryInstance, Module,
    ModuleImportResolver, ModuleInstance, RuntimeValue, Signature, ValueType,
};

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

struct EnvModuleResolver(wasmi::MemoryRef);
impl ModuleImportResolver for EnvModuleResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &wasmi::Signature,
    ) -> Result<wasmi::FuncRef, wasmi::Error> {
        let func = match field_name {
            "print" => FuncInstance::alloc_host(Signature::new(&[ValueType::F32][..], None), 0),
            _ => {
                return Err(InterpreterError::Function(format!(
                    "host module doesn't export function with name {}",
                    field_name
                )));
            }
        };
        Ok(func)
    }

    fn resolve_memory(
        &self,
        field_name: &str,
        _memory_type: &wasmi::MemoryDescriptor,
    ) -> Result<wasmi::MemoryRef, InterpreterError> {
        let mem = match field_name {
            "memory" => self.0.clone(),
            _ => panic!("HAHAHAH!!"),
        };
        Ok(mem)
    }
}

struct Runtime<W: Write>(Arc<Mutex<W>>);
impl<W: Write> Externals for Runtime<W> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: wasmi::RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, wasmi::Trap> {
        match index {
            0 => {
                let n: f32 = args.nth::<F32>(0).into();
                writeln!(self.0.lock().unwrap(), "{}", n).unwrap();
            }
            _ => panic!("HAHAHAH!!!"),
        };
        Ok(None)
    }
}

pub fn run_binary<W: Write + Send + 'static>(
    binary: &[u8],
    out: Arc<Mutex<W>>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    dump_hex(&binary);

    let mut runtime = Runtime(out);

    let module = Module::from_buffer(&binary)?;
    // let memory = Memory::new(&store, MemoryType::new(1, None, false)).unwrap();
    // let import_object = imports! {
    //     "env" => {
    //         "print" => Function::new_native_with_env(&store, writer, |out: &Writer<W>, x: f32| writeln!(&mut *out.w.lock().unwrap(), "{}", x)),
    //         "memory" => memory.clone(),
    //     }
    // };
    let memory = MemoryInstance::alloc(Pages(1), None).unwrap();
    let resolver = &EnvModuleResolver(memory.clone());
    let import_object = ImportsBuilder::default().with_resolver("env", resolver);
    let instance = ModuleInstance::new(&module, &import_object)?.assert_no_start();
    instance.invoke_export("main", &[], &mut runtime)?;
    let mut data = memory.direct_access().as_ref().to_owned();
    data.resize(100 * 100, 0);
    Ok(data)
}
