//! 错误处理测试
//!
//! 测试范围：
//! - 各种错误类型
//! - 错误转换
//! - 错误消息格式

use inversearch_service::error::{
    InversearchError, IndexError, SearchError, EncoderError, StorageError, CacheError,
};

// ============================================================================
// IndexError 测试
// ============================================================================

/// 测试 IndexError::EmptyContent 显示
#[test]
fn test_index_error_empty_content() {
    let err = IndexError::EmptyContent;
    assert_eq!(format!("{}", err), "Empty content");
}

/// 测试 IndexError::InvalidId 显示
#[test]
fn test_index_error_invalid_id() {
    let err = IndexError::InvalidId(999);
    assert!(format!("{}", err).contains("999"));
}

/// 测试 IndexError::NotFound 显示
#[test]
fn test_index_error_not_found() {
    let err = IndexError::NotFound(123);
    assert!(format!("{}", err).contains("123"));
}

/// 测试 IndexError::Encoding 显示
#[test]
fn test_index_error_encoding() {
    let err = IndexError::Encoding("test error".to_string());
    assert!(format!("{}", err).contains("test error"));
}

/// 测试 IndexError 转换为 InversearchError
#[test]
fn test_index_error_conversion() {
    let err: InversearchError = IndexError::EmptyContent.into();
    assert!(matches!(err, InversearchError::Index(_)));
}

// ============================================================================
// SearchError 测试
// ============================================================================

/// 测试 SearchError::EmptyQuery 显示
#[test]
fn test_search_error_empty_query() {
    let err = SearchError::EmptyQuery;
    assert_eq!(format!("{}", err), "Empty query");
}

/// 测试 SearchError::InvalidOptions 显示
#[test]
fn test_search_error_invalid_options() {
    let err = SearchError::InvalidOptions("bad option".to_string());
    assert!(format!("{}", err).contains("bad option"));
}

/// 测试 SearchError::NoResults 显示
#[test]
fn test_search_error_no_results() {
    let err = SearchError::NoResults;
    assert_eq!(format!("{}", err), "No results found");
}

/// 测试 SearchError::Timeout 显示
#[test]
fn test_search_error_timeout() {
    let err = SearchError::Timeout;
    assert_eq!(format!("{}", err), "Search timeout");
}

/// 测试 SearchError 转换为 InversearchError
#[test]
fn test_search_error_conversion() {
    let err: InversearchError = SearchError::EmptyQuery.into();
    assert!(matches!(err, InversearchError::Search(_)));
}

// ============================================================================
// EncoderError 测试
// ============================================================================

/// 测试 EncoderError::InvalidRegex 显示
#[test]
fn test_encoder_error_invalid_regex() {
    let err = EncoderError::InvalidRegex("bad regex".to_string());
    assert!(format!("{}", err).contains("bad regex"));
}

/// 测试 EncoderError::Encoding 显示
#[test]
fn test_encoder_error_encoding() {
    let err = EncoderError::Encoding("encode failed".to_string());
    assert!(format!("{}", err).contains("encode failed"));
}

/// 测试 EncoderError::Normalization 显示
#[test]
fn test_encoder_error_normalization() {
    let err = EncoderError::Normalization("normalize failed".to_string());
    assert!(format!("{}", err).contains("normalize failed"));
}

/// 测试 EncoderError 转换为 InversearchError
#[test]
fn test_encoder_error_conversion() {
    let err: InversearchError = EncoderError::InvalidRegex("test".to_string()).into();
    assert!(matches!(err, InversearchError::Encoder(_)));
}

// ============================================================================
// StorageError 测试
// ============================================================================

/// 测试 StorageError::Connection 显示
#[test]
fn test_storage_error_connection() {
    let err = StorageError::Connection("connection refused".to_string());
    assert!(format!("{}", err).contains("connection refused"));
}

/// 测试 StorageError::Query 显示
#[test]
fn test_storage_error_query() {
    let err = StorageError::Query("query failed".to_string());
    assert!(format!("{}", err).contains("query failed"));
}

/// 测试 StorageError::Serialization 显示
#[test]
fn test_storage_error_serialization() {
    let err = StorageError::Serialization("serialize failed".to_string());
    assert!(format!("{}", err).contains("serialize failed"));
}

/// 测试 StorageError::Deserialization 显示
#[test]
fn test_storage_error_deserialization() {
    let err = StorageError::Deserialization("deserialize failed".to_string());
    assert!(format!("{}", err).contains("deserialize failed"));
}

