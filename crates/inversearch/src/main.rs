// Service entry point - only compiled when "service" feature is enabled
#[cfg(feature = "service")]
use inversearch_service::service::{run_server, ServiceConfig};

#[cfg(feature = "service")]
fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(
                "inversearch=info"
                    .parse()
                    .expect("Failed to parse log level directive"),
            ),
        )
        .init();

    tracing::info!("Starting Inversearch service");

    if let Err(e) = run() {
        tracing::error!("Service error: {}", e);
        std::process::exit(1);
    }
}

// Library mode - when "service" feature is disabled, this is not a valid binary
#[cfg(not(feature = "service"))]
fn main() {
    eprintln!("Inversearch is compiled in library mode. This binary is not intended for direct execution.");
    eprintln!("To build as a service, compile with: cargo build --features service");
    eprintln!("To use as a library, add 'inversearch' as a dependency in your Cargo.toml");
    std::process::exit(1);
}

#[cfg(feature = "service")]
fn run() -> anyhow::Result<()> {
    // Use tokio runtime for async gRPC server
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let config = ServiceConfig::default();
        match run_server(config).await {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Service error: {}", e)),
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {
        // 主函数入口测试
    }
}
