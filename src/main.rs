use anyhow::Result;
use clap::Parser;
use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming, WriteMode};
use graphdb::config::Config;

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

/// 初始化日志系统
fn init_logger(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    Logger::try_with_str(&config.log_level)?
        .log_to_file(
            FileSpec::default()
                .basename(&config.log_file)
                .directory(&config.log_dir),
        )
        .rotate(
            Criterion::Size(config.max_log_file_size),
            Naming::Numbers,
            Cleanup::KeepLogFiles(config.max_log_files),
        )
        .write_mode(WriteMode::Async)
        .append()
        .start()?;

    log::info!("日志系统初始化完成: {}/{}", config.log_dir, config.log_file);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Serve { config } => {
            println!("Starting GraphDB service with config: {}", config);
            println!("Process ID: {}", std::process::id());

            // 加载配置
            let cfg = match Config::load(&config) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("加载配置文件失败: {}, 使用默认配置", e);
                    Config::default()
                }
            };

            // 初始化日志系统
            if let Err(e) = init_logger(&cfg) {
                eprintln!("初始化日志系统失败: {}", e);
            }

            // Initialize and start service
            api::start_service_with_config(cfg).await?;
        }
        Cli::Query { query } => {
            println!("Executing query: {}", query);
            println!("Process ID: {}", std::process::id());

            // 使用默认配置初始化日志
            let cfg = Config::default();
            if let Err(e) = init_logger(&cfg) {
                eprintln!("初始化日志系统失败: {}", e);
            }

            // Execute query directly
            api::execute_query(&query).await?;
        }
    }

    Ok(())
}
