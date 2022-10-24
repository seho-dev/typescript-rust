use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use typescript_jit::{Module, Value};

pub fn check(module: Arc<Module>, name: &str, goal: f64) -> Result<(), String> {
    match module.namespace.variables.get(name) {
        Some(var) => match &**var {
            Value::Number(n) => {
                if *n == goal {
                    Ok(())
                } else {
                    Err(format!(
                        "number in '{}' not as expected {} != {}",
                        name, n, goal
                    ))
                }
            }
            _ => Err(format!("expected number in '{}' but got: {:?}", name, var)),
        },
        None => Err(format!("expected variable '{}'", name)),
    }
}

pub struct TestLogger {
    file: Mutex<File>,
}

impl TestLogger {
    pub fn new(name: &str) -> Box<Self> {
        let mut path = PathBuf::new();
        path.push(name);

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
        }

        Box::new(Self {
            file: Mutex::new(File::create(path).unwrap()),
        })
    }
}

impl log::Log for TestLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn flush(&self) {
        self.file.lock().unwrap().flush().unwrap();
    }

    fn log(&self, record: &log::Record) {
        let mut file = self.file.lock().unwrap();
        writeln!(
            file,
            "{:6} - {:25} - {}",
            record.level(),
            record.target(),
            record.args()
        )
        .unwrap();
    }
}
