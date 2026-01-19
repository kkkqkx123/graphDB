use anyhow::Result;
use clap::Parser;

// 导入库模块
use graphdb::api;
use graphdb::common::process::ProcessManager;

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
            
            // Initialize process manager
            let pm = ProcessManager::new();
            println!("Process manager initialized");
            let info = pm.current_process_info().expect("Failed to get process info");
            println!("Process info: {:?}", info);
            
            // Initialize and start service
            api::start_service(config).await?;
        }
        Cli::Query { query } => {
            println!("Executing query: {}", query);
            
            // Initialize process manager
            let pm = ProcessManager::new();
            println!("Process manager initialized");
            let info = pm.current_process_info().expect("Failed to get process info");
            println!("Process info: {:?}", info);
            
            // Execute query directly
            api::execute_query(&query).await?;
        }
    }

    Ok(())
}
