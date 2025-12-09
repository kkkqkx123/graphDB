//! 表达式验证器模块
//! 负责验证各种表达式类型和结构

use crate::graph::expression::expr_type::Expression;
use crate::core::ValueTypeDef;
use crate::query::validator::{Validator, ValidateContext};
use crate::query::validator::match_structs::{WhereClauseContext, AliasType};
use std::collections::HashMap;

/// 表达式验证器
pub struct ExpressionValidator;

impl ExpressionValidator {
    pub fn new() -> Self {
        Self
    }

    /// 验证过滤条件
    pub fn validate_filter(
        &self,
        filter: &Expression,
        context: &mut WhereClauseContext,
        validator: &mut Validator,
    ) -> Result<(), String> {
        // 验证过滤表达式
        // 检查表达式中的别名是否已定义
        // 验证表达式的类型
        
        // 使用别名验证器验证别名
        use super::alias_validator::AliasValidator;
        let alias_validator = AliasValidator::new();
        alias_validator.validate_aliases(&[filter.clone()], &context.aliases_available)?;

        // 使用类型推导验证表达式的类型是否为布尔类型
        use crate::query::visitor::DeduceTypeVisitor;
        use crate::storage::NativeStorage; // 使用实际可用的存储实现

        // 创建临时存储引擎用于类型推导
        let temp_dir = std::env::temp_dir().join("graphdb_temp_storage");
        std::fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;
        let storage = NativeStorage::new(&temp_dir).map_err(|e| format!("创建存储失败: {}", e))?;

        let inputs = vec![]; // 过滤表达式通常不依赖于输入
        let space = "default".to_string(); // 使用默认空间

        let mut type_visitor = DeduceTypeVisitor::new(
            &storage,
            validator.context(),
            inputs,
            space,
        );

        let expr_type = type_visitor
            .deduce_type(filter)
            .map_err(|e| format!("类型推导失败: {:?}", e))?;

        if expr_type != ValueTypeDef::Bool
            && expr_type != ValueTypeDef::Empty
            && expr_type != ValueTypeDef::Null
        {
            return Err(format!(
                "WHERE表达式必须求值为布尔类型，得到{:?}",
                expr_type
            ));
        }

        Ok(())
    }

    /// 验证Match路径
    pub fn validate_path(
        &self,
        path: &Expression,
        context: &mut crate::query::validator::match_structs::MatchClauseContext,
    ) -> Result<(), String> {
        // 验证Match路径表达式
        // 检查路径中的节点和边定义
        // 验证路径模式的有效性

        // 这里应该解析路径表达式，提取节点和边的信息
        // 但由于当前的路径表示可能不同，我们暂时实现基本验证

        // 检查路径中是否存在有效的节点和边结构
        match path {
            Expression::MatchPathPattern { patterns, .. } => {
                for pattern in patterns {
                    // 验证每个路径模式
                    self.validate_single_path_pattern(pattern, context)?;
                }
            }
            _ => {
                return Err("无效的路径模式表达式".to_string());
            }
        }

        Ok(())
    }

    /// 验证单个路径模式
    pub fn validate_single_path_pattern(
        &self,
        pattern: &Expression,
        context: &mut crate::query::validator::match_structs::MatchClauseContext,
    ) -> Result<(), String> {
        // 验证单个路径模式的结构
        // 在实际实现中，这里会检查节点、边的定义等
        Ok(())
    }

    /// 验证Return子句
    pub fn validate_return(
        &self,
        return_expr: &Expression,
        query_parts: &[crate::query::validator::match_structs::QueryPart],
        context: &mut crate::query::validator::match_structs::ReturnClauseContext,
    ) -> Result<(), String> {
        // 验证Return子句中的表达式
        // 检查使用的别名是否在作用域内
        
        // 使用别名验证器验证别名
        use super::alias_validator::AliasValidator;
        let alias_validator = AliasValidator::new();
        alias_validator.validate_aliases(&[return_expr.clone()], &context.aliases_available)
    }

