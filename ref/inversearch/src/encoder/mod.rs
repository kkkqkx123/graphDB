mod transform;
mod validator;

pub use transform::*;
pub use validator::EncoderValidator;

use crate::r#type::EncoderOptions;
use crate::error::Result;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

lazy_static::lazy_static! {
    static ref WHITESPACE: Regex = Regex::new(r"[^\p{L}\p{N}]+")
        .expect("Failed to compile WHITESPACE regex");
    static ref NORMALIZE: Regex = Regex::new(r"[\u{0300}-\u{036f}]")
        .expect("Failed to compile NORMALIZE regex");
    static ref NUMERIC_SPLIT_LENGTH: Regex = Regex::new(r"(\d{3})")
        .expect("Failed to compile NUMERIC_SPLIT_LENGTH regex");
    static ref NUMERIC_SPLIT_PREV_CHAR: Regex = Regex::new(r"(\D)(\d{3})")
        .expect("Failed to compile NUMERIC_SPLIT_PREV_CHAR regex");
    static ref NUMERIC_SPLIT_NEXT_CHAR: Regex = Regex::new(r"(\d{3})(\D)")
        .expect("Failed to compile NUMERIC_SPLIT_NEXT_CHAR regex");
}

#[derive(Clone)]
pub struct Encoder {
    pub normalize: NormalizeOption,
    pub split: SplitOption,
    pub numeric: bool,
    pub prepare: Option<Arc<dyn TextTransformer>>,
    pub finalize: Option<Arc<dyn TokenFinalizer>>,
    pub filter: Option<FilterOption>,
    pub dedupe: bool,
    pub matcher: Option<HashMap<String, String>>,
    pub mapper: Option<HashMap<char, char>>,
    pub stemmer: Option<HashMap<String, String>>,
    pub replacer: Option<Vec<(Regex, String)>>,
    pub minlength: usize,
    pub maxlength: usize,
    pub rtl: bool,
    pub cache: Option<Cache>,
}

#[derive(Clone)]
pub enum NormalizeOption {
    Bool(bool),
    Function(Arc<dyn TextTransformer>),
}

#[derive(Clone)]
pub enum SplitOption {
    String(String),
    Regex(Regex),
    Bool(bool),
}

#[derive(Clone)]
pub enum FilterOption {
    Set(HashSet<String>),
    Function(Arc<dyn TextFilter>),
}

#[derive(Clone)]
pub struct Cache {
    pub size: usize,
    pub cache_enc: Arc<RwLock<lru::LruCache<String, Vec<String>>>>,
    pub cache_term: Arc<RwLock<lru::LruCache<String, String>>>,
    pub cache_enc_length: usize,
    pub cache_term_length: usize,
}

