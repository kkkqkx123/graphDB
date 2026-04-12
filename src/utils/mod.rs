// Utility module - Only used for exporting submodules, no specific implementation

// ID generation module
pub mod id_gen;
pub use id_gen::{generate_id, IdGenerator};

// Logging module
pub mod logging;
pub use logging::{
    init as init_logging, is_initialized as is_logging_initialized, shutdown as shutdown_logging,
};

// Output stream module
pub mod output;
pub use output::{
    print, print_error, print_info, print_json, print_json_compact, print_success, print_table,
    print_warning, println, Format, JsonFormatter, OutputConfig, OutputError, OutputManager,
    OutputMode, Result as OutputResult, StreamOutput, TableFormatter,
};
