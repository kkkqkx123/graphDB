//! Hot and Cold Cache Manager
//!
//! Core Manager for data flow and management of the three-tier caching architecture

use crate::error::Result;
use crate::r#type::{DocId, EnrichedSearchResults, SearchResults};
use crate::storage::cold_warm_cache::{
    background::BackgroundTaskManager,
    config::{ColdWarmCacheConfig, WALConfig},
};
use crate::storage::common::{
    compression::{compress_data, decompress_data},
    io::{atomic_write, load_from_file},
    types::FileStorageData,
    StorageInfo,
};
use crate::{Index, StorageInterface};
use dashmap::DashMap;
use memmap2::Mmap;
use postcard::{from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::fs as tokio_fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexData {
    pub data: HashMap<String, Vec<DocId>>,
    pub context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    pub documents: HashMap<DocId, String>,
}

impl From<FileStorageData> for IndexData {
    fn from(file_data: FileStorageData) -> Self {
        Self {
            data: file_data.data,
            context_data: file_data.context_data,
            documents: file_data.documents,
        }
    }
}

impl From<IndexData> for FileStorageData {
    fn from(index_data: IndexData) -> Self {
        Self {
            version: "1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: index_data.data,
            context_data: index_data.context_data,
            documents: index_data.documents,
        }
    }
}

#[derive(Debug)]
struct HotCacheEntry {
    data: IndexData,
    last_access: Instant,
    size: usize,
}

impl HotCacheEntry {
    fn new(data: IndexData, size: usize) -> Self {
        Self {
            data,
            last_access: Instant::now(),
            size,
        }
    }
}

#[derive(Debug)]
struct WarmCacheEntry {
    file_path: PathBuf,
    last_access: Instant,
    #[allow(dead_code)]
    size: usize,
    compressed: bool,
}

impl WarmCacheEntry {
    fn new(file_path: PathBuf, size: usize, compressed: bool) -> Self {
        Self {
            file_path,
            last_access: Instant::now(),
            size,
            compressed,
        }
    }

    async fn load(&self) -> Result<FileStorageData> {
        let file = tokio_fs::File::open(&self.file_path).await?;
        let std_file = file.into_std().await;
        let mmap = unsafe { Mmap::map(&std_file) }
            .map_err(|e| crate::error::StorageError::Generic(e.to_string()))?;

        let data: FileStorageData = from_bytes(&mmap[..])
            .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;

        if self.compressed {
            let bytes = to_allocvec(&data)?;
            let decompressed = decompress_data(&bytes)?;
            let decompressed_data = from_bytes::<FileStorageData>(&decompressed)
                .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;
            Ok(decompressed_data)
        } else {
            Ok(data)
        }
    }
}

struct ColdStorageEntry {
    file_path: PathBuf,
    compressed: bool,
}

impl ColdStorageEntry {
    fn new(file_path: PathBuf, compressed: bool) -> Self {
        Self {
            file_path,
            compressed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WALEntry {
    Add {
        index_name: String,
        data: IndexData,
        timestamp: u64,
    },
    Remove {
        index_name: String,
        timestamp: u64,
    },
    Update {
        index_name: String,
        data: IndexData,
        timestamp: u64,
    },
}

pub struct WALManager {
    config: WALConfig,
    current_wal: Arc<Mutex<WALWriter>>,
    wal_files: Arc<Mutex<Vec<PathBuf>>>,
    checkpoint_info: Arc<Mutex<CheckpointInfo>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CheckpointInfo {
    timestamp: u64,
    wal_sequence: u64,
    index_states: HashMap<String, IndexData>,
}

struct WALWriter {
    file_path: PathBuf,
    file: tokio::fs::File,
    size: usize,
    entry_count: usize,
}

#[allow(dead_code)]
impl WALWriter {
    async fn new(file_path: PathBuf) -> Result<Self> {
        let file = tokio_fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .await?;

        let metadata = tokio_fs::metadata(&file_path).await?;
        let size = metadata.len() as usize;

        Ok(Self {
            file_path,
            file,
            size,
            entry_count: 0,
        })
    }

    async fn write_entry(&mut self, entry: &WALEntry) -> Result<()> {
        let bytes = to_allocvec(entry)
            .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;
        let len = bytes.len() as u32;

        self.file.write_all(&len.to_le_bytes()).await?;
        self.file.write_all(&bytes).await?;
        self.file.flush().await?;

        self.size += 4 + bytes.len();
        self.entry_count += 1;

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        self.file.flush().await?;
        self.file.sync_all().await?;
        Ok(())
    }

    fn size(&self) -> usize {
        self.size
    }

    fn entry_count(&self) -> usize {
        #[allow(dead_code)]
        let _ = self.entry_count;
        self.entry_count
    }
}

impl WALManager {
    async fn new(config: WALConfig) -> Result<Self> {
        tokio_fs::create_dir_all(&config.base_path).await?;

        let wal_path = config.base_path.join("wal.current");
        let current_wal = WALWriter::new(wal_path).await?;

        let checkpoint_path = config.base_path.join("checkpoint.json");
        let checkpoint_info = if checkpoint_path.exists() {
            let bytes = tokio_fs::read(&checkpoint_path).await?;
            if bytes.is_empty() {
                CheckpointInfo::default()
            } else {
                serde_json::from_slice(&bytes).unwrap_or_default()
            }
        } else {
            CheckpointInfo::default()
        };

        let mut wal_files = Vec::new();
        let mut entries = tokio_fs::read_dir(&config.base_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "wal") {
                wal_files.push(path);
            }
        }

        Ok(Self {
            config,
            current_wal: Arc::new(Mutex::new(current_wal)),
            wal_files: Arc::new(Mutex::new(wal_files)),
            checkpoint_info: Arc::new(Mutex::new(checkpoint_info)),
        })
    }

    async fn write_entry(&self, entry: WALEntry) -> Result<()> {
        let mut writer = self.current_wal.lock().await;
        writer.write_entry(&entry).await?;

        if writer.size() > self.config.max_wal_size {
            self.rotate_wal(&mut writer).await?;
        }

        Ok(())
    }

    async fn rotate_wal(&self, writer: &mut WALWriter) -> Result<()> {
        writer.flush().await?;

        let old_path = writer.file_path.clone();
        let new_path = self.config.base_path.join(format!(
            "wal.{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

        tokio_fs::rename(&old_path, &new_path).await?;

        self.wal_files.lock().await.push(new_path);

        *writer = WALWriter::new(old_path).await?;

        self.cleanup_old_wals().await?;

        Ok(())
    }

    async fn cleanup_old_wals(&self) -> Result<()> {
        let mut wal_files = self.wal_files.lock().await;
        while wal_files.len() > self.config.max_wal_files {
            if !wal_files.is_empty() {
                let old_file = wal_files.remove(0);
                let _ = tokio_fs::remove_file(&old_file).await;
            }
        }
        Ok(())
    }

    async fn create_checkpoint(&self, index_states: HashMap<String, IndexData>) -> Result<()> {
        let checkpoint = CheckpointInfo {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            wal_sequence: 0,
            index_states,
        };

        let checkpoint_path = self.config.base_path.join("checkpoint.json");
        let bytes = serde_json::to_vec(&checkpoint)?;
        atomic_write(&checkpoint_path, &bytes).await?;

        *self.checkpoint_info.lock().await = checkpoint;

        Ok(())
    }

    async fn recover(&self) -> Result<HashMap<String, IndexData>> {
        let checkpoint = self.checkpoint_info.lock().await.clone();
        let mut index_states = checkpoint.index_states;

        let current_wal_path = self.config.base_path.join("wal.current");
        if current_wal_path.exists() {
            let entries = self.read_wal_file(&current_wal_path).await?;
            for entry in entries {
                match entry {
                    WALEntry::Add {
                        index_name, data, ..
                    } => {
                        index_states.insert(index_name, data);
                    }
                    WALEntry::Remove { index_name, .. } => {
                        index_states.remove(&index_name);
                    }
                    WALEntry::Update {
                        index_name, data, ..
                    } => {
                        index_states.insert(index_name, data);
                    }
                }
            }
        }

        Ok(index_states)
    }

    async fn read_wal_file(&self, path: &PathBuf) -> Result<Vec<WALEntry>> {
        let metadata = match tokio_fs::metadata(path).await {
            Ok(m) => m,
            Err(_) => return Ok(Vec::new()),
        };

        if metadata.len() == 0 {
            return Ok(Vec::new());
        }

        let bytes = tokio_fs::read(path).await?;
        let mut entries = Vec::new();
        let mut offset = 0;

        while offset + 4 <= bytes.len() {
            let len = u32::from_le_bytes(
                bytes[offset..offset + 4].try_into().unwrap()
            ) as usize;
            offset += 4;

            if offset + len > bytes.len() {
                break;
            }

            let entry = from_bytes::<WALEntry>(&bytes[offset..offset + len])
                .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;
            entries.push(entry);
            offset += len;
        }

        Ok(entries)
    }
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub hot_hit: AtomicU64,
    pub warm_hit: AtomicU64,
    pub cold_hit: AtomicU64,
    pub miss: AtomicU64,
    pub evict_to_warm: AtomicU64,
    pub evict_to_cold: AtomicU64,
    pub wal_writes: AtomicU64,
    pub wal_rotations: AtomicU64,
    pub checkpoint_count: AtomicU64,
    pub flush_count: AtomicU64,
    pub merge_count: AtomicU64,
    pub cleanup_count: AtomicU64,
}

impl CacheStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total_hits(&self) -> u64 {
        self.hot_hit.load(Ordering::Relaxed)
            + self.warm_hit.load(Ordering::Relaxed)
            + self.cold_hit.load(Ordering::Relaxed)
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.total_hits() + self.miss.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        self.total_hits() as f64 / total as f64
    }

    pub fn hot_hit_rate(&self) -> f64 {
        let total = self.total_hits();
        if total == 0 {
            return 0.0;
        }
        self.hot_hit.load(Ordering::Relaxed) as f64 / total as f64
    }

    pub fn warm_hit_rate(&self) -> f64 {
        let total = self.total_hits();
        if total == 0 {
            return 0.0;
        }
        self.warm_hit.load(Ordering::Relaxed) as f64 / total as f64
    }

    pub fn cold_hit_rate(&self) -> f64 {
        let total = self.total_hits();
        if total == 0 {
            return 0.0;
        }
        self.cold_hit.load(Ordering::Relaxed) as f64 / total as f64
    }

    pub fn reset(&self) {
        self.hot_hit.store(0, Ordering::Relaxed);
        self.warm_hit.store(0, Ordering::Relaxed);
        self.cold_hit.store(0, Ordering::Relaxed);
        self.miss.store(0, Ordering::Relaxed);
        self.evict_to_warm.store(0, Ordering::Relaxed);
        self.evict_to_cold.store(0, Ordering::Relaxed);
        self.wal_writes.store(0, Ordering::Relaxed);
        self.wal_rotations.store(0, Ordering::Relaxed);
        self.checkpoint_count.store(0, Ordering::Relaxed);
        self.flush_count.store(0, Ordering::Relaxed);
        self.merge_count.store(0, Ordering::Relaxed);
        self.cleanup_count.store(0, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> CacheStatsSnapshot {
        CacheStatsSnapshot {
            hot_hit: self.hot_hit.load(Ordering::Relaxed),
            warm_hit: self.warm_hit.load(Ordering::Relaxed),
            cold_hit: self.cold_hit.load(Ordering::Relaxed),
            miss: self.miss.load(Ordering::Relaxed),
            evict_to_warm: self.evict_to_warm.load(Ordering::Relaxed),
            evict_to_cold: self.evict_to_cold.load(Ordering::Relaxed),
            wal_writes: self.wal_writes.load(Ordering::Relaxed),
            wal_rotations: self.wal_rotations.load(Ordering::Relaxed),
            checkpoint_count: self.checkpoint_count.load(Ordering::Relaxed),
            flush_count: self.flush_count.load(Ordering::Relaxed),
            merge_count: self.merge_count.load(Ordering::Relaxed),
            cleanup_count: self.cleanup_count.load(Ordering::Relaxed),
            total_hits: self.total_hits(),
            hit_rate: self.hit_rate(),
            hot_hit_rate: self.hot_hit_rate(),
            warm_hit_rate: self.warm_hit_rate(),
            cold_hit_rate: self.cold_hit_rate(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStatsSnapshot {
    pub hot_hit: u64,
    pub warm_hit: u64,
    pub cold_hit: u64,
    pub miss: u64,
    pub evict_to_warm: u64,
    pub evict_to_cold: u64,
    pub wal_writes: u64,
    pub wal_rotations: u64,
    pub checkpoint_count: u64,
    pub flush_count: u64,
    pub merge_count: u64,
    pub cleanup_count: u64,
    pub total_hits: u64,
    pub hit_rate: f64,
    pub hot_hit_rate: f64,
    pub warm_hit_rate: f64,
    pub cold_hit_rate: f64,
}

#[allow(dead_code)]
pub struct ColdWarmCacheManager {
    config: ColdWarmCacheConfig,
    hot_cache: Arc<DashMap<String, Arc<HotCacheEntry>>>,
    warm_cache: Arc<DashMap<String, WarmCacheEntry>>,
    cold_storage: Arc<DashMap<String, ColdStorageEntry>>,
    wal_manager: Option<Arc<WALManager>>,
    stats: Arc<CacheStats>,
    hot_cache_size: AtomicUsize,
    warm_cache_size: AtomicUsize,
    background_tasks: Arc<Mutex<Option<BackgroundTaskManager>>>,
    current_wal_sequence: AtomicU64,
}

impl ColdWarmCacheManager {
    pub async fn new() -> Result<Arc<Self>> {
        Self::with_config(ColdWarmCacheConfig::default()).await
    }

    pub async fn with_config(config: ColdWarmCacheConfig) -> Result<Arc<Self>> {
        tokio_fs::create_dir_all(&config.cold_storage_path).await?;
        tokio_fs::create_dir_all(&config.wal_path).await?;

        let wal_manager = if config.wal_enabled {
            let wal_config = WALConfig {
                base_path: config.wal_path.clone(),
                max_wal_size: config.wal_max_size,
                max_wal_files: config.wal_max_files,
                flush_interval: config.wal_flush_interval,
                auto_rotate: config.wal_auto_rotate,
                compression: config.cold_storage_compression,
                compression_level: config.cold_storage_compression_level,
            };
            Some(Arc::new(WALManager::new(wal_config).await?))
        } else {
            None
        };

        let manager = Self {
            config: config.clone(),
            hot_cache: Arc::new(DashMap::new()),
            warm_cache: Arc::new(DashMap::new()),
            cold_storage: Arc::new(DashMap::new()),
            wal_manager,
            stats: Arc::new(CacheStats::new()),
            hot_cache_size: AtomicUsize::new(0),
            warm_cache_size: AtomicUsize::new(0),
            background_tasks: Arc::new(Mutex::new(None)),
            current_wal_sequence: AtomicU64::new(0),
        };

        manager.recover().await?;

        if manager.config.warmup_enabled {
            manager.warmup().await?;
        }

        let manager_arc = Arc::new(manager);
        let bg_manager = BackgroundTaskManager::new(manager_arc.clone());
        *manager_arc.background_tasks.lock().await = Some(bg_manager);

        Ok(manager_arc)
    }

    async fn recover(&self) -> Result<()> {
        if let Some(wal_manager) = &self.wal_manager {
            let index_states = wal_manager.recover().await?;
            for (index_name, data) in index_states {
                let size = self.calculate_data_size(&data);
                if size < self.config.hot_cache_max_size / 10 {
                    let entry = Arc::new(HotCacheEntry::new(data, size));
                    self.hot_cache.insert(index_name, entry);
                    self.hot_cache_size.fetch_add(size, Ordering::Relaxed);
                } else {
                    self.persist_to_cold_storage(&index_name, &data).await?;
                }
            }
        }
        Ok(())
    }

    async fn warmup(&self) -> Result<()> {
        let mut cold_entries: Vec<(String, std::path::PathBuf, u64)> = Vec::new();

        let mut entries = tokio_fs::read_dir(&self.config.cold_storage_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "cold") {
                if let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy().to_string();
                    if let Ok(meta) = tokio_fs::metadata(&path).await {
                        cold_entries.push((name, path, meta.len()));
                    }
                }
            }
        }

        cold_entries.sort_by(|a, b| b.2.cmp(&a.2));

        let warmup_count = cold_entries.len().min(self.config.warmup_max_entries);
        for (name, path, _size) in cold_entries.into_iter().take(warmup_count) {
            if self.hot_cache.contains_key(&name) || self.warm_cache.contains_key(&name) {
                continue;
            }

            let file_data = load_from_file(&path).await?;
            let data: IndexData = file_data.into();
            self.persist_to_warm(&name, &data).await?;
        }

        tracing::info!("Warmup completed: loaded {} entries into warm cache", warmup_count);
        Ok(())
    }

    fn calculate_data_size(&self, data: &IndexData) -> usize {
        let mut size = 0;
        for (k, v) in &data.data {
            size += k.len() + v.len() * std::mem::size_of::<DocId>();
        }
        for (ctx, map) in &data.context_data {
            size += ctx.len();
            for (k, v) in map {
                size += k.len() + v.len() * std::mem::size_of::<DocId>();
            }
        }
        for doc in data.documents.values() {
            size += std::mem::size_of::<DocId>() + doc.len();
        }
        size
    }

    pub async fn get_index(&self, index_name: &str) -> Result<Option<IndexData>> {
        if let Some(entry) = self.hot_cache.get(index_name) {
            self.stats.hot_hit.fetch_add(1, Ordering::Relaxed);
            return Ok(Some(entry.data.clone()));
        }

        if let Some(entry) = self.warm_cache.get(index_name) {
            let file_data = entry.load().await?;
            let data: IndexData = file_data.into();
            self.stats.warm_hit.fetch_add(1, Ordering::Relaxed);
            self.promote_to_hot(index_name, data.clone()).await?;
            return Ok(Some(data));
        }

        if let Some(entry) = self.cold_storage.get(index_name) {
            let file_data = load_from_file(&entry.file_path).await?;
            let data: IndexData = if entry.compressed {
                let bytes = to_allocvec(&file_data)?;
                let decompressed = decompress_data(&bytes)?;
                let decompressed_data = from_bytes::<IndexData>(&decompressed)
                    .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;
                decompressed_data
            } else {
                file_data.into()
            };
            self.stats.cold_hit.fetch_add(1, Ordering::Relaxed);
            self.promote_to_hot(index_name, data.clone()).await?;
            return Ok(Some(data));
        }

        self.stats.miss.fetch_add(1, Ordering::Relaxed);
        Ok(None)
    }

    pub async fn insert_index(&self, index_name: &str, data: IndexData) -> Result<()> {
        if let Some(ref wal_manager) = self.wal_manager {
            let entry = WALEntry::Add {
                index_name: index_name.to_string(),
                data: data.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            wal_manager.write_entry(entry).await?;
            self.stats.wal_writes.fetch_add(1, Ordering::Relaxed);
        }

        let size = self.calculate_data_size(&data);
        let entry = Arc::new(HotCacheEntry::new(data, size));
        self.hot_cache.insert(index_name.to_string(), entry);
        self.hot_cache_size.fetch_add(size, Ordering::Relaxed);

        self.evict_if_needed().await?;

        Ok(())
    }

    pub fn get_flush_interval(&self) -> Duration {
        self.config.flush_interval
    }

    pub fn get_merge_interval(&self) -> Duration {
        self.config.merge_interval
    }

    pub fn get_cleanup_interval(&self) -> Duration {
        self.config.cleanup_interval
    }

    pub fn get_checkpoint_interval(&self) -> Duration {
        self.config.checkpoint_interval
    }

    pub async fn update_index(&self, index_name: &str, data: IndexData) -> Result<()> {
        if let Some(ref wal_manager) = self.wal_manager {
            let entry = WALEntry::Update {
                index_name: index_name.to_string(),
                data: data.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            wal_manager.write_entry(entry).await?;
            self.stats.wal_writes.fetch_add(1, Ordering::Relaxed);
        }

        self.hot_cache.remove(index_name);
        self.warm_cache.remove(index_name);

        let size = self.calculate_data_size(&data);
        let entry = Arc::new(HotCacheEntry::new(data, size));
        self.hot_cache.insert(index_name.to_string(), entry);
        self.hot_cache_size.fetch_add(size, Ordering::Relaxed);

        self.evict_if_needed().await?;

        Ok(())
    }

    pub async fn remove_index(&self, index_name: &str) -> Result<()> {
        if let Some(ref wal_manager) = self.wal_manager {
            let entry = WALEntry::Remove {
                index_name: index_name.to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            wal_manager.write_entry(entry).await?;
            self.stats.wal_writes.fetch_add(1, Ordering::Relaxed);
        }

        if let Some((_, entry)) = self.hot_cache.remove(index_name) {
            self.hot_cache_size.fetch_sub(entry.size, Ordering::Relaxed);
        }
        self.warm_cache.remove(index_name);
        self.cold_storage.remove(index_name);

        Ok(())
    }

    async fn promote_to_hot(&self, index_name: &str, data: IndexData) -> Result<()> {
        let size = self.calculate_data_size(&data);
        if size < self.config.hot_cache_max_size / 10 {
            let entry = Arc::new(HotCacheEntry::new(data, size));
            self.hot_cache.insert(index_name.to_string(), entry);
            self.hot_cache_size.fetch_add(size, Ordering::Relaxed);
            self.evict_if_needed().await?;
        }
        Ok(())
    }

    async fn evict_if_needed(&self) -> Result<()> {
        let hot_size = self.hot_cache_size.load(Ordering::Relaxed);
        if hot_size > self.config.hot_cache_max_size {
            self.evict_from_hot().await?;
        }

        let warm_size = self.warm_cache_size.load(Ordering::Relaxed);
        if warm_size > self.config.warm_cache_max_size {
            self.evict_from_warm().await?;
        }

        Ok(())
    }

    async fn evict_from_hot(&self) -> Result<()> {
        let mut keys_to_evict = Vec::new();

        let target_size = self.config.hot_cache_max_size * 8 / 10;

        let mut entries: Vec<_> = self
            .hot_cache
            .iter()
            .map(|entry| {
                let key = entry.key().clone();
                let last_access = entry.last_access;
                let size = entry.size;
                (key, last_access, size)
            })
            .collect();
        entries.sort_by_key(|(_, last_access, _)| *last_access);

        let current_size = self.hot_cache_size.load(Ordering::Relaxed);
        if current_size <= target_size {
            return Ok(());
        }

        let mut size_to_free = current_size - target_size;

        for (key, _, size) in entries {
            if size_to_free == 0 {
                break;
            }
            keys_to_evict.push(key.clone());
            size_to_free = size_to_free.saturating_sub(size);
        }

        for key in keys_to_evict {
            if let Some((_, entry)) = self.hot_cache.remove(&key) {
                self.hot_cache_size.fetch_sub(entry.size, Ordering::Relaxed);
                self.persist_to_warm(&key, &entry.data).await?;
                self.stats.evict_to_warm.fetch_add(1, Ordering::Relaxed);
            }
        }

        Ok(())
    }

    async fn evict_from_warm(&self) -> Result<()> {
        let mut keys_to_evict = Vec::new();

        let target_size = self.config.warm_cache_max_size * 8 / 10;
        let current_size = self.warm_cache_size.load(Ordering::Relaxed);

        if current_size <= target_size {
            return Ok(());
        }

        let mut entries: Vec<_> = self
            .warm_cache
            .iter()
            .map(|entry| {
                let key = entry.key().clone();
                let last_access = entry.last_access;
                let size = entry.size;
                (key, last_access, size)
            })
            .collect();
        entries.sort_by_key(|(_, last_access, _)| *last_access);

        let mut size_to_free = current_size - target_size;

        for (key, _, size) in entries {
            if size_to_free == 0 {
                break;
            }
            keys_to_evict.push(key.clone());
            size_to_free = size_to_free.saturating_sub(size);
        }

        for key in keys_to_evict {
            if let Some((_, entry)) = self.warm_cache.remove(&key) {
                self.warm_cache_size
                    .fetch_sub(entry.size, Ordering::Relaxed);
                let file_data = entry.load().await?;
                let data: IndexData = file_data.into();
                self.persist_to_cold_storage(&key, &data).await?;
                self.stats.evict_to_cold.fetch_add(1, Ordering::Relaxed);
            }
        }

        Ok(())
    }

    async fn persist_to_warm(&self, index_name: &str, data: &IndexData) -> Result<()> {
        let file_path = self
            .config
            .cold_storage_path
            .join(format!("{}.warm", index_name));
        let file_data: FileStorageData = data.clone().into();
        let bytes = to_allocvec(&file_data)
            .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;

        let (final_bytes, compressed) = if self.config.cold_storage_compression {
            (
                compress_data(&bytes, self.config.cold_storage_compression_level)?,
                true,
            )
        } else {
            (bytes, false)
        };

        atomic_write(&file_path, &final_bytes).await?;

        let entry = WarmCacheEntry::new(file_path, final_bytes.len(), compressed);
        self.warm_cache.insert(index_name.to_string(), entry);
        self.warm_cache_size
            .fetch_add(final_bytes.len(), Ordering::Relaxed);

        Ok(())
    }

    async fn persist_to_cold_storage(&self, index_name: &str, data: &IndexData) -> Result<()> {
        let file_path = self
            .config
            .cold_storage_path
            .join(format!("{}.cold", index_name));
        let file_data: FileStorageData = data.clone().into();
        let bytes = to_allocvec(&file_data)
            .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;

        let (final_bytes, compressed) = if self.config.cold_storage_compression {
            (
                compress_data(&bytes, self.config.cold_storage_compression_level)?,
                true,
            )
        } else {
            (bytes, false)
        };

        atomic_write(&file_path, &final_bytes).await?;

        let entry = ColdStorageEntry::new(file_path, compressed);
        self.cold_storage.insert(index_name.to_string(), entry);

        Ok(())
    }

    pub async fn flush_hot_to_warm(&self) -> Result<()> {
        let current_size = self.hot_cache_size.load(Ordering::Relaxed);
        let target_size = self.config.hot_cache_max_size / 2;

        if current_size <= target_size {
            return Ok(());
        }

        let mut entries: Vec<_> = self
            .hot_cache
            .iter()
            .map(|entry| {
                let key = entry.key().clone();
                let last_access = entry.last_access;
                let size = entry.size;
                (key, last_access, size)
            })
            .collect();
        entries.sort_by_key(|(_, last_access, _)| *last_access);

        let mut flushed_count = 0;
        for (key, _, _) in entries {
            if self.hot_cache_size.load(Ordering::Relaxed) <= target_size {
                break;
            }

            if let Some((_, entry)) = self.hot_cache.remove(&key) {
                self.hot_cache_size.fetch_sub(entry.size, Ordering::Relaxed);
                self.persist_to_warm(&key, &entry.data).await?;
                flushed_count += 1;
            }
        }

        self.stats
            .flush_count
            .fetch_add(flushed_count, Ordering::Relaxed);
        Ok(())
    }

    pub async fn merge_warm_to_cold(&self) -> Result<()> {
        let warm_entries: Vec<_> = self
            .warm_cache
            .iter()
            .map(|entry| {
                let key = entry.key().clone();
                let entry = entry.value();
                (key, entry.file_path.clone(), entry.size, entry.compressed)
            })
            .collect();

        for (key, file_path, size, _compressed) in warm_entries {
            let file_data = load_from_file(&file_path).await?;
            let data: IndexData = file_data.into();
            self.persist_to_cold_storage(&key, &data).await?;
            self.warm_cache.remove(&key);
            self.warm_cache_size.fetch_sub(size, Ordering::Relaxed);
        }

        self.stats.merge_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub async fn create_checkpoint(&self) -> Result<()> {
        if let Some(ref wal_manager) = self.wal_manager {
            let mut index_states = HashMap::new();
            for entry in self.hot_cache.iter() {
                index_states.insert(entry.key().clone(), entry.data.clone());
            }

            wal_manager.create_checkpoint(index_states).await?;
            self.stats.checkpoint_count.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    pub async fn cleanup_expired(&self) -> Result<()> {
        let now = Instant::now();
        let mut keys_to_remove = Vec::new();

        for entry in self.hot_cache.iter() {
            let elapsed = now.duration_since(entry.last_access);
            if elapsed > Duration::from_secs(3600) {
                keys_to_remove.push(entry.key().clone());
            }
        }

        for key in keys_to_remove {
            if let Some((_, entry)) = self.hot_cache.remove(&key) {
                self.hot_cache_size.fetch_sub(entry.size, Ordering::Relaxed);
            }
        }

        self.stats.cleanup_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert(
            "hot_cache_size".to_string(),
            format!(
                "{:.2} MB",
                self.hot_cache_size.load(Ordering::Relaxed) as f64 / 1024.0 / 1024.0
            ),
        );
        stats.insert(
            "warm_cache_size".to_string(),
            format!(
                "{:.2} MB",
                self.warm_cache_size.load(Ordering::Relaxed) as f64 / 1024.0 / 1024.0
            ),
        );
        stats.insert(
            "hot_hit_rate".to_string(),
            format!("{:.2}%", self.stats.hot_hit_rate() * 100.0),
        );
        stats.insert(
            "warm_hit_rate".to_string(),
            format!("{:.2}%", self.stats.warm_hit_rate() * 100.0),
        );
        stats.insert(
            "cold_hit_rate".to_string(),
            format!("{:.2}%", self.stats.cold_hit_rate() * 100.0),
        );
        stats.insert(
            "total_hit_rate".to_string(),
            format!("{:.2}%", self.stats.hit_rate() * 100.0),
        );
        stats
    }

    pub async fn clear(&self) -> Result<()> {
        self.hot_cache.clear();
        self.warm_cache.clear();
        self.cold_storage.clear();
        self.hot_cache_size.store(0, Ordering::Relaxed);
        self.warm_cache_size.store(0, Ordering::Relaxed);

        let mut entries = tokio_fs::read_dir(&self.config.cold_storage_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let _ = tokio_fs::remove_file(&path).await;
            }
        }

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        if let Some(bg_manager) = self.background_tasks.lock().await.take() {
            bg_manager.shutdown().await;
        }

        self.create_checkpoint().await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageInterface for ColdWarmCacheManager {
    async fn mount(&self, index: &Index) -> Result<()> {
        let index_name = "default";
        let data = IndexData {
            data: index.map.index.values()
                .flat_map(|m| m.iter())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            context_data: HashMap::new(),
            documents: index.documents.clone(),
        };
        self.insert_index(index_name, data).await
    }

    async fn open(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        self.shutdown().await
    }

    async fn destroy(&self) -> Result<()> {
        self.clear().await
    }

    async fn commit(&self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let index_name = "default";
        let data = IndexData {
            data: index.map.index.values()
                .flat_map(|m| m.iter())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            context_data: HashMap::new(),
            documents: index.documents.clone(),
        };

        if self.hot_cache.contains_key(index_name)
            || self.warm_cache.contains_key(index_name)
            || self.cold_storage.contains_key(index_name)
        {
            self.update_index(index_name, data).await
        } else {
            self.insert_index(index_name, data).await
        }
    }

    async fn get(
        &self,
        key: &str,
        ctx: Option<&str>,
        limit: usize,
        offset: usize,
        _resolve: bool,
        _enrich: bool,
    ) -> Result<SearchResults> {
        let mut all_results = Vec::new();

        // Collect all index names from all tiers
        let mut index_names: Vec<String> = Vec::new();
        for entry in self.hot_cache.iter() {
            index_names.push(entry.key().clone());
        }
        for entry in self.warm_cache.iter() {
            if !index_names.contains(entry.key()) {
                index_names.push(entry.key().clone());
            }
        }
        for entry in self.cold_storage.iter() {
            if !index_names.contains(entry.key()) {
                index_names.push(entry.key().clone());
            }
        }

        for name in index_names {
            if let Some(index_data) = self.get_index(&name).await? {
                let results = if let Some(ctx_key) = ctx {
                    index_data.context_data
                        .get(ctx_key)
                        .and_then(|m| m.get(key))
                        .cloned()
                        .unwrap_or_default()
                } else {
                    index_data.data
                        .get(key)
                        .cloned()
                        .unwrap_or_default()
                };
                all_results.extend(results);
            }
        }

        all_results.sort();
        all_results.dedup();

        let total = all_results.len();
        let start = offset.min(total);
        let end = (offset + limit).min(total);
        Ok(all_results[start..end].to_vec())
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let mut results = EnrichedSearchResults::new();

        let mut index_names: Vec<String> = Vec::new();
        for entry in self.hot_cache.iter() {
            index_names.push(entry.key().clone());
        }
        for entry in self.warm_cache.iter() {
            if !index_names.contains(entry.key()) {
                index_names.push(entry.key().clone());
            }
        }
        for entry in self.cold_storage.iter() {
            if !index_names.contains(entry.key()) {
                index_names.push(entry.key().clone());
            }
        }

        for name in index_names {
            if let Some(index_data) = self.get_index(&name).await? {
                for &id in ids {
                    if let Some(content) = index_data.documents.get(&id) {
                        results.push(crate::r#type::EnrichedSearchResult {
                            id,
                            doc: Some(serde_json::json!({
                                "content": content,
                                "id": id
                            })),
                            highlight: None,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        let mut index_names: Vec<String> = Vec::new();
        for entry in self.hot_cache.iter() {
            index_names.push(entry.key().clone());
        }
        for entry in self.warm_cache.iter() {
            if !index_names.contains(entry.key()) {
                index_names.push(entry.key().clone());
            }
        }
        for entry in self.cold_storage.iter() {
            if !index_names.contains(entry.key()) {
                index_names.push(entry.key().clone());
            }
        }

        for name in index_names {
            if let Ok(Some(index_data)) = self.get_index(&name).await {
                if index_data.documents.contains_key(&id) {
                    return Ok(true);
                }
                for doc_ids in index_data.data.values() {
                    if doc_ids.contains(&id) {
                        return Ok(true);
                    }
                }
                for ctx_map in index_data.context_data.values() {
                    for doc_ids in ctx_map.values() {
                        if doc_ids.contains(&id) {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    async fn remove(&self, ids: &[DocId]) -> Result<()> {
        let mut index_names: Vec<String> = Vec::new();
        for entry in self.hot_cache.iter() {
            index_names.push(entry.key().clone());
        }
        for entry in self.warm_cache.iter() {
            if !index_names.contains(entry.key()) {
                index_names.push(entry.key().clone());
            }
        }
        for entry in self.cold_storage.iter() {
            if !index_names.contains(entry.key()) {
                index_names.push(entry.key().clone());
            }
        }

        for name in index_names {
            if let Ok(Some(mut index_data)) = self.get_index(&name).await {
                for &id in ids {
                    index_data.documents.remove(&id);
                    for doc_ids in index_data.data.values_mut() {
                        doc_ids.retain(|&doc_id| doc_id != id);
                    }
                    for ctx_map in index_data.context_data.values_mut() {
                        for doc_ids in ctx_map.values_mut() {
                            doc_ids.retain(|&doc_id| doc_id != id);
                        }
                    }
                }
                self.update_index(&name, index_data).await?;
            }
        }

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.clear().await
    }

    async fn info(&self) -> Result<StorageInfo> {
        let doc_count: usize = {
            let mut count = 0;
            for entry in self.hot_cache.iter() {
                count += entry.data.documents.len();
            }
            count
        };

        Ok(StorageInfo {
            name: "ColdWarmCache".to_string(),
            version: "1.0".to_string(),
            size: self.hot_cache_size.load(Ordering::Relaxed) as u64,
            document_count: doc_count,
            index_count: self.hot_cache.len(),
            is_connected: true,
        })
    }
}
