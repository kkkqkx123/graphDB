//! Match语句验证器（新体系）
//! 使用trait+枚举架构，替代原有的策略模式

use super::structs::{
    AliasType, MatchStepRange, PaginationContext, Path, QueryPart, ReturnClauseContext,
    UnwindClauseContext, WhereClauseContext, WithClauseContext, YieldClauseContext, YieldColumn,
};
use super::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult,
};
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::parser::ast::stmt::{MatchStmt, ReturnClause, ReturnItem, OrderByClause};
use crate::query::parser::ast::Pattern;
use std::collections::HashMap;

/// 验证后的MATCH信息
#[derive(Debug, Clone)]
pub struct ValidatedMatch {
    pub space_id: u64,
    pub patterns: Vec<Pattern>,
    pub where_clause: Option<Expression>,
    pub return_clause: Option<ReturnClause>,
    pub order_by: Option<OrderByClause>,
    pub limit: Option<usize>,
    pub skip: Option<usize>,
    pub optional: bool,
    pub aliases: HashMap<String, AliasType>,
}

/// Match语句验证器
#[derive(Debug)]
pub struct MatchValidator {
    /// 输入列
    inputs: Vec<ColumnDef>,
    /// 输出列
    outputs: Vec<ColumnDef>,
    /// 验证后的结果
    validated_result: Option<ValidatedMatch>,
    /// 别名映射
    aliases: HashMap<String, AliasType>,
    /// 路径列表
    paths: Vec<Path>,
    /// 查询部分
    query_parts: Vec<QueryPart>,
    /// 分页上下文
    pagination: Option<PaginationContext>,
    /// 是否为可选匹配
    optional: bool,
    /// 表达式属性
    expression_props: ExpressionProps,
    /// 用户定义变量
    user_defined_vars: Vec<String>,
}

impl MatchValidator {
    /// 创建新的Match验证器
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            validated_result: None,
            aliases: HashMap::new(),
            paths: Vec::new(),
            query_parts: Vec::new(),
            pagination: None,
            optional: false,
            expression_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    /// 创建带分页上下文的验证器
    pub fn with_pagination(skip: i64, limit: i64) -> Self {
        let mut validator = Self::new();
        validator.pagination = Some(PaginationContext { skip, limit });
        validator
    }

    /// 获取验证后的结果
    pub fn validated_result(&self) -> Option<&ValidatedMatch> {
        self.validated_result.as_ref()
    }

