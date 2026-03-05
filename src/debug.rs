use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::{Mutex, OnceLock};

static LOGGER: OnceLock<Mutex<BufWriter<std::fs::File>>> = OnceLock::new();

fn init() {
    LOGGER.get_or_init(|| {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open("search_debug.log")
            .unwrap();

        Mutex::new(BufWriter::new(file))
    });
}

pub fn debug_log(depth: usize, message: &str) {
    init();

    let indent = "\t".repeat(depth);
    let line = format!("{}{}\n", indent, message);

    let logger = LOGGER.get().unwrap();
    let mut writer = logger.lock().unwrap();

    let _ = writer.write_all(line.as_bytes());
    let _ = writer.flush();
}
