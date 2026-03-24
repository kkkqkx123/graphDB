//! Merge 语句验证器
//! 用于验证 MERGE 语句（Cypher 风格的模式创建/匹配）
//! 参考 nebula-graph MaintainValidator.cpp 中的 MergeZoneValidator 实现

use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::Expression;
use crate::query::parser::ast::stmt::{Ast, MergeStmt, SetClause};
use crate::query::parser::ast::Pattern;
use crate::query::validator::structs::validation_info::ValidationInfo;
use crate::query::validator::structs::AliasType;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};
use crate::query::QueryContext;

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
                format!(
                    "Variable name must start with a letter or underscore: {}",
                    name
                ),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    /// 验证属性名
    fn validate_property_name(&self, name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::new(
                "Property name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 属性名必须以字母或下划线开头
        let first_char = name.chars().next().expect("属性名已验证非空");
        if !first_char.is_alphabetic() && first_char != '_' {
            return Err(ValidationError::new(
                format!(
                    "Property name must start with a letter or underscore: {}",
                    name
                ),
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
    fn validate_properties(&self, props: &ContextualExpression) -> Result<(), ValidationError> {
        if let Some(e) = props.get_expression() {
            self.validate_properties_internal(&e)
        } else {
            Err(ValidationError::new(
                "属性表达式无效".to_string(),
                ValidationErrorType::SemanticError,
            ))
        }
    }

    fn validate_properties_internal(&self, props: &Expression) -> Result<(), ValidationError> {
        match props {
            Expression::Map(items) => {
                if items.is_empty() {
                    return Err(ValidationError::new(
                        "属性不能为空".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }

                for (key, value) in items {
                    self.validate_property_name(key)?;
                    self.validate_expression_recursive(value)?;
                }
                Ok(())
            }
            _ => Err(ValidationError::new(
                "属性必须是映射类型".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 验证属性值
    fn validate_property_value(&self, value: &ContextualExpression) -> Result<(), ValidationError> {
        if let Some(e) = value.get_expression() {
            self.validate_property_value_internal(&e)
        } else {
            Err(ValidationError::new(
                "属性值表达式无效".to_string(),
                ValidationErrorType::SemanticError,
            ))
        }
    }

    fn validate_property_value_internal(&self, value: &Expression) -> Result<(), ValidationError> {
        self.validate_expression_recursive(value)
    }

    /// 递归验证表达式
    fn validate_expression_recursive(&self, expr: &Expression) -> Result<(), ValidationError> {
        match expr {
            Expression::Literal(_) => Ok(()),
            Expression::Variable(_) => Ok(()),
            Expression::Function { args, .. } => {
                if args.is_empty() {
                    return Err(ValidationError::new(
                        "函数调用必须有参数".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                for arg in args.iter() {
                    self.validate_expression_recursive(arg)?;
                }
                Ok(())
            }
            Expression::Binary { left, right, .. } => {
                self.validate_expression_recursive(left)?;
                self.validate_expression_recursive(right)?;
                Ok(())
            }
            Expression::Unary { operand, .. } => {
                self.validate_expression_recursive(operand)?;
                Ok(())
            }
            Expression::List(items) => {
                for item in items.iter() {
                    self.validate_expression_recursive(item)?;
                }
                Ok(())
            }
            Expression::Map(items) => {
                for (_, value) in items.iter() {
                    self.validate_expression_recursive(value)?;
                }
                Ok(())
            }
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                if let Some(test) = test_expr {
                    self.validate_expression_recursive(test)?;
                }
                for (cond, val) in conditions.iter() {
                    self.validate_expression_recursive(cond)?;
                    self.validate_expression_recursive(val)?;
                }
                if let Some(def) = default {
                    self.validate_expression_recursive(def)?;
                }
                Ok(())
            }
            Expression::Property { object, .. } => {
                self.validate_expression_recursive(object)?;
                Ok(())
            }
            Expression::Aggregate { arg, .. } => {
                self.validate_expression_recursive(arg)?;
                Ok(())
            }
            Expression::TypeCast { expression, .. } => {
                self.validate_expression_recursive(expression)?;
                Ok(())
            }
            Expression::Subscript {
                collection, index, ..
            } => {
                self.validate_expression_recursive(collection)?;
                self.validate_expression_recursive(index)?;
                Ok(())
            }
            Expression::Range {
                collection,
                start,
                end,
                ..
            } => {
                self.validate_expression_recursive(collection)?;
                if let Some(s) = start {
                    self.validate_expression_recursive(s)?;
                }
                if let Some(e) = end {
                    self.validate_expression_recursive(e)?;
                }
                Ok(())
            }
            Expression::Path(exprs) => {
                for expr in exprs.iter() {
                    self.validate_expression_recursive(expr)?;
                }
                Ok(())
            }
            Expression::Label(_) => Ok(()),
            Expression::ListComprehension {
                source,
                filter,
                map,
                ..
            } => {
                self.validate_expression_recursive(source)?;
                if let Some(f) = filter {
                    self.validate_expression_recursive(f)?;
                }
                if let Some(m) = map {
                    self.validate_expression_recursive(m)?;
                }
                Ok(())
            }
            Expression::LabelTagProperty { tag, .. } => {
                self.validate_expression_recursive(tag)?;
                Ok(())
            }
            Expression::TagProperty { .. } => Ok(()),
            Expression::EdgeProperty { .. } => Ok(()),
            Expression::Predicate { args, .. } => {
                for arg in args.iter() {
                    self.validate_expression_recursive(arg)?;
                }
                Ok(())
            }
            Expression::Reduce {
                initial,
                source,
                mapping,
                ..
            } => {
                self.validate_expression_recursive(initial)?;
                self.validate_expression_recursive(source)?;
                self.validate_expression_recursive(mapping)?;
                Ok(())
            }
            Expression::PathBuild(exprs) => {
                for expr in exprs.iter() {
                    self.validate_expression_recursive(expr)?;
                }
                Ok(())
            }
            Expression::Parameter(_) => Ok(()),
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
        self.outputs = vec![ColumnDef {
            name: "result".to_string(),
            type_: ValueType::Vertex, // 可能是顶点或边
        }];
    }

    fn extract_pattern_info(&self, pattern: &Pattern, info: &mut ValidationInfo) {
        use crate::query::parser::ast::Pattern;

        match pattern {
            Pattern::Node(node) => {
                if let Some(ref var) = node.variable {
                    info.add_alias(var.clone(), AliasType::Node);
                }
                for label in &node.labels {
                    if !info.semantic_info.referenced_tags.contains(label) {
                        info.semantic_info.referenced_tags.push(label.clone());
                    }
                }
            }
            Pattern::Edge(edge) => {
                if let Some(ref var) = edge.variable {
                    info.add_alias(var.clone(), AliasType::Edge);
                }
                for edge_type in &edge.edge_types {
                    if !info.semantic_info.referenced_edges.contains(edge_type) {
                        info.semantic_info.referenced_edges.push(edge_type.clone());
                    }
                }
            }
            Pattern::Path(path) => {
                for element in &path.elements {
                    self.extract_path_element_info(element, info);
                }
            }
            Pattern::Variable(var) => {
                info.add_alias(var.name.clone(), AliasType::Variable);
            }
        }
    }

    fn extract_path_element_info(
        &self,
        element: &crate::query::parser::ast::PathElement,
        info: &mut ValidationInfo,
    ) {
        use crate::query::parser::ast::PathElement;

        match element {
            PathElement::Node(node) => {
                if let Some(ref var) = node.variable {
                    info.add_alias(var.clone(), AliasType::Node);
                }
                for label in &node.labels {
                    if !info.semantic_info.referenced_tags.contains(label) {
                        info.semantic_info.referenced_tags.push(label.clone());
                    }
                }
            }
            PathElement::Edge(edge) => {
                if let Some(ref var) = edge.variable {
                    info.add_alias(var.clone(), AliasType::Edge);
                }
                for edge_type in &edge.edge_types {
                    if !info.semantic_info.referenced_edges.contains(edge_type) {
                        info.semantic_info.referenced_edges.push(edge_type.clone());
                    }
                }
            }
            PathElement::Alternative(patterns) => {
                for pattern in patterns {
                    self.extract_pattern_info(pattern, info);
                }
            }
            PathElement::Optional(inner) => {
                self.extract_path_element_info(inner, info);
            }
            PathElement::Repeated(inner, _) => {
                self.extract_path_element_info(inner, info);
            }
        }
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
/// - validate 方法接收 Arc<Ast> 和 Arc<QueryContext>
impl StatementValidator for MergeValidator {
    fn validate(
        &mut self,
        ast: Arc<Ast>,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let merge_stmt = match &ast.stmt {
            crate::query::parser::ast::Stmt::Merge(merge_stmt) => merge_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected MERGE statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        self.validate_impl(merge_stmt)?;

        let mut info = ValidationInfo::new();

        if let Some(ref pattern) = self.pattern {
            self.extract_pattern_info(pattern, &mut info);
        }

        Ok(ValidationResult::success_with_info(info))
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
