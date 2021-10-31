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

test_output!(
    (print_12, "print 12", "12\n")
    (print_n8, "print -8", "-8\n")
    (mult_print, "print 12 print -8 print 44 print 0.1 print -1e-02", "12\n-8\n44\n0.1\n-0.01\n")
    (print_1p1, "print (1 + 1)", "2\n")
    (print_expr3, "print ((3*2) - (21/7))", "3\n")
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
