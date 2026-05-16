pub mod common;
pub mod factory;
pub mod manager;
pub mod storage_enum;
pub mod tantivy;

pub use common::{r#trait::StorageInterface, types::{Bm25Stats, StorageInfo}};
pub use factory::StorageFactory;
pub use manager::{DefaultStorage, MutableStorageManager, StorageManager, StorageManagerBuilder};
pub use storage_enum::StorageEnum;
pub use tantivy::TantivyStorage;
