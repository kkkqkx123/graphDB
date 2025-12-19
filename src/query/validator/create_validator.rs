//! CREATE语句验证器
//! 对应 NebulaGraph CreateValidator 的功能

use crate::core::error::{DBError, DBResult, ValidationError};
use crate::query::context::{QueryContext, AstContext};
use crate::query::context::ast_context::ColumnDefinition;
use crate::query::parser::cypher::ast::{CypherStatement, CreateClause};
use crate::query::planner::plan::execution_plan::ExecutionPlan;
use crate::query::planner::plan::core::PlanNode;
use crate::query::validator::validator_trait::{Validator, ValidatorExt};
use crate::query::validator::base_validator::BaseValidator;
use std::sync::Arc;

/// CREATE语句验证器
pub struct CreateValidator {
    base: BaseValidator,
    create_clause: CreateClause,
}

impl CreateValidator {
    pub fn new(statement: CypherStatement, qctx: Arc<QueryContext>) -> DBResult<Self> {
        let create_clause = extract_create_clause(&statement)?;
        let base = BaseValidator::new(statement, qctx);
        
        Ok(Self {
            base,
            create_clause,
        })
    }
    
    /// 验证创建模式
    fn validate_create_pattern(&mut self) -> DBResult<()> {
        let patterns = self.create_clause.patterns.clone();
        for pattern in &patterns {
            self.validate_pattern(pattern)?;
        }
        Ok(())
    }
    
    /// 验证模式
    fn validate_pattern(&mut self, pattern: &crate::query::parser::cypher::ast::patterns::Pattern) -> DBResult<()> {
        for pattern_part in &pattern.parts {
            self.validate_pattern_part(pattern_part)?;
        }
        Ok(())
    }
    
    /// 验证模式部分
    fn validate_pattern_part(&mut self, pattern_part: &crate::query::parser::cypher::ast::patterns::PatternPart) -> DBResult<()> {
        // 验证节点创建
        self.validate_node_creation(&pattern_part.node)?;
        
        // 验证关系创建
        for rel in &pattern_part.relationships {
            self.validate_relationship_creation(rel)?;
        }
        
        Ok(())
    }
    
    /// 验证节点创建
    fn validate_node_creation(&mut self, node: &crate::query::parser::cypher::ast::patterns::NodePattern) -> DBResult<()> {
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
    
    /// 验证关系创建
    fn validate_relationship_creation(&mut self, rel: &crate::query::parser::cypher::ast::patterns::RelationshipPattern) -> DBResult<()> {
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
    
    /// 验证属性
    fn validate_properties(&mut self, properties: &std::collections::HashMap<String, crate::query::parser::cypher::ast::expressions::Expression>) -> DBResult<()> {
        for (name, expr) in properties {
            self.validate_expression(expr)?;
        }
        Ok(())
    }
    
    /// 验证表达式
    fn validate_expression(&mut self, expr: &crate::query::parser::cypher::ast::expressions::Expression) -> DBResult<()> {
        match expr {
            crate::query::parser::cypher::ast::expressions::Expression::FunctionCall(_) => {
                self.validate_function_call(expr)?;
            }
            crate::query::parser::cypher::ast::expressions::Expression::Binary(_) => {
                self.validate_binary_expression(expr)?;
            }
            _ => {} // 其他表达式类型的验证
        }
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
    
    /// 创建插入顶点节点
    fn create_insert_vertices_node(&self, patterns: &[crate::query::parser::cypher::ast::patterns::Pattern]) -> DBResult<std::sync::Arc<dyn PlanNode>> {
        let vertices = self.extract_vertices_from_patterns(patterns)?;
        // Ok(Arc::new(InsertVerticesNode::new(vertices)))
        Err(DBError::Validation(
            ValidationError::PlanError(
                "InsertVerticesNode not implemented yet".to_string()
            )
        ))
    }
    
    /// 创建插入边节点
    fn create_insert_edges_node(&self, patterns: &[crate::query::parser::cypher::ast::patterns::Pattern]) -> DBResult<std::sync::Arc<dyn PlanNode>> {
        let edges = self.extract_edges_from_patterns(patterns)?;
        // Ok(Arc::new(InsertEdgesNode::new(edges)))
        Err(DBError::Validation(
            ValidationError::PlanError(
                "InsertEdgesNode not implemented yet".to_string()
            )
        ))
    }
    
    /// 从模式中提取顶点
    fn extract_vertices_from_patterns(&self, patterns: &[crate::query::parser::cypher::ast::patterns::Pattern]) -> DBResult<Vec<crate::core::Vertex>> {
        // 实现顶点提取逻辑
        Ok(vec![])
    }
    
    /// 从模式中提取边
    fn extract_edges_from_patterns(&self, patterns: &[crate::query::parser::cypher::ast::patterns::Pattern]) -> DBResult<Vec<crate::core::Edge>> {
        // 实现边提取逻辑
        Ok(vec![])
    }
}

impl ValidatorExt for CreateValidator {
    fn validate_impl(&mut self) -> DBResult<()> {
        // 验证创建模式
        self.validate_create_pattern()?;
        
        Ok(())
    }
    
    fn to_plan_impl(&mut self) -> DBResult<ExecutionPlan> {
        // 创建插入顶点节点
        let insert_vertices_node = self.create_insert_vertices_node(&self.create_clause.patterns)?;
        
        // 创建插入边节点
        let insert_edges_node = self.create_insert_edges_node(&self.create_clause.patterns)?;
        
        // 创建输出节点
        // let output_node = Box::new(OutputNode::new(insert_edges_node));
        let output_node = insert_edges_node;
        
        Ok(ExecutionPlan {
            root: Some(output_node),
            id: -1, // 将在后续分配
            optimize_time_in_us: 0,
            format: "default".to_string(),
        })
    }
}

impl Validator for CreateValidator {
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
        "CreateValidator"
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

/// 从CypherStatement中提取CreateClause
fn extract_create_clause(statement: &CypherStatement) -> DBResult<CreateClause> {
    match statement {
        CypherStatement::Create(create_clause) => Ok(create_clause.clone()),
        _ => Err(DBError::Validation(
            ValidationError::SyntaxError(
                "Expected CREATE statement".to_string()
            )
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::cypher::ast::clauses::CreateClause;
    use crate::query::context::managers::r#impl::{
        MockIndexManager, MockMetaClient, MockSchemaManager, MockStorageClient,
    };

    #[test]
    fn test_create_validator_creation() {
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

        let statement = CypherStatement::Create(CreateClause {
            patterns: Vec::new(),
        });

        let validator = CreateValidator::new(statement, qctx).unwrap();
        assert_eq!(validator.name(), "CreateValidator");
    }

    #[test]
    fn test_extract_create_clause() {
        let create_clause = CreateClause {
            patterns: Vec::new(),
        };
        let statement = CypherStatement::Create(create_clause.clone());
        
        let extracted = extract_create_clause(&statement).unwrap();
        assert_eq!(extracted.patterns.len(), 0);
        
        // 测试非CREATE语句
        let match_statement = CypherStatement::Match(MatchClause {
            patterns: Vec::new(),
            where_clause: None,
            optional: false,
        });
        
        assert!(extract_create_clause(&match_statement).is_err());
    }
}