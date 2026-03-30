#[cfg(feature = "server")]
mod server_main {
    use clap::Parser;
    use graphdb::api;
    use graphdb::config::Config;
    use graphdb::core::error::DBResult;
    use graphdb::utils::logging;

    #[derive(Parser)]
    #[clap(version = "0.1.0", author = "GraphDB Contributors")]
    enum Cli {
        /// Start the GraphDB service
        Serve {
            #[clap(short, long, default_value = "config.toml")]
            config: String,
        },
        /// Execute a query directly
        Query {
            #[clap(short, long)]
            query: String,
        },
    }

    pub fn main() -> DBResult<()> {
        let cli = Cli::parse();

        match cli {
            Cli::Serve { config } => {
                println!("Starting GraphDB service with config: {}", config);
                println!("Process ID: {}", std::process::id());

                // Load configuration
                let cfg = match Config::load(&config) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        eprintln!(
                            "Failed to load configuration file: {}, using default configuration",
                            e
                        );
                        Config::default()
                    }
                };

                // Initialize logging system
                if let Err(e) = logging::init(&cfg) {
                    eprintln!("Failed to initialize logging system: {}", e);
                }

                // Initialize and start service
                let result = api::start_service_with_config(cfg);

                // Ensure logging is flushed before exiting
                logging::shutdown();
                result?;
            }
            Cli::Query { query } => {
                println!("Executing query: {}", query);
                println!("Process ID: {}", std::process::id());

                // Use default configuration to initialize logging
                let cfg = Config::default();
                if let Err(e) = logging::init(&cfg) {
                    eprintln!("Failed to initialize logging system: {}", e);
                }

                // Execute query directly using tokio runtime
                let rt = tokio::runtime::Runtime::new()?;
                let result = rt.block_on(api::execute_query(&query));

                // Ensure logging is flushed before exiting
                logging::shutdown();
                result?;
            }
        }

        Ok(())
    }
}

#[cfg(feature = "server")]
use graphdb::core::error::DBResult;

#[cfg(feature = "server")]
fn main() -> DBResult<()> {
    server_main::main()
}

#[cfg(not(feature = "server"))]
fn main() {
    eprintln!("Error: server feature is not enabled, cannot run server program");
    eprintln!("Please recompile using the following command:");
    eprintln!("  cargo run --features server");
    std::process::exit(1);
}
