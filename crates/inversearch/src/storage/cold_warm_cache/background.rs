//! 后台任务管理器
//!
//! 管理定期执行的后台任务：
//! - Flush: 将热缓存数据降级到温缓存
//! - Merge: 将温缓存数据合并到冷存储
//! - Cleanup: 清理过期数据
//! - Checkpoint: 创建检查点

use crate::storage::cold_warm_cache::manager::ColdWarmCacheManager;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

pub struct BackgroundTaskManager {
    flush_handle: Option<JoinHandle<()>>,
    merge_handle: Option<JoinHandle<()>>,
    cleanup_handle: Option<JoinHandle<()>>,
    checkpoint_handle: Option<JoinHandle<()>>,
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl BackgroundTaskManager {
    pub fn new(manager: Arc<ColdWarmCacheManager>) -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

        let flush_interval = manager.get_flush_interval();
        let flush_handle = if flush_interval > Duration::from_secs(0) {
            Some(tokio::spawn({
                let manager = manager.clone();
                let mut shutdown = shutdown_tx.subscribe();
                async move {
                    let mut ticker = interval(flush_interval);
                    loop {
                        tokio::select! {
                            _ = ticker.tick() => {
                                if let Err(e) = manager.flush_hot_to_warm().await {
                                    tracing::error!("Flush task error: {}", e);
                                }
                            }
                            _ = shutdown.recv() => {
                                break;
                            }
                        }
                    }
                }
            }))
        } else {
            None
        };

        let merge_interval = manager.get_merge_interval();
        let merge_handle = if merge_interval > Duration::from_secs(0) {
            Some(tokio::spawn({
                let manager = manager.clone();
                let mut shutdown = shutdown_tx.subscribe();
                async move {
                    let mut ticker = interval(merge_interval);
                    loop {
                        tokio::select! {
                            _ = ticker.tick() => {
                                if let Err(e) = manager.merge_warm_to_cold().await {
                                    tracing::error!("Merge task error: {}", e);
                                }
                            }
                            _ = shutdown.recv() => {
                                break;
                            }
                        }
                    }
                }
            }))
        } else {
            None
        };

        let cleanup_interval = manager.get_cleanup_interval();
        let cleanup_handle = if cleanup_interval > Duration::from_secs(0) {
            Some(tokio::spawn({
                let manager = manager.clone();
                let mut shutdown = shutdown_tx.subscribe();
                async move {
                    let mut ticker = interval(cleanup_interval);
                    loop {
                        tokio::select! {
                            _ = ticker.tick() => {
                                if let Err(e) = manager.create_checkpoint().await {
                                    tracing::error!("Cleanup task error: {}", e);
                                }
                            }
                            _ = shutdown.recv() => {
                                break;
                            }
                        }
                    }
                }
            }))
        } else {
            None
        };

        let checkpoint_interval = manager.get_checkpoint_interval();
        let checkpoint_handle = if checkpoint_interval > Duration::from_secs(0) {
            Some(tokio::spawn({
                let manager = manager.clone();
                let mut shutdown = shutdown_tx.subscribe();
                async move {
                    let mut ticker = interval(checkpoint_interval);
                    loop {
                        tokio::select! {
                            _ = ticker.tick() => {
                                if let Err(e) = manager.create_checkpoint().await {
                                    tracing::error!("Checkpoint task error: {}", e);
                                }
                            }
                            _ = shutdown.recv() => {
                                break;
                            }
                        }
                    }
                }
            }))
        } else {
            None
        };

        Self {
            flush_handle,
            merge_handle,
            cleanup_handle,
            checkpoint_handle,
            shutdown_tx: Some(shutdown_tx),
        }
    }

    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        if let Some(handle) = self.flush_handle.take() {
            let _ = handle.await;
        }
        if let Some(handle) = self.merge_handle.take() {
            let _ = handle.await;
        }
        if let Some(handle) = self.cleanup_handle.take() {
            let _ = handle.await;
        }
        if let Some(handle) = self.checkpoint_handle.take() {
            let _ = handle.await;
        }
    }
}
