use std::error::Error;

use typescript_jit::{Runtime, Value};
use typescript_tests::TestLogger;

#[test]
fn run_ifs() -> Result<(), String> {
    log::set_boxed_logger(TestLogger::new("results/ifs.log")).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let rt = Runtime::new();

    let module = rt.load_file(
        "tests/ifs.ts",
        Some("results/ifs.ir".into())
    ).map_err(|e| e.to_string())?;

    match module.namespace.variables.get("c") {
        Some(var) => {

            match &**var {
                Value::Number(n) => {
                    if *n == 3.0 {
                        Ok(())
                    }
                    else {
                        Err(format!("number {} != 3", n))
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