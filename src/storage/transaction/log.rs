//! 事务日志 - 记录事务操作日志
//!
//! 实现事务日志的写入和恢复：
//! - TransactionLog: 事务日志
//! - LogRecord: 日志记录
//! - LogType: 日志类型
//! - 日志恢复

use super::{TransactionId, TransactionState};
use crate::core::StorageError;
use bincode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};

/// 日志类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogType {
    /// 开始日志
    Begin,
    /// 更新日志
    Update,
    /// 插入日志
    Insert,
    /// 删除日志
    Delete,
    /// 提交日志
    Commit,
    /// 回滚日志
    Rollback,
    /// 检查点日志
    Checkpoint,
    /// 补偿日志（用于回滚）
    Compensation,
}

impl LogType {
    pub fn is_undoable(&self) -> bool {
        matches!(self, LogType::Update | LogType::Insert | LogType::Delete)
    }

    pub fn is_redoable(&self) -> bool {
        matches!(self, LogType::Update | LogType::Insert)
    }
}

/// 日志记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    /// 日志序列号（LSN）
    pub lsn: u64,
    /// 事务 ID
    pub tx_id: TransactionId,
    /// 日志类型
    pub log_type: LogType,
    /// 时间戳
    pub timestamp: u64,
    /// 数据键
    pub key: Option<String>,
    /// 旧值（用于回滚）
    pub old_value: Option<Vec<u8>>,
    /// 新值（用于重做）
    pub new_value: Option<Vec<u8>>,
    /// 版本号
    pub version: Option<u64>,
    /// 下一个 LSN（用于撤销链表）
    pub prev_lsn: u64,
    /// 是否刷新到磁盘
    pub flushed: bool,
}

impl LogRecord {
    pub fn new(tx_id: TransactionId, log_type: LogType) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;
        Self {
            lsn: 0,
            tx_id,
            log_type,
            timestamp,
            key: None,
            old_value: None,
            new_value: None,
            version: None,
            prev_lsn: 0,
            flushed: false,
        }
    }

    pub fn with_key(mut self, key: &str) -> Self {
        self.key = Some(key.to_string());
        self
    }

    pub fn with_old_value(mut self, value: &[u8]) -> Self {
        self.old_value = Some(value.to_vec());
        self
    }

    pub fn with_new_value(mut self, value: &[u8]) -> Self {
        self.new_value = Some(value.to_vec());
        self
    }

    pub fn with_version(mut self, version: u64) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_prev_lsn(mut self, prev_lsn: u64) -> Self {
        self.prev_lsn = prev_lsn;
        self
    }

    pub fn size(&self) -> usize {
        let mut size = 24; // lsn + tx_id + log_type + timestamp
        if let Some(ref key) = self.key {
            size += key.len() + 4;
        }
        if let Some(ref old) = self.old_value {
            size += old.len() + 4;
        }
        if let Some(ref new) = self.new_value {
            size += new.len() + 4;
        }
        size
    }
}

/// 事务日志
///
/// 管理事务日志的写入、读取和恢复
#[derive(Debug)]
pub struct TransactionLog {
    /// 日志文件路径
    log_dir: PathBuf,
    /// 当前活跃日志文件
    current_log: Arc<Mutex<Option<BufWriter<File>>>>,
    /// 日志文件列表
    log_files: RwLock<Vec<PathBuf>>,
    /// LSN 计数器
    lsn_counter: Arc<Mutex<u64>>,
    /// 最后一个刷写 LSN
    flushed_lsn: Arc<Mutex<u64>>,
    /// 日志缓冲区
    buffer: Arc<Mutex<Vec<LogRecord>>>,
    /// 配置
    config: LogConfig,
    /// 统计信息
    stats: Arc<Mutex<LogStats>>,
}

impl Default for TransactionLog {
    fn default() -> Self {
        Self::new(PathBuf::from("transaction_logs"), LogConfig::default())
    }
}

