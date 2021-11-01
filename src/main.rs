use std::io::Write;
use std::sync::{Arc, Mutex};

mod run_wasm;
mod wasm_macro;
use wasm_macro::wasm;

mod compiler;

#[cfg(test)]
mod test;

/// Compile the given chasm source code in a wasm binary.
pub fn compile(source: &str) -> anyhow::Result<Vec<u8>> {
    let mut main_function = Vec::new();
    // code
    compiler::Parser::parse(source, &mut main_function)?;

    wasm!(&mut main_function, (end));

    let binary = wasm!( new
        (magic version)
        (section type (vec
            (functype (vec f32) (vec))
            (functype (vec) (vec))))
        (section import (vec (import "env" "print" function 0x0)))
        (section function (vec 1))
        (section export (vec (export "main" function 0x1)))
        (section code (vec (data &main_function)))
    );

    Ok(binary)
}

fn main() -> anyhow::Result<()> {
    pub struct ToWriteFmt<T>(pub T);

    impl<T> std::fmt::Write for ToWriteFmt<T>
    where
        T: std::io::Write,
    {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            self.0.write_all(s.as_bytes()).map_err(|_| std::fmt::Error)
        }
    }

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
        run_wasm::run_binary(&binary, Arc::new(Mutex::new(ToWriteFmt(std::io::stdout()))))?;
    }
}
