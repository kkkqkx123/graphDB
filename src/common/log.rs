use chrono::{DateTime, Local};
use std::collections::VecDeque;
use std::fmt;
use std::fmt::Write;
use std::fs::{File, OpenOptions};
use std::io::Write as IoWrite;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

/// Log level definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

impl LogLevel {
    /// Check if this log level should be displayed given a minimum level
    pub fn is_enabled(&self, min_level: LogLevel) -> bool {
        *self >= min_level
    }
}

/// Log entry structure
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub target: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub message: String,
    pub module_path: Option<String>,
}

impl LogEntry {
    pub fn new(
        level: LogLevel,
        target: String,
        file: Option<&str>,
        line: Option<u32>,
        message: String,
        module_path: Option<&str>,
    ) -> Self {
        Self {
            timestamp: Local::now(),
            level,
            target,
            file: file.map(|s| s.to_string()),
            line,
            message,
            module_path: module_path.map(|s| s.to_string()),
        }
    }
}

/// Log writer trait for different output destinations
pub trait LogWriter: Send + Sync {
    fn write(&self, entry: &LogEntry) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Console log writer
pub struct ConsoleWriter {
    min_level: LogLevel,
    show_level: bool,
    show_target: bool,
    show_location: bool,
}

impl ConsoleWriter {
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            min_level,
            show_level: true,
            show_target: true,
            show_location: true,
        }
    }

    pub fn with_level(mut self, min_level: LogLevel) -> Self {
        self.min_level = min_level;
        self
    }

    pub fn with_show_level(mut self, show: bool) -> Self {
        self.show_level = show;
        self
    }

    pub fn with_show_target(mut self, show: bool) -> Self {
        self.show_target = show;
        self
    }

    pub fn with_show_location(mut self, show: bool) -> Self {
        self.show_location = show;
        self
    }
}

impl LogWriter for ConsoleWriter {
    fn write(&self, entry: &LogEntry) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !entry.level.is_enabled(self.min_level) {
            return Ok(());
        }

        let mut output = String::new();

        if self.show_level {
            write!(output, "[{}] ", entry.level)?;
        }

        write!(
            output,
            "{} ",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f")
        )?;

        if self.show_target {
            write!(output, "[{}] ", entry.target)?;
        }

        if let (Some(file), Some(line)) = (&entry.file, entry.line) {
            if self.show_location {
                write!(output, "[{}:{}]", file, line)?;
            }
        }

        write!(output, " - {}", entry.message)?;

        println!("{}", output);
        Ok(())
    }

    fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Console output is typically unbuffered in Rust, so just return Ok
        Ok(())
    }
}

/// File log writer
pub struct FileWriter {
    file: Arc<Mutex<File>>,
    min_level: LogLevel,
}

impl FileWriter {
    pub fn new<P: AsRef<Path>>(path: P, min_level: LogLevel) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)?;

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            min_level,
        })
    }
}

impl LogWriter for FileWriter {
    fn write(&self, entry: &LogEntry) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !entry.level.is_enabled(self.min_level) {
            return Ok(());
        }

        let mut file = self
            .file
            .lock()
            .expect("File writer lock should not be poisoned");

        writeln!(
            file,
            "[{}] {} [{}] [{}:{}] - {}",
            entry.level,
            entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            entry.target,
            match &entry.file {
                Some(f) => f.as_str(),
                None => "<unnamed>",
            },
            match entry.line {
                Some(l) => l,
                None => 0,
            },
            entry.message
        )?;

        Ok(())
    }

    fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut file = self
            .file
            .lock()
            .expect("File writer lock should not be poisoned");
        file.flush()?;
        Ok(())
    }
}

/// Memory log writer for testing
pub struct MemoryWriter {
    entries: Arc<Mutex<VecDeque<LogEntry>>>,
    max_entries: usize,
    min_level: LogLevel,
}

impl MemoryWriter {
    pub fn new(max_entries: usize, min_level: LogLevel) -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::with_capacity(max_entries))),
            max_entries,
            min_level,
        }
    }

    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.entries
            .lock()
            .expect("Memory writer entries lock should not be poisoned")
            .clone()
            .into()
    }

    pub fn clear(&self) {
        self.entries
            .lock()
            .expect("Memory writer entries lock should not be poisoned")
            .clear();
    }
}

impl LogWriter for MemoryWriter {
    fn write(&self, entry: &LogEntry) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !entry.level.is_enabled(self.min_level) {
            return Ok(());
        }