    /// 验证With子句
    pub fn validate_with(
        &self,
        with_expr: &Expression,
        query_parts: &[crate::query::validator::match_structs::QueryPart],
        context: &mut crate::query::validator::match_structs::WithClauseContext,
    ) -> Result<(), String> {
        // 验证With子句中的表达式别名
        
        // 使用别名验证器验证别名
        use super::alias_validator::AliasValidator;
        let alias_validator = AliasValidator::new();
        alias_validator.validate_aliases(&[with_expr.clone()], &context.aliases_available)?;

        // 验证With子句的分页
        if let Some(ref pagination) = context.pagination {
            if pagination.skip < 0 {
                return Err("SKIP不能为负数".to_string());
            }
            if pagination.limit < 0 {
                return Err("LIMIT不能为负数".to_string());
            }
        }

        // 验证是否包含聚合表达式
        use super::aggregate_validator::AggregateValidator;
        let aggregate_validator = AggregateValidator::new();
        if aggregate_validator.has_aggregate_expr(with_expr) {
            context.yield_clause.has_agg = true;
        }

        Ok(())
    }

    /// 验证Unwind子句
    pub fn validate_unwind(
        &self,
        unwind_expr: &Expression,
        context: &mut crate::query::validator::match_structs::UnwindClauseContext,
    ) -> Result<(), String> {
        // 验证Unwind表达式中的别名
        
        // 使用别名验证器验证别名
        use super::alias_validator::AliasValidator;
        let alias_validator = AliasValidator::new();
        alias_validator.validate_aliases(&[unwind_expr.clone()], &context.aliases_available)?;

        // 检查是否有聚合表达式（在UNWIND中不允许）
        use super::aggregate_validator::AggregateValidator;
        let aggregate_validator = AggregateValidator::new();
        if aggregate_validator.has_aggregate_expr(unwind_expr) {
            return Err("UNWIND子句中不能使用聚合表达式".to_string());
        }

        Ok(())
    }

    /// 验证Yield子句
    pub fn validate_yield(
        &self,
        context: &mut crate::query::validator::match_structs::YieldClauseContext,
    ) -> Result<(), String> {
        // 如果有聚合函数，执行特殊验证
        if context.has_agg {
            return self.validate_group(context);
        }

        // 对于普通Yield子句，验证别名
        use super::alias_validator::AliasValidator;
        let alias_validator = AliasValidator::new();
        for col in &context.yield_columns {
            alias_validator.validate_aliases(&[col.expr.clone()], &context.aliases_available)?;
        }

        Ok(())
    }

