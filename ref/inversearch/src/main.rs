fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("inversearch=info".parse()
                    .expect("Failed to parse log level directive")),
        )
        .init();

    tracing::info!("Starting Inversearch service");

    if let Err(e) = run() {
        tracing::error!("Service error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    tracing::info!("Inversearch service started successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {
        assert!(true);
    }
}
