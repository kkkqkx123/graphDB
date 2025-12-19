//! MATCH语句验证器
//! 对应 NebulaGraph MatchValidator 的功能

use crate::core::error::{DBError, DBResult, ValidationError};
use crate::query::context::{QueryContext, AstContext};
use crate::query::context::ast_context::ColumnDefinition;
use crate::query::parser::cypher::ast::{CypherStatement, MatchClause, WhereClause, ReturnClause};
use crate::query::planner::plan::execution_plan::ExecutionPlan;
use crate::query::planner::plan::core::PlanNode;
use crate::query::validator::validator_trait::{Validator, ValidatorExt};
use crate::query::validator::base_validator::BaseValidator;
use std::sync::Arc;

/// MATCH语句验证器
pub struct MatchValidator {
    base: BaseValidator,
    match_clause: MatchClause,
}

impl MatchValidator {
    pub fn new(statement: CypherStatement, qctx: Arc<QueryContext>) -> DBResult<Self> {
        let match_clause = extract_match_clause(&statement)?;
        let base = BaseValidator::new(statement, qctx);
        
        Ok(Self {
            base,
            match_clause,
        })
    }
    
    /// 验证模式
    fn validate_pattern(&mut self) -> DBResult<()> {
        let patterns = self.match_clause.patterns.clone();
        for pattern in &patterns {
            for pattern_part in &pattern.parts {
                self.validate_pattern_part(pattern_part)?;
            }
        }
        Ok(())
    }
    
    /// 验证模式部分
    fn validate_pattern_part(&mut self, pattern_part: &crate::query::parser::cypher::ast::patterns::PatternPart) -> DBResult<()> {
        // 验证节点模式
        self.validate_node_pattern(&pattern_part.node)?;
        
        // 验证关系模式
        for rel in &pattern_part.relationships {
            self.validate_relationship_pattern(rel)?;
        }
        
        Ok(())
    }
    
    /// 验证节点模式
    fn validate_node_pattern(&mut self, node: &crate::query::parser::cypher::ast::patterns::NodePattern) -> DBResult<()> {
        // 验证标签是否存在
        for label in &node.labels {
            if !self.base.query_context().schema_manager.has_schema(label) {
                return Err(DBError::Validation(
                    ValidationError::SemanticError(
                        format!("Tag not found: {}", label)
                    )
                ));
            }
        }
        
        // 验证属性
        if let Some(properties) = &node.properties {
            self.validate_properties(properties)?;
        }
        
        Ok(())
    }
    
    /// 验证关系模式
    fn validate_relationship_pattern(&mut self, rel: &crate::query::parser::cypher::ast::patterns::RelationshipPattern) -> DBResult<()> {
        // 验证边类型是否存在
        for edge_type in &rel.types {
            if !self.base.query_context().schema_manager.has_schema(edge_type) {
                return Err(DBError::Validation(
                    ValidationError::SemanticError(
                        format!("Edge type not found: {}", edge_type)
                    )
                ));
            }
        }
        
        // 验证属性
        if let Some(properties) = &rel.properties {
            self.validate_properties(properties)?;
        }
        
        Ok(())
    }
    
    /// 验证WHERE子句
    fn validate_where_clause(&mut self) -> DBResult<()> {
        if let Some(where_clause) = &self.match_clause.where_clause {
            let expr = where_clause.expression.clone();
            self.validate_expression(&expr)?;
        }
        Ok(())
    }
    
    /// 验证RETURN子句
    fn validate_return_clause(&mut self) -> DBResult<()> {
        // 这里需要从statement中提取RETURN子句
        // 暂时跳过具体实现
        Ok(())
    }
    
