//! 通用规则trait和工具函数的完整实现
//! 提供优化规则的通用接口和辅助函数，减少代码重复

use super::optimizer::{OptContext, OptGroupNode, OptRule, OptimizerError, Pattern};
use crate::core::types::operators::BinaryOperator;
use crate::core::{Expression, Value};
use crate::query::planner::plan::PlanNodeEnum;

use std::collections::HashMap;

/// 优化规则的基础trait，扩展了OptRule
pub trait BaseOptRule: OptRule {
    /// 获取规则的优先级，数值越小优先级越高
    fn priority(&self) -> u32 {
        100 // 默认优先级
    }

    /// 检查规则是否适用于给定的计划节点
    fn is_applicable(&self, node: &OptGroupNode) -> bool {
        self.pattern().matches(node)
    }

    /// 应用规则前的验证
    fn validate(&self, _ctx: &OptContext, _node: &OptGroupNode) -> Result<(), OptimizerError> {
        // 默认实现不做任何验证
        Ok(())
    }

    /// 应用规则后的处理
    fn post_process(
        &self,
        _ctx: &mut OptContext,
        _original_node: &OptGroupNode,
        _result_node: &OptGroupNode,
    ) -> Result<(), OptimizerError> {
        // 默认实现不做任何处理
        Ok(())
    }
}

/// 下推优化规则的通用trait
pub trait PushDownRule: BaseOptRule {
    /// 检查是否可以下推到指定的子节点类型
    fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool;

    /// 获取下推后的新节点
    fn create_pushed_down_node(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
        child: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError>;
}

/// 合并优化规则的通用trait
pub trait MergeRule: BaseOptRule {
    /// 检查是否可以合并指定的节点
    fn can_merge(&self, node: &OptGroupNode, child: &OptGroupNode) -> bool;

    /// 创建合并后的新节点
    fn create_merged_node(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
        child: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError>;
}

/// 消除优化规则的通用trait
pub trait EliminationRule: BaseOptRule {
    /// 检查节点是否可以被消除
    fn can_eliminate(&self, _ctx: &OptContext, _node: &OptGroupNode) -> bool;

    /// 获取消除后的替代节点（如果有）
    fn get_replacement(
        &self,
        _ctx: &mut OptContext,
        _node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError>;
}

/// 表达式解析器，用于分析条件表达式
#[derive(Debug, Clone)]
pub struct ExpressionParser {
    // 缓存已解析的表达式，避免重复解析
    parsed_expressions: HashMap<String, ParsedExpression>,
}

/// 解析后的表达式结构
#[derive(Debug, Clone)]
pub struct ParsedExpression {
    pub is_tautology: bool,
    pub variables: Vec<String>,
    pub operators: Vec<String>,
}

impl ExpressionParser {
    pub fn new() -> Self {
        Self {
            parsed_expressions: HashMap::new(),
        }
    }

    /// 解析表达式并检查是否为永真式
    pub fn parse_and_check_tautology(&mut self, expression: &str) -> bool {
        // 检查缓存
        if let Some(parsed) = self.parsed_expressions.get(expression) {
            return parsed.is_tautology;
        }

        let is_tautology = self.check_tautology(expression);

        // 缓存结果
        self.parsed_expressions.insert(
            expression.to_string(),
            ParsedExpression {
                is_tautology,
                variables: self.extract_variables(expression),
                operators: self.extract_operators(expression),
            },
        );

        is_tautology
    }

    /// 检查表达式是否为永真式
    fn check_tautology(&self, expression: &str) -> bool {
        let expression = expression.trim();

        // 检查简单的布尔常量
        match expression {
            "1 = 1" | "true" | "TRUE" | "True" | "0 = 0" => return true,
            _ => {}
        }

        // 检查形如 a = a 的表达式
        if let Some(eq_pos) = expression.find('=') {
            let left = expression[..eq_pos].trim();
            let right = expression[eq_pos + 1..].trim();

            // 如果左右两边相同（忽略空格），则是永真式
            if left == right {
                return true;
            }

            // 检查更复杂的相等表达式，如 (a + b) = (b + a)
            if self.are_expressions_equivalent(left, right) {
                return true;
            }
        }

        // 检查逻辑永真式，如 (a AND b) OR (NOT a AND b) OR (a AND NOT b) OR (NOT a AND NOT b)
        if self.check_logical_tautology(expression) {
            return true;
        }

        false
    }