        let mut entries = self
            .entries
            .lock()
            .expect("Memory writer entries lock should not be poisoned");
        entries.push_back(entry.clone());

        if entries.len() > self.max_entries {
            entries.pop_front();
        }

        Ok(())
    }

    fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // No-op for memory writer
        Ok(())
    }
}

/// Main logger implementation
pub struct Logger {
    writers: Vec<Arc<dyn LogWriter>>,
    min_level: LogLevel,
    module_filters: Vec<(String, LogLevel)>,
}

impl Logger {
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            writers: Vec::new(),
            min_level,
            module_filters: Vec::new(),
        }
    }

    pub fn add_writer(&mut self, writer: Arc<dyn LogWriter>) {
        self.writers.push(writer);
    }

    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    pub fn add_module_filter(&mut self, module: &str, level: LogLevel) {
        self.module_filters.push((module.to_string(), level));
    }

    fn get_target_level(&self, target: &str) -> LogLevel {
        for (module, level) in &self.module_filters {
            if target.starts_with(module) {
                return *level;
            }
        }
        self.min_level
    }

    pub fn log(
        &self,
        level: LogLevel,
        target: &str,
        file: Option<&str>,
        line: Option<u32>,
        message: String,
        module_path: Option<&str>,
    ) {
        let min_level = self.get_target_level(target);
        if !level.is_enabled(min_level) {
            return;
        }

        let entry = LogEntry::new(level, target.to_string(), file, line, message, module_path);

        for writer in &self.writers {
            if let Err(e) = writer.write(&entry) {
                eprintln!("Error writing log: {}", e);
            }
        }
    }

    pub fn flush(&self) {
        for writer in &self.writers {
            if let Err(e) = writer.flush() {
                eprintln!("Error flushing log: {}", e);
            }
        }
    }
}

/// Global logger instance
static GLOBAL_LOGGER: OnceLock<Arc<Mutex<Logger>>> = OnceLock::new();

/// Initialize the global logger
pub fn init_logger(min_level: LogLevel) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = GLOBAL_LOGGER.get_or_init(|| Arc::new(Mutex::new(Logger::new(min_level))));
    Ok(())
}

/// Get a reference to the global logger
pub fn logger() -> Option<Arc<Mutex<Logger>>> {
    GLOBAL_LOGGER.get().cloned()
}

/// Log a message at the specified level
pub fn log(level: LogLevel, target: &str, message: &str) {
    if let Some(logger) = logger() {
        let logger = logger
            .lock()
            .expect("Global logger lock should not be poisoned");
        logger.log(
            level,
            target,
            None, // file
            None, // line
            message.to_string(),
            None, // module_path
        );
    }
}

/// Log a message with file and line information
pub fn log_with_location(
    level: LogLevel,
    target: &str,
    file: &str,
    line: u32,
    message: &str,
    module_path: &str,
) {
    if let Some(logger) = logger() {
        let logger = logger
            .lock()
            .expect("Global logger lock should not be poisoned");
        logger.log(
            level,
            target,
            Some(file),
            Some(line),
            message.to_string(),
            Some(module_path),
        );
    }
}

/// Convenience macros for different log levels
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::log::log($crate::log::LogLevel::Trace, module_path!(), &format!($($arg)*))
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::log::log($crate::log::LogLevel::Debug, module_path!(), &format!($($arg)*))
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log::log($crate::log::LogLevel::Info, module_path!(), &format!($($arg)*))
    }
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log::log($crate::log::LogLevel::Warn, module_path!(), &format!($($arg)*))
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log::log($crate::log::LogLevel::Error, module_path!(), &format!($($arg)*))
    }
}

#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {
        $crate::log::log($crate::log::LogLevel::Fatal, module_path!(), &format!($($arg)*));
        std::process::exit(1);
    }
}

/// Log configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: LogLevel,
    pub file_path: Option<String>,
    pub max_file_size: u64, // in bytes
    pub max_files: u32,
    pub enable_console: bool,
    pub format: LogFormat,
    pub module_filters: Vec<(String, LogLevel)>,
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    Plain,
    Json,
    Syslog,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            file_path: None,
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
            enable_console: true,
            format: LogFormat::Plain,
            module_filters: Vec::new(),
        }
    }
}