    /// 验证表达式
    fn validate_expression(&mut self, expr: &crate::query::parser::cypher::ast::expressions::Expression) -> DBResult<()> {
        match expr {
            crate::query::parser::cypher::ast::expressions::Expression::Property(_) => {
                // 验证属性表达式
                self.validate_property_expression(expr)?;
            }
            crate::query::parser::cypher::ast::expressions::Expression::FunctionCall(_) => {
                // 验证函数调用
                self.validate_function_call(expr)?;
            }
            crate::query::parser::cypher::ast::expressions::Expression::Binary(_) => {
                // 验证二元表达式
                self.validate_binary_expression(expr)?;
            }
            _ => {} // 其他表达式类型的验证
        }
        Ok(())
    }
    
    /// 验证属性表达式
    fn validate_property_expression(&mut self, _prop: &crate::query::parser::cypher::ast::expressions::Expression) -> DBResult<()> {
        // 验证属性是否存在
        // 这里需要根据变量名和属性名检查schema
        Ok(())
    }
    
    /// 验证函数调用
    fn validate_function_call(&mut self, func: &crate::query::parser::cypher::ast::expressions::Expression) -> DBResult<()> {
        // 验证函数是否存在
        if let crate::query::parser::cypher::ast::expressions::Expression::FunctionCall(func_call) = func {
            if self.base.query_context().get_function(&func_call.function_name).is_none() {
                return Err(DBError::Validation(
                    ValidationError::SemanticError(
                        format!("Function not found: {}", func_call.function_name)
                    )
                ));
            }
            
            // 验证参数数量
            // 验证参数类型
        }
        Ok(())
    }
    
    /// 验证二元表达式
    fn validate_binary_expression(&mut self, bin: &crate::query::parser::cypher::ast::expressions::Expression) -> DBResult<()> {
        if let crate::query::parser::cypher::ast::expressions::Expression::Binary(bin_expr) = bin {
            self.validate_expression(&bin_expr.left)?;
            self.validate_expression(&bin_expr.right)?;
        }
        Ok(())
    }
    
    /// 验证属性
    fn validate_properties(&mut self, properties: &std::collections::HashMap<String, crate::query::parser::cypher::ast::expressions::Expression>) -> DBResult<()> {
        for (name, expr) in properties {
            self.validate_expression(expr)?;
        }
        Ok(())
    }
    
    /// 创建扫描节点
    fn create_scan_node(&self, pattern: &crate::query::parser::cypher::ast::patterns::Pattern) -> DBResult<std::sync::Arc<dyn PlanNode>> {
        // 根据模式创建适当的扫描节点
        if !pattern.parts.is_empty() && !pattern.parts[0].node.labels.is_empty() {
            // 创建标签扫描
            // Ok(Arc::new(IndexScanNode::new(pattern.clone())))
            Err(DBError::Validation(
                ValidationError::PlanError(
                    "IndexScanNode not implemented yet".to_string()
                )
            ))
        } else {
            // 创建全图扫描
            // Ok(Arc::new(ScanVerticesNode::new()))
            Err(DBError::Validation(
                ValidationError::PlanError(
                    "ScanVerticesNode not implemented yet".to_string()
                )
            ))
        }
    }
    
    /// 创建过滤节点
    fn create_filter_node(&self, where_clause: &Option<WhereClause>, input: std::sync::Arc<dyn PlanNode>) -> DBResult<std::sync::Arc<dyn PlanNode>> {
        if let Some(where_clause) = where_clause {
            // Ok(Arc::new(FilterNode::new(where_clause.expression.clone(), input)))
            Err(DBError::Validation(
                ValidationError::PlanError(
                    "FilterNode not implemented yet".to_string()
                )
            ))
        } else {
            Ok(input)
        }
    }
    
    /// 创建投影节点
    fn create_project_node(&self, return_clause: &Option<ReturnClause>, input: std::sync::Arc<dyn PlanNode>) -> DBResult<std::sync::Arc<dyn PlanNode>> {
        // 根据RETURN子句创建投影节点
        // Ok(Arc::new(ProjectNode::new(vec![], input)))
        Err(DBError::Validation(
            ValidationError::PlanError(
                "ProjectNode not implemented yet".to_string()
            )
        ))
    }
}

