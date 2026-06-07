//! Text preprocessor for embeddings

use serde::{Deserialize, Serialize};

/// Preprocessor configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PreprocessorConfig {
    /// No preprocessing (default)
    #[default]
    None,
    /// Simple prefix
    Prefix { prefix: String },
    /// Template with {text} placeholder
    Template { template: String },
    /// Nomic-Embed task type
    Nomic { task_type: NomicTaskType },
    /// Stella task type
    Stella { task_type: StellaTaskType },
}

/// Text preprocessor trait
pub trait Preprocessor: Send + Sync {
    /// Preprocess a single text
    fn preprocess(&self, text: &str) -> String;

    /// Preprocess a batch of texts
    fn process_batch(&self, texts: &[&str]) -> Vec<String> {
        texts.iter().map(|&t| self.preprocess(t)).collect()
    }
}

/// No-op preprocessor
#[derive(Debug, Clone, Default)]
pub struct NoopPreprocessor;

impl Preprocessor for NoopPreprocessor {
    fn preprocess(&self, text: &str) -> String {
        text.to_string()
    }
}

/// Prefix preprocessor - adds a prefix to each text
#[derive(Debug, Clone)]
pub struct PrefixPreprocessor {
    prefix: String,
}

impl PrefixPreprocessor {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl Preprocessor for PrefixPreprocessor {
    fn preprocess(&self, text: &str) -> String {
        format!("{}{}", self.prefix, text)
    }
}

/// Template preprocessor - applies a template to each text
#[derive(Debug, Clone)]
pub struct TemplatePreprocessor {
    template: String,
}

impl TemplatePreprocessor {
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }
}

impl Preprocessor for TemplatePreprocessor {
    fn preprocess(&self, text: &str) -> String {
        self.template.replace("{{text}}", text)
    }
}

/// Nomic task type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NomicTaskType {
    SearchQuery,
    SearchDocument,
    Classification,
    Clustering,
}

/// Nomic preprocessor
#[derive(Debug, Clone)]
pub struct NomicPreprocessor {
    task_type: NomicTaskType,
}

impl NomicPreprocessor {
    pub fn new(task_type: NomicTaskType) -> Self {
        Self { task_type }
    }

    fn get_prefix(&self) -> &'static str {
        match self.task_type {
            NomicTaskType::SearchQuery => "search_query: ",
            NomicTaskType::SearchDocument => "search_document: ",
            NomicTaskType::Classification => "classification: ",
            NomicTaskType::Clustering => "clustering: ",
        }
    }
}

impl Preprocessor for NomicPreprocessor {
    fn preprocess(&self, text: &str) -> String {
        format!("{}{}", self.get_prefix(), text)
    }
}

/// Stella task type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum StellaTaskType {
    S2PQuery,
    S2SDocument,
    P2PQuery,
    P2PDocument,
}

/// Stella preprocessor
#[derive(Debug, Clone)]
pub struct StellaPreprocessor {
    task_type: StellaTaskType,
}

impl StellaPreprocessor {
    pub fn new(task_type: StellaTaskType) -> Self {
        Self { task_type }
    }

    fn get_prefix(&self) -> &'static str {
        match self.task_type {
            StellaTaskType::S2PQuery => {
                "Instruct: Given a web search query, retrieve relevant passages. Query: "
            }
            StellaTaskType::S2SDocument => {
                "Instruct: Given a web search query, retrieve relevant passages. Document: "
            }
            StellaTaskType::P2PQuery => {
                "Instruct: Given a passage, retrieve relevant passages. Query: "
            }
            StellaTaskType::P2PDocument => {
                "Instruct: Given a passage, retrieve relevant passages. Document: "
            }
        }
    }
}

impl Preprocessor for StellaPreprocessor {
    fn preprocess(&self, text: &str) -> String {
        format!("{}{}", self.get_prefix(), text)
    }
}

/// Chained preprocessor - combines multiple preprocessors
pub struct ChainedPreprocessor {
    preprocessors: Vec<Box<dyn Preprocessor>>,
}

