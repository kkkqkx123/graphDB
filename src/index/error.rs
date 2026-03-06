//! 索引错误类型
//!
//! 涵盖索引创建、更新和查询过程中的错误

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("Index creation error: {0}")]
    IndexCreationError(String),
    #[error("Index update error: {0}")]
    IndexUpdateError(String),
    #[error("Index query error: {0}")]
    IndexQueryError(String),
    #[error("Index not found: {0}")]
    IndexNotFound(String),
    #[error("Index status error: {0}")]
    IndexStatusError(String),
}

impl From<String> for IndexError {
    fn from(msg: String) -> Self {
        IndexError::IndexQueryError(msg)
    }
}

impl From<&str> for IndexError {
    fn from(msg: &str) -> Self {
        IndexError::IndexQueryError(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_error_creation() {
        let error = IndexError::IndexCreationError("Creation failed".to_string());
        assert_eq!(format!("{}", error), "Index creation error: Creation failed");
    }

    #[test]
    fn test_index_error_not_found() {
        let error = IndexError::IndexNotFound("test_index".to_string());
        assert_eq!(format!("{}", error), "Index not found: test_index");
    }

    #[test]
    fn test_index_error_from_string() {
        let error: IndexError = "Test error".to_string().into();
        assert_eq!(format!("{}", error), "Index query error: Test error");
    }
}
