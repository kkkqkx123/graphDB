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
pub mod resolver;
pub mod search;
pub mod serialize;
pub mod storage;

pub mod r#type;
pub mod async_;

// Service-related modules - only compiled when "service" feature is enabled
#[cfg(feature = "service")]
pub mod proto;

#[cfg(feature = "service")]
pub mod service;

// Re-export document types
pub use document::{
    Document,
    DocumentConfig,
    Field,
    FieldConfig,
    FieldType,
    Fields,
    parse_tree,
    parse_tree_cached,
    TreePath,
    PathCache,
    EvaluationStrategy,
    PathParseError,
    TagSystem,
    TagConfig,
    Batch,
    BatchOperation,
    BatchExecutor,
    BatchExecutorFn,
    BatchResult,
    BatchStatus,
    BatchMetadata,
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
pub use common::{Arena, ArenaTokenizer, ArenaVec, ArenaStringBuilder};
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
// Export highlight modules with specific names to avoid conflicts
pub use highlight::{
    highlight_fields, highlight_document, highlight_single_document,
    highlight_document_structured, highlight_single_document_structured,
    highlight_results, highlight_results_with_complete,
    HighlightProcessor
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
    Enricher,
    FieldSelector,
    TagIntegrationConfig,
    HighlightConfig,
    MetadataSource,
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
    BoostStrategy,
    FieldBoostConfig,
    FieldSearch,
    multi_field_search,
    multi_field_search_with_weights,
    MultiFieldSearchConfig,
};
pub use serialize::*;
pub use storage::common::r#trait::StorageInterface;
pub use storage::common::types::StorageInfo;

#[cfg(feature = "store-memory")]
pub use storage::memory::MemoryStorage;

#[cfg(feature = "store-file")]
pub use storage::file::FileStorage;

#[cfg(feature = "store-wal")]
pub use storage::wal_storage::WALStorage;

#[cfg(feature = "store-wal")]
pub use storage::wal::{WALManager, IndexChange};

#[cfg(feature = "store-wal")]
pub use config::WALConfig as StorageWALConfig;


pub use async_::*;
// Export specific types from r#type module to avoid conflicts
pub use r#type::{
    IndexOptions, ContextOptions, SearchOptions, FieldOption, TagOption,
    EncoderOptions, DocumentSearchResult, DocumentSearchResults,
    EnrichedDocumentSearchResult, EnrichedDocumentSearchResults,
    MergedDocumentSearchEntry, MergedDocumentSearchResults,
    DocId, SearchResults, IntermediateSearchResults
};