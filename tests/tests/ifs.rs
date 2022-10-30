use typescript_jit::Runtime;
use typescript_tests::{TestLogger, check};

#[test]
fn run_ifs() -> Result<(), String> {
    log::set_boxed_logger(TestLogger::new("results/ifs.log")).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let rt = Runtime::new();

    let module = rt.load_file(
        "tests/ifs.ts",
        Some("results/ifs.ir".into())
    ).map_err(|e| e.to_string())?;

    check(module, "c", 3.0)
}