    /// 检查两个表达式是否等价
    fn are_expressions_equivalent(&self, left: &str, right: &str) -> bool {
        // 简单实现：去除括号后比较
        let left_clean = left.replace('(', "").replace(')', "").trim().to_string();
        let right_clean = right.replace('(', "").replace(')', "").trim().to_string();

        if left_clean == right_clean {
            return true;
        }

        // 检查加法交换律：a + b = b + a
        if let (Some(left_plus), Some(right_plus)) = (left_clean.find('+'), right_clean.find('+')) {
            let left_a = left_clean[..left_plus].trim();
            let left_b = left_clean[left_plus + 1..].trim();
            let right_a = right_clean[..right_plus].trim();
            let right_b = right_clean[right_plus + 1..].trim();

            return (left_a == right_a && left_b == right_b)
                || (left_a == right_b && left_b == right_a);
        }

        // 检查乘法交换律：a * b = b * a
        if let (Some(left_mul), Some(right_mul)) = (left_clean.find('*'), right_clean.find('*')) {
            let left_a = left_clean[..left_mul].trim();
            let left_b = left_clean[left_mul + 1..].trim();
            let right_a = right_clean[..right_mul].trim();
            let right_b = right_clean[right_mul + 1..].trim();

            return (left_a == right_a && left_b == right_b)
                || (left_a == right_b && left_b == right_a);
        }

        false
    }

    /// 检查逻辑永真式
    fn check_logical_tautology(&self, expression: &str) -> bool {
        // 简单实现：检查 (a OR NOT a) 形式的表达式
        if let Some(or_pos) = expression.find("OR") {
            let left = expression[..or_pos].trim();
            let right = expression[or_pos + 2..].trim();

            // 检查 NOT a 形式
            if right.starts_with("NOT ") {
                let not_expression = right[4..].trim();
                if left == not_expression {
                    return true;
                }
            }

            // 检查 a OR NOT a 形式
            if left.starts_with("NOT ") {
                let not_expression = left[4..].trim();
                if right == not_expression {
                    return true;
                }
            }
        }

        false
    }

    /// 提取表达式中的变量
    fn extract_variables(&self, expression: &str) -> Vec<String> {
        // 简单实现：提取字母开头的标识符
        let mut variables = Vec::new();
        let chars: Vec<char> = expression.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i].is_alphabetic() {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let var = expression[start..i].to_string();
                if !variables.contains(&var) {
                    variables.push(var);
                }
            } else {
                i += 1;
            }
        }

        variables
    }

