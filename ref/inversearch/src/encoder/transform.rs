use std::sync::Arc;

/// Trait for text transformation operations
pub trait TextTransformer: Send + Sync {
    fn transform(&self, text: String) -> String;
}

/// Trait for text filtering operations
pub trait TextFilter: Send + Sync {
    fn should_include(&self, text: &str) -> bool;
}

/// Trait for finalization operations on token collections
pub trait TokenFinalizer: Send + Sync {
    fn finalize(&self, tokens: Vec<String>) -> Option<Vec<String>>;
}

/// Wrapper for function-based transformers
pub struct FunctionTransformer<F>
where
    F: Fn(String) -> String + Send + Sync,
{
    func: Arc<F>,
}

impl<F> FunctionTransformer<F>
where
    F: Fn(String) -> String + Send + Sync,
{
    pub fn new(func: F) -> Self {
        Self {
            func: Arc::new(func),
        }
    }
}

impl<F> TextTransformer for FunctionTransformer<F>
where
    F: Fn(String) -> String + Send + Sync,
{
    fn transform(&self, text: String) -> String {
        (self.func)(text)
    }
}

/// Wrapper for function-based filters
pub struct FunctionFilter<F>
where
    F: Fn(&str) -> bool + Send + Sync,
{
    func: Arc<F>,
}

impl<F> FunctionFilter<F>
where
    F: Fn(&str) -> bool + Send + Sync,
{
    pub fn new(func: F) -> Self {
        Self {
            func: Arc::new(func),
        }
    }
}

impl<F> TextFilter for FunctionFilter<F>
where
    F: Fn(&str) -> bool + Send + Sync,
{
    fn should_include(&self, text: &str) -> bool {
        (self.func)(text)
    }
}

/// Wrapper for function-based finalizers
pub struct FunctionFinalizer<F>
where
    F: Fn(Vec<String>) -> Option<Vec<String>> + Send + Sync,
{
    func: Arc<F>,
}

impl<F> FunctionFinalizer<F>
where
    F: Fn(Vec<String>) -> Option<Vec<String>> + Send + Sync,
{
    pub fn new(func: F) -> Self {
        Self {
            func: Arc::new(func),
        }
    }
}

impl<F> TokenFinalizer for FunctionFinalizer<F>
where
    F: Fn(Vec<String>) -> Option<Vec<String>> + Send + Sync,
{
    fn finalize(&self, tokens: Vec<String>) -> Option<Vec<String>> {
        (self.func)(tokens)
    }
}

/// Built-in transformers
pub struct LowercaseTransformer;
pub struct UnicodeNormalizer;

impl TextTransformer for LowercaseTransformer {
    fn transform(&self, text: String) -> String {
        text.to_lowercase()
    }
}

impl TextTransformer for UnicodeNormalizer {
    fn transform(&self, text: String) -> String {
        use unicode_normalization::UnicodeNormalization;
        text.chars().nfkd().collect::<String>()
    }
}

/// Built-in filters
pub struct SetFilter {
    excluded: std::collections::HashSet<String>,
}

impl SetFilter {
    pub fn new(excluded: std::collections::HashSet<String>) -> Self {
        Self { excluded }
    }
}

impl TextFilter for SetFilter {
    fn should_include(&self, text: &str) -> bool {
        !self.excluded.contains(text)
    }
}

/// Performance-optimized transformer with caching
pub struct CachedTransformer<T: TextTransformer> {
    inner: T,
    cache: std::sync::RwLock<std::collections::HashMap<String, String>>,
    max_cache_size: usize,
}

impl<T: TextTransformer> CachedTransformer<T> {
    pub fn new(inner: T, max_cache_size: usize) -> Self {
        Self {
            inner,
            cache: std::sync::RwLock::new(std::collections::HashMap::new()),
            max_cache_size,
        }
    }
}

impl<T: TextTransformer> TextTransformer for CachedTransformer<T> {
    fn transform(&self, text: String) -> String {
        // Fast path: check cache
        {
            let cache = self.cache.read().unwrap();
            if let Some(cached) = cache.get(&text) {
                return cached.clone();
            }
        }
        
        // Slow path: transform and cache
        let transformed = self.inner.transform(text.clone());
        
        {
            let mut cache = self.cache.write().unwrap();
            if cache.len() >= self.max_cache_size {
                // Simple eviction: remove oldest entry
                if let Some(key) = cache.keys().next().cloned() {
                    cache.remove(&key);
                }
            }
            cache.insert(text, transformed.clone());
        }
        
        transformed
    }
}