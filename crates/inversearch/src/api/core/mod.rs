// Core API module - re-exports all core functionality
// These modules are defined at the crate root level

// Re-export error types
pub use crate::error::{
    CacheError, EncoderError, IndexError, InversearchError as Error, Result, SearchError,
    StorageError,
};

// Re-export specific types from modules (avoid glob exports to prevent conflicts)
pub use crate::async_::{AsyncIndex, AsyncIndexTask, AsyncSearchTask};

pub use crate::charset::{
    charset_cjk, charset_exact, charset_latin_advanced, charset_latin_balance, charset_latin_extra,
    charset_latin_soundex, charset_normalize, get_charset_cjk, get_charset_default,
    get_charset_exact, get_charset_latin_advanced, get_charset_latin_balance,
    get_charset_latin_extra, get_charset_latin_soundex, get_charset_normalize,
};

pub use crate::common::{Arena, ArenaStringBuilder, ArenaTokenizer, ArenaVec};

pub use crate::compress::{
    compress_string, compress_string_with_options, lcg, lcg64, lcg_for_number, to_radix,
    CompressCache, RadixTable, DEFAULT_CACHE_SIZE,
};

pub use crate::config::{Config, StorageBackend, StorageConfig};

pub use crate::document::{
    parse_tree, parse_tree_cached, Batch, BatchExecutor, BatchExecutorFn, BatchMetadata,
    BatchOperation, BatchResult, BatchStatus, Document, DocumentConfig, EvaluationStrategy, Field,
    FieldConfig, FieldType, Fields, PathCache, PathParseError, TagConfig, TagSystem, TreePath,
};

pub use crate::encoder::{Encoder, EncoderValidator};

pub use crate::highlight::{
    highlight_document, highlight_document_structured, highlight_fields, highlight_results,
    highlight_results_with_complete, highlight_single_document,
    highlight_single_document_structured, BoundaryState, BoundaryTerm, HighlightProcessor,
};

pub use crate::index::{Index, Register, ScoreFn, TokenizeMode};

pub use crate::intersect::{
    intersect, intersect_simple, intersect_union, union, union_simple, Bm25Scorer, ScoreConfig,
    ScoredId, SuggestionConfig, SuggestionEngine, TfIdfScorer,
};

pub use crate::keystore::{DocId, KeystoreMap, KeystoreSet};

pub use crate::metrics::Metrics;

pub use crate::resolver::{
    combine_search_results, exclusion, intersect_and, resolve_default, union_op, xor_op, Enricher,
    FieldSelector, Handler, HighlightConfig, MetadataSource, Resolver, ResolverError,
    ResolverOptions, ResolverResult, TagIntegrationConfig,
};

pub use crate::search::{
    multi_field_search, multi_field_search_with_weights, multi_term_search, resolve_default_search,
    search, single_term_query, BoostStrategy, CacheKeyGenerator, CacheStats, CachedSearch,
    CombineStrategy, FieldBoostConfig, FieldSearch, MultiFieldSearchConfig,
    MultiFieldSearchOptions, SearchCache, SearchCoordinator, SearchResult, SingleTermResult,
};

pub use crate::serialize::{
    ChunkDataProvider, ChunkedSerializer, CompressionAlgorithm, ExportData, IndexExportData,
    IndexInfo, SerializeConfig, SerializeFormat,
};

pub use crate::storage::common::r#trait::StorageInterface;
pub use crate::storage::common::types::StorageInfo;

pub use crate::tokenizer::{Tokenizer, TokenizerMode};

// Re-export types from r#type module
pub use crate::r#type::{
    ContextOptions, DocumentSearchResult, DocumentSearchResults, EncoderOptions,
    EnrichedDocumentSearchResult, EnrichedDocumentSearchResults, EnrichedSearchResult,
    EnrichedSearchResults, FieldOption, HighlightBoundaryOptions, HighlightEllipsisOptions,
    HighlightOptions, IndexOptions, IntermediateSearchResults, MergedDocumentSearchEntry,
    MergedDocumentSearchResults, SearchOptions, SearchResults, TagOption,
};

// Storage backends - conditionally available
#[cfg(feature = "store-file")]
pub use crate::storage::file::FileStorage;

#[cfg(feature = "store-redis")]
pub use crate::storage::redis::RedisStorage;

#[cfg(feature = "store-wal")]
pub use crate::storage::wal::{IndexChange, WALManager, WALStorage};

#[cfg(feature = "store-wal")]
pub use crate::config::WALConfig as StorageWalConfig;

#[cfg(feature = "store-cold-warm-cache")]
pub use crate::storage::cold_warm_cache::{ColdWarmCacheConfig, ColdWarmCacheManager};