impl ChainedPreprocessor {
    pub fn new(preprocessors: Vec<Box<dyn Preprocessor>>) -> Self {
        Self { preprocessors }
    }
}

impl Preprocessor for ChainedPreprocessor {
    fn preprocess(&self, text: &str) -> String {
        self.preprocessors
            .iter()
            .fold(text.to_string(), |acc, p| p.preprocess(&acc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_preprocessor() {
        let p = NoopPreprocessor;
        assert_eq!(p.preprocess("hello world"), "hello world");
    }

    #[test]
    fn test_noop_preprocessor_empty() {
        let p = NoopPreprocessor;
        assert_eq!(p.preprocess(""), "");
    }

    #[test]
    fn test_prefix_preprocessor() {
        let p = PrefixPreprocessor::new("query: ");
        assert_eq!(p.preprocess("rust"), "query: rust");
    }

    #[test]
    fn test_prefix_preprocessor_no_space() {
        let p = PrefixPreprocessor::new("cls:");
        assert_eq!(p.preprocess("text"), "cls:text");
    }

    #[test]
    fn test_template_preprocessor() {
        let p = TemplatePreprocessor::new("classify: {{text}}");
        assert_eq!(p.preprocess("hello"), "classify: hello");
    }

    #[test]
    fn test_template_preprocessor_multiple_placeholders() {
        let p = TemplatePreprocessor::new("{{text}} and {{text}}");
        assert_eq!(p.preprocess("x"), "x and x");
    }

    #[test]
    fn test_nomic_preprocessor_search_query() {
        let p = NomicPreprocessor::new(NomicTaskType::SearchQuery);
        assert_eq!(p.preprocess("rust"), "search_query: rust");
    }

    #[test]
    fn test_nomic_preprocessor_search_document() {
        let p = NomicPreprocessor::new(NomicTaskType::SearchDocument);
        assert_eq!(p.preprocess("doc"), "search_document: doc");
    }

    #[test]
    fn test_nomic_preprocessor_classification() {
        let p = NomicPreprocessor::new(NomicTaskType::Classification);
        assert_eq!(p.preprocess("text"), "classification: text");
    }

    #[test]
    fn test_nomic_preprocessor_clustering() {
        let p = NomicPreprocessor::new(NomicTaskType::Clustering);
        assert_eq!(p.preprocess("data"), "clustering: data");
    }

    #[test]
    fn test_stella_preprocessor_s2p_query() {
        let p = StellaPreprocessor::new(StellaTaskType::S2PQuery);
        assert!(p.preprocess("test").contains("web search query"));
        assert!(p.preprocess("test").contains("test"));
    }

    #[test]
    fn test_stella_preprocessor_s2s_document() {
        let p = StellaPreprocessor::new(StellaTaskType::S2SDocument);
        assert!(p.preprocess("doc").contains("Document:"));
    }

    #[test]
    fn test_stella_preprocessor_p2p_query() {
        let p = StellaPreprocessor::new(StellaTaskType::P2PQuery);
        assert!(p.preprocess("q").contains("passage"));
        assert!(p.preprocess("q").contains("q"));
    }

    #[test]
    fn test_stella_preprocessor_p2p_document() {
        let p = StellaPreprocessor::new(StellaTaskType::P2PDocument);
        assert!(p.preprocess("d").contains("Document:"));
    }

    #[test]
    fn test_chained_preprocessor() {
        let chain = ChainedPreprocessor::new(vec![
            Box::new(PrefixPreprocessor::new("Q: ")),
            Box::new(TemplatePreprocessor::new("{{text}} [END]")),
        ]);
        assert_eq!(chain.preprocess("hello"), "Q: hello [END]");
    }

    #[test]
    fn test_chained_preprocessor_empty_chain() {
        let chain = ChainedPreprocessor::new(vec![]);
        assert_eq!(chain.preprocess("text"), "text");
    }

    #[test]
    fn test_batch_processing() {
        let p = PrefixPreprocessor::new("> ");
        let texts = vec!["a", "b", "c"];
        let result = p.process_batch(&texts);
        assert_eq!(result, vec!["> a", "> b", "> c"]);
    }
}
