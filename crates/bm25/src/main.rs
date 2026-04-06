#![cfg(feature = "service")]

use bm25_service::{init_logging, run_server, ServiceConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    tracing::info!("Starting BM25 service");

    let config = ServiceConfig::from_env().unwrap_or_else(|_| ServiceConfig::default());
    tracing::info!("Loaded configuration: {:?}", config);

    run_server(config).await?;

    Ok(())
}
