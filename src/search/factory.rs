use std::path::Path;
use std::sync::Arc;

use crate::search::engine::{SearchEngine, EngineType};
use crate::search::adapters::{Bm25SearchEngine, InversearchEngine, InversearchConfig};
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
                let engine = Bm25SearchEngine::open_or_create(&engine_path)?;
                Ok(Arc::new(engine))
            }
            EngineType::Inversearch => {
                let config = InversearchConfig {
                    persistence_path: Some(engine_path.with_extension("bin")),
                    ..Default::default()
                };
                let engine = if config.persistence_path.as_ref().unwrap().exists() {
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
                let engine = Bm25SearchEngine::open_or_create(&engine_path)?;
                Ok(Arc::new(engine))
            }
            EngineType::Inversearch => {
                let mut inv_config = config.inversearch.clone();
                inv_config.persistence_path = Some(engine_path.with_extension("bin"));

                let engine = if inv_config.persistence_path.as_ref().unwrap().exists() {
                    InversearchEngine::load(&engine_path.with_extension("bin"), inv_config)?
                } else {
                    InversearchEngine::new(inv_config)?
                };
                Ok(Arc::new(engine))
            }
        }
    }
}
