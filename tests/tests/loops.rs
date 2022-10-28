use typescript_jit::Runtime;
use typescript_tests::{TestLogger, check};

#[test]
fn run_loops() -> Result<(), String> {
    log::set_boxed_logger(TestLogger::new("results/loops.log")).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let rt = Runtime::new();

    let module = rt.load_file(
        "tests/loops.ts",
        Some("results/loops.ir".into())
    ).map_err(|e| e.to_string())?;

    check(module.clone(), "sum", 3.0)?;
    check(module.clone(), "forofSum", 6.0)?;

    Ok(())
}