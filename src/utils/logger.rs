/// A simple logger wrapper for consistent logging format
pub struct Logger;

impl Logger {
    pub fn info(message: &str) {
        println!("[INFO] {}", message);
    }

    pub fn warn(message: &str) {
        eprintln!("[WARN] {}", message);
    }

    pub fn error(message: &str) {
        eprintln!("[ERROR] {}", message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger() {
        // These tests just verify the methods don't panic
        // In a real scenario, you might capture stdout/stderr to verify output
        Logger::info("Test info message");
        Logger::warn("Test warning message");
        Logger::error("Test error message");
    }
}
