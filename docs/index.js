import init, { compile } from './chasm_wasm.js';

const codeArea = document.getElementById("code");
const canvas = document.getElementById("canvas");
const compileButton = document.getElementById("compile");
const outputArea = document.getElementById("output");

let keywords = [ "print", "var", "while", "endwhile", "if", "endif", "else", "proc", "endproc" ];
CodeMirror.defineSimpleMode("simplemode", {
    start: [
        {
            regex: new RegExp(`(${keywords.join("|")})`),
            token: "keyword"
        },
        {
            regex: /0x[a-f\d]+|[-+]?(?:\.\d+|\d+\.?\d*)(?:e[-+]?\d+)?/i,
            token: "number"
        },
        { regex: /[-+\/*=<>!]+/, token: "operator" },
        { regex: /[a-z$][\w$]*/, token: "variable" }
    ]
});
const editor = CodeMirror.fromTextArea(codeArea, {
    mode: "simplemode",
    theme: "abcdef",
    lineNumbers: true
});

const WIDTH = 100;
const HEIGHT = 100;

const ctx = canvas.getContext("2d");
ctx.fillStyle = "black";
ctx.fillRect(0, 0, WIDTH, HEIGHT);

// input: h in [0,360] and s,v in [0,1] - output: r,g,b in [0,1]
function hsl2rgb(h,s,l) 
{
  let a=s*Math.min(l,1-l);
  let f= (n,k=(n+h/30)%12) => l - a*Math.max(Math.min(k-3,9-k,1),-1);                 
  return [f(0),f(8),f(4)];
}
window.hsl = hsl2rgb;

function grad(c) {
    // let [r,g,b] = hsl2rgb(c/255*360,1,Math.min(0.5, (1 - (1 - c/255)**2)) + c/510);
    // return [r*255,g*255,b*255];
    return [c,c,c];
}

function updateCanvas(buffer) {
    const imageData = ctx.createImageData(WIDTH, HEIGHT);
    for (let i = 0; i < WIDTH * HEIGHT * 4; i+=4) {
        let c = buffer[i/4];
        let [r,g,b] = grad(c);
        imageData.data[i+0] = r;
        imageData.data[i+1] = g;
        imageData.data[i+2] = b;
        imageData.data[i+3] = 255;
    }
    ctx.putImageData(imageData, 0, 0);
};

let marker;
const logMessage = (message) => (outputArea.value = outputArea.value + message + "\n");
const markError = (token) => {
    marker = editor.markText({ line: token.line, ch: token.char }, { line: token.line, ch: token.char + token.value.length }, { className: "error" });
    console.log(marker);
};

async function runCode() {
    const source = editor.getValue();

    if (marker) {
        marker.clear();
    }

    try {
        const bin = compile(source);

        let memory = new WebAssembly.Memory({initial:1});
        let wasm = await WebAssembly.instantiate(bin, {
            env: {
                print: logMessage,
                memory: memory,
            }
        })

        logMessage(`Executing ... `);
        wasm.instance.exports.main();
        const buffer = new Uint8Array(memory.buffer);
        updateCanvas(buffer);
    }
    catch (e) {
        // logMessage(error.message);
        // markError(error.token);
        console.log(e);
        const error = JSON.parse(e);
        console.log(error);
        logMessage(error.message);
        markError(error.token);
    }
}
window.runCode = runCode;

async function run() {
    await init();


    await runCode();
}
run();

compileButton.addEventListener("click", async () => {
    // compileButton.classList.add("active");
    // interpretButton.classList.remove("active");
    // await run(compilerRuntime);
    runCode();
});
