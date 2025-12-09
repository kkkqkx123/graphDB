//! MatchValidator - Match语句验证器主模块
//! 对应 NebulaGraph MatchValidator.h/.cpp 的功能
//! 整合所有子验证器模块的功能

use crate::query::validator::{Validator, ValidateContext};
use crate::graph::expression::expr_type::Expression;
use std::collections::HashMap;

// 导入子验证器模块
use super::alias_validator::AliasValidator;
use super::aggregate_validator::AggregateValidator;
use super::pagination_validator::PaginationValidator;
use super::expression_validator::ExpressionValidator;
use super::clause_validator::ClauseValidator;

// 导入结构定义
use crate::query::validator::match_structs::*;

pub struct MatchValidator {
    base: Validator,
    query_parts: Vec<QueryPart>,
    alias_validator: AliasValidator,
    aggregate_validator: AggregateValidator,
    pagination_validator: PaginationValidator,
    expression_validator: ExpressionValidator,
    clause_validator: ClauseValidator,
}

impl MatchValidator {
    pub fn new(context: ValidateContext) -> Self {
        Self {
            base: Validator::new(context),
            query_parts: Vec::new(),
            alias_validator: AliasValidator::new(),
            aggregate_validator: AggregateValidator::new(),
            pagination_validator: PaginationValidator::new(),
            expression_validator: ExpressionValidator::new(),
            clause_validator: ClauseValidator::new(),
        }
    }

