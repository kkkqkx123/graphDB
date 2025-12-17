//! UNWIND 子句规划器
//!
//! 负责将 Cypher 查询中的 UNWIND 子句转换为执行计划。
//!
//! # 功能概述
//!
//! UNWIND 子句用于将集合（列表）展开为多行，每行包含集合中的一个元素。
//! 这是 Cypher 中处理集合数据的重要操作，常用于：
//! - 展开列表为独立行
//! - 与其他子句组合进行数据处理
//! - 实现类似 SQL 的 UNNEST 功能
//!
//! # 处理逻辑
//!
//! 1. 验证 UNWIND 表达式的有效性
//! 2. 创建 UNWIND 计划节点
//! 3. 设置展开表达式和别名
//! 4. 连接到输入计划
//!
//! # 示例
//!
//! ```cypher
//! UNWIND [1, 2, 3] AS number
//! RETURN number
//! ```
//!
//! 将产生三行结果，每行包含一个数字。
//!
//! ```cypher
//! WITH ['Alice', 'Bob', 'Charlie'] AS names
//! UNWIND names AS name
//! RETURN name
//! ```
//!
//! 将名字列表展开为独立的行。
//!
//! # 注意事项
//!
//! - UNWIND 表达式必须求值为列表
//! - 别名不能与现有变量冲突
//! - 空列表将产生零行结果
//! - NULL 值将产生零行结果

use crate::query::planner::match_planning::core::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::match_planning::utils::connector::SegmentsConnector;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::plan::core::plan_node_traits::PlanNodeMutable;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};
use std::sync::Arc;

/// UNWIND子句规划器
/// 负责规划UNWIND操作来展开集合
#[derive(Debug)]
pub struct UnwindClausePlanner;

impl UnwindClausePlanner {
    pub fn new() -> Self {
        Self
    }
}

impl CypherClausePlanner for UnwindClausePlanner {
    /// 将 UNWIND 子句上下文转换为执行计划
    ///
    /// # 参数
    /// * `clause_ctx` - Cypher 子句上下文，必须是 Unwind 类型
    ///
    /// # 返回
    /// * `Result<SubPlan, PlannerError>` - 执行计划或错误
    ///
    /// # 错误处理
    /// * 如果上下文不是 Unwind 类型，返回 InvalidAstContext 错误
    /// * 如果无法提取 UnwindClauseContext，返回 InvalidAstContext 错误
    /// * 如果别名验证失败，返回相应的 PlannerError
    /// * 如果表达式验证失败，返回相应的 PlannerError
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        // 验证输入上下文类型
        if !matches!(clause_ctx.kind(), CypherClauseKind::Unwind) {
            return Err(PlannerError::InvalidAstContext(
                "UnwindClausePlanner 只能处理 UNWIND 子句上下文".to_string(),
            ));
        }

        // 提取具体的 UNWIND 子句上下文
        let unwind_clause_ctx = match clause_ctx {
            CypherClauseContext::Unwind(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "无法提取 UnwindClauseContext".to_string(),
                ))
            }
        };

        // 验证 UNWIND 子句上下文的完整性
        validate_unwind_clause(&unwind_clause_ctx)?;

        // UNWIND 子句不应该创建起始节点
        // 它应该接收来自上游子句的输入数据
        // 这里我们创建一个不包含输入的 UNWIND 节点
        // 实际的输入连接由更高层的规划器负责
        let unwind_node = create_unwind_node_without_input(&unwind_clause_ctx)?;

        // 创建包含 UNWIND 节点的子计划
        // 注意：这个计划没有输入，需要由调用者连接到上游计划
        let unwind_plan = SubPlan::new(Some(unwind_node.clone()), Some(unwind_node));

        Ok(unwind_plan)
    }
}

/// 验证 UNWIND 子句的有效性
///
/// # 参数
/// * `ctx` - UNWIND 子句上下文
///
/// # 返回
/// * `Result<(), PlannerError>` - 验证结果
fn validate_unwind_clause(ctx: &crate::query::validator::structs::UnwindClauseContext) -> Result<(), PlannerError> {
    // 验证别名不能为空
    if ctx.alias.trim().is_empty() {
        return Err(PlannerError::PlanGenerationFailed(
            "UNWIND 别名不能为空".to_string(),
        ));
    }

    // 验证别名是否符合标识符规范
    if !is_valid_identifier(&ctx.alias) {
        return Err(PlannerError::PlanGenerationFailed(
            format!("UNWIND 别名 '{}' 不是有效的标识符", ctx.alias)
        ));
    }

    // 验证表达式不能为空（在实际实现中可能需要更复杂的验证）
    // 这里我们只做基本检查，实际的表达式验证可能在解析阶段完成

    Ok(())
}

/// 检查字符串是否是有效的标识符
///
/// # 参数
/// * `identifier` - 要检查的标识符
///
/// # 返回
/// * `bool` - 是否是有效标识符
fn is_valid_identifier(identifier: &str) -> bool {
    if identifier.is_empty() {
        return false;
    }

    // 标识符必须以字母或下划线开头
    let first_char = identifier.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    // 其余字符可以是字母、数字或下划线
    for c in identifier.chars() {
        if !c.is_alphanumeric() && c != '_' {
            return false;
        }
    }

    true
}