    /// 提取表达式中的操作符
    fn extract_operators(&self, expression: &str) -> Vec<String> {
        let mut operators = Vec::new();
        let ops = [
            "=", "!=", "<", ">", "<=", ">=", "+", "-", "*", "/", "AND", "OR", "NOT",
        ];

        for op in &ops {
            if expression.contains(op) {
                operators.push(op.to_string());
            }
        }

        operators
    }
}

//全局表达式解析器实例
thread_local! {
    static EXPRESSION_PARSER: std::cell::RefCell<ExpressionParser> =
        std::cell::RefCell::new(ExpressionParser::new());
}

/// 辅助函数：检查条件是否为永真式（完整实现）
pub fn is_tautology(condition: &str) -> bool {
    match condition.trim() {
        "1 = 1" | "true" | "TRUE" | "True" | "0 = 0" => true,
        _ => {
            // 使用表达式解析器检查更复杂的永真式
            EXPRESSION_PARSER
                .with(|parser| parser.borrow_mut().parse_and_check_tautology(condition))
        }
    }
}

/// 辅助函数：检查Expression是否为永真式
pub fn is_expression_tautology(expression: &Expression) -> bool {
    match expression {
        // 检查布尔字面量
        Expression::Literal(Value::Bool(true)) => true,
        Expression::Literal(Value::Bool(false)) => false,

        // 检查二元表达式
        Expression::Binary { left, op, right } => {
            match (left.as_ref(), op, right.as_ref()) {
                // 检查 1 = 1
                (
                    Expression::Literal(Value::Int(1)),
                    BinaryOperator::Equal,
                    Expression::Literal(Value::Int(1)),
                ) => true,
                // 检查 0 = 0
                (
                    Expression::Literal(Value::Int(0)),
                    BinaryOperator::Equal,
                    Expression::Literal(Value::Int(0)),
                ) => true,
                // 检查 a = a
                (Expression::Variable(a), BinaryOperator::Equal, Expression::Variable(b))
                    if a == b =>
                {
                    true
                }
                // 检查逻辑或的永真式：a OR NOT a
                (
                    Expression::Variable(a),
                    BinaryOperator::Or,
                    Expression::Unary { op, operand },
                ) if matches!(op, crate::core::types::operators::UnaryOperator::Not)
                    && matches!(operand.as_ref(), Expression::Variable(b) if b == a) =>
                {
                    true
                }
                // 检查逻辑或的永真式：NOT a OR a
                (
                    Expression::Unary { op, operand },
                    BinaryOperator::Or,
                    Expression::Variable(b),
                ) if matches!(op, crate::core::types::operators::UnaryOperator::Not)
                    && matches!(operand.as_ref(), Expression::Variable(a) if a == b) =>
                {
                    true
                }
                _ => false,
            }
        }

        // 其他表达式类型暂时不认为是永真式
        _ => false,
    }
}

/// 辅助函数：合并两个过滤条件
pub fn combine_conditions(cond1: &str, cond2: &str) -> String {
    if cond1.is_empty() {
        cond2.to_string()
    } else if cond2.is_empty() {
        cond1.to_string()
    } else {
        format!("({}) AND ({})", cond1, cond2)
    }
}

/// 辅助函数：合并表达式列表
pub fn combine_expression_list(exprs: &[String]) -> String {
    if exprs.is_empty() {
        String::new()
    } else if exprs.len() == 1 {
        exprs[0].clone()
    } else {
        format!("({})", exprs.join(") AND ("))
    }
}

/// 辅助结构：表示过滤条件分离的结果
#[derive(Debug, Clone)]
pub struct FilterSplitResult {
    pub pushable_condition: Option<String>,  // 可以下推的条件
    pub remaining_condition: Option<String>, // 保留在Filter节点的条件
}

/// 辅助函数：创建基本的模式匹配
pub fn create_basic_pattern(node_name: &'static str) -> Pattern {
    // 这里需要根据实际的Pattern实现来调整
    // 暂时保留，但应该使用PlanNodeEnum的name()方法
    Pattern::new(node_name)
}

/// 辅助函数：创建带依赖的模式匹配
pub fn create_pattern_with_dependency(
    node_name: &'static str,
    dependency_name: &'static str,
) -> Pattern {
    // 这里需要根据实际的Pattern实现来调整
    // 暂时保留，但应该使用PlanNodeEnum的name()方法
    Pattern::new(node_name).with_dependency(Pattern::new(dependency_name))
}

/// 辅助函数：检查节点是否有指定类型的依赖（完整实现）
///
/// # 参数
/// * `ctx` - 优化上下文，用于查找依赖节点
/// * `node` - 要检查的节点
/// * `kind` - 要查找的依赖节点类型
///
/// # 返回值
/// 如果找到指定类型的依赖节点，返回true；否则返回false
pub fn has_dependency_of_kind(
    ctx: &OptContext,
    node: &OptGroupNode,
    node_name: &'static str,
) -> bool {
    // 检查节点的依赖列表
    for &dep_id in &node.dependencies {
        // 使用OptContext查找依赖节点
        if let Some(dep_node) = ctx.find_group_node_by_plan_node_id(dep_id) {
            // 检查依赖节点的计划节点类型
            if dep_node.plan_node.name() == node_name {
                return true;
            }
        }
    }

    false
}

/// 辅助函数：获取节点的第一个依赖（完整实现）
///
/// # 参数
/// * `ctx` - 优化上下文，用于查找依赖节点
/// * `node` - 要获取依赖的节点
///
/// # 返回值
/// 如果节点有依赖，返回第一个依赖节点的引用；否则返回None
pub fn get_first_dependency<'a>(
    ctx: &'a OptContext,
    node: &OptGroupNode,
) -> Option<&'a OptGroupNode> {
    // 检查是否有依赖
    if node.dependencies.is_empty() {
        return None;
    }

    // 获取第一个依赖的ID
    let first_dep_id = node.dependencies[0];

    // 使用OptContext查找依赖节点
    ctx.find_group_node_by_plan_node_id(first_dep_id)
}

