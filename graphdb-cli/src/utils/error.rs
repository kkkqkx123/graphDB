use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Query execution failed: {0}")]
    QueryError(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Command error: {0}")]
    CommandError(String),

    #[error("No active connection")]
    NotConnected,

    #[error("No space selected")]
    NoSpaceSelected,

    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("Script file not found: {0}")]
    ScriptNotFound(String),

    #[error("{0}")]
    Other(String),
}

impl CliError {
    pub fn connection(msg: impl Into<String>) -> Self {
        CliError::ConnectionError(msg.into())
    }

    pub fn auth(msg: impl Into<String>) -> Self {
        CliError::AuthError(msg.into())
    }

    pub fn query(msg: impl Into<String>) -> Self {
        CliError::QueryError(msg.into())
    }

    pub fn session(msg: impl Into<String>) -> Self {
        CliError::SessionError(msg.into())
    }

    pub fn config(msg: impl Into<String>) -> Self {
        CliError::ConfigError(msg.into())
    }

    pub fn command(msg: impl Into<String>) -> Self {
        CliError::CommandError(msg.into())
    }
}

pub type Result<T> = std::result::Result<T, CliError>;
