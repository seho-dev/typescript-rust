use typescript_jit::Runtime;
use typescript_tests::{TestLogger, check};

#[test]
fn run_trycatch() -> Result<(), String> {
    log::set_boxed_logger(TestLogger::new("results/trycatch.log")).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let rt = Runtime::new();

    let module = rt.load_file(
        "tests/trycatch.ts",
        Some("results/trycatch.ir".into())
    ).map_err(|e| e.to_string())?;

    check(module, "nuff", 1.0)?;

    Ok(())
}