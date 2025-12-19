//! 验证器核心trait定义
//! 实现验证+规划的一体化设计

use crate::core::error::{DBError, DBResult, ValidationError};
use crate::query::context::{QueryContext, AstContext};
use crate::query::parser::cypher::ast::CypherStatement;
use crate::query::planner::plan::execution_plan::ExecutionPlan;
use crate::query::context::ast_context::ColumnDefinition;

/// 验证器核心trait
pub trait Validator: Send + Sync {
    /// 验证语句的语义正确性
    fn validate(&mut self) -> DBResult<()>;
    
    /// 将验证后的AST转换为执行计划
    fn to_plan(&mut self) -> DBResult<ExecutionPlan>;
    
    /// 获取AST上下文
    fn ast_context(&self) -> &AstContext;
    
    /// 获取验证器名称
    fn name(&self) -> &'static str;
    
    /// 获取输入变量名
    fn input_var_name(&self) -> Option<&str>;
    
    /// 设置输入变量名
    fn set_input_var_name(&mut self, name: String);
    
    /// 获取输出列定义
    fn output_columns(&self) -> &[ColumnDefinition];
    
    /// 获取输入列定义
    fn input_columns(&self) -> &[ColumnDefinition];
}

/// BaseValidator的扩展trait，供具体验证器实现
pub trait ValidatorExt {
    /// 具体验证逻辑
    fn validate_impl(&mut self) -> DBResult<()>;
    
    /// 具体规划逻辑
    fn to_plan_impl(&mut self) -> DBResult<ExecutionPlan>;
}

/// 验证器创建器trait
pub trait ValidatorCreator: Send + Sync {
    fn create(&self, statement: &CypherStatement, qctx: std::sync::Arc<QueryContext>) -> DBResult<Box<dyn Validator>>;
}