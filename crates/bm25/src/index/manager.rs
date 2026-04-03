use anyhow::Result;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tantivy::{schema::*, Index, IndexReader, IndexWriter, ReloadPolicy};

#[derive(Debug, Clone)]
pub struct IndexManagerConfig {
    pub writer_buffer_size: usize,
    pub reader_cache_enabled: bool,
}

impl Default for IndexManagerConfig {
    fn default() -> Self {
        Self {
            writer_buffer_size: 50_000_000,
            reader_cache_enabled: true,
        }
    }
}

impl IndexManagerConfig {
    pub fn new(writer_buffer_size: usize, reader_cache_enabled: bool) -> Self {
        Self {
            writer_buffer_size,
            reader_cache_enabled,
        }
    }

    pub fn with_writer_buffer_size(mut self, size: usize) -> Self {
        self.writer_buffer_size = size;
        self
    }

    pub fn with_reader_cache(mut self, enabled: bool) -> Self {
        self.reader_cache_enabled = enabled;
        self
    }
}

#[derive(Clone)]
pub struct IndexManager {
    index: Index,
    schema: Schema,
    config: IndexManagerConfig,
    cached_reader: Arc<RwLock<Option<IndexReader>>>,
}

impl IndexManager {
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::create_with_config(path, IndexManagerConfig::default())
    }

    pub fn create_with_config<P: AsRef<Path>>(path: P, config: IndexManagerConfig) -> Result<Self> {
        let path = path.as_ref();
        std::fs::create_dir_all(path)?;
        let schema = Self::build_schema();
        let index = Index::create_in_dir(path, schema.clone())?;
        Ok(Self {
            index,
            schema,
            config,
            cached_reader: Arc::new(RwLock::new(None)),
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open_with_config(path, IndexManagerConfig::default())
    }

    pub fn open_with_config<P: AsRef<Path>>(path: P, config: IndexManagerConfig) -> Result<Self> {
        let index = Index::open_in_dir(path)?;
        let schema = index.schema();
        Ok(Self {
            index,
            schema: schema.clone(),
            config,
            cached_reader: Arc::new(RwLock::new(None)),
        })
    }

    fn build_schema() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("document_id", STRING | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("content", TEXT | STORED);
        schema_builder.build()
    }

    pub fn writer(&self) -> Result<IndexWriter> {
        Ok(self.index.writer(self.config.writer_buffer_size)?)
    }

    pub fn reader(&self) -> Result<IndexReader> {
        if self.config.reader_cache_enabled {
            if let Ok(reader_guard) = self.cached_reader.read() {
                if let Some(reader) = reader_guard.as_ref() {
                    return Ok(reader.clone());
                }
            }

            let new_reader = self.create_reader()?;

            if let Ok(mut writer_guard) = self.cached_reader.write() {
                *writer_guard = Some(new_reader.clone());
            }

            Ok(new_reader)
        } else {
            self.create_reader()
        }
    }

    fn create_reader(&self) -> Result<IndexReader> {
        Ok(self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?)
    }

    pub fn reload_reader(&self) -> Result<IndexReader> {
        let new_reader = self.create_reader()?;

        if let Ok(mut writer_guard) = self.cached_reader.write() {
            *writer_guard = Some(new_reader.clone());
        }

        Ok(new_reader)
    }

    pub fn clear_reader_cache(&self) {
        if let Ok(mut writer_guard) = self.cached_reader.write() {
            *writer_guard = None;
        }
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn config(&self) -> &IndexManagerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_open() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("test_index");

        let manager = IndexManager::create(&path)?;
        assert!(manager.reader().is_ok());

        let opened = IndexManager::open(&path)?;
        assert!(opened.reader().is_ok());

        Ok(())
    }

    #[test]
    fn test_config() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("test_config");

        let config = IndexManagerConfig::default()
            .with_writer_buffer_size(100_000_000)
            .with_reader_cache(false);

        let manager = IndexManager::create_with_config(&path, config)?;
        assert_eq!(manager.config().writer_buffer_size, 100_000_000);
        assert!(!manager.config().reader_cache_enabled);

        Ok(())
    }

    #[test]
    fn test_reader_caching() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("test_cache");

        let manager = IndexManager::create(&path)?;

        let reader1 = manager.reader()?;
        let reader2 = manager.reader()?;

        assert!(std::ptr::eq(&reader1, &reader2));

        Ok(())
    }

    #[test]
    fn test_reload_reader() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("test_reload");

        let manager = IndexManager::create(&path)?;
        let reader1 = manager.reader()?;

        manager.clear_reader_cache();
        let reader2 = manager.reload_reader()?;

        assert!(!std::ptr::eq(&reader1, &reader2));

        Ok(())
    }
}
