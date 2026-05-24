use std::path::Path;
use std::sync::Arc;

use crate::search::engine::{EngineType, SearchEngine};
use crate::search::error::SearchError;
use crate::search::tantivy_index::{TantivyConfig, TantivySearchEngine};

pub struct SearchEngineFactory;

impl SearchEngineFactory {
    pub fn create(
        engine_type: EngineType,
        index_name: &str,
        base_path: &Path,
    ) -> Result<Arc<dyn SearchEngine>, SearchError> {
        let engine_path = base_path.join(index_name);

        match engine_type {
            EngineType::Bm25 => {
                let engine =
                    TantivySearchEngine::open_or_create(&engine_path, TantivyConfig::default())?;
                Ok(Arc::new(engine))
            }
        }
    }

    pub fn from_config(
        engine_type: EngineType,
        index_name: &str,
        base_path: &Path,
        config: &crate::search::config::FulltextConfig,
    ) -> Result<Arc<dyn SearchEngine>, SearchError> {
        let engine_path = base_path.join(index_name);

        match engine_type {
            EngineType::Bm25 => {
                let engine =
                    TantivySearchEngine::open_or_create(&engine_path, config.tantivy.clone())?;
                Ok(Arc::new(engine))
            }
        }
    }
}