impl TransactionLog {
    pub fn new(log_dir: PathBuf, config: LogConfig) -> Self {
        std::fs::create_dir_all(&log_dir).ok();

        let mut log_files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&log_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map(|e| e.to_str()) == Some(Some("log")) {
                    log_files.push(entry.path());
                }
            }
        }
        log_files.sort();

        Self {
            log_dir,
            current_log: Arc::new(Mutex::new(None)),
            log_files: RwLock::new(log_files),
            lsn_counter: Arc::new(Mutex::new(0)),
            flushed_lsn: Arc::new(Mutex::new(0)),
            buffer: Arc::new(Mutex::new(Vec::new())),
            config,
            stats: Arc::new(Mutex::new(LogStats::default())),
        }
    }

    /// 分配 LSN
    fn allocate_lsn(&self) -> u64 {
        let mut counter = self.lsn_counter.lock().unwrap();
        *counter += 1;
        *counter
    }

    /// 写入日志记录
    pub fn write(&self, record: LogRecord) -> Result<u64, StorageError> {
        let lsn = self.allocate_lsn();
        let mut record = record;
        record.lsn = lsn;

        if record.log_type == LogType::Begin {
            record.prev_lsn = 0;
        }

        let mut buffer = self.buffer.lock().unwrap();
        buffer.push(record.clone());

        if buffer.len() >= self.config.buffer_size || record.log_type == LogType::Commit {
            self.flush_buffer()?;
        }

        let mut stats = self.stats.lock().unwrap();
        stats.bytes_written += record.size() as u64;
        stats.records_written += 1;

        Ok(lsn)
    }

    /// 写入开始日志
    pub fn write_begin(&self, tx_id: TransactionId) -> Result<u64, StorageError> {
        let record = LogRecord::new(tx_id, LogType::Begin);
        self.write(record)
    }

    /// 写入更新日志
    pub fn write_update(
        &self,
        tx_id: TransactionId,
        key: &str,
        old_value: &[u8],
        new_value: &[u8],
        prev_lsn: u64,
    ) -> Result<u64, StorageError> {
        let record = LogRecord::new(tx_id, LogType::Update)
            .with_key(key)
            .with_old_value(old_value)
            .with_new_value(new_value)
            .with_prev_lsn(prev_lsn);
        self.write(record)
    }

    /// 写入插入日志
    pub fn write_insert(
        &self,
        tx_id: TransactionId,
        key: &str,
        value: &[u8],
        prev_lsn: u64,
    ) -> Result<u64, StorageError> {
        let record = LogRecord::new(tx_id, LogType::Insert)
            .with_key(key)
            .with_new_value(value)
            .with_prev_lsn(prev_lsn);
        self.write(record)
    }

    /// 写入删除日志
    pub fn write_delete(
        &self,
        tx_id: TransactionId,
        key: &str,
        old_value: &[u8],
        prev_lsn: u64,
    ) -> Result<u64, StorageError> {
        let record = LogRecord::new(tx_id, LogType::Delete)
            .with_key(key)
            .with_old_value(old_value)
            .with_prev_lsn(prev_lsn);
        self.write(record)
    }

    /// 写入提交日志
    pub fn write_commit(&self, tx_id: TransactionId, prev_lsn: u64) -> Result<u64, StorageError> {
        let record = LogRecord::new(tx_id, LogType::Commit).with_prev_lsn(prev_lsn);
        self.write(record)
    }

    /// 写入回滚日志
    pub fn write_rollback(&self, tx_id: TransactionId, prev_lsn: u64) -> Result<u64, StorageError> {
        let record = LogRecord::new(tx_id, LogType::Rollback).with_prev_lsn(prev_lsn);
        self.write(record)
    }

    /// 刷新缓冲区到磁盘
    fn flush_buffer(&self) -> Result<(), StorageError> {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.is_empty() {
            return Ok(());
        }

        let records: Vec<LogRecord> = buffer.drain(..).collect();

        let mut log_file = self.current_log.lock().unwrap();

        if log_file.is_none() {
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_millis();
            let file_name = format!("transaction_{}.log", timestamp);
            let path = self.log_dir.join(&file_name);
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| StorageError::DbError(format!("IO error opening log: {}", e)))?;

            let writer = BufWriter::new(file);
            *log_file = Some(writer);

            let mut log_files = self.log_files.write().unwrap();
            log_files.push(path);
        }

        if let Some(ref mut writer) = *log_file {
            for record in &records {
                let bytes = bincode::serde::encode_to_vec(record, bincode::config::standard())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;

                let len_bytes = (bytes.len() as u32).to_le_bytes();
                writer.write_all(&len_bytes).map_err(|e| {
                    StorageError::DbError(format!("IO error writing log: {}", e))
                })?;
                writer.write_all(&bytes).map_err(|e| {
                    StorageError::DbError(format!("IO error writing log: {}", e))
                })?;

                let mut flushed = self.flushed_lsn.lock().unwrap();
                *flushed = record.lsn;
            }

            writer.flush().map_err(|e| {
                StorageError::DbError(format!("IO error flushing log: {}", e))
            })?;
        }

        Ok(())
    }

    /// 强制刷新到磁盘
    pub fn flush(&self) -> Result<u64, StorageError> {
        self.flush_buffer()?;
        let flushed = self.flushed_lsn.lock().unwrap();
        Ok(*flushed)
    }

    /// 从日志恢复
    pub fn recover(&self) -> RecoveryResult {
        let mut transactions = HashMap::new();
        let mut dirty_pages = HashMap::new();
        let mut commit_lsns = HashMap::new();
        let mut undo_lsns = HashMap::new();

        let log_files = self.log_files.read().unwrap();

        for log_path in log_files.iter().rev() {
            if let Ok(file) = File::open(log_path) {
                let reader = BufReader::new(file);
                let mut stream = reader.bytes();

                while let Some(Ok(len_bytes)) = stream.next() {
                    if let Some(Ok(len_bytes2)) = stream.next() {
                        if let Some(Ok(len_bytes3)) = stream.next() {
                            if let Some(Ok(len_bytes4)) = stream.next() {
                                let len = u32::from_le_bytes([len_bytes, len_bytes2, len_bytes3, len_bytes4]) as usize;
                                let mut bytes = vec![0u8; len];

                                for byte in bytes.iter_mut() {
                                    if let Some(Ok(b)) = stream.next() {
                                        *byte = b;
                                    } else {
                                        break;
                                    }
                                }

                                if let Ok((record, _)) = bincode::serde::decode_from_slice::<LogRecord, _>(&bytes, bincode::config::standard()) {
                                    self.process_recovery_record(
                                        &record,
                                        &mut transactions,
                                        &mut dirty_pages,
                                        &mut commit_lsns,
                                        &mut undo_lsns,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        RecoveryResult {
            transactions,
            dirty_pages,
            commit_lsns,
            undo_lsns,
        }
    }

    fn process_recovery_record(
        &self,
        record: &LogRecord,
        transactions: &mut HashMap<super::TransactionId, TransactionInfo>,
        dirty_pages: &mut HashMap<String, u64>,
        commit_lsns: &mut HashMap<super::TransactionId, u64>,
        undo_lsns: &mut HashMap<super::TransactionId, u64>,
    ) {
        let tx_id = record.tx_id;

        match record.log_type {
            LogType::Begin => {
                transactions.insert(tx_id, TransactionInfo {
                    state: super::TransactionState::Active,
                    first_lsn: record.lsn,
                    last_lsn: record.lsn,
                });
            }
            LogType::Update | LogType::Insert | LogType::Delete => {
                if let Some(info) = transactions.get_mut(&tx_id) {
                    info.last_lsn = record.lsn;
                }
                if let Some(ref key) = record.key {
                    dirty_pages.insert(key.clone(), record.lsn);
                }
                undo_lsns.insert(tx_id, record.lsn);
            }
            LogType::Commit => {
                transactions.insert(tx_id, TransactionInfo {
                    state: super::TransactionState::Committed,
                    last_lsn: record.lsn,
                    first_lsn: transactions.get(&tx_id).map(|t| t.first_lsn).unwrap_or(0),
                });
                commit_lsns.insert(tx_id, record.lsn);
            }
            LogType::Rollback => {
                transactions.insert(tx_id, TransactionInfo {
                    state: super::TransactionState::Aborted,
                    last_lsn: record.lsn,
                    first_lsn: transactions.get(&tx_id).map(|t| t.first_lsn).unwrap_or(0),
                });
            }
            LogType::Checkpoint => {
            }
            _ => {}
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> LogStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    /// 清理旧日志文件
    pub fn cleanup_old_logs(&self, min_lsn: u64) {
        let mut log_files = self.log_files.write().unwrap();
        let flushed = self.flushed_lsn.lock().unwrap();

        log_files.retain(|path| {
            if let Ok(metadata) = path.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let age = SystemTime::now().duration_since(modified).unwrap_or(Duration::ZERO);
                    age < Duration::from_secs(self.config.retain_duration_secs as u64)
                } else {
                    true
                }
            } else {
                true
            }
        });
    }
}

/// 事务信息（恢复用）
#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub state: super::TransactionState,
    pub first_lsn: u64,
    pub last_lsn: u64,
}

/// 恢复结果
#[derive(Debug, Default)]
pub struct RecoveryResult {
    pub transactions: HashMap<super::TransactionId, TransactionInfo>,
    pub dirty_pages: HashMap<String, u64>,
    pub commit_lsns: HashMap<super::TransactionId, u64>,
    pub undo_lsns: HashMap<super::TransactionId, u64>,
}

impl RecoveryResult {
    pub fn needs_redo(&self) -> Vec<super::TransactionId> {
        self.commit_lsns
            .keys()
            .filter(|tx_id| self.undo_lsns.contains_key(tx_id))
            .cloned()
            .collect()
    }

    pub fn needs_undo(&self) -> Vec<super::TransactionId> {
        self.transactions
            .iter()
            .filter(|(_, info)| info.state == super::TransactionState::Active)
            .map(|(tx_id, _)| *tx_id)
            .collect()
    }
}

/// 日志配置
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// 日志目录
    pub log_dir: PathBuf,
    /// 缓冲区大小
    pub buffer_size: usize,
    /// 刷新间隔
    pub flush_interval: Duration,
    /// 日志保留时间（秒）
    pub retain_duration_secs: u64,
    /// 是否启用异步写入
    pub async_write: bool,
    /// 检查点间隔
    pub checkpoint_interval: Duration,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("transaction_logs"),
            buffer_size: 100,
            flush_interval: Duration::from_secs(1),
            retain_duration_secs: 86400,
            async_write: true,
            checkpoint_interval: Duration::from_secs(300),
        }
    }
}

/// 日志统计信息
#[derive(Debug, Clone, Default)]
pub struct LogStats {
    pub records_written: u64,
    pub bytes_written: u64,
    pub records_read: u64,
    pub bytes_read: u64,
    pub flush_count: u64,
    pub checkpoint_count: u64,
}

impl LogStats {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_record() {
        let record = LogRecord::new(TransactionId::new(1), LogType::Begin);
        assert_eq!(record.log_type, LogType::Begin);

        let record = record
            .with_key("vertex:space1:v1")
            .with_old_value(b"old")
            .with_new_value(b"new");

        assert_eq!(record.key, Some("vertex:space1:v1".to_string()));
        assert_eq!(record.old_value, Some(b"old".to_vec()));
        assert_eq!(record.new_value, Some(b"new".to_vec()));
    }

    #[test]
    fn test_log_type() {
        assert!(LogType::Update.is_undoable());
        assert!(LogType::Insert.is_redoable());
        assert!(!LogType::Commit.is_undoable());
        assert!(!LogType::Begin.is_redoable());
    }

    #[test]
    fn test_transaction_log_write() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().to_path_buf();
        let config = LogConfig {
            log_dir: log_dir.clone(),
            buffer_size: 10,
            ..Default::default()
        };

        let log = TransactionLog::new(log_dir, config);
        let tx_id = TransactionId::new(1);

        let lsn = log.write_begin(tx_id).unwrap();
        assert!(lsn > 0);
    }

    #[test]
    fn test_recovery_result() {
        let mut result = RecoveryResult::default();
        let tx_id = TransactionId::new(1);

        result.transactions.insert(tx_id, TransactionInfo {
            state: TransactionState::Active,
            first_lsn: 1,
            last_lsn: 10,
        });

        result.commit_lsns.insert(tx_id, 10);

        let needs_undo = result.needs_undo();
        assert!(needs_undo.contains(&tx_id));
    }
}
