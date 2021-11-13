# chasm-rs

This crate is a single-pass compiler to WebAssembly for the language chasm.

chasm is a very simple language [created by Colin
Eberhardt](https://github.com/ColinEberhardt/chasm), to introduce the basic
building blocks of compilers, and reveal some of the inner workings of
WebAssembly. This is a implementation of the compiler in Rust.

## Usage

Specify the dependency in Cargo.toml:
```toml
[dependencies]
chasm-rs = "0.1.0"
```

And then simply call `chasm_rs::compile` to compile a source code to a
WebAssembly module:

```rust
let source = "
    var y = 0
    while (y < 100)
        var x = 0
        while (x < 100)
            c = ((y/100)*255)
            setpixel (x, y, c)
            x = (x + 1)
        endwhile
        y = (y + 1)
    endwhile";

let wasm = chasm_rs::compile(source);

assert!(wasm.is_ok());
```

## About
### License
Copyright © 2021, [Rodrigodd](https://github.com/Rodrigodd).
Released under the [MIT License](LICENSE).
