use tantivy::schema::{Schema, Field, STRING, TEXT, STORED};
use tantivy::schema::TantivyDocument;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IndexSchema {
    pub document_id: Field,
    pub title: Field,
    pub content: Field,
}

impl IndexSchema {
    pub fn new() -> Self {
        let mut schema_builder = Schema::builder();
        let document_id = schema_builder.add_text_field("document_id", STRING | STORED);
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let content = schema_builder.add_text_field("content", TEXT | STORED);
        
        IndexSchema {
            document_id,
            title,
            content,
        }
    }

    pub fn schema(&self) -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("document_id", STRING | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("content", TEXT | STORED);
        schema_builder.build()
    }

    pub fn to_document(&self, document_id: &str, fields: &HashMap<String, String>) -> TantivyDocument {
        let mut doc = TantivyDocument::new();
        doc.add_text(self.document_id, document_id);
        
        for (key, value) in fields {
            if key == "title" {
                doc.add_text(self.title, value);
            } else if key == "content" {
                doc.add_text(self.content, value);
            }
        }
        
        doc
    }
}

impl Default for IndexSchema {
    fn default() -> Self {
        Self::new()
    }
}