/// 创建 UNWIND 节点（不包含输入）
///
/// # 参数
/// * `ctx` - UNWIND 子句上下文
///
/// # 返回
/// * `Result<Arc<dyn PlanNode>, PlannerError>` - UNWIND 节点或错误
///
/// # 说明
/// 此函数创建一个 UNWIND 节点，但不包含输入节点。
/// 实际的输入连接由更高层的规划器负责。
/// 这种设计确保了数据流的正确性和模块化。
fn create_unwind_node_without_input(
    ctx: &crate::query::validator::structs::UnwindClauseContext,
) -> Result<Arc<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
    use crate::query::planner::plan::SingleDependencyNode;

    // 创建一个占位符节点，实际执行时会被替换为真正的输入
    // 注意：这不是起始节点，只是一个占位符
    let placeholder_node = Arc::new(SingleDependencyNode {
        id: -1,
        kind: PlanNodeKind::Start, // 使用 Start 作为占位符类型
        dependencies: vec![],
        output_var: None,
        col_names: vec![],
        cost: 0.0,
    });

    // 创建 UNWIND 节点
    let mut unwind_node = SingleInputNode::new(PlanNodeKind::Unwind, placeholder_node);

    // 设置 UNWIND 节点的属性
    // 将表达式和别名信息存储在列名中，供执行器使用
    // 使用特殊格式存储 UNWIND 信息
    unwind_node.set_col_names(vec![
        format!("unwind_expr:{}", serialize_expression(&ctx.unwind_expr)),
        format!("unwind_alias:{}", ctx.alias),
    ]);

    // 设置输出变量为别名
    // 注意：这里可能需要根据实际的变量系统进行调整
    // unwind_node.output_var = Some(Variable::new(&ctx.alias));

    Ok(Arc::new(unwind_node))
}

/// 序列化表达式为字符串
/// 用于在计划节点中存储表达式信息
///
/// # 参数
/// * `expr` - 要序列化的表达式
///
/// # 返回
/// * `String` - 序列化后的字符串
fn serialize_expression(expr: &crate::graph::expression::Expression) -> String {
    // 这里应该实现完整的表达式序列化逻辑
    // 目前使用简化的实现，实际项目中可能需要更复杂的序列化
    match expr {
        crate::graph::expression::Expression::Variable(name) => format!("${}", name),
        crate::graph::expression::Expression::Literal(_) => "literal".to_string(),
        crate::graph::expression::Expression::List(_) => "list".to_string(),
        _ => "expression".to_string(),
    }
}

/// 将 UNWIND 计划连接到输入计划
///
/// # 参数
/// * `input_plan` - 输入计划
/// * `unwind_plan` - UNWIND 计划
///
/// # 返回
/// * `Result<SubPlan, PlannerError>` - 连接后的计划或错误
///
/// # 说明
/// 此函数用于将 UNWIND 节点正确连接到前一个子句的输出，
/// 确保数据流的正确性。在实际的查询规划过程中，
/// 这个连接操作通常由更高层的规划器负责。
///
/// # 正确的数据流
/// input_plan -> unwind_plan
///
/// # 注意事项
/// - UNWIND 节点不应该创建起始节点
/// - 输入计划必须存在且有效
/// - 连接后确保数据流的正确性
pub fn connect_unwind_to_input(
    input_plan: SubPlan,
    unwind_plan: SubPlan,
) -> Result<SubPlan, PlannerError> {
    // 验证输入计划的有效性
    if input_plan.root.is_none() {
        return Err(PlannerError::PlanGenerationFailed(
            "UNWIND 子句必须有有效的输入计划".to_string(),
        ));
    }

    // 使用 SegmentsConnector 连接两个计划
    let connector = SegmentsConnector::new();
    
    // 将 UNWIND 计划连接到输入计划的输出
    // 数据流：input_plan -> unwind_plan
    let connected_plan = connector.add_input(unwind_plan, input_plan, true);
    
    Ok(connected_plan)
}

/// 验证 UNWIND 表达式是否返回列表类型
///
/// # 参数
/// * `expr` - 要验证的表达式
///
/// # 返回
/// * `Result<(), PlannerError>` - 验证结果
///
/// # 说明
/// 在实际实现中，这里应该进行类型检查以确保表达式返回列表。
/// 目前这是一个占位符实现，实际的类型检查可能需要符号表信息。
///
/// # 重要提示
/// UNWIND 表达式必须求值为列表类型，否则会在执行时出错。
pub fn validate_unwind_expression_type(
    _expr: &crate::graph::expression::Expression,
) -> Result<(), PlannerError> {
    // TODO: 实现完整的类型检查逻辑
    // 需要访问符号表或类型推断系统来验证表达式类型
    
    // 目前假设所有表达式都是有效的
    // 在实际实现中，应该检查表达式是否能求值为列表
    Ok(())
}

// 注意：UNWIND 子句规划器不应该创建起始节点
// 起始节点应该在查询的最开始（如 MATCH 子句）创建
// UNWIND 子句必须接收来自上游子句的输入数据
//
// # 正确的架构设计
// 1. 数据流从上游子句流向 UNWIND 子句
// 2. UNWIND 子句处理展开逻辑
// 3. 结果传递给下游子句
//
// # 错误的做法
// - 在 UNWIND 子句中创建起始节点
// - 假设 UNWIND 是查询的起点
// - 忽略输入数据的依赖关系
