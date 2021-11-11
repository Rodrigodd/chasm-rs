# chasm

## A very simple compile-to-WebAssembly language

You can [play with chasm online](https://rodrigodd.github.io/chasm-rs/).

This is a rewrite in Rust of the compiler for the language chasm. chasm is a very simple programming language with a two-fold purpose:

1. Introduce the basic building blocks of compilers - and show that they aren't that scary or difficult!
2. Reveal some of the inner workings of WebAssembly.

chasm was first created by Colin Eberhardt to accompany a talk at FullStack Conference NYC. You can see the original implementation of the compiler [here](https://github.com/ColinEberhardt/chasm). My implementation loosely follow his [blog post](https://blog.scottlogic.com/2019/05/17/webassembly-compiler.html). His implementation uses an AST, but mine is a single-pass compiler.

## Example

Below is an example of a Mandelbrot fractal renderer. Other examples can be found in the `./examples` folder. You can also test these examples in the [online demo](https://rodrigodd.github.io/chasm-rs/).

``` php
var y = 0
while (y < 100)
  var x  = 0
  while (x < 100)
    var cr = ((y / 50) - 1.5)
    var ci = ((x / 50) - 1)

    var i = 0
    var j = 0
    var iter = 0

    while ((((i * i) + (j * j)) < 4) && (iter < 255))
      var ni = (((i * i) - (j * j)) + cr)
      j = (((2 * i) * j) + ci)
      i = ni
      iter = (iter + 1)
    endwhile
    setpixel (x, y, iter)
    x = (x + 1)
  endwhile
  y = (y + 1)
endwhile
```

## Build and Run

### CLI

You can pass a file as an argument to the CLI to run and render chasm code in the file. For example, to render the Julia set in `./examples/julia.chasm`, you can run the command (assuming you are in the repo root):

``` console
cargo run -- examples/julia.chasm
```

If you pass any second argument it will render in the terminal as ASCII, and if you pass no argument it will run a bad REPL.

### WebAssembly

To build the compiler for WebAssembly, you need to use [wasm-pack](https://github.com/rustwasm/wasm-pack):
``` console
cd chasm-wasm
wasm-pack build --release --target web
```
This will output a folder `./chasm-wasm/pkg` with the files `chasm_wasm.js` and `chasm_wasm_bg.wasm` (and some others). You can then move these two files to `./docs` to test them on the web page `./docs/index.html`.