    /// 获取别名映射
    pub fn aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases
    }

    /// 获取路径列表
    pub fn paths(&self) -> &[Path] {
        &self.paths
    }

    /// 验证完整的 MATCH 语句
    pub fn validate_match_statement(&mut self, match_stmt: &MatchStmt) -> Result<(), ValidationError> {
        // 1. 验证模式不为空
        if match_stmt.patterns.is_empty() {
            return Err(ValidationError::new(
                "MATCH 语句必须包含至少一个模式".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 验证每个模式
        for (idx, pattern) in match_stmt.patterns.iter().enumerate() {
            if let Err(e) = self.validate_pattern(pattern, idx) {
                return Err(e);
            }
        }

        // 3. 验证 RETURN 子句存在性
        if match_stmt.return_clause.is_none() {
            return Err(ValidationError::new(
                "MATCH 语句必须包含 RETURN 子句".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 4. 验证 WHERE 子句（如果存在）
        if let Some(ref where_clause) = match_stmt.where_clause {
            if let Err(e) = self.validate_where_clause(where_clause) {
                return Err(e);
            }
        }

        // 5. 验证 RETURN 子句
        if let Some(ref return_clause) = match_stmt.return_clause {
            if let Err(e) = self.validate_return_clause(return_clause) {
                return Err(e);
            }
        }

        // 6. 验证 ORDER BY 子句（如果存在）
        if let Some(ref order_by) = match_stmt.order_by {
            if let Err(e) = self.validate_order_by(order_by) {
                return Err(e);
            }
        }

        // 7. 验证分页参数
        if let (Some(skip), Some(limit)) = (match_stmt.skip, match_stmt.limit) {
            if skip >= limit {
                return Err(ValidationError::new(
                    format!("SKIP 值 ({}) 必须小于 LIMIT 值 ({})", skip, limit),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        // 8. 收集别名
        self.collect_aliases_from_patterns(&match_stmt.patterns)?;

        Ok(())
    }

    /// 验证单个模式
    fn validate_pattern(&mut self, pattern: &Pattern, idx: usize) -> Result<(), ValidationError> {
        match pattern {
            Pattern::Node(node_pattern) => {
                // 验证节点模式
                if node_pattern.variable.is_none() && node_pattern.labels.is_empty() {
                    return Err(ValidationError::new(
                        format!("第 {} 个模式: 匿名节点必须指定标签", idx + 1),
                        ValidationErrorType::SemanticError,
                    ));
                }
                
                // 如果有变量名，添加到别名映射
                if let Some(ref var) = node_pattern.variable {
                    self.aliases.insert(var.clone(), AliasType::Node);
                }
            }
            Pattern::Edge(edge_pattern) => {
                // 验证边模式
                if edge_pattern.edge_types.is_empty() && edge_pattern.variable.is_none() {
                    // 警告：匿名边类型，但不报错
                }
                
                // 如果有变量名，添加到别名映射
                if let Some(ref var) = edge_pattern.variable {
                    self.aliases.insert(var.clone(), AliasType::Edge);
                }
            }
            Pattern::Path(path_pattern) => {
                // 验证路径模式
                if path_pattern.elements.is_empty() {
                    return Err(ValidationError::new(
                        format!("第 {} 个模式: 路径不能为空", idx + 1),
                        ValidationErrorType::SemanticError,
                    ));
                }
                
                // 如果有变量名，添加到别名映射
                if let Some(ref var) = path_pattern.variable {
                    self.aliases.insert(var.clone(), AliasType::Path);
                }
            }
            Pattern::Variable(var_pattern) => {
                // 变量模式 - 检查变量是否已定义
                if !self.aliases.contains_key(&var_pattern.name) {
                    return Err(ValidationError::new(
                        format!("第 {} 个模式: 引用了未定义的变量 '{}'", idx + 1, var_pattern.name),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }
        Ok(())
    }

    /// 从模式中收集别名
    fn collect_aliases_from_patterns(&mut self, patterns: &[Pattern]) -> Result<(), ValidationError> {
        for (idx, pattern) in patterns.iter().enumerate() {
            match pattern {
                Pattern::Node(node) => {
                    if let Some(ref var) = node.variable {
                        self.aliases.insert(var.clone(), AliasType::Node);
                    }
                }
                Pattern::Edge(edge) => {
                    if let Some(ref var) = edge.variable {
                        self.aliases.insert(var.clone(), AliasType::Edge);
                    }
                }
                Pattern::Path(path) => {
                    if let Some(ref var) = path.variable {
                        self.aliases.insert(var.clone(), AliasType::Path);
                    }
                }
                Pattern::Variable(var) => {
                    if !self.aliases.contains_key(&var.name) {
                        return Err(ValidationError::new(
                            format!("第 {} 个模式: 引用了未定义的变量 '{}'", idx + 1, var.name),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// 验证 WHERE 子句
    fn validate_where_clause(&mut self, where_expr: &Expression) -> Result<(), ValidationError> {
        // 验证 WHERE 表达式是否有效
        match where_expr {
            Expression::Binary { op, .. } => {
                // 检查比较操作符
                use crate::core::BinaryOperator;
                match op {
                    BinaryOperator::Equal | BinaryOperator::NotEqual | BinaryOperator::LessThan |
                    BinaryOperator::LessThanOrEqual | BinaryOperator::GreaterThan | BinaryOperator::GreaterThanOrEqual |
                    BinaryOperator::And | BinaryOperator::Or => Ok(()),
                    _ => Err(ValidationError::new(
                        "WHERE 子句包含无效的操作符".to_string(),
                        ValidationErrorType::TypeError,
                    )),
                }
            }
            Expression::Unary { op, .. } => {
                use crate::core::UnaryOperator;
                match op {
                    UnaryOperator::Not => Ok(()),
                    _ => Err(ValidationError::new(
                        "WHERE 子句包含无效的一元操作符".to_string(),
                        ValidationErrorType::TypeError,
                    )),
                }
            }
            _ => Ok(()), // 其他表达式类型暂时通过
        }
    }

    /// 验证 RETURN 子句
    fn validate_return_clause(
        &mut self,
        return_clause: &ReturnClause,
    ) -> Result<(), ValidationError> {
        // 检查是否为空（除非是 RETURN *）
        if return_clause.items.is_empty() {
            return Err(ValidationError::new(
                "RETURN 子句必须包含至少一个返回项".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证每个返回项
        for (idx, item) in return_clause.items.iter().enumerate() {
            match item {
                ReturnItem::All => {
                    // RETURN * 是有效的
                }
                ReturnItem::Expression { expression, alias } => {
                    // 验证表达式
                    if let Err(e) = self.validate_return_expression(expression, idx) {
                        return Err(e);
                    }
                    
                    // 验证别名（如果存在）
                    if let Some(ref alias_name) = alias {
                        if alias_name.is_empty() {
                            return Err(ValidationError::new(
                                format!("第 {} 个返回项的别名不能为空", idx + 1),
                                ValidationErrorType::SemanticError,
                            ));
                        }
                        // 将别名添加到映射
                        self.aliases.insert(alias_name.clone(), AliasType::Runtime);
                    }
                }
            }
        }

        Ok(())
    }

    /// 验证返回表达式
    fn validate_return_expression(
        &mut self,
        expr: &Expression,
        idx: usize,
    ) -> Result<(), ValidationError> {
        match expr {
            Expression::Variable(var_name) => {
                // 检查变量是否在上下文中定义
                if !self.aliases.contains_key(var_name) {
                    return Err(ValidationError::new(
                        format!("第 {} 个返回项引用了未定义的变量 '{}'", idx + 1, var_name),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            Expression::Property { object, property: _ } => {
                // 验证属性访问
                if let Expression::Variable(var_name) = object.as_ref() {
                    if !self.aliases.contains_key(var_name) {
                        return Err(ValidationError::new(
                            format!("第 {} 个返回项引用了未定义的变量 '{}'", idx + 1, var_name),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
            Expression::Function { name, args } => {
                // 验证函数调用
                for (arg_idx, arg) in args.iter().enumerate() {
                    if let Err(e) = self.validate_return_expression(arg, arg_idx) {
                        return Err(e);
                    }
                }
                // TODO: 验证函数名是否有效
            }
            Expression::Binary { left, right, .. } => {
                // 验证二元表达式
                if let Err(e) = self.validate_return_expression(left, idx) {
                    return Err(e);
                }
                if let Err(e) = self.validate_return_expression(right, idx) {
                    return Err(e);
                }
            }
            Expression::Unary { operand, .. } => {
                // 验证一元表达式
                if let Err(e) = self.validate_return_expression(operand, idx) {
                    return Err(e);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// 验证 ORDER BY 子句
    fn validate_order_by(
        &mut self,
        order_by: &OrderByClause,
    ) -> Result<(), ValidationError> {
        if order_by.items.is_empty() {
            return Err(ValidationError::new(
                "ORDER BY 子句必须包含至少一个排序项".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for (idx, item) in order_by.items.iter().enumerate() {
            // 验证排序表达式
            match &item.expression {
                Expression::Variable(var_name) => {
                    if !self.aliases.contains_key(var_name) {
                        return Err(ValidationError::new(
                            format!("第 {} 个排序项引用了未定义的变量 '{}'", idx + 1, var_name),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
                Expression::Property { object, .. } => {
                    if let Expression::Variable(var_name) = object.as_ref() {
                        if !self.aliases.contains_key(var_name) {
                            return Err(ValidationError::new(
                                format!("第 {} 个排序项引用了未定义的变量 '{}'", idx + 1, var_name),
                                ValidationErrorType::SemanticError,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// 验证别名
    pub fn validate_aliases(
        &mut self,
        exprs: &[Expression],
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        for (idx, expr) in exprs.iter().enumerate() {
            match expr {
                Expression::Variable(var_name) => {
                    if !aliases.contains_key(var_name) {
                        return Err(ValidationError::new(
                            format!("第 {} 个表达式引用了未定义的别名 '{}'", idx + 1, var_name),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// 检查表达式是否包含聚合函数
    pub fn has_aggregate_expression(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Function { name, .. } => {
                // 检查是否为聚合函数
                let agg_functions = ["count", "sum", "avg", "min", "max", "collect"];
                agg_functions.iter().any(|&f| f.eq_ignore_ascii_case(name))
            }
            Expression::Binary { left, right, .. } => {
                self.has_aggregate_expression(left) || self.has_aggregate_expression(right)
            }
            Expression::Unary { operand, .. } => {
                self.has_aggregate_expression(operand)
            }
            _ => false,
        }
    }

    /// 验证分页
    pub fn validate_pagination(
        &mut self,
        skip_expression: Option<&Expression>,
        limit_expression: Option<&Expression>,
        context: &PaginationContext,
    ) -> Result<(), ValidationError> {
        // 验证 skip 值
        if context.skip < 0 {
            return Err(ValidationError::new(
                "SKIP 值不能为负数".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证 limit 值
        if context.limit < 0 {
            return Err(ValidationError::new(
                "LIMIT 值不能为负数".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证 skip < limit
        if context.skip >= context.limit && context.limit > 0 {
            return Err(ValidationError::new(
                format!("SKIP 值 ({}) 必须小于 LIMIT 值 ({})", context.skip, context.limit),
                ValidationErrorType::SemanticError,
            ));
        }

        self.pagination = Some(context.clone());
        Ok(())
    }

    /// 验证步数范围
    pub fn validate_step_range(&self, range: &MatchStepRange) -> Result<(), ValidationError> {
        if range.min() > range.max() {
            return Err(ValidationError::new(
                format!("步数范围无效: min ({}) 大于 max ({})", range.min(), range.max()),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证过滤条件
    pub fn validate_filter(
        &mut self,
        filter: &Expression,
        _context: &WhereClauseContext,
    ) -> Result<(), ValidationError> {
        // 复用 WHERE 子句验证逻辑
        self.validate_where_clause(filter)
    }

    /// 验证Return子句（完整上下文版本）
    pub fn validate_return(
        &mut self,
        _return_expression: &Expression,
        return_items: &[YieldColumn],
        _context: &ReturnClauseContext,
    ) -> Result<(), ValidationError> {
        if return_items.is_empty() {
            return Err(ValidationError::new(
                "RETURN 子句必须包含至少一个返回项".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证With子句
    pub fn validate_with(
        &mut self,
        _with_expression: &Expression,
        with_items: &[YieldColumn],
        _context: &WithClauseContext,
    ) -> Result<(), ValidationError> {
        if with_items.is_empty() {
            return Err(ValidationError::new(
                "WITH 子句必须包含至少一个项".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证Unwind子句
    pub fn validate_unwind(
        &mut self,
        unwind_expression: &Expression,
        context: &UnwindClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证 unwind 表达式
        match unwind_expression {
            Expression::Variable(var_name) => {
                if !self.aliases.contains_key(var_name) {
                    return Err(ValidationError::new(
                        format!("UNWIND 引用了未定义的变量 '{}'", var_name),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            _ => {}
        }
        
        // 添加 unwind 别名
        self.aliases.insert(context.alias.clone(), AliasType::Variable);
        Ok(())
    }

    /// 验证Yield子句
    pub fn validate_yield(&mut self, context: &YieldClauseContext) -> Result<(), ValidationError> {
        if context.yield_columns.is_empty() {
            return Err(ValidationError::new(
                "YIELD 子句必须包含至少一个列".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 构建所有命名别名的列
    pub fn build_columns_for_all_named_aliases(
        &mut self,
        query_parts: &[QueryPart],
        columns: &mut Vec<YieldColumn>,
    ) -> Result<(), ValidationError> {
        for part in query_parts {
            for (alias, alias_type) in &part.aliases_generated {
                // 根据别名类型构建列
                let expr = Expression::Variable(alias.clone());
                let col = YieldColumn::new(expr, alias.clone());
                columns.push(col);
            }
        }
        Ok(())
    }

    /// 结合别名
    pub fn combine_aliases(
        &mut self,
        cur_aliases: &mut HashMap<String, AliasType>,
        last_aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        for (alias, alias_type) in last_aliases {
            if cur_aliases.contains_key(alias) {
                // 检查类型是否一致
                if cur_aliases.get(alias) != Some(alias_type) {
                    return Err(ValidationError::new(
                        format!("别名 '{}' 的类型不一致", alias),
                        ValidationErrorType::SemanticError,
                    ));
                }
            } else {
                cur_aliases.insert(alias.clone(), alias_type.clone());
            }
        }
        Ok(())
    }

    /// 构建输出
    pub fn build_outputs(&mut self, paths: &mut Vec<Path>) -> Result<(), ValidationError> {
        // 构建输出列
        for path in paths.iter() {
            for node_info in &path.node_infos {
                if !node_info.alias.is_empty() {
                    let col = ColumnDef {
                        name: node_info.alias.clone(),
                        type_: super::ValueType::Vertex,
                    };
                    self.outputs.push(col);
                }
            }
            for edge_info in &path.edge_infos {
                if !edge_info.alias.is_empty() {
                    let col = ColumnDef {
                        name: edge_info.alias.clone(),
                        type_: super::ValueType::Edge,
                    };
                    self.outputs.push(col);
                }
            }
        }
        Ok(())
    }

    /// 检查别名
    pub fn check_alias(
        &mut self,
        ref_expression: &Expression,
        aliases_available: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        match ref_expression {
            Expression::Variable(var_name) => {
                if !aliases_available.contains_key(var_name) {
                    return Err(ValidationError::new(
                        format!("引用了未定义的别名 '{}'", var_name),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// 生成输出列
    fn generate_output_columns(&mut self, match_stmt: &MatchStmt) {
        self.outputs.clear();
        
        if let Some(ref return_clause) = match_stmt.return_clause {
            for item in &return_clause.items {
                match item {
                    ReturnItem::All => {
                        // RETURN * - 添加所有别名作为输出
                        for (alias, _) in &self.aliases {
                            let col = ColumnDef {
                                name: alias.clone(),
                                type_: super::ValueType::Unknown,
                            };
                            self.outputs.push(col);
                        }
                    }
                    ReturnItem::Expression { expression, alias } => {
                        let name = alias.clone().unwrap_or_else(|| {
                            // 生成默认名称
                            match expression {
                                Expression::Variable(v) => v.clone(),
                                _ => format!("col_{}", self.outputs.len()),
                            }
                        });
                        let col = ColumnDef {
                            name,
                            type_: super::ValueType::Unknown,
                        };
                        self.outputs.push(col);
                    }
                }
            }
        }
    }
}

impl StatementValidator for MatchValidator {
    fn validate(
        &mut self,
        query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Result<ValidationResult, ValidationError> {
        // 1. 检查是否需要空间
        if !self.is_global_statement(ast) && query_context.is_none() {
            return Err(ValidationError::new(
                "未选择图空间，请先执行 USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 获取 MATCH 语句
        let match_stmt = if let Some(ref stmt) = ast.sentence() {
            match stmt {
                crate::query::parser::ast::Stmt::Match(m) => m.clone(),
                _ => {
                    return Err(ValidationError::new(
                        "期望 MATCH 语句".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        } else {
            return Err(ValidationError::new(
                "AST 中未找到语句".to_string(),
                ValidationErrorType::SemanticError,
            ));
        };

        // 3. 验证 MATCH 语句
        if let Err(e) = self.validate_match_statement(&match_stmt) {
            return Err(e);
        }

        // 4. 获取 space_id
        let space_id = query_context
            .map(|qc| qc.space_id())
            .filter(|&id| id != 0)
            .or_else(|| ast.space().space_id.map(|id| id as u64))
            .unwrap_or(0);

        // 5. 创建验证结果
        let validated = ValidatedMatch {
            space_id,
            patterns: match_stmt.patterns.clone(),
            where_clause: match_stmt.where_clause.clone(),
            return_clause: match_stmt.return_clause.clone(),
            order_by: match_stmt.order_by.clone(),
            limit: match_stmt.limit,
            skip: match_stmt.skip,
            optional: match_stmt.optional,
            aliases: self.aliases.clone(),
        };

        self.validated_result = Some(validated);
        self.optional = match_stmt.optional;

        // 6. 生成输出列
        self.generate_output_columns(&match_stmt);

        // 7. 返回验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Match
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expression_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for MatchValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::core::Value;

    #[test]
    fn test_match_validator_creation() {
        let validator = MatchValidator::new();
        assert_eq!(validator.inputs.len(), 0);
        assert_eq!(validator.outputs.len(), 0);
    }

    #[test]
    fn test_match_validator_with_pagination() {
        let validator = MatchValidator::with_pagination(10, 100);
        assert!(validator.pagination.is_some());
        let ctx = validator.pagination.unwrap();
        assert_eq!(ctx.skip, 10);
        assert_eq!(ctx.limit, 100);
    }

    #[test]
    fn test_validate_step_range() {
        let validator = MatchValidator::new();

        // 测试有效的范围（min <= max）
        let valid_range = MatchStepRange::new(1, 3);
        assert!(validator.validate_step_range(&valid_range).is_ok());

        // 测试无效的范围（min > max）
        let invalid_range = MatchStepRange::new(3, 1);
        assert!(validator.validate_step_range(&invalid_range).is_err());
    }

    #[test]
    fn test_validate_aliases() {
        let mut validator = MatchValidator::new();

        // 创建一个别名映射
        let mut aliases = HashMap::new();
        aliases.insert("n".to_string(), AliasType::Node);
        aliases.insert("e".to_string(), AliasType::Edge);

        // 测试有效的别名引用
        let expression = Expression::Variable("n".to_string());
        assert!(validator.validate_aliases(&[expression], &aliases).is_ok());

        // 测试无效的别名引用
        let invalid_expression = Expression::Variable("invalid".to_string());
        assert!(validator
            .validate_aliases(&[invalid_expression], &aliases)
            .is_err());
    }

    #[test]
    fn test_has_aggregate_expression() {
        let validator = MatchValidator::new();

        // 测试没有聚合函数的表达式
        let non_agg_expression = Expression::Literal(Value::Int(1));
        assert_eq!(validator.has_aggregate_expression(&non_agg_expression), false);

        // 测试有聚合函数的表达式
        let agg_expression = Expression::Function {
            name: "count".to_string(),
            args: vec![Expression::Variable("n".to_string())],
        };
        assert_eq!(validator.has_aggregate_expression(&agg_expression), true);
    }

    #[test]
    fn test_combine_aliases() {
        let mut validator = MatchValidator::new();

        let mut cur_aliases = HashMap::new();
        cur_aliases.insert("a".to_string(), AliasType::Node);

        let mut last_aliases = HashMap::new();
        last_aliases.insert("b".to_string(), AliasType::Edge);
        last_aliases.insert("c".to_string(), AliasType::Path);

        // 组合别名
        assert!(validator
            .combine_aliases(&mut cur_aliases, &last_aliases)
            .is_ok());
        assert_eq!(cur_aliases.len(), 3);
        assert!(cur_aliases.contains_key("a"));
        assert!(cur_aliases.contains_key("b"));
        assert!(cur_aliases.contains_key("c"));
    }

    #[test]
    fn test_validate_pagination() {
        let mut validator = MatchValidator::new();

        // 测试有效的分页
        let ctx = PaginationContext { skip: 0, limit: 10 };
        assert!(validator.validate_pagination(None, None, &ctx).is_ok());

        // 测试无效的 skip
        let invalid_ctx = PaginationContext { skip: -1, limit: 10 };
        assert!(validator.validate_pagination(None, None, &invalid_ctx).is_err());

        // 测试 skip >= limit
        let invalid_ctx2 = PaginationContext { skip: 10, limit: 5 };
        assert!(validator.validate_pagination(None, None, &invalid_ctx2).is_err());
    }

    #[test]
    fn test_statement_type() {
        let validator = MatchValidator::new();
        assert_eq!(validator.statement_type(), StatementType::Match);
    }

    #[test]
    fn test_requires_space() {
        let validator = MatchValidator::new();
        assert!(validator.requires_space());
    }

    #[test]
    fn test_requires_write_permission() {
        let validator = MatchValidator::new();
        assert!(!validator.requires_write_permission());
    }
}
