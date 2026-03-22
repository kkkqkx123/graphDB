use tantivy::{
    schema::*,
    Index, IndexWriter, IndexReader, ReloadPolicy,
};
use std::path::Path;
use anyhow::Result;

#[derive(Clone)]
pub struct IndexManager {
    index: Index,
    schema: Schema,
}

impl IndexManager {
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let schema = Self::build_schema();
        let index = Index::create_in_dir(path, schema.clone())?;
        Ok(Self { index, schema })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let index = Index::open_in_dir(path)?;
        let schema = index.schema();
        Ok(Self { index, schema: schema.clone() })
    }

    fn build_schema() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("document_id", STRING | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("content", TEXT | STORED);
        schema_builder.build()
    }

    pub fn writer(&self) -> Result<IndexWriter> {
        Ok(self.index.writer(50_000_000)?)
    }

    pub fn reader(&self) -> Result<IndexReader> {
        Ok(self.index.reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?)
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn index(&self) -> &Index {
        &self.index
    }
}
