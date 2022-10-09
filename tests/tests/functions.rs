use std::error::Error;

use typescript_jit::Runtime;
use typescript_tests::TestLogger;

#[test]
fn run_functions() -> Result<(), Box<dyn Error>> {
    log::set_boxed_logger(TestLogger::new("results/functions.log")).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let rt = Runtime::new();

    rt.load_file(
        "tests/functions.ts",
        Some("results/functions.ir".into())
    )?;

    Ok(())
}