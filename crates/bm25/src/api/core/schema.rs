use std::collections::HashMap;
use tantivy::schema::TantivyDocument;
use tantivy::schema::{
    Field, IndexRecordOption, Schema, TextFieldIndexing, TextOptions, STORED, STRING,
};

const MIXED_TOKENIZER_NAME: &str = "mixed";

#[derive(Debug, Clone)]
pub struct IndexSchema {
    pub document_id: Field,
    pub tag_name: Field,
    pub field_name: Field,
    pub content: Field,
}

impl IndexSchema {
    pub fn new() -> Self {
        let mut schema_builder = Schema::builder();

        let document_id = schema_builder.add_text_field("document_id", STRING | STORED);

        let tag_name = schema_builder.add_text_field("tag_name", STRING | STORED);

        let field_name = schema_builder.add_text_field("field_name", STRING | STORED);

        let content_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(MIXED_TOKENIZER_NAME)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        let content = schema_builder.add_text_field("content", content_options);

        IndexSchema {
            document_id,
            tag_name,
            field_name,
            content,
        }
    }

    pub fn schema(&self) -> Schema {
        Self::build_schema()
    }

    fn build_schema() -> Schema {
        let mut schema_builder = Schema::builder();

        schema_builder.add_text_field("document_id", STRING | STORED);
        schema_builder.add_text_field("tag_name", STRING | STORED);
        schema_builder.add_text_field("field_name", STRING | STORED);

        let content_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(MIXED_TOKENIZER_NAME)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        schema_builder.add_text_field("content", content_options);

        schema_builder.build()
    }

    pub fn to_document(
        &self,
        document_id: &str,
        fields: &HashMap<String, String>,
    ) -> TantivyDocument {
        let mut doc = TantivyDocument::new();
        doc.add_text(self.document_id, document_id);

        for (key, value) in fields {
            match key.as_str() {
                "tag_name" => doc.add_text(self.tag_name, value),
                "field_name" => doc.add_text(self.field_name, value),
                "content" => doc.add_text(self.content, value),
                _ => {}
            }
        }

        doc
    }

    pub fn searchable_fields(&self) -> Vec<Field> {
        vec![self.content]
    }
}

impl Default for IndexSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let schema = IndexSchema::new();
        let built_schema = schema.schema();

        assert!(built_schema.get_field("document_id").is_ok());
        assert!(built_schema.get_field("tag_name").is_ok());
        assert!(built_schema.get_field("field_name").is_ok());
        assert!(built_schema.get_field("content").is_ok());
    }

    #[test]
    fn test_to_document_basic() {
        let schema = IndexSchema::new();
        let mut fields = HashMap::new();
        fields.insert("tag_name".to_string(), "person".to_string());
        fields.insert("field_name".to_string(), "description".to_string());
        fields.insert("content".to_string(), "Test content".to_string());

        let doc = schema.to_document("doc1", &fields);
        assert_eq!(doc.field_values().count(), 4);
    }

    #[test]
    fn test_searchable_fields() {
        let schema = IndexSchema::new();
        let fields = schema.searchable_fields();
        assert_eq!(fields.len(), 1);
    }
}
