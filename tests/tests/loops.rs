use typescript_jit::{Runtime, Value};
use typescript_tests::TestLogger;

#[test]
fn run_loops() -> Result<(), String> {
    log::set_boxed_logger(TestLogger::new("results/loops.log")).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let rt = Runtime::new();

    let module = rt.load_file(
        "tests/loops.ts",
        Some("results/loops.ir".into())
    ).map_err(|e| e.to_string())?;

    if let Some(n) = module.namespace.variables.get("sum") {
        if let Value::Number(n) = &**n {
            if *n == 3.0 {
                return Ok(())
            }
        }
        Err(format!("expected {:?} to be 3.0", n))
    }
    else {
        Err("expected value 'sum'".into())
    }
}