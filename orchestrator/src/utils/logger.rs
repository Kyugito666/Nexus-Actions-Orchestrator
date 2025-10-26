// src/utils/logger.rs - Logging utilities

use chrono::Local;
use log::{Record, Level, Metadata};
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct FileLogger {
    log_dir: PathBuf,
    current_file: Mutex<Option<std::fs::File>>,
}

impl FileLogger {
    pub fn new(log_dir: PathBuf) -> Self {
        create_dir_all(&log_dir).ok();
        
        Self {
            log_dir,
            current_file: Mutex::new(None),
        }
    }
    
    fn get_log_file(&self, category: &str) -> PathBuf {
        self.log_dir.join(format!("{}.log", category))
    }
    
    pub fn log_to_file(&self, category: &str, message: &str) {
        let log_path = self.get_log_file(category);
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
        {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(file, "[{}] {}", timestamp, message).ok();
        }
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }
    
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let category = match record.level() {
                Level::Error => "error",
                Level::Warn => "warning",
                _ => "orchestrator",
            };
            
            let message = format!(
                "[{}] {} - {}",
                record.level(),
                record.target(),
                record.args()
            );
            
            self.log_to_file(category, &message);
        }
    }
    
    fn flush(&self) {}
}

pub fn setup_logging(log_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let logger = Box::new(FileLogger::new(log_dir));
    log::set_boxed_logger(logger)?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(())
}