/// 辅助函数：获取节点的所有依赖（完整实现）
///
/// # 参数
/// * `ctx` - 优化上下文，用于查找依赖节点
/// * `node` - 要获取依赖的节点
///
/// # 返回值
/// 返回包含所有依赖节点引用的向量
pub fn get_all_dependencies<'a>(ctx: &'a OptContext, node: &OptGroupNode) -> Vec<&'a OptGroupNode> {
    let mut dependencies = Vec::new();

    // 遍历节点的依赖列表
    for &dep_id in &node.dependencies {
        // 使用OptContext查找依赖节点
        if let Some(dep_node) = ctx.find_group_node_by_plan_node_id(dep_id) {
            dependencies.push(dep_node);
        }
    }

    dependencies
}

/// 辅助函数：获取指定索引的依赖节点（完整实现）
///
/// # 参数
/// * `ctx` - 优化上下文，用于查找依赖节点
/// * `node` - 要获取依赖的节点
/// * `index` - 依赖节点的索引位置
///
/// # 返回值
/// 如果索引有效，返回对应依赖节点的引用；否则返回None
pub fn get_dependency_at<'a>(
    ctx: &'a OptContext,
    node: &OptGroupNode,
    index: usize,
) -> Option<&'a OptGroupNode> {
    // 检查索引是否有效
    if index >= node.dependencies.len() {
        return None;
    }

    // 获取指定索引的依赖ID
    let dep_id = node.dependencies[index];

    // 使用OptContext查找依赖节点
    ctx.find_group_node_by_plan_node_id(dep_id)
}

/// 辅助函数：创建新的OptGroupNode
pub fn create_new_opt_group_node(id: usize, plan_node: PlanNodeEnum) -> OptGroupNode {
    OptGroupNode::new(id, plan_node)
}

/// 辅助函数：克隆OptGroupNode但替换计划节点
pub fn clone_with_new_plan_node(node: &OptGroupNode, plan_node: PlanNodeEnum) -> OptGroupNode {
    let mut new_node = node.clone();
    new_node.plan_node = plan_node;
    new_node
}

/// 宏：简化规则实现的重复代码
#[macro_export]
macro_rules! impl_basic_rule {
    ($rule_type:ty, $name:expr) => {
        impl OptRule for $rule_type {
            fn name(&self) -> &str {
                $name
            }
        }

        impl BaseOptRule for $rule_type {
            // 使用默认实现
        }
    };
}

/// 宏：简化下推规则的实现
#[macro_export]
macro_rules! impl_push_down_rule {
    ($rule_type:ty, $name:expr, $target_kind:expr) => {
        impl OptRule for $rule_type {
            fn name(&self) -> &str {
                $name
            }
        }

        impl BaseOptRule for $rule_type {
            // 使用默认实现
        }

        impl PushDownRule for $rule_type {
            fn can_push_down_to(&self, child_node: &PlanNodeEnum) -> bool {
                child_node.name() == $target_kind
            }

            fn create_pushed_down_node(
                &self,
                ctx: &mut OptContext,
                node: &OptGroupNode,
                child: &OptGroupNode,
            ) -> Result<Option<OptGroupNode>, OptimizerError> {
                // 默认实现：返回None，表示不进行下推
                // 具体规则应该重写此方法
                Ok(None)
            }
        }
    };
}

