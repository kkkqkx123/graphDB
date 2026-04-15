//! Output stream module for GraphDB
//!
//! Provides unified output control with support for multiple formats and destinations.
//!
//! # Examples
//!
//! ```rust
//! use graphdb::utils::output;
//!
//! // Global convenience functions
//! output::println("Hello, World!").unwrap();
//! output::print_success("Operation completed").unwrap();
//! output::print_error("Something went wrong").unwrap();
//! ```
//!
//! ```rust
//! use graphdb::utils::output::{OutputManager, Format};
//!
//! // Instance-based usage for customization
//! let manager = OutputManager::new()
//!     .with_format(Format::Json);
//!
//! manager.println("{ \"status\": \"ok\" }").unwrap();
//! ```

// Error types
mod error;
pub use error::{OutputError, Result};

// Writer implementations
mod writer;
pub use writer::{FileWriter, MultiWriter, StderrWriter, StdoutWriter};

// Manager and format
mod manager;
pub use manager::{
    get_global_format, print, print_error, print_info, print_success, print_warning, println,
    set_global_format, Format, OutputManager,
};

// JSON formatter (Phase 2)
mod json;
pub use json::{
    print_json, print_json_compact, print_json_to, to_json_string, to_json_string_compact,
    JsonFormatter,
};

// Table formatter (Phase 2)
mod table;
pub use table::{print_table, print_table_to, TableFormatter};

// Configuration (Phase 3)
mod config;
pub use config::{OutputConfig, OutputMode};

// Stream output (Phase 3)
mod stream;
pub use stream::StreamOutput;

/// Initialize the output module with default settings
pub fn init() {
    // The global manager is lazily initialized on first use
}

/// Initialize the output module with a specific format
pub fn init_with_format(format: Format) {
    set_global_format(format);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Ensure all public types are accessible
        let _ = Format::Plain;
        let _ = Format::Json;
        let _ = Format::Table;
    }

    #[test]
    fn test_output_manager_creation() {
        let manager = OutputManager::new();
        assert_eq!(manager.format(), Format::Plain);
    }
}
