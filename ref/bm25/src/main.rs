#![cfg(feature = "service")]

use bm25_service::{Config, init_logging, init_metrics, run_server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    init_metrics();

    tracing::info!("Starting BM25 service");

    let config = Config::from_env().unwrap_or_else(|_| Config::default());
    tracing::info!("Loaded configuration: {:?}", config);

    run_server(config).await?;

    Ok(())
}
