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

                // 加载配置
                let cfg = match Config::load(&config) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        eprintln!("加载配置文件失败: {}, 使用默认配置", e);
                        Config::default()
                    }
                };

                // 初始化日志系统
                if let Err(e) = logging::init(&cfg) {
                    eprintln!("初始化日志系统失败: {}", e);
                }

                // Initialize and start service
                let result = api::start_service_with_config(cfg);

                // 确保日志 flush 后再退出
                logging::shutdown();
                result?;
            }
            Cli::Query { query } => {
                println!("Executing query: {}", query);
                println!("Process ID: {}", std::process::id());

                // 使用默认配置初始化日志
                let cfg = Config::default();
                if let Err(e) = logging::init(&cfg) {
                    eprintln!("初始化日志系统失败: {}", e);
                }

                // Execute query directly using tokio runtime
                let rt = tokio::runtime::Runtime::new()?;
                let result = rt.block_on(api::execute_query(&query));

                // 确保日志 flush 后再退出
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
    eprintln!("错误：server feature 未启用，无法运行服务端程序");
    eprintln!("请使用以下命令重新编译：");
    eprintln!("  cargo run --features server");
    std::process::exit(1);
}
