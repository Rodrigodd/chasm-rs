use super::*;
use core::panic;
use std::sync::{Arc, Mutex};

use crate::Error;
use crate::compiler::Token;

macro_rules! test_output {
	($name:ident, $source:expr, $output:expr) => {
		#[test]
        fn $name() {
            check_output($source, $output)
        }
	};
    ($( ( $($e:tt)* ) )*) => {
        $( test_output!( $($e)* ); )*
    };
}

fn check_output(source: &str, expected: Result<&str, Error>) {
    let binary = compile(source);
    match (expected, binary) {
        (Err(expected), Err(binary)) => assert_eq!(expected, binary),
        (Ok(expected), Ok(binary))  => {
            let out = Arc::new(Mutex::new(String::new()));
            run_wasm::run_binary(&binary, out.clone()).unwrap();
            assert_eq!(*out.lock().unwrap(), expected);
        }
        (expected, binary) => panic!("expected {:?}, received {:?}", expected, binary)
    }
}

#[rustfmt::skip]
test_output!(
    (print_12, "print 12", Ok("12\n"))
    (print_n8, "print -8", Ok("-8\n"))
    (mult_print, "print 12 print -8 print 44 print 0.1 print -1e-02", Ok("12\n-8\n44\n0.1\n-0.01\n"))
    (print_1p1, "print (1 + 1)", Ok("2\n"))
    (print_expr3, "print ((3*2) - (21/7))", Ok("3\n"))
    (print_var_a, "var a = 12 print a", Ok("12\n"))
    (print_var_b, "var b = (46*72) b = (b/46) print b", Ok("72\n"))
    (fibonacci, "
     var a = 0
     var b = 1
     var i = 0
     while (i < 10)
        print a
        b = (a + b)
        a = (b - a)
        i = (i + 1)
     endwhile",
     Ok("0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n"))
    (if_block, "if (1==1) print 1 endif print 2", Ok("1\n2\n"))
    (if_block_no, "if (1==2) print 1 endif print 2", Ok("2\n"))
    (if_else_block, "if (1==1) print 1 else print 3 endif print 2", Ok("1\n2\n"))
    (if_else_block_no, "if (1==2) print 1 else print 3 endif print 2", Ok("3\n2\n"))
    (proc_call, "proc a(x) print x endproc a(10)", Ok("10\n"))
    (proc_call3, "proc func(a,b,c) print (a+(b+c)) endproc func(5,2,7)", Ok("14\n"))
    (proc_call_local, "proc func(a,b,c) x = 14 print ((a+(b+c))/x) endproc a = 5 m = 2 n = 7 func(a,m,n)", Ok("1\n"))
    (recur_call, "
     proc A () B() endproc
     proc B () print 5 endproc
     A()",
     Ok("5\n"))
    (recur_call3, "
     proc A (x) B(x, 2) endproc
     proc B (x, y) C(x, y, 4) endproc
     proc C (x, y, z) print ((x+y)+z) endproc
     A(1)",
     Ok("7\n"))
    (setpixel_side_effect, "print 0 setpixel(0, 1, 2) print x print y print color", Ok("0\n0\n1\n2\n"))
    (print_print, 
        "print print",
        Err(Error::UnexpectedToken {
            received: (Token::Print, 6..11),
            expected: &[Token::Number, Token::LeftParen]
        })
    )
);

#[test]
fn mandelbrot() -> Result<(), Error> {
    let source = "
var y  = 0
while (y < 100)
  y = (y + 1)
  var x  = 0
  while (x < 100)
    x = (x + 1)

    var e = ((y / 50) - 1.5)
    var f = ((x / 50) - 1)

    var a = 0
    var b = 0
    var i = 0
    var j = 0
    var c = 0

    while ((((i * i) + (j * j)) < 4) && (c < 255))
      i = (((a * a) - (b * b)) + e)
      j = (((2 * a) * b) + f)
      a = i
      b = j
      c = (c + 1)
    endwhile
    setpixel (x, y, c)
  endwhile
endwhile";
    let binary = compile(source)?;
    let out = Arc::new(Mutex::new(String::new()));
    let output = run_wasm::run_binary(&binary, out.clone()).unwrap();

    let hash = blake3::hash(&output);
    assert_eq!(&hash.to_hex()[0..16], "28ad088dd153090f");

    Ok(())
}

