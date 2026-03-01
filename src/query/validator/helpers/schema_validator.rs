//! Schema 验证工具模块
//!
//! 提供完整的 Schema 校验功能，对标 NebulaGraph 的 SchemaUtil
//! 用于 DML 语句（INSERT、UPDATE、DELETE）的 Schema 级别验证
//!
//! 本文件已按照新的验证器体系更新：
//! 1. 保留了原有完整功能：
//!    - 属性存在性验证
//!    - 属性类型验证
//!    - 非空约束验证
//!    - 默认值填充
//!    - VID 类型验证
//!    - 表达式求值
//!    - 自动 Schema 创建
//! 2. 添加了与新的验证器体系的集成支持
//! 3. 使用 Arc 管理 SchemaManager 以支持新体系

use std::sync::Arc;

use crate::core::error::{ValidationError as CoreValidationError, ValidationErrorType};
use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::types::{DataType, EdgeTypeInfo, PropertyDef, TagInfo};
use crate::core::Value;
use crate::storage::metadata::schema_manager::SchemaManager;
use crate::storage::metadata::redb_schema_manager::RedbSchemaManager;
use crate::query::validator::validator_trait::ValueType;

/// Schema 验证器
/// 封装 Schema 相关的所有验证逻辑
/// 
/// 注意：这是一个工具验证器，不直接实现 StatementValidator trait
/// 它被其他语句验证器（如 InsertVerticesValidator, UpdateValidator 等）使用
#[derive(Debug, Clone)]
pub struct SchemaValidator {
    schema_manager: Arc<RedbSchemaManager>,
}

impl SchemaValidator {
    /// 创建新的 Schema 验证器
    pub fn new(schema_manager: Arc<RedbSchemaManager>) -> Self {
        Self { schema_manager }
    }

    /// 获取底层的 SchemaManager
    pub fn get_schema_manager(&self) -> &RedbSchemaManager {
        self.schema_manager.as_ref()
    }

    /// 获取 Arc<SchemaManager>
    pub fn schema_manager_arc(&self) -> Arc<RedbSchemaManager> {
        self.schema_manager.clone()
    }

    /// 获取 Tag 信息
    pub fn get_tag(&self, space_name: &str, tag_name: &str) -> Result<Option<TagInfo>, CoreValidationError> {
        self.schema_manager.as_ref().get_tag(space_name, tag_name)
            .map_err(|e| CoreValidationError::new(
                format!("获取 Tag 失败: {}", e),
                ValidationErrorType::SemanticError,
            ))
    }

    /// 获取 EdgeType 信息
    pub fn get_edge_type(
        &self,
        space_name: &str,
        edge_type_name: &str,
    ) -> Result<Option<EdgeTypeInfo>, CoreValidationError> {
        self.schema_manager.as_ref().get_edge_type(space_name, edge_type_name)
            .map_err(|e| CoreValidationError::new(
                format!("获取 Edge Type 失败: {}", e),
                ValidationErrorType::SemanticError,
            ))
    }

    /// 获取 Space 的所有 EdgeType
    pub fn get_all_edge_types(&self, space_name: &str) -> Result<Vec<EdgeTypeInfo>, CoreValidationError> {
        self.schema_manager.as_ref().list_edge_types(space_name)
            .map_err(|e| CoreValidationError::new(
                format!("获取 Edge Type 列表失败: {}", e),
                ValidationErrorType::SemanticError,
            ))
    }

