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

pub mod async_;
pub mod r#type;

// Service-related modules - only compiled when "service" feature is enabled
#[cfg(feature = "service")]
pub mod proto;

#[cfg(feature = "service")]
pub mod service;

// Re-export document types
pub use document::{
    parse_tree, parse_tree_cached, Batch, BatchExecutor, BatchExecutorFn, BatchMetadata,
    BatchOperation, BatchResult, BatchStatus, Document, DocumentConfig, EvaluationStrategy, Field,
    FieldConfig, FieldType, Fields, PathCache, PathParseError, TagConfig, TagSystem, TreePath,
};

// Re-export charset modules with specific names to avoid conflicts
pub use charset::{
    charset_cjk, charset_exact, charset_latin_advanced, charset_latin_balance, charset_latin_extra,
    charset_latin_soundex, charset_normalize, get_charset_cjk, get_charset_default,
    get_charset_exact, get_charset_latin_advanced, get_charset_latin_balance,
    get_charset_latin_extra, get_charset_latin_soundex, get_charset_normalize,
};
pub use common::*;
pub use common::{Arena, ArenaStringBuilder, ArenaTokenizer, ArenaVec};
pub use compress::{
    compress_string, compress_string_with_options, lcg, lcg64, lcg_for_number, to_radix,
    CompressCache, RadixTable, DEFAULT_CACHE_SIZE,
};
pub use config::*;
pub use encoder::*;
pub use error::*;
// Export highlight modules with specific names to avoid conflicts
pub use highlight::{
    highlight_document, highlight_document_structured, highlight_fields, highlight_results,
    highlight_results_with_complete, highlight_single_document,
    highlight_single_document_structured, HighlightProcessor,
};
pub use index::*;
pub use intersect::*;
pub use keystore::*;
pub use metrics::*;
pub use resolver::{
    combine_search_results, exclusion, intersect_and, resolve_default, union_op, xor_op, Enricher,
    FieldSelector, Handler, HighlightConfig, MetadataSource, Resolver, ResolverError,
    ResolverOptions, ResolverResult, TagIntegrationConfig,
};
pub use search::{
    multi_field_search, multi_field_search_with_weights, multi_term_search, resolve_default_search,
    search, single_term_query, BoostStrategy, CacheKeyGenerator, CacheStats, CachedSearch,
    CombineStrategy, FieldBoostConfig, FieldSearch, MultiFieldSearchConfig,
    MultiFieldSearchOptions, SearchCache, SearchCoordinator, SearchResult, SingleTermResult,
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
pub use storage::wal::{IndexChange, WALManager};

#[cfg(feature = "store-wal")]
pub use config::WALConfig as StorageWALConfig;

pub use async_::*;
// Export specific types from r#type module to avoid conflicts
pub use r#type::{
    ContextOptions, DocId, DocumentSearchResult, DocumentSearchResults, EncoderOptions,
    EnrichedDocumentSearchResult, EnrichedDocumentSearchResults, FieldOption, IndexOptions,
    IntermediateSearchResults, MergedDocumentSearchEntry, MergedDocumentSearchResults,
    SearchOptions, SearchResults, TagOption,
};
