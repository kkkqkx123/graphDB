use std::path::Path;
use std::sync::Arc;

use crate::search::adapters::{Bm25Config, Bm25SearchEngine, InversearchConfig, InversearchEngine};
use crate::search::engine::{EngineType, SearchEngine};
use crate::search::error::SearchError;

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
                let engine = Bm25SearchEngine::open_or_create(&engine_path, Bm25Config::default())?;
                Ok(Arc::new(engine))
            }
            EngineType::Inversearch => {
                let config = InversearchConfig::builder()
                    .path(engine_path.with_extension("bin"))
                    .build();
                let engine = if engine_path.with_extension("bin").exists() {
                    InversearchEngine::load(&engine_path.with_extension("bin"), config)?
                } else {
                    InversearchEngine::new(config)?
                };
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
                let engine = Bm25SearchEngine::open_or_create(&engine_path, config.bm25.clone())?;
                Ok(Arc::new(engine))
            }
            EngineType::Inversearch => {
                let mut inv_config = config.inversearch.clone();
                inv_config.index_path = Some(engine_path.with_extension("bin"));

                let engine = if engine_path.with_extension("bin").exists() {
                    InversearchEngine::load(&engine_path.with_extension("bin"), inv_config)?
                } else {
                    InversearchEngine::new(inv_config)?
                };
                Ok(Arc::new(engine))
            }
        }
    }
}