    pub fn validate(&mut self) -> Result<(), String> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), String> {
        // 由于当前实现缺乏访问查询子句的方法，这里先实现基本框架
        // 在实际应用中，这里会遍历查询的所有子句

        // 初始化第一个查询部分
        self.query_parts.push(QueryPart {
            matchs: Vec::new(),
            boundary: None,
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: Vec::new(),
        });

        // 在实际实现中，这里会遍历查询的各个子句
        // for clause in clauses {
        //     match clause.kind() {
        //         ClauseKind::Match => { ... }
        //         ClauseKind::With => { ... }
        //         ClauseKind::Unwind => { ... }
        //     }
        // }

        // 模拟验证逻辑，先添加一些模拟数据来展示流程
        let mut aliases_available = HashMap::new();

        // 模拟处理匹配子句
        // 这里先创建一个模拟的Match子句上下文
        let mut match_clause_ctx = MatchClauseContext {
            paths: Vec::new(),
            aliases_available: aliases_available.clone(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };

        // 将生成的别名添加到可用别名中
        aliases_available.extend(match_clause_ctx.aliases_generated.clone());
        self.query_parts.last_mut().unwrap().matchs.push(match_clause_ctx);

        // 模拟验证返回子句
        let return_context = ReturnClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: Vec::new(),
                aliases_available: aliases_available.clone(),
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
            aliases_available: aliases_available.clone(),
            aliases_generated: HashMap::new(),
            pagination: None,
            order_by: None,
            distinct: false,
        };

        // 验证返回子句
        self.clause_validator.validate_return_clause(&return_context, &mut self.base)?;

        // 构建输出
        self.clause_validator.build_outputs(&mut self.query_parts.last_mut().unwrap().matchs[0].paths)?;

        Ok(())
    }

    /// 获取验证上下文的可变引用
    pub fn context_mut(&mut self) -> &mut ValidateContext {
        self.base.context_mut()
    }

    /// 获取验证上下文的引用
    pub fn context(&self) -> &ValidateContext {
        self.base.context()
    }

    // 以下方法委托给相应的子验证器

    /// 验证别名（委托给AliasValidator）
    pub fn validate_aliases(
        &mut self,
        exprs: &[Expression],
        aliases: &std::collections::HashMap<String, AliasType>,
    ) -> Result<(), String> {
        self.alias_validator.validate_aliases(exprs, aliases)
    }

    /// 检查表达式是否包含聚合函数（委托给AggregateValidator）
    pub fn has_aggregate_expr(&self, expr: &Expression) -> bool {
        self.aggregate_validator.has_aggregate_expr(expr)
    }

    /// 验证分页（委托给PaginationValidator）
    pub fn validate_pagination(
        &mut self,
        skip_expr: Option<&Expression>,
        limit_expr: Option<&Expression>,
        context: &PaginationContext,
    ) -> Result<(), String> {
        self.pagination_validator.validate_pagination(skip_expr, limit_expr, context)
    }

    /// 验证步数范围（委托给PaginationValidator）
    pub fn validate_step_range(&self, range: &MatchStepRange) -> Result<(), String> {
        self.pagination_validator.validate_step_range(range)
    }

    /// 验证过滤条件（委托给ExpressionValidator）
    pub fn validate_filter(
        &mut self,
        filter: &Expression,
        context: &mut WhereClauseContext,
    ) -> Result<(), String> {
        self.expression_validator.validate_filter(filter, context, &mut self.base)
    }

    /// 验证Return子句（委托给ExpressionValidator）
    pub fn validate_return(
        &mut self,
        return_expr: &Expression,
        query_parts: &[QueryPart],
        context: &mut ReturnClauseContext,
    ) -> Result<(), String> {
        self.expression_validator.validate_return(return_expr, query_parts, context)
    }

    /// 验证With子句（委托给ExpressionValidator）
    pub fn validate_with(
        &mut self,
        with_expr: &Expression,
        query_parts: &[QueryPart],
        context: &mut WithClauseContext,
    ) -> Result<(), String> {
        self.expression_validator.validate_with(with_expr, query_parts, context)
    }

    /// 验证Unwind子句（委托给ExpressionValidator）
    pub fn validate_unwind(
        &mut self,
        unwind_expr: &Expression,
        context: &mut UnwindClauseContext,
    ) -> Result<(), String> {
        self.expression_validator.validate_unwind(unwind_expr, context)
    }

    /// 验证Yield子句（委托给ClauseValidator）
    pub fn validate_yield(&mut self, context: &mut YieldClauseContext) -> Result<(), String> {
        self.clause_validator.validate_yield_clause(context)
    }

    /// 构建所有命名别名的列（委托给ClauseValidator）
    pub fn build_columns_for_all_named_aliases(
        &mut self,
        query_parts: &[QueryPart],
        columns: &mut Vec<YieldColumn>,
    ) -> Result<(), String> {
        self.clause_validator.build_columns_for_all_named_aliases(query_parts, columns)
    }

    /// 结合别名（委托给AliasValidator）
    pub fn combine_aliases(
        &mut self,
        cur_aliases: &mut std::collections::HashMap<String, AliasType>,
        last_aliases: &std::collections::HashMap<String, AliasType>,
    ) -> Result<(), String> {
        self.alias_validator.combine_aliases(cur_aliases, last_aliases)
    }

    /// 构建输出（委托给ClauseValidator）
    pub fn build_outputs(&mut self, paths: &mut Vec<Path>) -> Result<(), String> {
        self.clause_validator.build_outputs(paths)
    }

    /// 检查别名（委托给AliasValidator）
    pub fn check_alias(
        &mut self,
        ref_expr: &Expression,
        aliases_available: &std::collections::HashMap<String, AliasType>,
    ) -> Result<(), String> {
        self.alias_validator.check_alias(ref_expr, aliases_available)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::expr_type::Expression;
    use std::collections::HashMap;

    #[test]
    fn test_match_validator_creation() {
        let context = ValidateContext::new();
        let validator = MatchValidator::new(context);

        assert_eq!(validator.query_parts.len(), 0);
    }

    #[test]
    fn test_basic_validation() {
        let context = ValidateContext::new();
        let mut validator = MatchValidator::new(context);

        // 简单验证应该成功
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_validate_pagination() {
        let context = ValidateContext::new();
        let mut validator = MatchValidator::new(context);

        // 测试有效的分页表达式
        let skip_expr = Expression::Constant(crate::core::Value::Int(1));
        let limit_expr = Expression::Constant(crate::core::Value::Int(10));
        let pagination_ctx = PaginationContext { skip: 0, limit: 10 };

        assert!(validator.validate_pagination(Some(&skip_expr), Some(&limit_expr), &pagination_ctx).is_ok());
    }

    #[test]
    fn test_validate_aliases() {
        let context = ValidateContext::new();
        let mut validator = MatchValidator::new(context);

        // 创建一个别名映射
        let mut aliases = HashMap::new();
        aliases.insert("n".to_string(), AliasType::Node);
        aliases.insert("e".to_string(), AliasType::Edge);

        // 测试有效的别名引用
        let expr = Expression::Variable("n".to_string());
        assert!(validator.validate_aliases(&[expr], &aliases).is_ok());

        // 测试无效的别名引用
        let invalid_expr = Expression::Variable("invalid".to_string());
        assert!(validator.validate_aliases(&[invalid_expr], &aliases).is_err());
    }

    #[test]
    fn test_has_aggregate_expr() {
        let context = ValidateContext::new();
        let validator = MatchValidator::new(context);

        // 测试没有聚合函数的表达式
        let non_agg_expr = Expression::Constant(crate::core::Value::Int(1));
        assert_eq!(validator.has_aggregate_expr(&non_agg_expr), false);
    }

    #[test]
    fn test_combine_aliases() {
        let context = ValidateContext::new();
        let mut validator = MatchValidator::new(context);

        let mut cur_aliases = HashMap::new();
        cur_aliases.insert("a".to_string(), AliasType::Node);

        let mut last_aliases = HashMap::new();
        last_aliases.insert("b".to_string(), AliasType::Edge);
        last_aliases.insert("c".to_string(), AliasType::Path);

        // 组合别名
        assert!(validator.combine_aliases(&mut cur_aliases, &last_aliases).is_ok());
        assert_eq!(cur_aliases.len(), 3);
        assert!(cur_aliases.contains_key("a"));
        assert!(cur_aliases.contains_key("b"));
        assert!(cur_aliases.contains_key("c"));
    }

    #[test]
    fn test_validate_step_range() {
        let context = ValidateContext::new();
        let mut validator = MatchValidator::new(context);

        // 测试有效的范围（min <= max）
        let valid_range = MatchStepRange::new(1, 3);
        assert!(validator.validate_step_range(&valid_range).is_ok());

        // 测试无效的范围（min > max）
        let invalid_range = MatchStepRange::new(3, 1);
        assert!(validator.validate_step_range(&invalid_range).is_err());
    }

    #[test]
    fn test_context_access() {
        let mut context = ValidateContext::new();
        context.add_alias("test_alias".to_string(), ValueTypeDef::String);

        let mut validator = MatchValidator::new(context);

        // 测试上下文访问
        assert!(validator.context().get_alias_type("test_alias").is_some());

        validator.context_mut().add_error("Test error".to_string());
        assert!(validator.context().has_errors());
    }
}