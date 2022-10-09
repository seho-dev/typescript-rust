use std::error::Error;

use typescript_jit::{Runtime, Value};
use typescript_tests::TestLogger;

#[test]
fn run_switches() -> Result<(), String> {
    log::set_boxed_logger(TestLogger::new("results/switches.log")).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let rt = Runtime::new();

    let module = rt.load_file(
        "tests/switches.ts",
        Some("results/switches.ir".into())
    ).map_err(|e| e.to_string())?;

    match module.namespace.variables.get("choice") {
        Some(var) => {

            match &**var {
                Value::Number(n) => {
                    if *n == 2.0 {
                        Ok(())
                    }
                    else {
                        Err(format!("number {} != 2", n))
                    }
                }
                _ => {
                    Err(format!("expected number but got: {:?}", var))
                }
            }
        }
        None => {
            Err("expected variable 'choice'".into())
        }
    }
}