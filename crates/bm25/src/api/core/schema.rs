use std::collections::HashMap;
use tantivy::schema::TantivyDocument;
use tantivy::schema::{
    Field, IndexRecordOption, Schema, TextFieldIndexing, TextOptions, STORED, STRING, TEXT,
};

const MIXED_TOKENIZER_NAME: &str = "mixed";

#[derive(Debug, Clone)]
pub struct IndexSchema {
    pub document_id: Field,
    pub title: Field,
    pub content: Field,
    pub entity_type: Field,
    pub raw_name: Field,
    pub keywords: Field,
    pub file_path: Field,
    pub module_name: Field,
}

impl IndexSchema {
    pub fn new() -> Self {
        let mut schema_builder = Schema::builder();

        let document_id = schema_builder.add_text_field("document_id", STRING | STORED);

        let title = schema_builder.add_text_field("title", TEXT | STORED);

        let content_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(MIXED_TOKENIZER_NAME)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        let content = schema_builder.add_text_field("content", content_options);

        let entity_type = schema_builder.add_text_field("entity_type", STRING | STORED);

        let raw_name_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(MIXED_TOKENIZER_NAME)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        let raw_name = schema_builder.add_text_field("raw_name", raw_name_options);

        let keywords_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(MIXED_TOKENIZER_NAME)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        let keywords = schema_builder.add_text_field("keywords", keywords_options);

        let file_path = schema_builder.add_text_field("file_path", STRING | STORED);
        let module_name = schema_builder.add_text_field("module_name", STRING | STORED);

        IndexSchema {
            document_id,
            title,
            content,
            entity_type,
            raw_name,
            keywords,
            file_path,
            module_name,
        }
    }

    pub fn schema(&self) -> Schema {
        Self::build_schema()
    }

    fn build_schema() -> Schema {
        let mut schema_builder = Schema::builder();

        schema_builder.add_text_field("document_id", STRING | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);

        let content_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(MIXED_TOKENIZER_NAME)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        schema_builder.add_text_field("content", content_options);

        schema_builder.add_text_field("entity_type", STRING | STORED);

        let raw_name_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(MIXED_TOKENIZER_NAME)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        schema_builder.add_text_field("raw_name", raw_name_options);

        let keywords_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(MIXED_TOKENIZER_NAME)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        schema_builder.add_text_field("keywords", keywords_options);

        schema_builder.add_text_field("file_path", STRING | STORED);
        schema_builder.add_text_field("module_name", STRING | STORED);

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
                "title" => doc.add_text(self.title, value),
                "content" => doc.add_text(self.content, value),
                "entity_type" => doc.add_text(self.entity_type, value),
                "raw_name" => doc.add_text(self.raw_name, value),
                "keywords" => doc.add_text(self.keywords, value),
                "file_path" => doc.add_text(self.file_path, value),
                "module_name" => doc.add_text(self.module_name, value),
                _ => {}
            }
        }

        doc
    }

    pub fn searchable_fields(&self) -> Vec<Field> {
        vec![self.title, self.content, self.raw_name, self.keywords]
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
        assert!(built_schema.get_field("title").is_ok());
        assert!(built_schema.get_field("content").is_ok());
        assert!(built_schema.get_field("entity_type").is_ok());
        assert!(built_schema.get_field("raw_name").is_ok());
        assert!(built_schema.get_field("keywords").is_ok());
        assert!(built_schema.get_field("file_path").is_ok());
        assert!(built_schema.get_field("module_name").is_ok());
    }

    #[test]
    fn test_to_document_basic() {
        let schema = IndexSchema::new();
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "Test Title".to_string());
        fields.insert("content".to_string(), "Test content".to_string());

        let doc = schema.to_document("doc1", &fields);
        assert_eq!(doc.field_values().count(), 3);
    }

    #[test]
    fn test_to_document_all_fields() {
        let schema = IndexSchema::new();
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "Test".to_string());
        fields.insert("content".to_string(), "Content".to_string());
        fields.insert("entity_type".to_string(), "function".to_string());
        fields.insert("raw_name".to_string(), "calculate_total".to_string());
        fields.insert("keywords".to_string(), "calculate total price".to_string());
        fields.insert("file_path".to_string(), "src/main.rs".to_string());
        fields.insert("module_name".to_string(), "math".to_string());

        let doc = schema.to_document("doc1", &fields);
        assert_eq!(doc.field_values().count(), 8);
    }

    #[test]
    fn test_searchable_fields() {
        let schema = IndexSchema::new();
        let fields = schema.searchable_fields();
        assert_eq!(fields.len(), 4);
    }
}