impl Encoder {
    pub fn new(options: EncoderOptions) -> Result<Self> {
        // Validate configuration before creating encoder
        EncoderValidator::validate(&options)?;
        
        // Clone options for later use to avoid borrowing issues
        let options_clone = options.clone();
        
        // Extract values from options
        let normalize_flag = options.normalize;
        let split_string = options.split;
        let numeric_flag = options.numeric;
        let filter_vec = options.filter;
        let dedupe_flag = options.dedupe;
        let matcher_map = options.matcher;
        let mapper_map = options.mapper;
        let stemmer_map = options.stemmer;
        let replacer_vec = options.replacer;
        let minlength_val = options.minlength;
        let maxlength_val = options.maxlength;
        let rtl_flag = options.rtl;
        let cache_flag = options.cache;
        
        let normalize = match normalize_flag {
            Some(true) => NormalizeOption::Bool(true),
            Some(false) => NormalizeOption::Bool(false),
            None => NormalizeOption::Bool(true),
        };

        let split = if let Some(split) = split_string {
            if split.is_empty() {
                SplitOption::String(String::new())
            } else {
                SplitOption::String(split)
            }
        } else {
            SplitOption::Regex(WHITESPACE.clone())
        };

        let numeric = numeric_flag.unwrap_or(true);

        let filter = filter_vec.map(|filter| {
            if filter.is_empty() {
                FilterOption::Set(HashSet::new())
            } else {
                FilterOption::Set(filter.into_iter().collect())
            }
        });

        let dedupe = dedupe_flag.unwrap_or(true);

        let replacer = replacer_vec.map(|replacer| {
            replacer
                .into_iter()
                .map(|(pattern, replacement)| {
                    let regex = Regex::new(&pattern)
                        .unwrap_or_else(|e| {
                            eprintln!("Failed to compile regex pattern '{}': {}", pattern, e);
                            Regex::new("").expect("Failed to create empty regex")
                        });
                    (regex, replacement)
                })
                .collect()
        });

        let minlength = minlength_val.unwrap_or(1);
        let maxlength = maxlength_val.unwrap_or(1024);
        let rtl = rtl_flag.unwrap_or(false);

        let cache = if cache_flag.unwrap_or(true) {
            Some(Cache {
                size: 200_000,
                cache_enc: Arc::new(RwLock::new(lru::LruCache::unbounded())),
                cache_term: Arc::new(RwLock::new(lru::LruCache::unbounded())),
                cache_enc_length: 128,
                cache_term_length: 128,
            })
        } else {
            None
        };

        let encoder = Encoder {
            normalize,
            split,
            numeric,
            prepare: None,
            finalize: None,
            filter,
            dedupe,
            matcher: matcher_map,
            mapper: mapper_map,
            stemmer: stemmer_map,
            replacer,
            minlength,
            maxlength,
            rtl,
            cache,
        };

        // Validate final transformer configuration
        EncoderValidator::validate_transformers(
            encoder.prepare.as_ref().map(|arc| arc.as_ref()),
            encoder.finalize.as_ref().map(|arc| arc.as_ref()),
            encoder.filter.as_ref().and_then(|opt| match opt {
                FilterOption::Function(func) => Some(func.as_ref()),
                _ => None,
            }),
        )?;

        // Log optimization suggestions
        let suggestions = EncoderValidator::suggest_optimizations(&options_clone);
        for suggestion in suggestions {
            tracing::debug!("Encoder optimization suggestion: {}", suggestion);
        }

        Ok(encoder)
    }