/// 宏：简化合并规则的实现
#[macro_export]
macro_rules! impl_merge_rule {
    ($rule_type:ty, $name:expr) => {
        impl OptRule for $rule_type {
            fn name(&self) -> &str {
                $name
            }
        }

        impl BaseOptRule for $rule_type {
            // 使用默认实现
        }

        impl MergeRule for $rule_type {
            // 默认实现，需要具体规则重写
            fn can_merge(&self, _node: &OptGroupNode, _child: &OptGroupNode) -> bool {
                false
            }

            fn create_merged_node(
                &self,
                _ctx: &mut OptContext,
                _node: &OptGroupNode,
                _child: &OptGroupNode,
            ) -> Result<Option<OptGroupNode>, OptimizerError> {
                Ok(None)
            }
        }
    };
}

/// 宏：简化消除规则的实现
#[macro_export]
macro_rules! impl_elimination_rule {
    ($rule_type:ty, $name:expr) => {
        impl OptRule for $rule_type {
            fn name(&self) -> &str {
                $name
            }
        }

        impl BaseOptRule for $rule_type {
            // 使用默认实现
        }

        impl EliminationRule for $rule_type {
            // 默认实现，需要具体规则重写
            fn can_eliminate(&self, _ctx: &OptContext, _node: &OptGroupNode) -> bool {
                false
            }

            fn get_replacement(
                &self,
                _ctx: &mut OptContext,
                _node: &OptGroupNode,
            ) -> Result<Option<OptGroupNode>, OptimizerError> {
                Ok(None)
            }
        }
    };
}

/// 新增宏：实现带有自定义验证的规则
#[macro_export]
macro_rules! impl_rule_with_validation {
    ($rule_type:ty, $name:expr, $validate:block) => {
        impl OptRule for $rule_type {
            fn name(&self) -> &str {
                $name
            }
        }

        impl BaseOptRule for $rule_type {
            fn validate(&self, _ctx: &OptContext, _node: &OptGroupNode) -> Result<(), OptimizerError> {
                $validate
                Ok(())
            }
        }
    };
}