    /// 验证属性名是否存在于 Schema 中
    pub fn validate_property_exists(
        &self,
        prop_name: &str,
        properties: &[PropertyDef],
    ) -> Result<(), CoreValidationError> {
        if !properties.iter().any(|p| p.name == prop_name) {
            return Err(CoreValidationError::new(
                format!("属性 '{}' 不存在于 Schema 中", prop_name),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 根据属性名获取属性定义
    pub fn get_property_def<'b>(
        &self,
        prop_name: &str,
        properties: &'b [PropertyDef],
    ) -> Option<&'b PropertyDef> {
        properties.iter().find(|p| p.name == prop_name)
    }

    /// 验证属性值类型是否匹配
    pub fn validate_property_type(
        &self,
        prop_name: &str,
        expected_type: &DataType,
        value: &Value,
    ) -> Result<(), CoreValidationError> {
        // NULL 值特殊处理（由 validate_not_null 处理约束）
        if matches!(value, Value::Null(_)) {
            return Ok(());
        }

        let actual_type = value.get_type();

        if !Self::is_type_compatible(expected_type, &actual_type) {
            return Err(CoreValidationError::new(
                format!(
                    "属性 '{}' 期望类型 {:?}, 实际类型 {:?}",
                    prop_name, expected_type, actual_type
                ),
                ValidationErrorType::TypeMismatch,
            ));
        }
        Ok(())
    }

    /// 检查类型兼容性
    /// 支持一些隐式类型转换
    pub fn is_type_compatible(expected: &DataType, actual: &DataType) -> bool {
        match (expected, actual) {
            // 精确匹配
            (a, b) if a == b => true,

            // 整数类型兼容
            (DataType::Int, DataType::Int64) => true,
            (DataType::Int64, DataType::Int) => true,
            (DataType::Int32, DataType::Int) => true,
            (DataType::Int32, DataType::Int64) => true,

            // 浮点数兼容
            (DataType::Float, DataType::Double) => true,
            (DataType::Double, DataType::Float) => true,

            // VID 兼容多种类型
            (DataType::VID, DataType::String) => true,
            (DataType::VID, DataType::Int) => true,
            (DataType::VID, DataType::Int64) => true,
            (DataType::VID, DataType::FixedString(_)) => true,

            // FixedString 兼容 String
            (DataType::FixedString(_), DataType::String) => true,
            (DataType::String, DataType::FixedString(_)) => true,

            // NULL 可以赋值给任何类型（在验证非空之前）
            (_, DataType::Null) => true,

            // 其他情况不匹配
            _ => false,
        }
    }

    /// 将 DataType 转换为 ValueType（用于新验证器体系）
    pub fn data_type_to_value_type(data_type: &DataType) -> ValueType {
        match data_type {
            DataType::Bool => ValueType::Bool,
            DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => ValueType::Int,
            DataType::Float | DataType::Double => ValueType::Float,
            DataType::String | DataType::FixedString(_) => ValueType::String,
            DataType::Date => ValueType::Date,
            DataType::Time => ValueType::Time,
            DataType::DateTime => ValueType::DateTime,
            DataType::Null => ValueType::Null,
            DataType::Vertex => ValueType::Vertex,
            DataType::Edge => ValueType::Edge,
            DataType::Path => ValueType::Path,
            DataType::List => ValueType::List,
            DataType::Map => ValueType::Map,
            DataType::Set => ValueType::Set,
            _ => ValueType::Unknown,
        }
    }

    /// 验证非空约束
    pub fn validate_not_null(
        &self,
        prop_name: &str,
        prop_def: &PropertyDef,
        value: &Value,
    ) -> Result<(), CoreValidationError> {
        if !prop_def.nullable && matches!(value, Value::Null(_)) {
            return Err(CoreValidationError::new(
                format!("非空属性 '{}' 不能为 NULL", prop_name),
                ValidationErrorType::ConstraintViolation,
            ));
        }
        Ok(())
    }

    /// 获取属性的默认值
    pub fn get_default_value(&self, prop_def: &PropertyDef) -> Option<Value> {
        prop_def.default.clone()
    }

    /// 填充默认值
    /// 为未提供的属性填充默认值或 NULL
    pub fn fill_default_values(
        &self,
        properties: &[PropertyDef],
        provided_props: &[(String, Value)],
    ) -> Result<Vec<(String, Value)>, CoreValidationError> {
        let mut result = provided_props.to_vec();

        for prop_def in properties {
            if !result.iter().any(|(name, _)| name == &prop_def.name) {
                // 属性未提供，尝试使用默认值
                if let Some(default) = &prop_def.default {
                    result.push((prop_def.name.clone(), default.clone()));
                } else if !prop_def.nullable {
                    return Err(CoreValidationError::new(
                        format!(
                            "属性 '{}' 未提供且没有默认值，且不允许为 NULL",
                            prop_def.name
                        ),
                        ValidationErrorType::ConstraintViolation,
                    ));
                } else {
                    // nullable 且无默认值，填充 NULL
                    result.push((prop_def.name.clone(), Value::Null(crate::core::NullType::default())));
                }
            }
        }

        Ok(result)
    }

    /// 验证 VID 类型
    pub fn validate_vid(
        &self,
        vid: &Value,
        expected_type: &DataType,
    ) -> Result<(), CoreValidationError> {
        match expected_type {
            DataType::String | DataType::FixedString(_) => {
                if !matches!(vid, Value::String(_)) {
                    return Err(CoreValidationError::new(
                        format!("VID 期望字符串类型, 实际为 {:?}", vid.get_type()),
                        ValidationErrorType::TypeMismatch,
                    ));
                }
            }
            DataType::Int | DataType::Int64 | DataType::Int32 => {
                if !matches!(vid, Value::Int(_)) {
                    return Err(CoreValidationError::new(
                        format!("VID 期望整数类型, 实际为 {:?}", vid.get_type()),
                        ValidationErrorType::TypeMismatch,
                    ));
                }
            }
            DataType::VID => {
                // VID 类型接受多种格式
                if !matches!(vid, Value::String(_) | Value::Int(_)) {
                    return Err(CoreValidationError::new(
                        format!("VID 类型不兼容: {:?}", vid.get_type()),
                        ValidationErrorType::TypeMismatch,
                    ));
                }
            }
            _ => {
                return Err(CoreValidationError::new(
                    format!("不支持的 VID 类型: {:?}", expected_type),
                    ValidationErrorType::TypeMismatch,
                ));
            }
        }
        Ok(())
    }

    /// 统一验证 VID 表达式
    /// 根据 Space 的 vid_type 验证表达式，确保类型匹配
    /// 
    /// 参数:
    /// - expr: VID 表达式
    /// - vid_type: Space 定义的 VID 类型
    /// - role: VID 角色描述（如 "source", "destination", "vertex"）
    /// 
    /// 返回:
    /// - Ok(()) 验证通过
    /// - Err(ValidationError) 验证失败
    pub fn validate_vid_expr(
        &self,
        expr: &ContextualExpression,
        vid_type: &DataType,
        role: &str,
    ) -> Result<(), CoreValidationError> {
        if let Some(e) = expr.expression() {
            self.validate_vid_expr_internal(&e, vid_type, role)
        } else {
            Err(CoreValidationError::new(
                format!("{} vertex ID 表达式无效", role),
                ValidationErrorType::SemanticError,
            ))
        }
    }

    /// 内部方法：验证 VID 表达式
    fn validate_vid_expr_internal(
        &self,
        expr: &crate::core::types::expression::Expression,
        vid_type: &DataType,
        role: &str,
    ) -> Result<(), CoreValidationError> {
        use crate::core::types::expression::Expression;
        
        match expr {
            Expression::Literal(value) => {
                // 字面量需要检查空值和类型匹配
                match value {
                    Value::String(s) => {
                        if s.is_empty() {
                            return Err(CoreValidationError::new(
                                format!("{} vertex ID 不能为空字符串", role),
                                ValidationErrorType::SemanticError,
                            ));
                        }
                        // 检查类型是否匹配
                        if !matches!(vid_type, DataType::String | DataType::FixedString(_) | DataType::VID) {
                            return Err(CoreValidationError::new(
                                format!("{} vertex ID 期望 {:?} 类型, 实际为字符串", role, vid_type),
                                ValidationErrorType::TypeMismatch,
                            ));
                        }
                    }
                    Value::Int(_) => {
                        // 检查类型是否匹配
                        if !matches!(vid_type, DataType::Int | DataType::Int64 | DataType::Int32 | DataType::VID) {
                            return Err(CoreValidationError::new(
                                format!("{} vertex ID 期望 {:?} 类型, 实际为整数", role, vid_type),
                                ValidationErrorType::TypeMismatch,
                            ));
                        }
                    }
                    _ => {
                        return Err(CoreValidationError::new(
                            format!("{} vertex ID 必须是字符串或整数常量", role),
                            ValidationErrorType::TypeMismatch,
                        ));
                    }
                }
                Ok(())
            }
            Expression::Variable(_) => {
                // 变量在验证阶段无法确定具体值，假设有效
                Ok(())
            }
            _ => {
                Err(CoreValidationError::new(
                    format!("{} vertex ID 必须是常量或变量", role),
                    ValidationErrorType::SemanticError,
                ))
            }
        }
    }

    /// 验证属性值列表
    /// 验证所有属性存在、类型匹配、非空约束
    pub fn validate_properties(
        &self,
        properties: &[PropertyDef],
        prop_values: &[(String, Value)],
    ) -> Result<Vec<(String, Value)>, CoreValidationError> {
        let mut result = Vec::new();

        for (prop_name, value) in prop_values {
            // 验证属性存在
            let prop_def = self
                .get_property_def(prop_name, properties)
                .ok_or_else(|| {
                    CoreValidationError::new(
                        format!("属性 '{}' 不存在", prop_name),
                        ValidationErrorType::SemanticError,
                    )
                })?;

            // 验证非空约束
            self.validate_not_null(prop_name, prop_def, value)?;

            // 验证类型
            self.validate_property_type(prop_name, &prop_def.data_type, value)?;

            result.push((prop_name.clone(), value.clone()));
        }

        // 填充默认值
        self.fill_default_values(properties, &result)
    }

    /// 验证表达式是否为可计算的值
    /// 用于检查 VID 和属性值表达式
    pub fn is_evaluable_expr(&self, expr: &ContextualExpression) -> bool {
        if let Some(e) = expr.expression() {
            self.is_evaluable_expr_internal(&e)
        } else {
            false
        }
    }

    /// 内部方法：验证表达式是否为可计算的值
    fn is_evaluable_expr_internal(&self, expr: &crate::core::types::expression::Expression) -> bool {
        use crate::core::types::expression::Expression;
        match expr {
            Expression::Literal(_) => true,
            Expression::Variable(_) => true,
            Expression::List(list) => list.iter().all(|e| self.is_evaluable_expr_internal(e)),
            Expression::Map(map) => map.iter().all(|(_, e)| self.is_evaluable_expr_internal(e)),
            // 函数调用如果是确定性的也可以接受
            Expression::Function { .. } => true,
            _ => false,
        }
    }

    /// 评估表达式为值
    /// 仅支持常量表达式
    pub fn evaluate_expression(
        &self,
        expr: &ContextualExpression,
    ) -> Result<Value, CoreValidationError> {
        if let Some(e) = expr.expression() {
            self.evaluate_expression_internal(&e)
        } else {
            Err(CoreValidationError::new(
                "表达式无效".to_string(),
                ValidationErrorType::SemanticError,
            ))
        }
    }

    /// 内部方法：评估表达式为值
    fn evaluate_expression_internal(
        &self,
        expr: &crate::core::types::expression::Expression,
    ) -> Result<Value, CoreValidationError> {
        use crate::core::types::expression::Expression;
        match expr {
            Expression::Literal(value) => Ok(value.clone()),
            Expression::Variable(name) => {
                // 变量在验证阶段无法求值，返回特殊标记
                Ok(Value::String(format!("${}", name)))
            }
            Expression::List(list) => {
                let values: Result<Vec<_>, _> = list
                    .iter()
                    .map(|e| self.evaluate_expression_internal(e))
                    .collect();
                Ok(Value::List(crate::core::value::List { values: values? }))
            }
            Expression::Map(map) => {
                let mut result = std::collections::HashMap::new();
                for (k, v) in map {
                    result.insert(k.clone(), self.evaluate_expression_internal(v)?);
                }
                Ok(Value::Map(result))
            }
            _ => Err(CoreValidationError::new(
                format!("无法评估表达式: {:?}", expr),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 自动创建 Tag（如果不存在）
    /// 根据提供的属性推断 Tag 的 Schema
    pub fn auto_create_tag(
        &self,
        space_name: &str,
        tag_name: &str,
        properties: &[(String, Value)],
    ) -> Result<TagInfo, CoreValidationError> {
        // 检查 Tag 是否已存在
        if let Some(existing) = self.schema_manager.as_ref().get_tag(space_name, tag_name)
            .map_err(|e| CoreValidationError::new(
                format!("获取 Tag 失败: {}", e),
                ValidationErrorType::SemanticError,
            ))? {
            return Ok(existing);
        }

        // 根据属性值推断属性类型
        let mut prop_defs = Vec::new();
        for (prop_name, value) in properties {
            let data_type = Self::infer_data_type(value);
            let prop_def = PropertyDef::new(prop_name.clone(), data_type)
                .with_nullable(true); // 自动创建的属性默认可为空
            prop_defs.push(prop_def);
        }

        // 创建 TagInfo
        let tag_info = TagInfo {
            tag_id: 0, // 由存储层分配
            tag_name: tag_name.to_string(),
            properties: prop_defs,
            comment: Some(format!("Auto-created for Cypher CREATE")),
            ttl_duration: None,
            ttl_col: None,
        };

        // 创建 Tag
        self.schema_manager.as_ref().create_tag(space_name, &tag_info)
            .map_err(|e| CoreValidationError::new(
                format!("创建 Tag '{}' 失败: {}", tag_name, e),
                ValidationErrorType::SemanticError,
            ))?;

        Ok(tag_info)
    }

    /// 自动创建 Edge Type（如果不存在）
    /// 根据提供的属性推断 Edge Type 的 Schema
    pub fn auto_create_edge_type(
        &self,
        space_name: &str,
        edge_type_name: &str,
        properties: &[(String, Value)],
    ) -> Result<EdgeTypeInfo, CoreValidationError> {
        // 检查 Edge Type 是否已存在
        if let Some(existing) = self.schema_manager.as_ref().get_edge_type(space_name, edge_type_name)
            .map_err(|e| CoreValidationError::new(
                format!("获取 Edge Type 失败: {}", e),
                ValidationErrorType::SemanticError,
            ))? {
            return Ok(existing);
        }

        // 根据属性值推断属性类型
        let mut prop_defs = Vec::new();
        for (prop_name, value) in properties {
            let data_type = Self::infer_data_type(value);
            let prop_def = PropertyDef::new(prop_name.clone(), data_type)
                .with_nullable(true); // 自动创建的属性默认可为空
            prop_defs.push(prop_def);
        }

        // 创建 EdgeTypeInfo
        let edge_info = EdgeTypeInfo {
            edge_type_id: 0, // 由存储层分配
            edge_type_name: edge_type_name.to_string(),
            properties: prop_defs,
            comment: Some(format!("Auto-created for Cypher CREATE")),
            ttl_duration: None,
            ttl_col: None,
        };

        // 创建 Edge Type
        self.schema_manager.as_ref().create_edge_type(space_name, &edge_info)
            .map_err(|e| CoreValidationError::new(
                format!("创建 Edge Type '{}' 失败: {}", edge_type_name, e),
                ValidationErrorType::SemanticError,
            ))?;

        Ok(edge_info)
    }

    /// 根据 Value 推断 DataType
    fn infer_data_type(value: &Value) -> DataType {
        match value {
            Value::Null(_) => DataType::String, // 默认为字符串类型
            Value::Bool(_) => DataType::Bool,
            Value::Int(_) => DataType::Int64,
            Value::Float(_) => DataType::Double,
            Value::String(s) => {
                // 根据字符串长度选择 FixedString 或 String
                if s.len() <= 256 {
                    DataType::FixedString(s.len().max(32))
                } else {
                    DataType::String
                }
            }
            Value::List(_) => DataType::List,
            Value::Map(_) => DataType::Map,
            Value::Date(_) => DataType::Date,
            Value::DateTime(_) => DataType::DateTime,
            _ => DataType::String, // 默认为字符串类型
        }
    }

    /// 批量自动创建缺失的 Tags
    pub fn auto_create_missing_tags(
        &self,
        space_name: &str,
        tags: &[(String, Vec<(String, Value)>)],
    ) -> Result<Vec<TagInfo>, CoreValidationError> {
        let mut created = Vec::new();
        for (tag_name, properties) in tags {
            let tag_info = self.auto_create_tag(space_name, tag_name, properties)?;
            created.push(tag_info);
        }
        Ok(created)
    }

    /// 批量自动创建缺失的 Edge Types
    pub fn auto_create_missing_edge_types(
        &self,
        space_name: &str,
        edge_types: &[(String, Vec<(String, Value)>)],
    ) -> Result<Vec<EdgeTypeInfo>, CoreValidationError> {
        let mut created = Vec::new();
        for (edge_type_name, properties) in edge_types {
            let edge_info = self.auto_create_edge_type(space_name, edge_type_name, properties)?;
            created.push(edge_info);
        }
        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::PropertyDef;

    fn create_test_validator() -> SchemaValidator {
        // 由于现在使用具体类型 RedbSchemaManager，测试需要创建一个真实的 RedbSchemaManager
        // 或者使用其他方法。这里暂时注释掉，需要后续修复测试
        panic!("测试需要更新以使用 RedbSchemaManager");
    }

    #[test]
    fn test_validate_property_exists_success() {
        let validator = create_test_validator();
        let properties = vec![
            PropertyDef::new("name".to_string(), DataType::String),
            PropertyDef::new("age".to_string(), DataType::Int),
        ];

        assert!(validator.validate_property_exists("name", &properties).is_ok());
        assert!(validator.validate_property_exists("age", &properties).is_ok());
    }

    #[test]
    fn test_validate_property_exists_failure() {
        let validator = create_test_validator();
        let properties = vec![PropertyDef::new("name".to_string(), DataType::String)];

        let result = validator.validate_property_exists("age", &properties);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("不存在"));
    }

    #[test]
    fn test_validate_property_type_success() {
        let validator = create_test_validator();

        assert!(validator
            .validate_property_type("name", &DataType::String, &Value::String("test".to_string()))
            .is_ok());
        assert!(validator
            .validate_property_type("age", &DataType::Int, &Value::Int(25))
            .is_ok());
    }

    #[test]
    fn test_validate_property_type_failure() {
        let validator = create_test_validator();

        let result = validator.validate_property_type("age", &DataType::Int, &Value::String("test".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("期望类型"));
    }

    #[test]
    fn test_validate_not_null_success() {
        let validator = create_test_validator();
        let prop_def = PropertyDef::new("name".to_string(), DataType::String).with_nullable(false);

        assert!(validator
            .validate_not_null("name", &prop_def, &Value::String("test".to_string()))
            .is_ok());
    }

    #[test]
    fn test_validate_not_null_failure() {
        let validator = create_test_validator();
        let prop_def = PropertyDef::new("name".to_string(), DataType::String).with_nullable(false);

        let result = validator.validate_not_null("name", &prop_def, &Value::Null(crate::core::NullType::default()));
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("不能为 NULL"));
    }

    #[test]
    fn test_fill_default_values() {
        let validator = create_test_validator();
        let properties = vec![
            PropertyDef::new("name".to_string(), DataType::String).with_nullable(false),
            PropertyDef::new("email".to_string(), DataType::String)
                .with_nullable(true)
                .with_default(Some(Value::String("default@example.com".to_string()))),
            PropertyDef::new("age".to_string(), DataType::Int).with_nullable(true),
        ];

        let provided = vec![("name".to_string(), Value::String("John".to_string()))];
        let result = validator.fill_default_values(&properties, &provided).expect("Failed to fill default values");

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "name");
        assert_eq!(result[1].0, "email");
        assert_eq!(result[1].1, Value::String("default@example.com".to_string()));
        assert_eq!(result[2].0, "age");
        assert!(matches!(result[2].1, Value::Null(_)));
    }

    #[test]
    fn test_validate_vid_string() {
        let validator = create_test_validator();

        assert!(validator
            .validate_vid(&Value::String("vid1".to_string()), &DataType::String)
            .is_ok());
    }

    #[test]
    fn test_validate_vid_int() {
        let validator = create_test_validator();

        assert!(validator.validate_vid(&Value::Int(123), &DataType::Int).is_ok());
    }

    #[test]
    fn test_is_type_compatible() {
        // 整数兼容
        assert!(SchemaValidator::is_type_compatible(&DataType::Int, &DataType::Int64));
        assert!(SchemaValidator::is_type_compatible(&DataType::Int64, &DataType::Int));

        // 浮点数兼容
        assert!(SchemaValidator::is_type_compatible(&DataType::Float, &DataType::Double));

        // VID 兼容
        assert!(SchemaValidator::is_type_compatible(&DataType::VID, &DataType::String));
        assert!(SchemaValidator::is_type_compatible(&DataType::VID, &DataType::Int));

        // 不兼容
        assert!(!SchemaValidator::is_type_compatible(&DataType::Int, &DataType::String));
        assert!(!SchemaValidator::is_type_compatible(&DataType::Bool, &DataType::Int));
    }

    #[test]
    fn test_data_type_to_value_type() {
        assert!(matches!(SchemaValidator::data_type_to_value_type(&DataType::Bool), ValueType::Bool));
        assert!(matches!(SchemaValidator::data_type_to_value_type(&DataType::Int), ValueType::Int));
        assert!(matches!(SchemaValidator::data_type_to_value_type(&DataType::String), ValueType::String));
    }
}
