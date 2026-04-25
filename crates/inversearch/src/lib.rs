// Inversearch - High-performance search library and service
// Re-export all modules through the api module

// Core modules - always available
pub mod async_;
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
pub mod tokenizer;
pub mod r#type;

// API module organization
pub mod api;

// Re-export core types for backward compatibility
pub use api::core;

// Core types - always available
pub use crate::{
    document::Document, document::Field, document::FieldType, error::InversearchError as Error,
    error::Result, index::Index, index::IndexOptions, search::SearchResult,
};

// Re-export types from r#type module
pub use crate::r#type::SearchOptions;

// Embedded API - available when "embedded" feature is enabled
#[cfg(feature = "embedded")]
pub use api::embedded;

#[cfg(feature = "embedded")]
pub use api::embedded::{
    EmbeddedBatch, EmbeddedBatchOperation, EmbeddedBatchResult, EmbeddedIndex,
    EmbeddedIndexBuilder, EmbeddedIndexStats, EmbeddedSearchResult,
};

// Server API - available when "service" feature is enabled
#[cfg(feature = "service")]
pub mod proto;

#[cfg(feature = "service")]
pub mod service;

#[cfg(feature = "service")]
pub use api::server;

#[cfg(feature = "service")]
pub use api::server::{run_server, InversearchService, ServerConfig, ServiceConfig};

// Re-export document types
pub use document::{
    parse_tree, parse_tree_cached, Batch, BatchExecutor, BatchExecutorFn, BatchMetadata,
    BatchOperation, BatchResult, BatchStatus, DocumentConfig, EvaluationStrategy, FieldConfig,
    Fields, PathCache, PathParseError, TagConfig, TagSystem, TreePath,
};

// Re-export charset modules with specific names to avoid conflicts
pub use charset::{
    charset_cjk, charset_exact, charset_latin_advanced, charset_latin_balance, charset_latin_extra,
    charset_latin_soundex, charset_normalize, get_charset_cjk, get_charset_default,
    get_charset_exact, get_charset_latin_advanced, get_charset_latin_balance,
    get_charset_latin_extra, get_charset_latin_soundex, get_charset_normalize,
};
pub use common::{Arena, ArenaStringBuilder, ArenaTokenizer, ArenaVec};
pub use compress::{
    compress_string, compress_string_with_options, lcg, lcg64, lcg_for_number, to_radix,
    CompressCache, RadixTable, DEFAULT_CACHE_SIZE,
};
pub use config::{
    Config, EmbeddedConfig, EmbeddedConfigBuilder, StorageBackend, StorageConfig,
    StorageConfigBuilder, TokenizeMode,
};
pub use encoder::Encoder;
pub use error::{
    CacheError, EncoderError, IndexError, InversearchError, SearchError, StorageError,
};
// Export highlight modules with specific names to avoid conflicts
pub use highlight::{
    highlight_document, highlight_document_structured, highlight_fields, highlight_results,
    highlight_results_with_complete, highlight_single_document,
    highlight_single_document_structured, HighlightProcessor,
};
pub use index::{Register, ScoreFn, TokenizeMode as IndexTokenizeMode};
pub use intersect::SuggestionEngine;
pub use keystore::{DocId, KeystoreMap, KeystoreSet};
pub use metrics::Metrics;
pub use resolver::{
    combine_search_results, exclusion, intersect_and, resolve_default, union_op, xor_op, Enricher,
    FieldSelector, Handler, HighlightConfig, MetadataSource, Resolver, ResolverError,
    ResolverOptions, ResolverResult, TagIntegrationConfig,
};
pub use search::{
    multi_field_search, multi_field_search_with_weights, multi_term_search, resolve_default_search,
    search, single_term_query, BoostStrategy, CacheKeyGenerator, CacheStats, CachedSearch,
    CombineStrategy, FieldBoostConfig, FieldSearch, MultiFieldSearchConfig,
    MultiFieldSearchOptions, SearchCache, SearchCoordinator, SingleTermResult,
};
pub use serialize::SerializeConfig;
pub use storage::common::r#trait::StorageInterface;
pub use storage::common::types::StorageInfo;
pub use storage::factory::StorageFactory;
pub use storage::manager::{DefaultStorage, StorageManager, StorageManagerBuilder};
pub use storage::persistence::{BackupInfo, IndexMetadata, IndexSnapshot, PersistenceManager};

// Storage backends - conditionally available
#[cfg(feature = "store-file")]
pub use storage::file::FileStorage;

#[cfg(feature = "store-redis")]
pub use storage::redis::RedisStorage;

#[cfg(feature = "store-wal")]
pub use storage::wal::{IndexChange, WALManager, WALStorage};

#[cfg(feature = "store-wal")]
pub use config::WALConfig as StorageWalConfig;

#[cfg(feature = "store-cold-warm-cache")]
pub use storage::cold_warm_cache::{ColdWarmCacheConfig, ColdWarmCacheManager};

pub use async_::{AsyncIndex, AsyncIndexTask, AsyncSearchTask};
// Export specific types from r#type module to avoid conflicts
pub use r#type::{
    ContextOptions, DocumentSearchResult, DocumentSearchResults, EncoderOptions,
    EnrichedDocumentSearchResult, EnrichedDocumentSearchResults, EnrichedSearchResult,
    EnrichedSearchResults, FieldOption, HighlightBoundaryOptions, HighlightEllipsisOptions,
    HighlightOptions, IntermediateSearchResults, MergedDocumentSearchEntry,
    MergedDocumentSearchResults, TagOption,
};