impl ValidatorExt for MatchValidator {
    fn validate_impl(&mut self) -> DBResult<()> {
        // 验证模式
        self.validate_pattern()?;
        
        // 验证WHERE子句
        self.validate_where_clause()?;
        
        // 验证RETURN子句
        self.validate_return_clause()?;
        
        Ok(())
    }
    
    fn to_plan_impl(&mut self) -> DBResult<ExecutionPlan> {
        // 创建扫描节点
        let scan_node = if !self.match_clause.patterns.is_empty() {
            self.create_scan_node(&self.match_clause.patterns[0])?
        } else {
            return Err(DBError::Validation(
                ValidationError::PlanError(
                    "No patterns found in MATCH clause".to_string()
                )
            ));
        };
        
        // 创建过滤节点
        let filter_node = self.create_filter_node(&self.match_clause.where_clause, scan_node)?;
        
        // 创建投影节点
        let project_node = self.create_project_node(&None, filter_node)?; // 暂时没有RETURN子句
        
        Ok(ExecutionPlan {
            root: Some(project_node),
            id: -1, // 将在后续分配
            optimize_time_in_us: 0,
            format: "default".to_string(),
        })
    }
}

impl Validator for MatchValidator {
    fn validate(&mut self) -> DBResult<()> {
        self.base.validate()?;
        self.validate_impl()
    }
    
    fn to_plan(&mut self) -> DBResult<ExecutionPlan> {
        self.to_plan_impl()
    }
    
    fn ast_context(&self) -> &AstContext {
        self.base.ast_context()
    }
    
    fn name(&self) -> &'static str {
        "MatchValidator"
    }
    
    fn input_var_name(&self) -> Option<&str> {
        self.base.input_var_name()
    }
    
    fn set_input_var_name(&mut self, name: String) {
        self.base.set_input_var_name(name);
    }
    
    fn output_columns(&self) -> &[ColumnDefinition] {
        self.base.output_columns()
    }
    
    fn input_columns(&self) -> &[ColumnDefinition] {
        self.base.input_columns()
    }
}

/// 从CypherStatement中提取MatchClause
fn extract_match_clause(statement: &CypherStatement) -> DBResult<MatchClause> {
    match statement {
        CypherStatement::Match(match_clause) => Ok(match_clause.clone()),
        _ => Err(DBError::Validation(
            ValidationError::SyntaxError(
                "Expected MATCH statement".to_string()
            )
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::cypher::ast::clauses::MatchClause;
    use crate::query::context::managers::r#impl::{
        MockIndexManager, MockMetaClient, MockSchemaManager, MockStorageClient,
    };

    #[test]
    fn test_match_validator_creation() {
        let schema_manager = Arc::new(MockSchemaManager::new());
        let index_manager = Arc::new(MockIndexManager::new());
        let meta_client = Arc::new(MockMetaClient::new());
        let storage_client = Arc::new(MockStorageClient::new());

        let qctx = Arc::new(QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        ));

        let statement = CypherStatement::Match(MatchClause {
            patterns: Vec::new(),
            where_clause: None,
            optional: false,
        });

        let validator = MatchValidator::new(statement, qctx).unwrap();
        assert_eq!(validator.name(), "MatchValidator");
    }

    #[test]
    fn test_extract_match_clause() {
        let match_clause = MatchClause {
            patterns: Vec::new(),
            where_clause: None,
            optional: false,
        };
        let statement = CypherStatement::Match(match_clause.clone());
        
        let extracted = extract_match_clause(&statement).unwrap();
        assert_eq!(extracted.patterns.len(), 0);
        assert!(extracted.where_clause.is_none());
        assert!(!extracted.optional);
        
        // 测试非MATCH语句
        let return_statement = CypherStatement::Return(crate::query::parser::cypher::ast::clauses::ReturnClause {
            return_items: Vec::new(),
            distinct: false,
            order_by: None,
            skip: None,
            limit: None,
        });
        
        assert!(extract_match_clause(&return_statement).is_err());
    }
}