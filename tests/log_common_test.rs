use graphdb::common::log::*;
use std::sync::Arc;

#[test]
fn test_log_level_display() {
    assert_eq!(format!("{}", LogLevel::Trace), "TRACE");
    assert_eq!(format!("{}", LogLevel::Debug), "DEBUG");
    assert_eq!(format!("{}", LogLevel::Info), "INFO");
    assert_eq!(format!("{}", LogLevel::Warn), "WARN");
    assert_eq!(format!("{}", LogLevel::Error), "ERROR");
    assert_eq!(format!("{}", LogLevel::Fatal), "FATAL");
}

#[test]
fn test_log_level_is_enabled() {
    assert!(LogLevel::Info.is_enabled(LogLevel::Debug));
    assert!(LogLevel::Info.is_enabled(LogLevel::Info));
    assert!(!LogLevel::Info.is_enabled(LogLevel::Warn));
}

#[test]
fn test_log_level_ord() {
    assert!(LogLevel::Trace < LogLevel::Debug);
    assert!(LogLevel::Debug < LogLevel::Info);
    assert!(LogLevel::Info < LogLevel::Warn);
    assert!(LogLevel::Warn < LogLevel::Error);
    assert!(LogLevel::Error < LogLevel::Fatal);
}

#[test]
fn test_log_level_clone() {
    let level1 = LogLevel::Info;
    let level2 = level1.clone();
    assert_eq!(level1, level2);
}

#[test]
fn test_log_level_partial_eq() {
    assert_eq!(LogLevel::Info, LogLevel::Info);
    assert_ne!(LogLevel::Info, LogLevel::Error);
}

#[test]
fn test_log_entry_new() {
    let entry = LogEntry::new(
        LogLevel::Info,
        "test".to_string(),
        Some("test.rs"),
        Some(42),
        "Test message".to_string(),
        Some("test_module"),
    );

    assert_eq!(entry.level, LogLevel::Info);
    assert_eq!(entry.target, "test");
    assert_eq!(entry.file, Some("test.rs".to_string()));
    assert_eq!(entry.line, Some(42));
    assert_eq!(entry.message, "Test message");
    assert_eq!(entry.module_path, Some("test_module".to_string()));
}

#[test]
fn test_log_entry_new_without_location() {
    let entry = LogEntry::new(
        LogLevel::Error,
        "test".to_string(),
        None,
        None,
        "Error message".to_string(),
        None,
    );

    assert_eq!(entry.level, LogLevel::Error);
    assert!(entry.file.is_none());
    assert!(entry.line.is_none());
    assert!(entry.module_path.is_none());
}

#[test]
fn test_log_entry_timestamp() {
    let entry = LogEntry::new(
        LogLevel::Info,
        "test".to_string(),
        None,
        None,
        "Message".to_string(),
        None,
    );

    assert!(entry.timestamp <= chrono::Local::now());
}

#[test]
fn test_log_entry_clone() {
    let entry1 = LogEntry::new(LogLevel::Info, "test".to_string(), None, None, "Message".to_string(), None);
    let entry2 = entry1.clone();
    assert_eq!(entry1.message, entry2.message);
    assert_eq!(entry1.level, entry2.level);
}

#[test]
fn test_console_writer_new() {
    let _writer = ConsoleWriter::new(LogLevel::Info);
}

#[test]
fn test_console_writer_with_level() {
    let _writer = ConsoleWriter::new(LogLevel::Debug).with_level(LogLevel::Error);
}

#[test]
fn test_console_writer_write() {
    let writer = ConsoleWriter::new(LogLevel::Info);
    let entry = LogEntry::new(
        LogLevel::Info,
        "test".to_string(),
        None,
        None,
        "Test message".to_string(),
        None,
    );
    let result = writer.write(&entry);
    assert!(result.is_ok());
}

#[test]
fn test_console_writer_filter_by_level() {
    let _writer = ConsoleWriter::new(LogLevel::Error);
    let debug_entry = LogEntry::new(LogLevel::Debug, "test".to_string(), None, None, "Debug".to_string(), None);
    assert!(!debug_entry.level.is_enabled(LogLevel::Error));
}

struct TestWriter {
    tx: std::sync::mpsc::Sender<LogEntry>,
}

impl TestWriter {
    fn new(tx: std::sync::mpsc::Sender<LogEntry>) -> Self {
        Self { tx }
    }
}

impl LogWriter for TestWriter {
    fn write(&self, entry: &LogEntry) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tx.send(entry.clone()).map_err(|e| e.into())
    }

    fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[test]
fn test_logger_new() {
    let _logger = Logger::new(LogLevel::Info);
}

#[test]
fn test_logger_with_min_level() {
    let _logger = Logger::new(LogLevel::Debug).with_min_level(LogLevel::Error);
}

#[test]
fn test_logger_add_writer() {
    let mut logger = Logger::new(LogLevel::Info);
    let writer = Arc::new(ConsoleWriter::new(LogLevel::Info));
    logger.add_writer(writer);
}

#[test]
fn test_logger_log() {
    let (tx, rx) = std::sync::mpsc::channel();
    let writer = TestWriter::new(tx);
    let mut logger = Logger::new(LogLevel::Info);
    let writer_arc = Arc::new(writer);
    logger.add_writer(writer_arc);

    logger.log(
        LogLevel::Info,
        "test",
        None,
        None,
        "Test message".to_string(),
        None,
    );

    let received = rx.recv_timeout(std::time::Duration::from_millis(100));
    assert!(received.is_ok());
    let entry = received.unwrap();
    assert_eq!(entry.message, "Test message");
}

#[test]
fn test_logger_log_with_location() {
    let (tx, rx) = std::sync::mpsc::channel();
    let writer = TestWriter::new(tx);
    let mut logger = Logger::new(LogLevel::Info);
    let writer_arc = Arc::new(writer);
    logger.add_writer(writer_arc);

    logger.log(
        LogLevel::Error,
        "test",
        Some("test.rs"),
        Some(42),
        "Error".to_string(),
        Some("test_module"),
    );

    let received = rx.recv_timeout(std::time::Duration::from_millis(100));
    assert!(received.is_ok());
    let entry = received.unwrap();
    assert_eq!(entry.file, Some("test.rs".to_string()));
    assert_eq!(entry.line, Some(42));
}

#[test]
fn test_logger_flush() {
    let (tx, _rx) = std::sync::mpsc::channel();
    let writer = TestWriter::new(tx);
    let mut logger = Logger::new(LogLevel::Info);
    let writer_arc = Arc::new(writer);
    logger.add_writer(writer_arc);
    logger.flush();
}

#[test]
fn test_logger_add_module_filter() {
    let mut logger = Logger::new(LogLevel::Info);
    logger.add_module_filter("test_module", LogLevel::Debug);
}
