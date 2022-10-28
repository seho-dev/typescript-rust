use std::{fs::File, io::Write, sync::Mutex};

use clap::Parser;
use typescript_jit as ts;

struct MyLogger{
    file: Mutex<File>,
}

impl log::Log for MyLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        false
    }

    fn flush(&self) {
        self.file.lock().unwrap().flush().unwrap();
    }

    fn log(&self, record: &log::Record) {
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{:6} - {:25} - {}", record.level(), record.target(), record.args()).unwrap();
    }
}

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    filename: String,
    /// show a execution log. This for debugging.
    #[arg(short, long)]
    log: Option<String>,
    /// shows the LLVM IR code. This for debugging.
    #[arg(short, long)]
    ir: Option<String>,
}

fn main() {
    let args = Args::parse();

    if let Some(log) = args.log {
        log::set_boxed_logger(Box::new(MyLogger {
            file: Mutex::new(File::create(log).unwrap()),
        })).unwrap();
        log::set_max_level(log::LevelFilter::Trace);
        // log::set_max_level(log::LevelFilter::Info);
    }

    let runtime = ts::Runtime::new();

    match runtime.load_file(args.filename, args.ir) {
        Ok(n) => {
            log::info!("an -> {:?}", n.namespace.variables.get("an"));
            log::info!("bu -> {:?}", n.namespace.variables.get("bu"));
        }
        Err(e) => {
            log::error!("{}", e);
        }
    }

}
