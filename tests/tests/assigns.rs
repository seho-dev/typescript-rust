use std::{error::Error, sync::Arc};

use typescript_jit::{Module, Runtime, Value};
use typescript_tests::TestLogger;

fn check(module: Arc<Module>, name: &str, goal: f64) -> Result<(), String> {
    match module.namespace.variables.get(name) {
        Some(var) => {
            match &**var {
                Value::Number(n) => {
                    if *n == goal {
                        Ok(())
                    }
                    else {
                        Err(format!("number {} != {}", n, goal))
                    }
                }
                _ => {
                    Err(format!("expected number but got: {:?}", var))
                }
            }
        }
        None => {
            Err("expected variable 'c'".into())
        }
    }
}

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