    pub fn encode(&self, str: &str) -> Result<Vec<String>> {
        let mut s = str.to_string();

        if let Some(cache) = &self.cache {
            if s.len() <= cache.cache_enc_length {
                if let Ok(cache_enc) = cache.cache_enc.read() {
                    if let Some(result) = cache_enc.peek(&s) {
                        return Ok(result.clone());
                    }
                }
            }
        }

        s = self.apply_normalize(&s);

        if let Some(prepare) = &self.prepare {
            s = prepare.transform(s);
        }

        if self.numeric && s.len() > 3 {
            s = NUMERIC_SPLIT_PREV_CHAR
                .replace_all(&s, "$1 $2")
                .to_string();
            s = NUMERIC_SPLIT_NEXT_CHAR
                .replace_all(&s, "$1 $2")
                .to_string();
            s = NUMERIC_SPLIT_LENGTH.replace_all(&s, "$1 ").to_string();
        }

        let words = self.apply_split(&s);

        let skip = !self.has_transformations();

        let mut final_terms = Vec::new();
        let mut dupes = HashSet::new();
        let mut last_term = String::new();
        let mut last_term_enc = String::new();

        for word in words {
            let base = word.clone();

            if word.is_empty() {
                continue;
            }

            if word.len() < self.minlength || word.len() > self.maxlength {
                continue;
            }

            if self.dedupe {
                if dupes.contains(&word) {
                    continue;
                }
                dupes.insert(word.clone());
            } else {
                if last_term == word {
                    continue;
                }
                last_term = word.clone();
            }

            if skip {
                final_terms.push(word);
                continue;
            }

            if let Some(filter) = &self.filter {
                if !self.apply_filter(filter, &word) {
                    continue;
                }
            }

            let mut word = word;

            if let Some(cache) = &self.cache {
                if base.len() <= cache.cache_term_length {
                    if let Ok(cache_term) = cache.cache_term.read() {
                        if let Some(tmp) = cache_term.peek(&base) {
                            if !tmp.is_empty() {
                                final_terms.push(tmp.clone());
                            }
                            continue;
                        }
                    }
                }
            }

            if let Some(stemmer) = &self.stemmer {
                word = self.apply_stemmer(&word, stemmer);
            }

            if self.mapper.is_some() || (self.dedupe && word.len() > 1) {
                word = self.apply_mapper(&word);
            }

            if let Some(matcher) = &self.matcher {
                word = self.apply_matcher(&word, matcher);
            }

            if let Some(replacer) = &self.replacer {
                word = self.apply_replacer(&word, replacer);
            }

            if let Some(cache) = &self.cache {
                if base.len() <= cache.cache_term_length {
                    if let Ok(mut cache_term) = cache.cache_term.write() {
                        cache_term.put(base.clone(), word.clone());
                    }
                }
            }

            if !word.is_empty() {
                if word != base {
                    if self.dedupe {
                        if dupes.contains(&word) {
                            continue;
                        }
                        dupes.insert(word.clone());
                    } else {
                        if last_term_enc == word {
                            continue;
                        }
                        last_term_enc = word.clone();
                    }
                }
                final_terms.push(word);
            }
        }

        if let Some(finalize) = &self.finalize {
            if let Some(result) = finalize.finalize(final_terms.clone()) {
                final_terms = result;
            }
        }

        if let Some(cache) = &self.cache {
            if s.len() <= cache.cache_enc_length {
                if let Ok(mut cache_enc) = cache.cache_enc.write() {
                    cache_enc.put(s.clone(), final_terms.clone());
                }
            }
        }

        Ok(final_terms)
    }

    fn apply_normalize(&self, str: &str) -> String {
        match &self.normalize {
            NormalizeOption::Bool(true) => {
                use unicode_normalization::UnicodeNormalization;
                NORMALIZE.replace_all(str.chars().nfkd().collect::<String>().as_str(), "")
                    .to_lowercase()
            }
            NormalizeOption::Bool(false) => str.to_lowercase(),
            NormalizeOption::Function(func) => func.transform(str.to_string()),
        }
    }

    fn apply_split(&self, str: &str) -> Vec<String> {
        match &self.split {
            SplitOption::String(s) if s.is_empty() => vec![str.to_string()],
            SplitOption::String(s) => str.split(s).map(|s| s.to_string()).collect(),
            SplitOption::Regex(regex) => {
                regex.split(str).map(|s| s.to_string()).collect()
            }
            SplitOption::Bool(false) => vec![str.to_string()],
            SplitOption::Bool(true) => {
                WHITESPACE.split(str).map(|s| s.to_string()).collect()
            }
        }
    }

    fn has_transformations(&self) -> bool {
        self.filter.is_some()
            || self.mapper.is_some()
            || self.matcher.is_some()
            || self.stemmer.is_some()
            || self.replacer.is_some()
    }

    fn apply_filter(&self, filter: &FilterOption, word: &str) -> bool {
        match filter {
            FilterOption::Set(set) => !set.contains(word),
            FilterOption::Function(func) => func.should_include(word),
        }
    }

    fn apply_stemmer(&self, word: &str, stemmer: &HashMap<String, String>) -> String {
        let mut word = word.to_string();
        let mut old = String::new();

        while old != word && word.len() > 2 {
            old = word.clone();
            for (key, value) in stemmer {
                if word.len() > key.len() && word.ends_with(key) {
                    word = format!("{}{}", &word[..word.len() - key.len()], value);
                    break;
                }
            }
        }

        word
    }

