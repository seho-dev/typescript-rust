use std::{fs::File, io::Write, sync::Mutex};

use typescript;

struct MyLogger{
    file: Mutex<File>,
}

impl log::Log for MyLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        false
    }

    fn flush(&self) {
        self.file.lock().unwrap().flush().unwrap();
    }

    fn log(&self, record: &log::Record) {
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{:6} - {:20} - {}", record.level(), record.target(), record.args()).unwrap();
    }
}

fn main() {
    log::set_boxed_logger(Box::new(MyLogger {
        file: Mutex::new(File::create("typescript.log").unwrap()),
    })).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    // log::set_max_level(log::LevelFilter::Info);

    let runtime = typescript::jit::Runtime::new();

    match runtime.load_file("samples/sample.ts") {
        Ok(n) => {
            log::info!("an -> {:?}", n.namespace.variables.get("an"));
            log::info!("bu -> {:?}", n.namespace.variables.get("bu"));
        }
        Err(e) => {
            log::error!("{}", e);
        }
    }

}
