//! Merge 语句验证器
//! 用于验证 MERGE 语句（Cypher 风格的模式创建/匹配）
//! 参考 nebula-graph MaintainValidator.cpp 中的 MergeZoneValidator 实现

use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::Expression;
use crate::query::QueryContext;
use crate::query::parser::ast::stmt::{MergeStmt, SetClause};
use crate::query::parser::ast::Pattern;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};

/// Merge 语句验证器
#[derive(Debug)]
pub struct MergeValidator {
    pattern: Option<Pattern>,
    on_create: Option<SetClause>,
    on_match: Option<SetClause>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl MergeValidator {
    /// 创建新的 Merge 验证器
    pub fn new() -> Self {
        Self {
            pattern: None,
            on_create: None,
            on_match: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    /// 验证模式
    fn validate_pattern(&self, pattern: &Pattern) -> Result<(), ValidationError> {
        use crate::query::parser::ast::Pattern;

        match pattern {
            Pattern::Node(node) => self.validate_node_pattern(node),
            Pattern::Edge(edge) => self.validate_edge_pattern(edge),
            Pattern::Path(path) => self.validate_path_pattern(path),
            Pattern::Variable(var) => self.validate_variable_pattern(var),
        }
    }

    /// 验证节点模式
    fn validate_node_pattern(
        &self,
        node: &crate::query::parser::ast::NodePattern,
    ) -> Result<(), ValidationError> {
        // 验证变量名（如果有）
        if let Some(ref var) = node.variable {
            self.validate_variable_name(var)?;
        }

        // 验证标签（如果有）
        for label in &node.labels {
            self.validate_label_name(label)?;
        }

        // 验证属性（如果有）
        if let Some(ref props) = node.properties {
            self.validate_properties(props)?;
        }

        Ok(())
    }

    /// 验证边模式
    fn validate_edge_pattern(
        &self,
        edge: &crate::query::parser::ast::EdgePattern,
    ) -> Result<(), ValidationError> {
        // 验证变量名（如果有）
        if let Some(ref var) = edge.variable {
            self.validate_variable_name(var)?;
        }

        // 验证边类型（如果有）
        for type_ in &edge.edge_types {
            self.validate_edge_type(type_)?;
        }

        // 验证属性（如果有）
        if let Some(ref props) = edge.properties {
            self.validate_properties(props)?;
        }

        Ok(())
    }

    /// 验证路径模式
    fn validate_path_pattern(
        &self,
        path: &crate::query::parser::ast::PathPattern,
    ) -> Result<(), ValidationError> {
        // 验证路径中的每个元素
        for element in &path.elements {
            self.validate_path_element(element)?;
        }
        Ok(())
    }

    /// 验证路径元素
    fn validate_path_element(
        &self,
        element: &crate::query::parser::ast::PathElement,
    ) -> Result<(), ValidationError> {
        use crate::query::parser::ast::PathElement;

        match element {
            PathElement::Node(node) => self.validate_node_pattern(node),
            PathElement::Edge(edge) => self.validate_edge_pattern(edge),
            PathElement::Alternative(patterns) => {
                for pattern in patterns {
                    self.validate_pattern(pattern)?;
                }
                Ok(())
            }
            PathElement::Optional(inner) => self.validate_path_element(inner),
            PathElement::Repeated(inner, _) => self.validate_path_element(inner),
        }
    }

    /// 验证变量模式
    fn validate_variable_pattern(
        &self,
        var: &crate::query::parser::ast::VariablePattern,
    ) -> Result<(), ValidationError> {
        // 验证变量名
        self.validate_variable_name(&var.name)
    }

    /// 验证变量名
    fn validate_variable_name(&self, name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::new(
                "Variable name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 变量名必须以字母或下划线开头
        let first_char = name.chars().next().expect("变量名已验证非空");
        if !first_char.is_alphabetic() && first_char != '_' {
            return Err(ValidationError::new(
                format!("Variable name must start with a letter or underscore: {}", name),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    /// 验证标签名
    fn validate_label_name(&self, name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::new(
                "Label name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证边类型
    fn validate_edge_type(&self, type_: &str) -> Result<(), ValidationError> {
        if type_.is_empty() {
            return Err(ValidationError::new(
                "Edge type cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证属性表达式
    fn validate_properties(
        &self,
        props: &ContextualExpression,
    ) -> Result<(), ValidationError> {
        if let Some(e) = props.get_expression() {
            self.validate_properties_internal(&e)
        } else {
            Err(ValidationError::new(
                "属性表达式无效".to_string(),
                ValidationErrorType::SemanticError,
            ))
        }
    }

    /// 内部方法：验证属性表达式
    fn validate_properties_internal(
        &self,
        props: &crate::core::types::expression::Expression,
    ) -> Result<(), ValidationError> {
        use crate::core::types::expression::Expression;

        // 属性表达式应该是一个 Map 或 Literal
        match props {
            Expression::Literal(_) => Ok(()),
            Expression::Variable(_) => Ok(()),
            _ => Ok(()), // 其他类型也允许
        }
    }

    /// 验证属性值
    fn validate_property_value(
        &self,
        value: &ContextualExpression,
    ) -> Result<(), ValidationError> {
        if let Some(e) = value.get_expression() {
            self.validate_property_value_internal(&e)
        } else {
            Err(ValidationError::new(
                "属性值表达式无效".to_string(),
                ValidationErrorType::SemanticError,
            ))
        }
    }

    /// 内部方法：验证属性值
    fn validate_property_value_internal(
        &self,
        value: &crate::core::types::expression::Expression,
    ) -> Result<(), ValidationError> {
        use crate::core::types::expression::Expression;

        // 属性值可以是常量、变量、列表等
        match value {
            Expression::Literal(_) => Ok(()),
            Expression::Variable(_) => Ok(()),
            _ => Ok(()), // 其他类型也允许
        }
    }

    /// 验证 SET 子句
    fn validate_set_clause(&self, set_clause: &SetClause) -> Result<(), ValidationError> {
        for assignment in &set_clause.assignments {
            // 验证属性名
            if assignment.property.is_empty() {
                return Err(ValidationError::new(
                    "Property name cannot be empty".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }

            // 验证赋值值
            self.validate_property_value(&assignment.value)?;
        }

        Ok(())
    }

    fn validate_impl(&mut self, stmt: &MergeStmt) -> Result<(), ValidationError> {
        // 验证模式
        self.validate_pattern(&stmt.pattern)?;

        // 验证 ON CREATE 子句
        if let Some(ref on_create) = stmt.on_create {
            self.validate_set_clause(on_create)?;
        }

        // 验证 ON MATCH 子句
        if let Some(ref on_match) = stmt.on_match {
            self.validate_set_clause(on_match)?;
        }

        // 保存信息
        self.pattern = Some(stmt.pattern.clone());
        self.on_create = stmt.on_create.clone();
        self.on_match = stmt.on_match.clone();

        // 设置输出列
        self.setup_outputs();

        Ok(())
    }

    fn setup_outputs(&mut self) {
        // MERGE 语句返回创建的/匹配的节点或边
        self.outputs = vec![
            ColumnDef {
                name: "result".to_string(),
                type_: ValueType::Vertex, // 可能是顶点或边
            },
        ];
    }
}

impl Default for MergeValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for MergeValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let merge_stmt = match stmt {
            crate::query::parser::ast::Stmt::Merge(merge_stmt) => merge_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected MERGE statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        self.validate_impl(merge_stmt)?;

        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Merge
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // MERGE 不是全局语句，需要在特定空间执行
        false
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_validator_new() {
        let validator = MergeValidator::new();
        assert_eq!(validator.statement_type(), StatementType::Merge);
        assert!(!validator.is_global_statement());
    }

    #[test]
    fn test_validate_variable_name() {
        let validator = MergeValidator::new();
        
        // 有效变量名
        assert!(validator.validate_variable_name("n").is_ok());
        assert!(validator.validate_variable_name("node1").is_ok());
        assert!(validator.validate_variable_name("_node").is_ok());
        
        // 无效变量名
        assert!(validator.validate_variable_name("").is_err());
        assert!(validator.validate_variable_name("1node").is_err());
    }

    #[test]
    fn test_validate_label_name() {
        let validator = MergeValidator::new();
        
        // 有效标签名
        assert!(validator.validate_label_name("Person").is_ok());
        
        // 无效标签名
        assert!(validator.validate_label_name("").is_err());
    }
}