    fn apply_mapper(&self, word: &str) -> String {
        let mut result = String::new();
        let mut prev = String::new();

        for char in word.chars() {
            if char.to_string() != prev || !self.dedupe {
                let tmp = self.mapper.as_ref().and_then(|m| m.get(&char).copied());
                if let Some(mapped) = tmp {
                    if mapped.to_string() != prev || !self.dedupe {
                        result.push(mapped);
                        prev = mapped.to_string();
                    }
                } else {
                    result.push(char);
                    prev = char.to_string();
                }
            }
        }

        result
    }

    fn apply_matcher(&self, word: &str, matcher: &HashMap<String, String>) -> String {
        let mut result = word.to_string();

        for (key, value) in matcher {
            result = result.replace(key, value);
        }

        result
    }

    fn apply_replacer(&self, word: &str, replacer: &[(Regex, String)]) -> String {
        let mut result = word.to_string();

        for (regex, replacement) in replacer {
            result = regex.replace_all(&result, replacement).to_string();
        }

        result
    }

    pub fn add_stemmer(&mut self, match_str: String, replace: String) {
        if self.stemmer.is_none() {
            self.stemmer = Some(HashMap::new());
        }
        if let Some(stemmer) = &mut self.stemmer {
            stemmer.insert(match_str, replace);
        }
        if let Some(cache) = &mut self.cache {
            if let Ok(mut cache_enc) = cache.cache_enc.write() {
                cache_enc.clear();
            }
            if let Ok(mut cache_term) = cache.cache_term.write() {
                cache_term.clear();
            }
        }
    }

    pub fn add_filter(&mut self, term: String) {
        if self.filter.is_none() {
            self.filter = Some(FilterOption::Set(HashSet::new()));
        }
        if let Some(FilterOption::Set(set)) = self.filter.as_mut() {
            set.insert(term);
        }
        if let Some(cache) = &mut self.cache {
            if let Ok(mut cache_enc) = cache.cache_enc.write() {
                cache_enc.clear();
            }
            if let Ok(mut cache_term) = cache.cache_term.write() {
                cache_term.clear();
            }
        }
    }

    pub fn add_mapper(&mut self, char_match: char, char_replace: char) {
        if self.mapper.is_none() {
            self.mapper = Some(HashMap::new());
        }
        if let Some(mapper) = &mut self.mapper {
            mapper.insert(char_match, char_replace);
        }
        if let Some(cache) = &mut self.cache {
            if let Ok(mut cache_enc) = cache.cache_enc.write() {
                cache_enc.clear();
            }
            if let Ok(mut cache_term) = cache.cache_term.write() {
                cache_term.clear();
            }
        }
    }

    pub fn add_matcher(&mut self, match_str: String, replace: String) {
        if self.matcher.is_none() {
            self.matcher = Some(HashMap::new());
        }
        if let Some(matcher) = &mut self.matcher {
            matcher.insert(match_str, replace);
        }
        if let Some(cache) = &mut self.cache {
            if let Ok(mut cache_enc) = cache.cache_enc.write() {
                cache_enc.clear();
            }
            if let Ok(mut cache_term) = cache.cache_term.write() {
                cache_term.clear();
            }
        }
    }

    pub fn add_replacer(&mut self, regex: Regex, replace: String) {
        if self.replacer.is_none() {
            self.replacer = Some(Vec::new());
        }
        if let Some(replacer) = &mut self.replacer {
            replacer.push((regex, replace));
        }
        if let Some(cache) = &mut self.cache {
            if let Ok(mut cache_enc) = cache.cache_enc.write() {
                cache_enc.clear();
            }
            if let Ok(mut cache_term) = cache.cache_term.write() {
                cache_term.clear();
            }
        }
    }

