//! MatchValidator - Match语句验证器
//! 对应 NebulaGraph MatchValidator.h/.cpp 的功能

use crate::query::validator::{Validator, ValidateContext};
use crate::graph::expression::expr_type::{Expression, ExpressionKind};
use crate::core::ValueTypeDef;

mod match_structs;
pub use match_structs::*;

pub struct MatchValidator {
    base: Validator,
    query_parts: Vec<QueryPart>,
}

impl MatchValidator {
    pub fn new(context: ValidateContext) -> Self {
        Self {
            base: Validator::new(context),
            query_parts: Vec::new(),
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
        self.validate_return_clause(&return_context)?;

        // 构建输出
        self.build_outputs(&mut self.query_parts.last_mut().unwrap().matchs[0].paths)?;

        Ok(())
    }

    /// 验证返回子句
    fn validate_return_clause(&mut self, context: &ReturnClauseContext) -> Result<(), String> {
        // 检查别名可用性
        for col in &context.yield_clause.yield_columns {
            self.validate_aliases(&[col.expr.clone()], &context.aliases_available)?;
        }

        // 验证分页
        if let Some(ref pagination) = context.pagination {
            if pagination.skip < 0 {
                return Err("SKIP should not be negative".to_string());
            }
            if pagination.limit < 0 {
                return Err("LIMIT should not be negative".to_string());
            }
        }

        // 验证排序
        if let Some(ref order_by) = context.order_by {
            // 在这里可以验证排序条件
            for &(index, _) in &order_by.indexed_order_factors {
                // 检查索引是否有效
                if index >= context.yield_clause.yield_columns.len() {
                    return Err(format!("Column index {} out of bounds", index));
                }
            }
        }

        Ok(())
    }

    /// 验证Match路径
    fn validate_path(&mut self, path: &Expression, context: &mut MatchClauseContext) -> Result<(), String> {
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
                return Err("Invalid path pattern expression".to_string());
            }
        }

