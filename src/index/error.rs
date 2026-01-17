//! 索引错误类型
//!
//! 涵盖索引创建、更新和查询过程中的错误

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("索引创建错误: {0}")]
    IndexCreationError(String),
    #[error("索引更新错误: {0}")]
    IndexUpdateError(String),
    #[error("索引查询错误: {0}")]
    IndexQueryError(String),
    #[error("索引不存在: {0}")]
    IndexNotFound(String),
    #[error("索引状态错误: {0}")]
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
        let error = IndexError::IndexCreationError("创建失败".to_string());
        assert_eq!(
            format!("{}", error),
            "索引创建错误: 创建失败"
        );
    }

    #[test]
    fn test_index_error_not_found() {
        let error = IndexError::IndexNotFound("test_index".to_string());
        assert_eq!(
            format!("{}", error),
            "索引不存在: test_index"
        );
    }

    #[test]
    fn test_index_error_from_string() {
        let error: IndexError = "测试错误".to_string().into();
        assert_eq!(format!("{}", error), "索引查询错误: 测试错误");
    }
}
