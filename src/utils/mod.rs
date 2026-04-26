// Utility module - Only used for exporting submodules, no specific implementation

// ID generation module
pub mod id_gen;
pub use id_gen::{generate_id, IdGenerator};

// Logging module
pub mod logging;
pub use logging::{
    init as init_logging, is_initialized as is_logging_initialized, shutdown as shutdown_logging,
};
