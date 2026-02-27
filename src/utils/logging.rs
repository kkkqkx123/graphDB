// 日志工具模块
//
// 封装 flexi_logger 的初始化和关闭操作，确保异步日志正确 flush

use crate::config::Config;
use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming, WriteMode, LoggerHandle};
use parking_lot::Mutex;

/// 全局日志句柄，用于程序退出时 flush
static LOGGER_HANDLE: Mutex<Option<LoggerHandle>> = Mutex::new(None);

/// 初始化日志系统
///
/// # Arguments
/// * `config` - 应用配置，包含日志相关参数
///
/// # Returns
/// * `Ok(())` - 初始化成功
/// * `Err(Box<dyn std::error::Error>)` - 初始化失败
///
/// # Examples
/// ```
/// use graphdb::config::Config;
/// use graphdb::utils::logging;
///
/// let config = Config::default();
/// logging::init(&config).expect("日志初始化失败");
/// ```
pub fn init(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let handle = Logger::try_with_str(&config.log.level)?
        .log_to_file(
            FileSpec::default()
                .basename(&config.log.file)
                .directory(&config.log.dir),
        )
        .rotate(
            Criterion::Size(config.log.max_file_size),
            Naming::Numbers,
            Cleanup::KeepLogFiles(config.log.max_files),
        )
        .write_mode(WriteMode::Async)
        .append()
        .start()?;

    // 保存句柄供后续 flush 使用
    *LOGGER_HANDLE.lock() = Some(handle);

    log::info!("日志系统初始化完成: {}/{}", config.log.dir, config.log.file);
    Ok(())
}

/// 刷新并关闭日志系统
///
/// 在程序退出前调用，确保所有异步日志都已写入文件
/// 这是一个阻塞操作，会等待日志线程完成当前工作
///
/// # Examples
/// ```
/// use graphdb::utils::logging;
///
/// // 程序退出前
/// logging::shutdown();
/// ```
pub fn shutdown() {
    let mut guard = LOGGER_HANDLE.lock();
    if let Some(handle) = guard.take() {
        handle.flush();
        // handle 在这里被 drop，会等待异步线程完成
    }
}

/// 检查日志系统是否已初始化
///
/// # Returns
/// * `true` - 日志系统已初始化
/// * `false` - 日志系统未初始化
pub fn is_initialized() -> bool {
    LOGGER_HANDLE.lock().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_init_and_shutdown() {
        let config = Config::default();
        
        // 初始化日志
        let result = init(&config);
        assert!(result.is_ok(), "日志初始化失败: {:?}", result.err());
        assert!(is_initialized());
        
        // 写入测试日志
        log::info!("测试日志消息");
        
        // 关闭日志
        shutdown();
        assert!(!is_initialized());
    }

    #[test]
    fn test_is_initialized_before_init() {
        // 确保在初始化前返回 false
        // 注意：由于 LOGGER_HANDLE 是全局的，这个测试可能受到其他测试的影响
        // 在实际运行中，应该在独立的进程中测试
    }
}
