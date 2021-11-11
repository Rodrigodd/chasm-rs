use std::io::Write;
use std::sync::{Arc, Mutex};
use wasmer::{imports, Function, Instance, Memory, MemoryType, Module, Store, WasmerEnv};

struct ToWriteFmt<T>(pub T);
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
            let b = art[y * 100 + x];
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
        let binary = match chasm_rs::compile(&code) {
            Ok(it) => it,
            Err(err) => {
                eprintln!("{}", err.to_string());
                std::process::exit(1);
            }
        };
        let out = Arc::new(Mutex::new(ToWriteFmt(std::io::stdout())));
        let art = run_binary(&binary, out)?;

        if args.len() > 2 {
            print_ascii_art(&art);
        } else {
            screen(&art)?;
        }

        return Ok(());
    }

    repl()
}

fn screen(art: &[u8]) -> anyhow::Result<()> {
    use minifb::{Key, Window, WindowOptions};
    const SCALE: usize = 3;
    const WIDTH: usize = 100 * SCALE;
    const HEIGHT: usize = 100 * SCALE;
    let mut window = Window::new("chasm", WIDTH, HEIGHT, WindowOptions::default())?;
    window.limit_update_rate(Some(std::time::Duration::from_micros(16666)));

    let mut buffer = vec![0; WIDTH * HEIGHT];
    for (i, &b) in art.iter().enumerate() {
        let x = SCALE * (i % 100);
        let y = SCALE * (i / 100);
        let c = u32::from_be_bytes([0, b, b, b]);
        for y in y..y + SCALE {
            for x in x..x + SCALE {
                buffer[x + WIDTH * y] = c;
            }
        }
    }
    window.update_with_buffer(&buffer, WIDTH, HEIGHT)?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update();
    }

    Ok(())
}

fn repl() -> anyhow::Result<()> {
    let mut line = String::new();
    loop {
        let mut stdout = std::io::stdout();
        write!(stdout, ">> ").unwrap();
        stdout.flush().unwrap();

        line.clear();
        std::io::stdin().read_line(&mut line).unwrap();
        let binary = match chasm_rs::compile(&line) {
            Ok(x) => x,
            Err(e) => {
                println!("error: {}", e);
                continue;
            }
        };
        let out = Arc::new(Mutex::new(ToWriteFmt(std::io::stdout())));
        run_binary(&binary, out)?;
    }
}

pub fn run_binary<W: std::fmt::Write + Send + 'static>(
    binary: &[u8],
    out: Arc<Mutex<W>>,
) -> anyhow::Result<Vec<u8>> {
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
    data.resize(100 * 100, 0);
    Ok(data)
}
