use anyhow::Result;
use clap::Parser;

// 导入库模块
use graphdb::api;

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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli {
        Cli::Serve { config } => {
            println!("Starting GraphDB service with config: {}", config);
            // Initialize and start the service
            api::start_service(config).await?;
        }
        Cli::Query { query } => {
            println!("Executing query: {}", query);
            // Execute the query directly
            api::execute_query(&query).await?;
        }
    }

    Ok(())
}