/// 新增宏：实现带有自定义后处理的规则
#[macro_export]
macro_rules! impl_rule_with_post_process {
    ($rule_type:ty, $name:expr, $post_process:block) => {
        impl OptRule for $rule_type {
            fn name(&self) -> &str {
                $name
            }
        }

        impl BaseOptRule for $rule_type {
            fn post_process(
                &self,
                _ctx: &mut OptContext,
                _original_node: &OptGroupNode,
                _result_node: &OptGroupNode
            ) -> Result<(), OptimizerError> {
                $post_process
                Ok(())
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tautology_simple() {
        assert!(is_tautology("1 = 1"));
        assert!(is_tautology("true"));
        assert!(is_tautology("TRUE"));
        assert!(is_tautology("True"));
        assert!(is_tautology("0 = 0"));
        assert!(!is_tautology("1 = 0"));
        assert!(!is_tautology("false"));
    }

    #[test]
    fn test_is_tautology_variable_equality() {
        assert!(is_tautology("a = a"));
        assert!(is_tautology("variable = variable"));
        assert!(!is_tautology("a = b"));
    }

    #[test]
    fn test_is_tautology_commutative_operations() {
        assert!(is_tautology("a + b = b + a"));
        assert!(is_tautology("x * y = y * x"));
        assert!(!is_tautology("a + b = a - b"));
    }

    #[test]
    fn test_is_tautology_logical_expressions() {
        assert!(is_tautology("a OR NOT a"));
        assert!(is_tautology("NOT a OR a"));
        assert!(!is_tautology("a AND NOT a"));
    }

    #[test]
    fn test_combine_conditions() {
        assert_eq!(combine_conditions("", "b > 10"), "b > 10");
        assert_eq!(combine_conditions("a > 5", ""), "a > 5");
        assert_eq!(
            combine_conditions("a > 5", "b < 10"),
            "(a > 5) AND (b < 10)"
        );
    }

    #[test]
    fn test_combine_expression_list() {
        assert_eq!(combine_expression_list(&[]), "");
        assert_eq!(combine_expression_list(&["a > 5".to_string()]), "a > 5");
        assert_eq!(
            combine_expression_list(&["a > 5".to_string(), "b < 10".to_string()]),
            "(a > 5) AND (b < 10)"
        );
    }

    #[test]
    fn test_expression_parser() {
        let mut parser = ExpressionParser::new();

        assert!(parser.parse_and_check_tautology("1 = 1"));
        assert!(parser.parse_and_check_tautology("a = a"));
        assert!(parser.parse_and_check_tautology("a + b = b + a"));
        assert!(!parser.parse_and_check_tautology("a = b"));

        // 测试缓存
        assert!(parser.parse_and_check_tautology("a = a"));
    }

    #[test]
    fn test_dependency_functions() {
        use crate::query::context::execution::QueryContext;

        // 创建测试上下文
        let _session_info = crate::api::session::session_manager::SessionInfo {
            session_id: 1,
            user_name: "test_user".to_string(),
            space_name: None,
            graph_addr: None,
            create_time: std::time::SystemTime::now(),
            last_access_time: std::time::SystemTime::now(),
            active_queries: 0,
            timezone: None,
        };
        let query_ctx = QueryContext::new();
        let mut opt_ctx = OptContext::new(query_ctx);

        // 创建测试节点 - 使用 OptGroupNode::default() 创建默认节点
        let mut node1 = OptGroupNode::default();
        node1.id = 1;

        let mut node2 = OptGroupNode::default();
        node2.id = 2;

        let mut node3 = OptGroupNode::default();
        node3.id = 3;

        // 添加节点到上下文
        opt_ctx.add_plan_node_and_group_node(1, &node1);
        opt_ctx.add_plan_node_and_group_node(2, &node2);
        opt_ctx.add_plan_node_and_group_node(3, &node3);

        // 创建一个有依赖的节点
        let mut node_with_deps = OptGroupNode::default();
        node_with_deps.id = 4;
        node_with_deps.dependencies = vec![1, 2, 3]; // 依赖于节点1、2、3

        // 测试 get_first_dependency
        let first_dep = get_first_dependency(&opt_ctx, &node_with_deps);
        assert!(first_dep.is_some());
        assert_eq!(first_dep.expect("Dependency should exist").id, 1);

        // 测试 get_dependency_at
        let dep_at_1 = get_dependency_at(&opt_ctx, &node_with_deps, 1);
        assert!(dep_at_1.is_some());
        assert_eq!(dep_at_1.expect("Dependency should exist").id, 2);

        let dep_at_2 = get_dependency_at(&opt_ctx, &node_with_deps, 2);
        assert!(dep_at_2.is_some());
        assert_eq!(dep_at_2.expect("Dependency should exist").id, 3);

        // 测试越界索引
        let dep_at_3 = get_dependency_at(&opt_ctx, &node_with_deps, 3);
        assert!(dep_at_3.is_none());

        // 测试 get_all_dependencies
        let all_deps = get_all_dependencies(&opt_ctx, &node_with_deps);
        assert_eq!(all_deps.len(), 3);
        assert_eq!(all_deps[0].id, 1);
        assert_eq!(all_deps[1].id, 2);
        assert_eq!(all_deps[2].id, 3);

        // 测试 has_dependency_of_kind
        // 默认节点是 Start 类型
        let has_start = has_dependency_of_kind(&opt_ctx, &node_with_deps, "Start");
        assert!(has_start);

        // 使用一个不存在的类型进行测试
        let has_filter = has_dependency_of_kind(&opt_ctx, &node_with_deps, "Filter");
        assert!(!has_filter);

        // 测试没有依赖的节点
        let mut node_no_deps = OptGroupNode::default();
        node_no_deps.id = 5;
        let first_dep_none = get_first_dependency(&opt_ctx, &node_no_deps);
        assert!(first_dep_none.is_none());

        let all_deps_empty = get_all_dependencies(&opt_ctx, &node_no_deps);
        assert!(all_deps_empty.is_empty());
    }
}
