use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use crate::config::common::fulltext::{
    FulltextConfig, FulltextEngineType, SyncConfig, SyncFailurePolicy, TantivyConfig, TokenizerKind,
};
