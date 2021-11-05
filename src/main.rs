use std::io::Write;
use std::sync::{Arc, Mutex};

mod run_wasm;
mod wasm_macro;
use wasm_macro::wasm;

mod compiler;

#[cfg(test)]
mod test;

pub fn write_section(w: &mut Vec<u8>, section_type: u8, f: impl Fn(&mut Vec<u8>)) {
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

pub struct ToWriteFmt<T>(pub T);
impl<T> std::fmt::Write for ToWriteFmt<T>
where
T: std::io::Write,
{
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| std::fmt::Error)
    }
}

fn print_ascii_art(art: &[u8]) {
    for y in 0..100 {
        for x in 0..100 {
            let b = art[y*100 + x];
            let c = [' ', '-', '=', '#'][(b / 64) as usize];
            print!("{}", c);
        }
        println!();
    }
}

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 1 {
        let code = std::fs::read_to_string(&args[1])?;
        let binary = compile(&code)?;
        let out = Arc::new(Mutex::new(ToWriteFmt(std::io::stdout())));
        let art = run_wasm::run_binary(&binary, out)?;
        print_ascii_art(&art);

        return Ok(());
    }

    repl()
}

fn repl() -> anyhow::Result<()> {
    let mut line = String::new();
    loop {
        let mut stdout = std::io::stdout();
        write!(stdout, ">> ").unwrap();
        stdout.flush().unwrap();

        line.clear();
        std::io::stdin().read_line(&mut line).unwrap();
        let binary = match compile(&line) {
            Ok(x) => x,
            Err(e) => {
                println!("error: {}", e);
                continue;
            }
        };
        let out = Arc::new(Mutex::new(ToWriteFmt(std::io::stdout())));
        run_wasm::run_binary(&binary, out)?;

    }
}
