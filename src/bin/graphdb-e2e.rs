//! E2E Test Runner for GraphDB
//!
//! This binary provides utilities for running E2E tests and generating reports.
//!
//! Usage:
//!   cargo run --bin graphdb-e2e -- --help
//!   cargo test --test e2e  # Run all E2E tests

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(version = "0.1.0", author = "GraphDB Contributors")]
enum Cli {
    /// Run E2E tests
    Run {
        /// Test suite to run (all, social, optimizer, extended, schema)
        #[clap(short, long, default_value = "all")]
        suite: String,

        /// Output format (text, json, junit)
        #[clap(short, long, default_value = "text")]
        format: String,

        /// Output file
        #[clap(short, long)]
        output: Option<PathBuf>,
    },

    /// List available test suites
    List,

    /// Run health check against a running server
    Health {
        /// Server host
        #[clap(short, long, default_value = "127.0.0.1")]
        host: String,

        /// Server port
        #[clap(short, long, default_value = "9758")]
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli {
        Cli::Run {
            suite,
            format,
            output,
        } => {
            println!("Running E2E tests: {}", suite);
            println!("Output format: {}", format);

            if let Some(path) = output {
                println!("Output file: {:?}", path);
            }

            println!("\nTo run E2E tests, use:");
            println!("  cargo test --test e2e");
            println!("  cargo test --test e2e social_network");
            println!("  cargo test --test e2e optimizer");
            println!("  cargo test --test e2e extended_types");
            println!("  cargo test --test e2e schema_manager");
        }
        Cli::List => {
            println!("Available E2E test suites:");
            println!("  - social_network: Social network scenario tests");
            println!("  - optimizer: Query optimizer tests");
            println!("  - extended_types: Extended type tests (geography, vector, fulltext)");
            println!("  - schema_manager: Schema manager initialization tests");
        }
        Cli::Health { host, port } => {
            println!("Checking server health at {}:{}", host, port);
            println!(
                "Use 'curl http://{}:{}/v1/health' to check manually",
                host, port
            );
        }
    }
}