    /// Set a custom text transformer for the prepare stage
    pub fn set_prepare_transformer<T: TextTransformer + 'static>(&mut self, transformer: T) {
        self.prepare = Some(Arc::new(transformer));
        if let Some(cache) = &mut self.cache {
            if let Ok(mut cache_enc) = cache.cache_enc.write() {
                cache_enc.clear();
            }
            if let Ok(mut cache_term) = cache.cache_term.write() {
                cache_term.clear();
            }
        }
    }

    /// Set a custom function-based text transformer for the prepare stage
    pub fn set_prepare_function<F>(&mut self, func: F)
    where
        F: Fn(String) -> String + Send + Sync + 'static,
    {
        self.set_prepare_transformer(FunctionTransformer::new(func));
    }

    /// Set a custom token finalizer
    pub fn set_finalize_transformer<T: TokenFinalizer + 'static>(&mut self, finalizer: T) {
        self.finalize = Some(Arc::new(finalizer));
        if let Some(cache) = &mut self.cache {
            if let Ok(mut cache_enc) = cache.cache_enc.write() {
                cache_enc.clear();
            }
            if let Ok(mut cache_term) = cache.cache_term.write() {
                cache_term.clear();
            }
        }
    }

    /// Set a custom function-based token finalizer
    pub fn set_finalize_function<F>(&mut self, func: F)
    where
        F: Fn(Vec<String>) -> Option<Vec<String>> + Send + Sync + 'static,
    {
        self.set_finalize_transformer(FunctionFinalizer::new(func));
    }

    /// Set a custom text filter
    pub fn set_filter_transformer<T: TextFilter + 'static>(&mut self, filter: T) {
        self.filter = Some(FilterOption::Function(Arc::new(filter)));
        if let Some(cache) = &mut self.cache {
            if let Ok(mut cache_enc) = cache.cache_enc.write() {
                cache_enc.clear();
            }
            if let Ok(mut cache_term) = cache.cache_term.write() {
                cache_term.clear();
            }
        }
    }

    /// Set a custom function-based text filter
    pub fn set_filter_function<F>(&mut self, func: F)
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.set_filter_transformer(FunctionFilter::new(func));
    }

    /// Set a cached transformer for better performance
    pub fn set_cached_prepare_transformer<T: TextTransformer + 'static>(
        &mut self,
        transformer: T,
        max_cache_size: usize,
    ) {
        self.set_prepare_transformer(CachedTransformer::new(transformer, max_cache_size));
    }

    /// 获取编码器选项
    pub fn get_options(&self) -> crate::r#type::EncoderOptions {
        crate::r#type::EncoderOptions {
            rtl: Some(self.rtl),
            dedupe: Some(self.dedupe),
            split: match &self.split {
                SplitOption::String(s) => Some(s.clone()),
                SplitOption::Regex(_) => None,
                SplitOption::Bool(_) => None,
            },
            numeric: Some(self.numeric),
            normalize: Some(matches!(self.normalize, NormalizeOption::Bool(true))),
            prepare: None,
            finalize: None,
            filter: match &self.filter {
                Some(FilterOption::Set(set)) => Some(set.iter().cloned().collect()),
                Some(FilterOption::Function(_)) => None,
                None => None,
            },
            matcher: self.matcher.clone(),
            mapper: self.mapper.clone(),
            stemmer: self.stemmer.clone(),
            replacer: self.replacer.as_ref().map(|r| {
                r.iter().map(|(regex, replacement)| {
                    (regex.as_str().to_string(), replacement.clone())
                }).collect()
            }),
            minlength: Some(self.minlength),
            maxlength: Some(self.maxlength),
            cache: Some(self.cache.is_some()),
        }
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Encoder::new(EncoderOptions::default()).expect("Default encoder configuration should be valid")
    }
}