        Ok(())
    }

    /// 验证单个路径模式
    fn validate_single_path_pattern(&mut self, pattern: &Expression, context: &mut MatchClauseContext) -> Result<(), String> {
        // 验证单个路径模式的结构
        // 在实际实现中，这里会检查节点、边的定义等
        Ok(())
    }

    /// 验证过滤条件
    fn validate_filter(&mut self, filter: &Expression, context: &mut WhereClauseContext) -> Result<(), String> {
        // 验证过滤表达式
        // 检查表达式中的别名是否已定义
        // 验证表达式的类型
        self.validate_aliases(&[filter.clone()], &context.aliases_available)?;

        // 使用类型推导验证表达式的类型是否为布尔类型
        use crate::query::visitor::DeduceTypeVisitor;
        use crate::storage::NativeStorage; // 使用实际可用的存储实现

        // 创建临时存储引擎用于类型推导
        let temp_dir = std::env::temp_dir().join("graphdb_temp_storage");
        std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let storage = NativeStorage::new(&temp_dir).map_err(|e| format!("Failed to create storage: {}", e))?;

        let inputs = vec![]; // 过滤表达式通常不依赖于输入
        let space = "default".to_string(); // 使用默认空间

        let mut type_visitor = DeduceTypeVisitor::new(
            &storage,
            self.base.context(),
            inputs,
            space,
        );

        let expr_type = type_visitor.deduce_type(filter)
            .map_err(|e| format!("Type deduction failed: {:?}", e))?;

        if expr_type != ValueTypeDef::Bool &&
           expr_type != ValueTypeDef::Empty &&
           expr_type != ValueTypeDef::Null {
            return Err(format!("WHERE expression must evaluate to a boolean type, got {:?}",
                             expr_type));
        }

        Ok(())
    }

    /// 验证Return子句
    fn validate_return(&mut self,
                      return_expr: &Expression,
                      query_parts: &[QueryPart],
                      context: &mut ReturnClauseContext) -> Result<(), String> {
        // 验证Return子句中的表达式
        // 检查使用的别名是否在作用域内
        self.validate_aliases(&[return_expr.clone()], &context.aliases_available)
    }

    /// 验证别名
    fn validate_aliases(&mut self,
                       exprs: &[Expression],
                       aliases: &std::collections::HashMap<String, AliasType>) -> Result<(), String> {
        // 验证表达式中使用的别名是否已定义
        for expr in exprs {
            self.validate_expression_aliases(expr, aliases)?;
        }
        Ok(())
    }

    /// 验证表达式中的别名
    fn validate_expression_aliases(&mut self,
                                  expr: &Expression,
                                  aliases: &std::collections::HashMap<String, AliasType>) -> Result<(), String> {
        // 首先检查表达式本身是否引用了一个别名
        if let Some(alias_name) = self.extract_alias_name(expr) {
            if !aliases.contains_key(&alias_name) {
                return Err(format!("Undefined variable alias: {}", alias_name));
            }
        }

        // 递归验证子表达式
        self.validate_subexpressions_aliases(expr, aliases)?;

        Ok(())
    }

    /// 从表达式中提取别名名称
    fn extract_alias_name(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Variable(name) => Some(name.clone()),
            Expression::Property(name) => Some(name.clone()),
            Expression::Label(name) => Some(name.clone()),
            // 根据实际的表达式类型，可能需要处理其他别名引用
            _ => None
        }
    }

    /// 递归验证子表达式中的别名
    fn validate_subexpressions_aliases(&mut self,
                                      expr: &Expression,
                                      aliases: &std::collections::HashMap<String, AliasType>) -> Result<(), String> {
        match expr {
            Expression::UnaryOp(_, operand) => {
                self.validate_expression_aliases(operand, aliases)
            },
            Expression::BinaryOp(left, _, right) => {
                self.validate_expression_aliases(left, aliases)?;
                self.validate_expression_aliases(right, aliases)
            },
            Expression::Property(_) => {
                // Property expression doesn't have sub-expressions
                Ok(())
            },
            Expression::Function(_, args) => {
                for arg in args {
                    self.validate_expression_aliases(arg, aliases)?;
                }
                Ok(())
            },
            // For constants, there are no sub-expressions
            Expression::Constant(_) => Ok(()),
            Expression::TagProperty { .. } |
            Expression::EdgeProperty { .. } |
            Expression::InputProperty(_) |
            Expression::VariableProperty { .. } |
            Expression::SourceProperty { .. } |
            Expression::DestinationProperty { .. } => {
                // These expressions don't have sub-expressions
                Ok(())
            },
            Expression::UnaryPlus(operand) => {
                self.validate_expression_aliases(operand, aliases)
            },
            Expression::UnaryNegate(operand) => {
                self.validate_expression_aliases(operand, aliases)
            },
            Expression::UnaryNot(operand) => {
                self.validate_expression_aliases(operand, aliases)
            },
            Expression::UnaryIncr(operand) => {
                self.validate_expression_aliases(operand, aliases)
            },
            Expression::UnaryDecr(operand) => {
                self.validate_expression_aliases(operand, aliases)
            },
            Expression::IsNull(operand) => {
                self.validate_expression_aliases(operand, aliases)
            },
            Expression::IsNotNull(operand) => {
                self.validate_expression_aliases(operand, aliases)
            },
            Expression::IsEmpty(operand) => {
                self.validate_expression_aliases(operand, aliases)
            },
            Expression::IsNotEmpty(operand) => {
                self.validate_expression_aliases(operand, aliases)
            },
            Expression::List(items) => {
                for item in items {
                    self.validate_expression_aliases(item, aliases)?;
                }
                Ok(())
            },
            Expression::Set(items) => {
                for item in items {
                    self.validate_expression_aliases(item, aliases)?;
                }
                Ok(())
            },
            Expression::Map(items) => {
                for (_, value) in items {
                    self.validate_expression_aliases(value, aliases)?;
                }
                Ok(())
            },
            Expression::TypeCasting { expr, .. } => {
                self.validate_expression_aliases(expr, aliases)
            },
            Expression::Case { conditions, default } => {
                for (condition, value) in conditions {
                    self.validate_expression_aliases(condition, aliases)?;
                    self.validate_expression_aliases(value, aliases)?;
                }
                if let Some(default_expr) = default {
                    self.validate_expression_aliases(default_expr, aliases)?;
                }
                Ok(())
            },
            Expression::Aggregate { arg, .. } => {
                self.validate_expression_aliases(arg, aliases)
            },
            Expression::ListComprehension { generator, condition } => {
                self.validate_expression_aliases(generator, aliases)?;
                if let Some(condition_expr) = condition {
                    self.validate_expression_aliases(condition_expr, aliases)?;
                }
                Ok(())
            },
            Expression::Predicate { list, condition } => {
                self.validate_expression_aliases(list, aliases)?;
                self.validate_expression_aliases(condition, aliases)
            },
            Expression::Reduce { list, initial, expr, .. } => {
                self.validate_expression_aliases(list, aliases)?;
                self.validate_expression_aliases(initial, aliases)?;
                self.validate_expression_aliases(expr, aliases)
            },
            Expression::PathBuild(items) => {
                for item in items {
                    self.validate_expression_aliases(item, aliases)?;
                }
                Ok(())
            },
            Expression::ESQuery(_) => {
                // ESQuery has no sub-expressions
                Ok(())
            },
            Expression::UUID => {
                // UUID has no sub-expressions
                Ok(())
            },
            Expression::Variable(_) => {
                // Variable has no sub-expressions
                Ok(())
            },
            Expression::Subscript { collection, index } => {
                self.validate_expression_aliases(collection, aliases)?;
                self.validate_expression_aliases(index, aliases)
            },
            Expression::SubscriptRange { collection, start, end } => {
                self.validate_expression_aliases(collection, aliases)?;
                if let Some(start_expr) = start {
                    self.validate_expression_aliases(start_expr, aliases)?;
                }
                if let Some(end_expr) = end {
                    self.validate_expression_aliases(end_expr, aliases)?;
                }
                Ok(())
            },
            Expression::Label(_) => {
                // Label has no sub-expressions
                Ok(())
            },
            Expression::MatchPathPattern { patterns, .. } => {
                for pattern in patterns {
                    self.validate_expression_aliases(pattern, aliases)?;
                }
                Ok(())
            },
        }
    }

    /// 验证With子句
    fn validate_with(&mut self,
                    with_expr: &Expression,
                    query_parts: &[QueryPart],
                    context: &mut WithClauseContext) -> Result<(), String> {
        // 验证With子句中的表达式别名
        self.validate_aliases(&[with_expr.clone()], &context.aliases_available)?;

        // 验证With子句的分页
        if let Some(ref pagination) = context.pagination {
            if pagination.skip < 0 {
                return Err("SKIP should not be negative".to_string());
            }
            if pagination.limit < 0 {
                return Err("LIMIT should not be negative".to_string());
            }
        }

        // 验证是否包含聚合表达式
        if self.has_aggregate_expr(with_expr) {
            context.yield_clause.has_agg = true;
        }

        Ok(())
    }

    /// 验证Unwind子句
    fn validate_unwind(&mut self,
                      unwind_expr: &Expression,
                      context: &mut UnwindClauseContext) -> Result<(), String> {
        // 验证Unwind表达式中的别名
        self.validate_aliases(&[unwind_expr.clone()], &context.aliases_available)?;

        // 检查是否有聚合表达式（在UNWIND中不允许）
        if self.has_aggregate_expr(unwind_expr) {
            return Err("Can't use aggregating expressions in UNWIND clause".to_string());
        }

        Ok(())
    }

    /// 检查表达式是否包含聚合函数
    fn has_aggregate_expr(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Aggregate { .. } => true,
            Expression::UnaryOp(_, operand) => self.has_aggregate_expr(operand),
            Expression::BinaryOp(left, _, right) =>
                self.has_aggregate_expr(left) || self.has_aggregate_expr(right),
            Expression::Function(_, args) => {
                args.iter().any(|arg| self.has_aggregate_expr(arg))
            },
            Expression::List(items) => {
                items.iter().any(|item| self.has_aggregate_expr(item))
            },
            Expression::Set(items) => {
                items.iter().any(|item| self.has_aggregate_expr(item))
            },
            Expression::Map(items) => {
                items.iter().any(|(_, value)| self.has_aggregate_expr(value))
            },
            Expression::Case { conditions, default } => {
                conditions.iter().any(|(cond, val)|
                    self.has_aggregate_expr(cond) || self.has_aggregate_expr(val)) ||
                default.as_ref().map_or(false, |d| self.has_aggregate_expr(d))
            },
            Expression::ListComprehension { generator, condition } => {
                self.has_aggregate_expr(generator) ||
                condition.as_ref().map_or(false, |c| self.has_aggregate_expr(c))
            },
            Expression::Predicate { list, condition } => {
                self.has_aggregate_expr(list) || self.has_aggregate_expr(condition)
            },
            Expression::Reduce { list, initial, expr, .. } => {
                self.has_aggregate_expr(list) ||
                self.has_aggregate_expr(initial) ||
                self.has_aggregate_expr(expr)
            },
            _ => false
        }
    }

    /// 验证分页
    fn validate_pagination(&mut self,
                          skip_expr: Option<&Expression>,
                          limit_expr: Option<&Expression>,
                          context: &PaginationContext) -> Result<(), String> {
        // 验证分页参数的有效性
        if context.skip < 0 {
            return Err("SKIP should not be negative".to_string());
        }
        if context.limit < 0 {
            return Err("LIMIT should not be negative".to_string());
        }

        // 验证表达式类型（如果提供了表达式）
        if let Some(skip) = skip_expr {
            self.validate_pagination_expr(skip, "SKIP")?;
        }

        if let Some(limit) = limit_expr {
            self.validate_pagination_expr(limit, "LIMIT")?;
        }

        Ok(())
    }

    /// 验证分页表达式
    fn validate_pagination_expr(&mut self, expr: &Expression, clause_name: &str) -> Result<(), String> {
        // 使用DeduceTypeVisitor来推导表达式的类型
        use crate::query::visitor::DeduceTypeVisitor;
        use crate::storage::NativeStorage; // 使用实际可用的存储实现

        // 创建临时存储引擎用于类型推导
        let temp_dir = std::env::temp_dir().join("graphdb_temp_storage");
        std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let storage = NativeStorage::new(&temp_dir).map_err(|e| format!("Failed to create storage: {}", e))?;

        let inputs = vec![]; // 分页表达式通常不依赖于输入
        let space = "default".to_string(); // 使用默认空间

        let mut type_visitor = DeduceTypeVisitor::new(
            &storage,
            self.base.context(),
            inputs,
            space,
        );

        let expr_type = type_visitor.deduce_type(expr)
            .map_err(|e| format!("Type deduction failed: {:?}", e))?;

        if expr_type != ValueTypeDef::Int && expr_type != ValueTypeDef::Empty && expr_type != ValueTypeDef::Null {
            return Err(format!("{} expression must evaluate to an integer type, got {:?}",
                             clause_name, expr_type));
        }

        Ok(())
    }

    /// 验证OrderBy子句
    fn validate_order_by(&mut self,
                        factors: &Vec<Expression>,  // 排序因子
                        yield_columns: &Vec<YieldColumn>,
                        context: &OrderByClauseContext) -> Result<(), String> {
        // 验证OrderBy子句
        for &(index, _) in &context.indexed_order_factors {
            if index >= yield_columns.len() {
                return Err(format!("Column index {} out of bounds", index));
            }
        }

        Ok(())
    }

    /// 验证Yield子句
    fn validate_yield(&mut self, context: &mut YieldClauseContext) -> Result<(), String> {
        // 如果有聚合函数，执行特殊验证
        if context.has_agg {
            return self.validate_group(context);
        }

        // 对于普通Yield子句，验证别名
        for col in &context.yield_columns {
            self.validate_aliases(&[col.expr.clone()], &context.aliases_available)?;
        }

        Ok(())
    }

    /// 验证分组子句
    fn validate_group(&mut self, yield_ctx: &mut YieldClauseContext) -> Result<(), String> {
        // 验证分组逻辑
        for col in &yield_ctx.yield_columns {
            // 如果表达式包含聚合函数，验证聚合表达式
            if self.has_aggregate_expr(&col.expr) {
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

    /// 构建所有命名别名的列
    fn build_columns_for_all_named_aliases(&mut self,
                                          query_parts: &[QueryPart],
                                          columns: &mut Vec<YieldColumn>) -> Result<(), String> {
        if query_parts.is_empty() {
            return Err("No alias declared.".to_string());
        }

        let curr_query_part = query_parts.last().unwrap();

        // 处理前一个查询部分的边界子句
        if query_parts.len() > 1 {
            let prev_query_part = &query_parts[query_parts.len() - 2];
            if let Some(ref boundary) = prev_query_part.boundary {
                match boundary {
                    BoundaryClauseContext::Unwind(unwind_ctx) => {
                        // 添加Unwind子句的别名
                        columns.push(YieldColumn::new(
                            Expression::Label(unwind_ctx.alias.clone()),
                            unwind_ctx.alias.clone()
                        ));

                        // 添加之前可用的别名
                        for (alias, _) in &prev_query_part.aliases_available {
                            columns.push(YieldColumn::new(
                                Expression::Label(alias.clone()),
                                alias.clone()
                            ));
                        }

                        // 添加之前生成的别名
                        for (alias, _) in &prev_query_part.aliases_generated {
                            columns.push(YieldColumn::new(
                                Expression::Label(alias.clone()),
                                alias.clone()
                            ));
                        }
                    }
                    BoundaryClauseContext::With(with_ctx) => {
                        // 添加With子句的列
                        for col in &with_ctx.yield_clause.yield_columns {
                            if !col.alias.is_empty() {
                                columns.push(YieldColumn::new(
                                    Expression::Label(col.alias.clone()),
                                    col.alias.clone()
                                ));
                            }
                        }
                    }
                }
            }
        }

        // 处理当前查询部分的匹配子句
        for match_ctx in &curr_query_part.matchs {
            for path in &match_ctx.paths {
                // 添加路径中节点和边的别名
                for i in 0..path.edge_infos.len() {
                    if !path.node_infos[i].anonymous {
                        columns.push(YieldColumn::new(
                            Expression::Label(path.node_infos[i].alias.clone()),
                            path.node_infos[i].alias.clone()
                        ));
                    }

                    if !path.edge_infos[i].anonymous {
                        columns.push(YieldColumn::new(
                            Expression::Label(path.edge_infos[i].alias.clone()),
                            path.edge_infos[i].alias.clone()
                        ));
                    }
                }

                // 添加最后的节点别名
                if !path.node_infos.last().unwrap().anonymous {
                    let last_node = path.node_infos.last().unwrap();
                    columns.push(YieldColumn::new(
                        Expression::Label(last_node.alias.clone()),
                        last_node.alias.clone()
                    ));
                }
            }

            // 添加路径别名
            for (alias, alias_type) in &match_ctx.aliases_generated {
                if *alias_type == AliasType::Path {
                    columns.push(YieldColumn::new(
                        Expression::Label(alias.clone()),
                        alias.clone()
                    ));
                }
            }
        }

        Ok(())
    }

    /// 结合别名
    fn combine_aliases(&mut self,
                      cur_aliases: &mut std::collections::HashMap<String, AliasType>,
                      last_aliases: &std::collections::HashMap<String, AliasType>) -> Result<(), String> {
        for (name, alias_type) in last_aliases {
            if !cur_aliases.contains_key(name) {
                if cur_aliases.insert(name.clone(), alias_type.clone()).is_some() {
                    return Err(format!("`{}': Redefined alias", name));
                }
            }
        }
        Ok(())
    }

    /// 构建输出
    fn build_outputs(&mut self, paths: &mut Vec<Path>) -> Result<(), String> {
        // 构建查询输出，包括列名和类型
        // 这里会根据路径信息构建最终的输出格式
        for path in paths {
            // 为每个路径构建输出信息
            // 在实际实现中，这里会构建具体的输出格式
        }
        Ok(())
    }

    /// 检查别名
    fn check_alias(&mut self,
                   ref_expr: &Expression,
                   aliases_available: &std::collections::HashMap<String, AliasType>) -> Result<(), String> {
        // 提取表达式中的别名名称
        if let Some(alias_name) = self.extract_alias_name(ref_expr) {
            if !aliases_available.contains_key(&alias_name) {
                return Err(format!("Undefined alias: {}", alias_name));
            }

            // 进一步验证别名类型是否匹配使用方式
            match ref_expr {
                Expression::SourceProperty { .. } | Expression::DestinationProperty { .. } => {
                    // 源/目标属性应指向节点类型的别名
                    if let Some(alias_type) = aliases_available.get(&alias_name) {
                        if alias_type == &AliasType::Edge || alias_type == &AliasType::Path {
                            return Err(format!("To get the src/dst vid of the edge/path, use src/dst/endNode({})", alias_name));
                        } else if alias_type != &AliasType::Node {
                            return Err(format!("Alias `{}` does not have the edge property src/dst", alias_name));
                        }
                    }
                }
                _ => {}
            }
        }

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

    /// 验证步数范围
    fn validate_step_range(&self, range: &MatchStepRange) -> Result<(), String> {
        if range.min > range.max {
            return Err(format!(
                "Max hop must be greater equal than min hop: {} vs. {}",
                range.max, range.min
            ));
        }
        Ok(())
    }
}