/// 测试 StorageError::Generic 显示
#[test]
fn test_storage_error_generic() {
    let err = StorageError::Generic("unknown error".to_string());
    assert!(format!("{}", err).contains("unknown error"));
}

/// 测试 StorageError 转换为 InversearchError
#[test]
fn test_storage_error_conversion() {
    let err: InversearchError = StorageError::Connection("test".to_string()).into();
    assert!(matches!(err, InversearchError::Storage(_)));
}

// ============================================================================
// CacheError 测试
// ============================================================================

/// 测试 CacheError::Miss 显示
#[test]
fn test_cache_error_miss() {
    let err = CacheError::Miss;
    assert_eq!(format!("{}", err), "Cache miss");
}

/// 测试 CacheError::Error 显示
#[test]
fn test_cache_error_generic() {
    let err = CacheError::Error("cache failed".to_string());
    assert!(format!("{}", err).contains("cache failed"));
}

/// 测试 CacheError 转换为 InversearchError
#[test]
fn test_cache_error_conversion() {
    let err: InversearchError = CacheError::Miss.into();
    assert!(matches!(err, InversearchError::Cache(_)));
}

// ============================================================================
// InversearchError 测试
// ============================================================================

/// 测试 InversearchError::Highlight 显示
#[test]
fn test_inversearch_error_highlight() {
    let err = InversearchError::Highlight("highlight failed".to_string());
    assert!(format!("{}", err).contains("highlight failed"));
}

/// 测试 InversearchError::Config 显示
#[test]
fn test_inversearch_error_config() {
    let err = InversearchError::Config("invalid config".to_string());
    assert!(format!("{}", err).contains("invalid config"));
}

/// 测试 InversearchError::Serialization 显示
#[test]
fn test_inversearch_error_serialization() {
    let err = InversearchError::Serialization("serialize error".to_string());
    assert!(format!("{}", err).contains("serialize error"));
}

/// 测试 InversearchError::Deserialization 显示
#[test]
fn test_inversearch_error_deserialization() {
    let err = InversearchError::Deserialization("deserialize error".to_string());
    assert!(format!("{}", err).contains("deserialize error"));
}

/// 测试 InversearchError::AsyncError 显示
#[test]
fn test_inversearch_error_async() {
    let err = InversearchError::AsyncError("async error".to_string());
    assert!(format!("{}", err).contains("async error"));
}

/// 测试 InversearchError::DuplicateFieldName 显示
#[test]
fn test_inversearch_error_duplicate_field() {
    let err = InversearchError::DuplicateFieldName("field1".to_string(), 5);
    let msg = format!("{}", err);
    assert!(msg.contains("field1"));
    assert!(msg.contains("5"));
}

// ============================================================================
// 错误链测试
// ============================================================================

/// 测试错误从底层错误转换
#[test]
fn test_error_chain_from_index() {
    let index_err = IndexError::NotFound(42);
    let inversearch_err: InversearchError = index_err.into();

    match inversearch_err {
        InversearchError::Index(e) => {
            match e {
                IndexError::NotFound(id) => assert_eq!(id, 42),
                _ => panic!("Expected NotFound error"),
            }
        }
        _ => panic!("Expected Index error"),
    }
}

/// 测试错误从底层错误转换 - Search
#[test]
fn test_error_chain_from_search() {
    let search_err = SearchError::Timeout;
    let inversearch_err: InversearchError = search_err.into();

    match inversearch_err {
        InversearchError::Search(e) => {
            match e {
                SearchError::Timeout => (),
                _ => panic!("Expected Timeout error"),
            }
        }
        _ => panic!("Expected Search error"),
    }
}

// ============================================================================
// Result 类型测试
// ============================================================================

/// 测试 Result 类型成功情况
#[test]
fn test_result_ok() {
    let result: inversearch_service::error::Result<u64> = Ok(42);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

/// 测试 Result 类型错误情况
#[test]
fn test_result_err() {
    let result: inversearch_service::error::Result<u64> = 
        Err(InversearchError::Index(IndexError::NotFound(1)));
    assert!(result.is_err());
}

/// 测试 Result 错误转换
#[test]
fn test_result_error_conversion() {
    fn returns_error() -> inversearch_service::error::Result<()> {
        Err(IndexError::EmptyContent.into())
    }

    let result = returns_error();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, InversearchError::Index(_)));
}
