//! Text preprocessing strategies for embedder
//!
//! Provides pluggable text preprocessing to support model-specific requirements
//! like task prefixes (Nomic-Embed) and instruction templates (Stella).

use serde::{Deserialize, Serialize};

/// Text preprocessing strategy trait
///
/// Implement this trait to create custom text preprocessing strategies.
/// Preprocessors can be chained using `ChainedPreprocessor`.
///
/// # Example
///
/// ```
/// use code_context_engine::embedding::preprocessor::{NomicPreprocessor, TextPreprocessor};
///
/// let preprocessor = NomicPreprocessor::search_document();
/// let processed = preprocessor.process("Hello world");
/// assert_eq!(processed, "search_document: Hello world");
/// ```
pub trait TextPreprocessor: Send + Sync {
    /// Process a single text
    fn process(&self, text: &str) -> String;

    /// Process multiple texts
    fn process_batch(&self, texts: &[&str]) -> Vec<String> {
        texts.iter().map(|text| self.process(text)).collect()
    }
}

/// No-op preprocessor (default behavior)
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopPreprocessor;

impl TextPreprocessor for NoopPreprocessor {
    fn process(&self, text: &str) -> String {
        text.to_string()
    }
}

/// Simple prefix preprocessor
#[derive(Debug, Clone)]
pub struct PrefixPreprocessor {
    prefix: String,
}

impl PrefixPreprocessor {
    /// Create a new prefix preprocessor
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl TextPreprocessor for PrefixPreprocessor {
    fn process(&self, text: &str) -> String {
        format!("{}{}", self.prefix, text)
    }
}

/// Template-based preprocessor with placeholder substitution
#[derive(Debug, Clone)]
pub struct TemplatePreprocessor {
    template: String,
}

impl TemplatePreprocessor {
    /// Create a new template preprocessor
    ///
    /// # Arguments
    ///
    /// * `template` - Template string with `{text}` placeholder
    ///
    /// # Example
    ///
    /// ```
    /// use code_context_engine::embedding::preprocessor::TemplatePreprocessor;
    ///
    /// let preprocessor = TemplatePreprocessor::new("Query: {text}");
    /// let result = preprocessor.process("Hello");
    /// assert_eq!(result, "Query: Hello");
    /// ```
    pub fn new(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
        }
    }

    /// Get the template string
    pub fn template(&self) -> &str {
        &self.template
    }
}

impl TextPreprocessor for TemplatePreprocessor {
    fn process(&self, text: &str) -> String {
        self.template.replace("{text}", text)
    }
}

/// Nomic-Embed task types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NomicTaskType {
    /// Document embedding for RAG indexing
    SearchDocument,
    /// Query embedding for RAG retrieval
    SearchQuery,
    /// Clustering task
    Clustering,
    /// Classification task
    Classification,
}

impl NomicTaskType {
    /// Get the prefix string for this task type
    pub fn as_prefix(&self) -> &'static str {
        match self {
            Self::SearchDocument => "search_document: ",
            Self::SearchQuery => "search_query: ",
            Self::Clustering => "clustering: ",
            Self::Classification => "classification: ",
        }
    }
}

/// Nomic-Embed preprocessor
///
/// Prepends task-specific prefixes as required by Nomic-Embed models.
/// Uses composition to reuse PrefixPreprocessor logic.
#[derive(Debug, Clone)]
pub struct NomicPreprocessor {
    task_type: NomicTaskType,
    inner: PrefixPreprocessor,
}

impl NomicPreprocessor {
    /// Create a new Nomic preprocessor with the given task type
    pub fn new(task_type: NomicTaskType) -> Self {
        Self {
            task_type,
            inner: PrefixPreprocessor::new(task_type.as_prefix()),
        }
    }

    /// Create a search_document preprocessor
    pub fn search_document() -> Self {
        Self::new(NomicTaskType::SearchDocument)
    }

    /// Create a search_query preprocessor
    pub fn search_query() -> Self {
        Self::new(NomicTaskType::SearchQuery)
    }

    /// Create a clustering preprocessor
    pub fn clustering() -> Self {
        Self::new(NomicTaskType::Clustering)
    }

    /// Create a classification preprocessor
    pub fn classification() -> Self {
        Self::new(NomicTaskType::Classification)
    }

    /// Get the task type
    pub fn task_type(&self) -> NomicTaskType {
        self.task_type
    }
}

impl TextPreprocessor for NomicPreprocessor {
    fn process(&self, text: &str) -> String {
        self.inner.process(text)
    }
}

/// Stella task types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StellaTaskType {
    /// Sentence-to-passage retrieval
    S2P,
    /// Sentence-to-sentence similarity
    S2S,
}

/// Stella-EN-400M preprocessor
///
/// Applies instruction templates as required by Stella models.
/// Uses composition to reuse TemplatePreprocessor logic.
#[derive(Debug, Clone)]
pub struct StellaPreprocessor {
    task_type: StellaTaskType,
    inner: TemplatePreprocessor,
}

impl StellaPreprocessor {
    /// Create a new Stella preprocessor with the given task type
    pub fn new(task_type: StellaTaskType) -> Self {
        Self {
            task_type,
            inner: TemplatePreprocessor::new(Self::get_template(task_type)),
        }
    }

    /// Create an S2P preprocessor
    pub fn s2p() -> Self {
        Self::new(StellaTaskType::S2P)
    }

    /// Create an S2S preprocessor
    pub fn s2s() -> Self {
        Self::new(StellaTaskType::S2S)
    }

