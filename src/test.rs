use super::*;

#[test]
fn print() -> anyhow::Result<()> {
    let binary = compile("print 12")?;
    let out = Arc::new(Mutex::new(String::new()));
    run_wasm::run_binary(&binary, out.clone())?;
    assert_eq!(*out.lock().unwrap(), "12\n");
    Ok(())
}