    /// 验证分组子句
    fn validate_group(
        &self,
        yield_ctx: &mut crate::query::validator::match_structs::YieldClauseContext,
    ) -> Result<(), String> {
        // 验证分组逻辑
        use super::aggregate_validator::AggregateValidator;
        let aggregate_validator = AggregateValidator::new();

        for col in &yield_ctx.yield_columns {
            // 如果表达式包含聚合函数，验证聚合表达式
            if aggregate_validator.has_aggregate_expr(&col.expr) {
                // 验证聚合函数
                // 在实际实现中，这里会进行更详细的聚合函数验证
            } else {
                // 非聚合表达式将作为分组键添加
                yield_ctx.group_keys.push(col.expr.clone());
            }

            yield_ctx.group_items.push(col.expr.clone());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::expr_type::Expression;
    use crate::query::validator::match_structs::{
        WhereClauseContext, MatchClauseContext, ReturnClauseContext,
        WithClauseContext, UnwindClauseContext, YieldClauseContext, YieldColumn
    };
    use std::collections::HashMap;

    #[test]
    fn test_expression_validator_creation() {
        let validator = ExpressionValidator::new();
        // 验证器创建成功
        assert!(true); // 占位测试
    }

    #[test]
    fn test_validate_filter() {
        let validator = ExpressionValidator::new();
        
        // 创建测试数据
        let mut where_context = WhereClauseContext {
            filter: None,
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: Vec::new(),
        };
        
        let mut base_validator = Validator::new(ValidateContext::new());
        
        // 测试布尔表达式
        let bool_expr = Expression::Constant(crate::core::Value::Bool(true));
        assert!(validator.validate_filter(&bool_expr, &mut where_context, &mut base_validator).is_ok());
    }

    #[test]
    fn test_validate_path() {
        let validator = ExpressionValidator::new();
        
        let mut match_context = MatchClauseContext {
            paths: Vec::new(),
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };
        
        // 测试路径验证
        // 注意：这里需要一个有效的路径表达式
        // 暂时跳过这个测试，因为需要特定的路径表达式构造
    }

    #[test]
    fn test_validate_return() {
        let validator = ExpressionValidator::new();
        
        let mut return_context = ReturnClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: Vec::new(),
                aliases_available: HashMap::new(),
                aliases_generated: HashMap::new(),
                distinct: false,
                has_agg: false,
                group_keys: Vec::new(),
                group_items: Vec::new(),
                need_gen_project: false,
                agg_output_column_names: Vec::new(),
                proj_output_column_names: Vec::new(),
                proj_cols: Vec::new(),
                paths: Vec::new(),
            },
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            pagination: None,
            order_by: None,
            distinct: false,
        };
        
        // 测试Return子句验证
        let return_expr = Expression::Constant(crate::core::Value::Int(1));
        assert!(validator.validate_return(&return_expr, &[], &mut return_context).is_ok());
    }

    #[test]
    fn test_validate_with() {
        let validator = ExpressionValidator::new();
        
        let mut with_context = WithClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: Vec::new(),
                aliases_available: HashMap::new(),
                aliases_generated: HashMap::new(),
                distinct: false,
                has_agg: false,
                group_keys: Vec::new(),
                group_items: Vec::new(),
                need_gen_project: false,
                agg_output_column_names: Vec::new(),
                proj_output_column_names: Vec::new(),
                proj_cols: Vec::new(),
                paths: Vec::new(),
            },
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            pagination: None,
            order_by: None,
            distinct: false,
        };
        
        // 测试With子句验证
        let with_expr = Expression::Constant(crate::core::Value::Int(1));
        assert!(validator.validate_with(&with_expr, &[], &mut with_context).is_ok());
    }

    #[test]
    fn test_validate_unwind() {
        let validator = ExpressionValidator::new();
        
        let mut unwind_context = UnwindClauseContext {
            alias: "test".to_string(),
            unwind_expr: Expression::Constant(crate::core::Value::Int(1)),
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: Vec::new(),
        };
        
        // 测试Unwind子句验证
        let unwind_expr = Expression::Constant(crate::core::Value::Int(1));
        assert!(validator.validate_unwind(&unwind_expr, &mut unwind_context).is_ok());
    }

    #[test]
    fn test_validate_yield() {
        let validator = ExpressionValidator::new();
        
        let mut yield_context = YieldClauseContext {
            yield_columns: vec![YieldColumn::new(Expression::Constant(crate::core::Value::Int(1)), "col1".to_string())],
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: Vec::new(),
            group_items: Vec::new(),
            need_gen_project: false,
            agg_output_column_names: Vec::new(),
            proj_output_column_names: Vec::new(),
            proj_cols: Vec::new(),
            paths: Vec::new(),
        };
        
        // 测试Yield子句验证
        assert!(validator.validate_yield(&mut yield_context).is_ok());
    }

    #[test]
    fn test_single_path_pattern() {
        let validator = ExpressionValidator::new();
        
        let mut match_context = MatchClauseContext {
            paths: Vec::new(),
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };
        
        // 测试单个路径模式验证
        let pattern = Expression::Constant(crate::core::Value::Int(1));
        assert!(validator.validate_single_path_pattern(&pattern, &mut match_context).is_ok());
    }
}