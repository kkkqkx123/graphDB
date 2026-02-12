//! 验证上下文基础数据类型定义
//! 包含所有验证上下文相关的核心数据结构

use crate::core::types::DataType;

/// 图空间信息
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub space_name: String,
    pub space_id: Option<u32>,
    pub is_default: bool,
    pub vid_type: DataType,
}

/// 列定义
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub type_: DataType,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub comment: Option<String>,
}

/// 列定义集合 - 一个变量包含多个列
pub type ColsDef = Vec<Column>;

/// 变量定义 - 在查询中定义的变量（如MATCH中的别名）
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub columns: ColsDef,
}

impl Variable {
    /// 创建新的变量
    pub fn new(name: String, columns: ColsDef) -> Self {
        Self { name, columns }
    }

    /// 检查变量是否有指定的列
    pub fn has_column(&self, col_name: &str) -> bool {
        self.columns.iter().any(|c| c.name == col_name)
    }

    /// 获取指定列的类型
    pub fn get_column_type(&self, col_name: &str) -> Option<&DataType> {
        self.columns
            .iter()
            .find(|c| c.name == col_name)
            .map(|c| &c.type_)
    }
}

impl Column {
    /// 创建新的列定义
    pub fn new(name: String, type_: DataType) -> Self {
        Column {
            name,
            type_,
            nullable: false,
            default_value: None,
            comment: None,
        }
    }

    /// 创建可空列
    pub fn nullable(name: String, type_: DataType) -> Self {
        Column {
            name,
            type_,
            nullable: true,
            default_value: None,
            comment: None,
        }
    }

    /// 设置默认值
    pub fn with_default(mut self, default_value: String) -> Self {
        self.default_value = Some(default_value);
        self
    }

    /// 设置注释
    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

impl SpaceInfo {
    pub fn new(space_name: String, space_id: Option<u32>, is_default: bool) -> Self {
        Self {
            space_name,
            space_id,
            is_default,
            vid_type: DataType::String,
        }
    }

    pub fn with_vid_type(mut self, vid_type: DataType) -> Self {
        self.vid_type = vid_type;
        self
    }
}

impl Default for SpaceInfo {
    fn default() -> Self {
        Self {
            space_name: String::new(),
            space_id: None,
            is_default: false,
            vid_type: DataType::String,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::DataType;

    #[test]
    fn test_variable_creation() {
        let cols = vec![
            Column::new("id".to_string(), DataType::Int),
            Column::new("name".to_string(), DataType::String),
        ];

        let var = Variable::new("person".to_string(), cols);
        assert_eq!(var.name, "person");
        assert_eq!(var.columns.len(), 2);
    }

    #[test]
    fn test_variable_has_column() {
        let cols = vec![
            Column::new("id".to_string(), DataType::Int),
            Column::new("name".to_string(), DataType::String),
        ];

        let var = Variable::new("person".to_string(), cols);
        assert!(var.has_column("id"));
        assert!(var.has_column("name"));
        assert!(!var.has_column("age"));
    }

    #[test]
    fn test_variable_get_column_type() {
        let cols = vec![
            Column::new("id".to_string(), DataType::Int),
            Column::new("name".to_string(), DataType::String),
        ];

        let var = Variable::new("person".to_string(), cols);
        assert_eq!(var.get_column_type("id"), Some(&DataType::Int));
        assert_eq!(var.get_column_type("name"), Some(&DataType::String));
        assert_eq!(var.get_column_type("age"), None);
    }

    #[test]
    fn test_space_info_creation() {
        let space = SpaceInfo::new("test_space".to_string(), Some(1), false);
        assert_eq!(space.space_name, "test_space");
        assert_eq!(space.space_id, Some(1));
        assert_eq!(space.is_default, false);
        assert_eq!(space.vid_type, DataType::String);
    }

    #[test]
    fn test_column_creation() {
        let col = Column::new("id".to_string(), DataType::Int);
        assert_eq!(col.name, "id");
        assert_eq!(col.type_, DataType::Int);
        assert!(!col.nullable);
        assert_eq!(col.default_value, None);
    }

    #[test]
    fn test_nullable_column() {
        let col = Column::nullable("age".to_string(), DataType::Int);
        assert_eq!(col.name, "age");
        assert_eq!(col.type_, DataType::Int);
        assert!(col.nullable);
    }

    #[test]
    fn test_column_with_default() {
        let col = Column::new("status".to_string(), DataType::String)
            .with_default("active".to_string());
        assert_eq!(col.default_value, Some("active".to_string()));
    }

    #[test]
    fn test_column_with_comment() {
        let col = Column::new("description".to_string(), DataType::String)
            .with_comment("This is a description".to_string());
        assert_eq!(col.comment, Some("This is a description".to_string()));
    }
}
