pub mod charset;
pub mod common;
pub mod compress;
pub mod config;
pub mod document;
pub mod encoder;
pub mod error;
pub mod highlight;
pub mod index;
pub mod intersect;
pub mod keystore;
pub mod metrics;
pub mod proto;
pub mod resolver;
pub mod search;
pub mod serialize;
pub mod storage;
pub mod tokenizer;
pub mod r#type;
pub mod async_;

// Re-export document types
pub use document::{
    Document,
    DocumentConfig,
    Field,
    FieldConfig,
    FieldType,
    Fields,
    parse_tree,
    TreePath,
    TagSystem,
    TagConfig,
    Batch,
    BatchOperation,
    BatchExecutor,
};

// Re-export charset modules with specific names to avoid conflicts
pub use charset::{
    charset_exact, charset_normalize, charset_cjk,
    charset_latin_balance, charset_latin_advanced, charset_latin_extra, charset_latin_soundex,
    get_charset_exact, get_charset_default, get_charset_normalize,
    get_charset_latin_balance, get_charset_latin_advanced, get_charset_latin_extra, get_charset_latin_soundex,
    get_charset_cjk
};
pub use common::*;
pub use compress::{
    compress_string,
    compress_string_with_options,
    lcg,
    lcg64,
    lcg_for_number,
    to_radix,
    RadixTable,
    CompressCache,
    DEFAULT_CACHE_SIZE,
};
pub use config::*;
pub use encoder::*;
pub use error::*;
// Export highlight types with specific names to avoid conflicts
pub use highlight::{
    highlight_fields, highlight_document, highlight_single_document,
    HighlightProcessor, HighlightConfig
};
pub use index::*;
pub use intersect::*;
pub use keystore::*;
pub use metrics::*;
pub use resolver::{
    Resolver,
    resolve_default,
    ResolverOptions,
    ResolverError,
    ResolverResult,
    Handler,
    intersect_and,
    union_op,
    exclusion,
    xor_op,
    combine_search_results,
    AsyncResolver,
    Enricher,
};
pub use search::{
    search,
    SearchResult,
    single_term_query,
    multi_term_search,
    SingleTermResult,
    SearchCache,
    CachedSearch,
    CacheStats,
    CacheKeyGenerator,
    resolve_default_search,
    SearchCoordinator,
    MultiFieldSearchOptions,
    CombineStrategy,
    multi_field_search,
};
pub use serialize::*;
pub use storage::{StorageInterface, StorageInfo, MemoryStorage, FileStorage};
pub use tokenizer::*;
pub use async_::*;
// Export specific types from r#type module to avoid conflicts
pub use r#type::{
    IndexOptions, ContextOptions, SearchOptions, FieldOption, TagOption,
    EncoderOptions, DocumentSearchResult, DocumentSearchResults,
    EnrichedDocumentSearchResult, EnrichedDocumentSearchResults,
    MergedDocumentSearchEntry, MergedDocumentSearchResults,
    DocId, SearchResults, IntermediateSearchResults
};