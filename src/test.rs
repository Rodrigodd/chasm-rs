use super::*;

macro_rules! test_output {
	($name:ident, $source:expr, $output:expr) => {
		#[test]
        fn $name() -> anyhow::Result<()> {
            check_output($source, $output)
        }
	};
    ($( ( $($e:tt)* ) )*) => {
        $( test_output!( $($e)* ); )*
    };
}

#[rustfmt::skip]
test_output!(
    (print_12, "print 12", "12\n")
    (print_n8, "print -8", "-8\n")
    (mult_print, "print 12 print -8 print 44 print 0.1 print -1e-02", "12\n-8\n44\n0.1\n-0.01\n")
    (print_1p1, "print (1 + 1)", "2\n")
    (print_expr3, "print ((3*2) - (21/7))", "3\n")
    (print_var_a, "var a = 12 print a", "12\n")
    (print_var_b, "var b = (46*72) b = (b/46) print b", "72\n")
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
     "0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n")
    (if_block, "if (1==1) print 1 endif print 2", "1\n2\n")
    (if_block_no, "if (1==2) print 1 endif print 2", "2\n")
    (if_else_block, "if (1==1) print 1 else print 3 endif print 2", "1\n2\n")
    (if_else_block_no, "if (1==2) print 1 else print 3 endif print 2", "3\n2\n")
    (proc_call, "proc a(x) print x endproc a(10)", "10\n")
    (proc_call3, "proc func(a,b,c) print (a+(b+c)) endproc func(5,2,7)", "14\n")
    (proc_call_local, "proc func(a,b,c) x = 14 print ((a+(b+c))/x) endproc a = 5 m = 2 n = 7 func(a,m,n)", "1\n")
    (recur_call, "
     proc A () B() endproc
     proc B () print 5 endproc
     A()",
     "5\n")
    (recur_call3, "
     proc A (x) B(x, 2) endproc
     proc B (x, y) C(x, y, 4) endproc
     proc C (x, y, z) print ((x+y)+z) endproc
     A(1)",
     "7\n")
);

#[test]
#[should_panic]
fn print_print() {
    check_output("print print", "").unwrap()
}

fn check_output(source: &str, expected: &str) -> anyhow::Result<()> {
    let binary = compile(source)?;
    let out = Arc::new(Mutex::new(String::new()));
    run_wasm::run_binary(&binary, out.clone())?;
    assert_eq!(*out.lock().unwrap(), expected);
    Ok(())
}
