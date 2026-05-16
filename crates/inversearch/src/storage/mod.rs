pub mod common;
pub mod cold_warm_cache;
pub mod factory;
pub mod file;
pub mod memory;
pub mod manager;
pub mod persistence;

pub use common::{
    compression::{compress_data, decompress_data},
    config::{StorageConfig, StorageType},
    error::{StorageError, StorageResult},
    io::{atomic_write, get_file_size, load_from_file, remove_file_safe, save_to_file},
    FileStorageData, StorageInfo, StorageInterface,
};

pub use factory::StorageFactory;
pub use manager::{StorageManager, StorageManagerBuilder};
pub use persistence::{BackupInfo, IndexMetadata, IndexSnapshot, PersistenceManager};