    /// Get the task type
    pub fn task_type(&self) -> StellaTaskType {
        self.task_type
    }

    /// Get the template for a given task type
    fn get_template(task_type: StellaTaskType) -> &'static str {
        match task_type {
            StellaTaskType::S2P => {
                "Instruct: Given a web search query, retrieve relevant passages that answer the query.\nQuery: {text}"
            }
            StellaTaskType::S2S => {
                "Instruct: Retrieve semantically similar text.\nQuery: {text}"
            }
        }
    }
}

impl TextPreprocessor for StellaPreprocessor {
    fn process(&self, text: &str) -> String {
        self.inner.process(text)
    }
}

/// Concrete preprocessor types for static dispatch
#[derive(Debug, Clone)]
pub enum ConcretePreprocessor {
    /// No-op preprocessor
    Noop(NoopPreprocessor),
    /// Prefix preprocessor
    Prefix(PrefixPreprocessor),
    /// Template preprocessor
    Template(TemplatePreprocessor),
    /// Nomic preprocessor
    Nomic(NomicPreprocessor),
    /// Stella preprocessor
    Stella(StellaPreprocessor),
}

impl TextPreprocessor for ConcretePreprocessor {
    fn process(&self, text: &str) -> String {
        match self {
            Self::Noop(p) => p.process(text),
            Self::Prefix(p) => p.process(text),
            Self::Template(p) => p.process(text),
            Self::Nomic(p) => p.process(text),
            Self::Stella(p) => p.process(text),
        }
    }
}

/// Chained preprocessor that applies multiple preprocessors in sequence
pub struct ChainedPreprocessor {
    preprocessors: Vec<ConcretePreprocessor>,
}

impl std::fmt::Debug for ChainedPreprocessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainedPreprocessor")
            .field("count", &self.preprocessors.len())
            .finish()
    }
}

impl ChainedPreprocessor {
    /// Create an empty chained preprocessor
    pub fn new() -> Self {
        Self {
            preprocessors: Vec::new(),
        }
    }

    /// Add a preprocessor to the chain
    pub fn with_preprocessor(mut self, preprocessor: ConcretePreprocessor) -> Self {
        self.preprocessors.push(preprocessor);
        self
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.preprocessors.is_empty()
    }

    /// Get the number of preprocessors in the chain
    pub fn len(&self) -> usize {
        self.preprocessors.len()
    }
}

impl Default for ChainedPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TextPreprocessor for ChainedPreprocessor {
    fn process(&self, text: &str) -> String {
        let mut result = text.to_string();
        // Pipeline semantics: process in add order (first added = first processed)
        for preprocessor in &self.preprocessors {
            result = preprocessor.process(&result);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_preprocessor() {
        let preprocessor = NoopPreprocessor;
        assert_eq!(preprocessor.process("Hello"), "Hello");
    }

    #[test]
    fn test_prefix_preprocessor() {
        let preprocessor = PrefixPreprocessor::new("prefix: ");
        assert_eq!(preprocessor.process("Hello"), "prefix: Hello");
    }

    #[test]
    fn test_template_preprocessor() {
        let preprocessor = TemplatePreprocessor::new("Query: {text}");
        assert_eq!(preprocessor.process("Hello"), "Query: Hello");
    }

    #[test]
    fn test_nomic_preprocessor() {
        let preprocessor = NomicPreprocessor::search_document();
        assert_eq!(
            preprocessor.process("Hello world"),
            "search_document: Hello world"
        );

        let preprocessor = NomicPreprocessor::search_query();
        assert_eq!(
            preprocessor.process("What is AI?"),
            "search_query: What is AI?"
        );
    }

    #[test]
    fn test_stella_preprocessor() {
        let preprocessor = StellaPreprocessor::s2p();
        let result = preprocessor.process("machine learning");
        assert!(result.contains("machine learning"));
        assert!(result.contains("Instruct:"));

        let preprocessor = StellaPreprocessor::s2s();
        let result = preprocessor.process("deep learning");
        assert!(result.contains("deep learning"));
        assert!(result.contains("Instruct:"));
    }

    #[test]
    fn test_chained_preprocessor() {
        // Pipeline semantics: first added = first processed
        // with_preprocessor("[A]").with_preprocessor("[B]").process("x") => "[B] [A] x"
        let preprocessor = ChainedPreprocessor::new()
            .with_preprocessor(ConcretePreprocessor::Prefix(PrefixPreprocessor::new(
                "[START] ",
            )))
            .with_preprocessor(ConcretePreprocessor::Prefix(PrefixPreprocessor::new(
                "[MIDDLE] ",
            )));

        assert_eq!(preprocessor.process("Hello"), "[MIDDLE] [START] Hello");
    }

    #[test]
    fn test_batch_processing() {
        let preprocessor = NomicPreprocessor::search_document();
        let texts = vec!["Hello", "World"];
        let results = preprocessor.process_batch(&texts);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "search_document: Hello");
        assert_eq!(results[1], "search_document: World");
    }

    #[test]
    fn test_nomic_task_type_prefixes() {
        assert_eq!(
            NomicTaskType::SearchDocument.as_prefix(),
            "search_document: "
        );
        assert_eq!(NomicTaskType::SearchQuery.as_prefix(), "search_query: ");
        assert_eq!(NomicTaskType::Clustering.as_prefix(), "clustering: ");
        assert_eq!(
            NomicTaskType::Classification.as_prefix(),
            "classification: "
        );
    }
}
