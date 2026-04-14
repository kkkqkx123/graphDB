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
    Nomic {
        task_type: NomicTaskType,
    },
    /// Stella task type
    Stella {
        task_type: StellaTaskType,
    },
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
            StellaTaskType::S2PQuery => "Instruct: Given a web search query, retrieve relevant passages. Query: ",
            StellaTaskType::S2SDocument => "Instruct: Given a web search query, retrieve relevant passages. Document: ",
            StellaTaskType::P2PQuery => "Instruct: Given a passage, retrieve relevant passages. Query: ",
            StellaTaskType::P2PDocument => "Instruct: Given a passage, retrieve relevant passages. Document: ",
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
