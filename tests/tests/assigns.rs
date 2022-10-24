use std::{error::Error, sync::Arc};

use typescript_jit::{Module, Runtime, Value};
use typescript_tests::{TestLogger, check};

#[test]
fn run_assigns() -> Result<(), String> {
    log::set_boxed_logger(TestLogger::new("results/assigns.log")).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let rt = Runtime::new();

    let module = rt.load_file(
        "tests/assigns.ts",
        Some("results/assigns.ir".into())
    ).map_err(|e| e.to_string())?;

    check(module.clone(), "a", 2.0)?;
    check(module.clone(), "b", -2.0)?;
    check(module.clone(), "c", 2.0)?;
    check(module.clone(), "d", 0.5)?;

    Ok(())
}