pub fn fallback_encoder(str: &str) -> Vec<String> {
    use unicode_normalization::UnicodeNormalization;
    NORMALIZE.replace_all(str.chars().nfkd().collect::<String>().as_str(), "")
        .to_lowercase()
        .trim()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_default() {
        let encoder = Encoder::default();
        let result = encoder.encode("Hello World").unwrap();
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_encoder_normalize() {
        let encoder = Encoder::default();
        let result = encoder.encode("Héllo Wörld").unwrap();
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_encoder_numeric() {
        let encoder = Encoder::default();
        let result = encoder.encode("123456").unwrap();
        assert_eq!(result, vec!["123", "456"]);
    }

    #[test]
    fn test_encoder_minlength() {
        let options = EncoderOptions {
            minlength: Some(3),
            ..Default::default()
        };
        let encoder = Encoder::new(options).unwrap();
        let result = encoder.encode("hi hello world").unwrap();
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_encoder_dedupe() {
        let options = EncoderOptions {
            dedupe: Some(true),
            ..Default::default()
        };
        let encoder = Encoder::new(options).unwrap();
        let result = encoder.encode("hello hello world world").unwrap();
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_encoder_filter() {
        let options = EncoderOptions {
            filter: Some(vec!["the".to_string(), "and".to_string()]),
            ..Default::default()
        };
        let encoder = Encoder::new(options).expect("Failed to create encoder");
        let result = encoder.encode("the cat and the dog").expect("Failed to encode");
        assert_eq!(result, vec!["cat", "dog"]);
    }

    #[test]
    fn test_encoder_stemmer() {
        let mut options = EncoderOptions::default();
        let mut stemmer = HashMap::new();
        stemmer.insert("ing".to_string(), "".to_string());
        options.stemmer = Some(stemmer);
        let encoder = Encoder::new(options).expect("Failed to create encoder");
        let result = encoder.encode("running jumping").expect("Failed to encode");
        assert_eq!(result, vec!["run", "jump"]);
    }

    #[test]
    fn test_encoder_mapper() {
        let mut options = EncoderOptions::default();
        options.dedupe = Some(false); // Disable dedupe for this test
        let mut mapper = HashMap::new();
        mapper.insert('a', 'b');
        options.mapper = Some(mapper);
        let encoder = Encoder::new(options).expect("Failed to create encoder");
        let result = encoder.encode("apple").expect("Failed to encode");
        assert_eq!(result, vec!["bpple"]);
    }

    #[test]
    fn test_encoder_matcher() {
        let mut options = EncoderOptions::default();
        let mut matcher = HashMap::new();
        matcher.insert("color".to_string(), "colour".to_string());
        options.matcher = Some(matcher);
        let encoder = Encoder::new(options).expect("Failed to create encoder");
        let result = encoder.encode("color").expect("Failed to encode");
        assert_eq!(result, vec!["colour"]);
    }

    #[test]
    fn test_fallback_encoder() {
        let result = fallback_encoder("Hello World");
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_trait_based_transformer() {
        let mut encoder = Encoder::default();
        
        // Test custom prepare transformer
        encoder.set_prepare_function(|text| text.replace("hello", "hi"));
        let result = encoder.encode("hello world").expect("Failed to encode");
        assert_eq!(result, vec!["hi", "world"]);
    }

    #[test]
    fn test_trait_based_filter() {
        let mut encoder = Encoder::default();
        encoder.dedupe = false; // Disable deduplication for this test
        
        // Test custom filter
        encoder.set_filter_function(|word| word.len() > 3);
        let result = encoder.encode("hi hello world").expect("Failed to encode");
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_trait_based_finalizer() {
        let mut encoder = Encoder::default();
        
        // Test custom finalizer
        encoder.set_finalize_function(|mut tokens| {
            tokens.retain(|t| t != "world");
            Some(tokens)
        });
        let result = encoder.encode("hello world").expect("Failed to encode");
        assert_eq!(result, vec!["hello"]);
    }

    #[test]
    fn test_validation_invalid_length() {
        let options = EncoderOptions {
            minlength: Some(10),
            maxlength: Some(5),
            ..Default::default()
        };
        
        let result = Encoder::new(options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_large_filter() {
        let mut options = EncoderOptions::default();
        let mut filter = Vec::new();
        for i in 0..10_001 {
            filter.push(format!("word{}", i));
        }
        options.filter = Some(filter);
        
        let result = Encoder::new(options);
        assert!(result.is_err());
    }
}
