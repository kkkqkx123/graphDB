use clap::Parser;

use crate::client::ConnectionMode;

#[derive(Parser, Debug)]
#[clap(
    name = "graphdb-cli",
    version = env!("CARGO_PKG_VERSION"),
    about = "GraphDB CLI - Interactive command-line client for GraphDB",
    long_about = "GraphDB CLI is an interactive command-line client for GraphDB,\n\
                  similar to PostgreSQL's psql. It supports GQL query execution,\n\
                  schema inspection, and various output formats.\n\
                  \n\
                  Connection modes:\n\
                  - HTTP mode (default): Connect to a remote GraphDB server\n\
                  - Embedded mode: Direct access to local database file"
)]
pub struct Cli {
    #[clap(
        short,
        long,
        default_value = "http",
        help = "Connection mode: http or embedded"
    )]
    pub mode: ConnectionModeArg,

    #[clap(
        short,
        long,
        default_value = "127.0.0.1",
        help = "Server host (HTTP mode)"
    )]
    pub host: String,

    #[clap(short, long, default_value_t = 8080, help = "Server port (HTTP mode)")]
    pub port: u16,

    #[clap(short, long, help = "Database file path (embedded mode)")]
    pub db_path: Option<String>,

    #[clap(
        short,
        long,
        default_value = "root",
        help = "Username for authentication"
    )]
    pub user: String,

    #[clap(short = 'W', long, help = "Prompt for password")]
    pub password: bool,

    #[clap(short, long, help = "Space name to connect to")]
    pub database: Option<String>,

    #[clap(short, long, help = "Execute single command and exit")]
    pub command: Option<String>,

    #[clap(short = 'f', long = "file", help = "Execute commands from file")]
    pub file: Option<String>,

    #[clap(short, long, help = "Output file for query results")]
    pub output: Option<String>,

    #[clap(
        long,
        default_value = "table",
        help = "Output format (table, csv, json, vertical, html)"
    )]
    pub format: String,

    #[clap(short, long, help = "Quiet mode - suppress non-essential output")]
    pub quiet: bool,

    #[clap(
        short = '1',
        long = "single-transaction",
        help = "Execute commands in a single transaction"
    )]
    pub single_transaction: bool,

    #[clap(long = "force", help = "Continue processing after errors")]
    pub force: bool,

    #[clap(
        short = 'v',
        long = "variable",
        value_name = "NAME=VALUE",
        help = "Set variable before execution"
    )]
    pub variables: Vec<String>,
}

/// Wrapper for ConnectionMode to implement clap::ValueEnum
#[derive(Debug, Clone, Copy)]
pub enum ConnectionModeArg {
    Http,
    Embedded,
}

impl std::str::FromStr for ConnectionModeArg {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "http" => Ok(ConnectionModeArg::Http),
            "embedded" => Ok(ConnectionModeArg::Embedded),
            _ => Err(format!(
                "Unknown connection mode: {}. Use 'http' or 'embedded'",
                s
            )),
        }
    }
}

impl std::fmt::Display for ConnectionModeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionModeArg::Http => write!(f, "http"),
            ConnectionModeArg::Embedded => write!(f, "embedded"),
        }
    }
}

impl From<ConnectionModeArg> for ConnectionMode {
    fn from(arg: ConnectionModeArg) -> Self {
        match arg {
            ConnectionModeArg::Http => ConnectionMode::Http,
            ConnectionModeArg::Embedded => ConnectionMode::Embedded,
        }
    }
}