/// Setup logger based on configuration
pub fn setup_logger(config: &LogConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logger(config.level)?;

    if let Some(logger) = logger() {
        let mut logger = logger
            .lock()
            .expect("Global logger lock should not be poisoned");

        // Add console writer if enabled
        if config.enable_console {
            let console_writer = Arc::new(
                ConsoleWriter::new(config.level)
                    .with_show_level(true)
                    .with_show_target(true)
                    .with_show_location(true),
            );
            logger.add_writer(console_writer);
        }

        // Add file writer if path is provided
        if let Some(ref path) = config.file_path {
            let file_writer = Arc::new(FileWriter::new(path, config.level)?);
            logger.add_writer(file_writer);
        }

        // Add module filters
        for (module, level) in &config.module_filters {
            logger.add_module_filter(module, *level);
        }
    }

    Ok(())
}

/// Log utilities
pub mod log_utils {
    use super::*;

    /// Format a log entry according to the specified format
    pub fn format_log_entry(entry: &LogEntry, format: &LogFormat) -> String {
        match format {
            LogFormat::Plain => format!(
                "[{}] {} [{}] [{}:{}] - {}",
                entry.level,
                entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                entry.target,
                match &entry.file {
                    Some(f) => f.as_str(),
                    None => "<unnamed>",
                },
                match entry.line {
                    Some(l) => l,
                    None => 0,
                },
                entry.message
            ),
            LogFormat::Json => format!(
                r#"{{"timestamp":"{}","level":"{}","target":"{}","file":"{}","line":{},"message":"{}"}}"#,
                entry.timestamp.to_rfc3339(),
                entry.level,
                entry.target,
                match &entry.file {
                    Some(f) => f.as_str(),
                    None => "",
                },
                match entry.line {
                    Some(l) => l,
                    None => 0,
                },
                entry.message.replace('"', "\\\"")
            ),
            LogFormat::Syslog => format!(
                "<{}>{} {}: [{}] {}",
                syslog_priority(&entry.level),
                entry.timestamp.format("%b %d %H:%M:%S"),
                entry.target,
                entry.level,
                entry.message
            ),
        }
    }

    /// Convert log level to syslog priority
    fn syslog_priority(level: &LogLevel) -> u8 {
        match level {
            LogLevel::Trace | LogLevel::Debug => 7, // Debug
            LogLevel::Info => 6,                    // Info
            LogLevel::Warn => 4,                    // Warning
            LogLevel::Error => 3,                   // Error
            LogLevel::Fatal => 2,                   // Critical
        }
    }

    /// Get the current timestamp for logging
    pub fn current_timestamp() -> DateTime<Local> {
        Local::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level() {
        assert!(LogLevel::Error.is_enabled(LogLevel::Info));
        assert!(!LogLevel::Debug.is_enabled(LogLevel::Error));
        assert!(LogLevel::Info.is_enabled(LogLevel::Info));
    }

    #[test]
    fn test_console_writer() {
        let writer = ConsoleWriter::new(LogLevel::Info);
        let entry = LogEntry::new(
            LogLevel::Info,
            "test".to_string(),
            Some("test.rs"),
            Some(42),
            "Test message".to_string(),
            Some("test_module"),
        );

        assert!(writer.write(&entry).is_ok());
    }

    #[test]
    fn test_memory_writer() {
        let writer = MemoryWriter::new(10, LogLevel::Debug);
        let entry = LogEntry::new(
            LogLevel::Debug,
            "test".to_string(),
            Some("test.rs"),
            Some(42),
            "Test message".to_string(),
            Some("test_module"),
        );

        assert!(writer.write(&entry).is_ok());
        assert_eq!(writer.get_entries().len(), 1);

        writer.clear();
        assert_eq!(writer.get_entries().len(), 0);
    }

    #[test]
    fn test_logger() {
        let mut logger = Logger::new(LogLevel::Info);
        let writer = Arc::new(MemoryWriter::new(10, LogLevel::Info));
        logger.add_writer(writer.clone());

        logger.log(
            LogLevel::Info,
            "test_target",
            Some("test.rs"),
            Some(42),
            "Test log message".to_string(),
            Some("test_module"),
        );

        let entries = writer.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, LogLevel::Info);
        assert_eq!(entries[0].message, "Test log message");
    }

    #[test]
    fn test_log_macro() {
        // This test is more for compilation checking
        // Since the macros depend on the crate structure, we'll just verify they're defined
        // In a real project, these would log actual messages
    }
}
