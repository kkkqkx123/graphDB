//! 表达式分析模块
//!
//! 提供表达式特性分析功能，包括：
//! - 确定性检查（是否包含非确定性函数）
//! - 复杂度评分
//! - 属性/变量/函数提取

use crate::core::types::expr::visitor::ExpressionVisitor;
use crate::core::types::expr::visitor_collectors::{
    FunctionCollector, PropertyCollector, VariableCollector,
};
use crate::core::types::ContextualExpression;
use crate::core::Expression;

/// 表达式分析结果
#[derive(Debug, Clone, Default)]
pub struct ExpressionAnalysis {
    /// 是否确定性（不含rand()、now()等非确定性函数）
    pub is_deterministic: bool,
    /// 复杂度评分（0-100）
    pub complexity_score: u32,
    /// 引用的属性列表
    pub referenced_properties: Vec<String>,
    /// 引用的变量列表
    pub referenced_variables: Vec<String>,
    /// 调用的函数列表
    pub called_functions: Vec<String>,
    /// 是否包含聚合函数
    pub contains_aggregate: bool,
    /// 是否包含子查询
    pub contains_subquery: bool,
    /// 节点数量
    pub node_count: u32,
}

impl ExpressionAnalysis {
    /// 创建空的分析结果
    pub fn new() -> Self {
        Self {
            is_deterministic: true, // 默认假设是确定性的
            ..Default::default()
        }
    }
}

/// 表达式分析模式
///
/// 预设的分析模式，简化配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisMode {
    /// 完整分析（默认）
    Full,
    /// 只检查确定性
    DeterministicOnly,
    /// 只提取属性引用
    PropertyExtractor,
    /// 只提取变量引用
    VariableExtractor,
}

/// 表达式分析选项
#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    /// 分析确定性
    pub check_deterministic: bool,
    /// 分析复杂度
    pub check_complexity: bool,
    /// 提取属性引用
    pub extract_properties: bool,
    /// 提取变量引用
    pub extract_variables: bool,
    /// 统计函数调用
    pub count_functions: bool,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            check_deterministic: true,
            check_complexity: true,
            extract_properties: true,
            extract_variables: true,
            count_functions: true,
        }
    }
}

impl AnalysisOptions {
    /// 从分析模式创建选项
    fn from_mode(mode: AnalysisMode) -> Self {
        match mode {
            AnalysisMode::Full => AnalysisOptions {
                check_deterministic: true,
                check_complexity: true,
                extract_properties: true,
                extract_variables: true,
                count_functions: true,
            },
            AnalysisMode::DeterministicOnly => AnalysisOptions {
                check_deterministic: true,
                check_complexity: false,
                extract_properties: false,
                extract_variables: false,
                count_functions: false,
            },
            AnalysisMode::PropertyExtractor => AnalysisOptions {
                check_deterministic: false,
                check_complexity: false,
                extract_properties: true,
                extract_variables: false,
                count_functions: false,
            },
            AnalysisMode::VariableExtractor => AnalysisOptions {
                check_deterministic: false,
                check_complexity: false,
                extract_properties: false,
                extract_variables: true,
                count_functions: false,
            },
        }
    }
}

/// 非确定性函数检查
///
/// 使用编译时静态匹配而非运行时HashMap，提高性能。
/// 非确定性函数每次调用可能返回不同结果（如rand()、now()等）。
pub struct NondeterministicChecker;

impl NondeterministicChecker {
    /// 检查函数是否非确定性
    ///
    /// 使用match进行编译时优化，比HashMap查找更高效
    pub fn is_nondeterministic(func_name: &str) -> bool {
        match func_name {
            // 时间相关函数
            "now" | "current_time" | "current_date" | "current_timestamp" | "localtime"
            | "localtimestamp" => true,

            // 随机数函数
            "rand" | "random" | "uuid" => true,

            // 窗口函数（结果依赖于行位置）
            "row_number" | "rank" | "dense_rank" | "percent_rank" | "cume_dist" => true,

            // 其他非确定性函数
            "last_insert_id" | "connection_id" | "current_user" | "session_user" => true,

            // 确定性函数
            _ => false,
        }
    }
}

/// 表达式分析器
///
/// 分析表达式的各种特性，支持按需分析（通过预设模式配置）。
#[derive(Debug, Clone)]
pub struct ExpressionAnalyzer {
    /// 分析选项
    options: AnalysisOptions,
}

impl ExpressionAnalyzer {
    /// 创建默认的表达式分析器（完整分析模式）
    pub fn new() -> Self {
        Self {
            options: AnalysisOptions::default(),
        }
    }

    /// 创建带选项的表达式分析器
    pub fn with_options(options: AnalysisOptions) -> Self {
        Self { options }
    }

    /// 创建只检查确定性的分析器
    pub fn deterministic_only() -> Self {
        Self {
            options: AnalysisOptions::from_mode(AnalysisMode::DeterministicOnly),
        }
    }

    /// 创建只提取属性引用的分析器
    pub fn property_extractor() -> Self {
        Self {
            options: AnalysisOptions::from_mode(AnalysisMode::PropertyExtractor),
        }
    }

    /// 创建只提取变量引用的分析器
    pub fn variable_extractor() -> Self {
        Self {
            options: AnalysisOptions::from_mode(AnalysisMode::VariableExtractor),
        }
    }

    /// 分析表达式（接受 ContextualExpression）
    ///
    /// # 参数
    /// - `ctx_expr`: 要分析的上下文表达式
    ///
    /// # 返回
    /// 表达式的分析结果
    pub fn analyze(&self, ctx_expr: &ContextualExpression) -> ExpressionAnalysis {
        let mut analysis = ExpressionAnalysis::new();

        // 通过 ContextualExpression 获取 Expression
        if let Some(expr_meta) = ctx_expr.expression() {
            let expr = expr_meta.inner();

            // 使用现有的 Collector 收集信息
            if self.options.extract_properties {
                let mut collector = PropertyCollector::new();
                collector.visit(expr);
                analysis.referenced_properties = collector.properties;
            }

            if self.options.extract_variables {
                let mut collector = VariableCollector::new();
                collector.visit(expr);
                analysis.referenced_variables = collector.variables;
            }

            if self.options.count_functions {
                let mut collector = FunctionCollector::new();
                collector.visit(expr);
                analysis.called_functions = collector.functions;
            }

            // 使用自定义 Visitor 进行复杂度和确定性分析
            let mut visitor = AnalysisVisitor::new(&mut analysis, self.options.clone());
            visitor.visit(expr);
        }

        analysis
    }

    /// 快速检查表达式是否确定性
    pub fn is_deterministic(&self, ctx_expr: &ContextualExpression) -> bool {
        let analysis = self.analyze(ctx_expr);
        analysis.is_deterministic
    }

    /// 快速提取表达式引用的属性
    pub fn extract_properties(&self, ctx_expr: &ContextualExpression) -> Vec<String> {
        let analysis = self.analyze(ctx_expr);
        analysis.referenced_properties
    }

    /// 快速提取表达式引用的变量
    pub fn extract_variables(&self, ctx_expr: &ContextualExpression) -> Vec<String> {
        let analysis = self.analyze(ctx_expr);
        analysis.referenced_variables
    }
}

/// 表达式分析 Visitor
///
/// 使用 Visitor 模式进行复杂度和确定性分析
struct AnalysisVisitor<'a> {
    analysis: &'a mut ExpressionAnalysis,
    options: AnalysisOptions,
}

impl<'a> AnalysisVisitor<'a> {
    fn new(analysis: &'a mut ExpressionAnalysis, options: AnalysisOptions) -> Self {
        Self { analysis, options }
    }
}

impl ExpressionVisitor for AnalysisVisitor<'_> {
    fn visit_literal(&mut self, _value: &crate::core::Value) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 1;
        }
        self.analysis.node_count += 1;
    }

    fn visit_variable(&mut self, _name: &str) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 2;
        }
        self.analysis.node_count += 1;
    }

    fn visit_property(&mut self, object: &Expression, _property: &str) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 5;
        }
        self.analysis.node_count += 1;
        self.visit(object);
    }

    fn visit_binary(
        &mut self,
        op: crate::core::types::BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 2;
            if op == crate::core::types::BinaryOperator::Like {
                self.analysis.complexity_score += 5;
            }
        }
        self.analysis.node_count += 1;
        self.visit(left);
        self.visit(right);
    }

    fn visit_unary(&mut self, _op: crate::core::types::UnaryOperator, operand: &Expression) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 1;
        }
        self.analysis.node_count += 1;
        self.visit(operand);
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) {
        if self.options.check_deterministic && NondeterministicChecker::is_nondeterministic(name) {
            self.analysis.is_deterministic = false;
        }
        if self.options.check_complexity {
            self.analysis.complexity_score += 10 + args.len() as u32 * 2;
        }
        self.analysis.node_count += 1;
        for arg in args {
            self.visit(arg);
        }
    }

    fn visit_aggregate(
        &mut self,
        _func: &crate::core::types::operators::AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) {
        self.analysis.contains_aggregate = true;
        if self.options.check_complexity {
            self.analysis.complexity_score += 20;
        }
        self.analysis.node_count += 1;
        self.visit(arg);
    }

    fn visit_case(
        &mut self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 5 + conditions.len() as u32 * 5;
        }
        self.analysis.node_count += 1;
        if let Some(test) = test_expr {
            self.visit(test);
        }
        for (when, then) in conditions {
            self.visit(when);
            self.visit(then);
        }
        if let Some(default_expr) = default {
            self.visit(default_expr);
        }
    }

    fn visit_type_cast(
        &mut self,
        expression: &Expression,
        _target_type: &crate::core::types::DataType,
    ) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 3;
        }
        self.analysis.node_count += 1;
        self.visit(expression);
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 4;
        }
        self.analysis.node_count += 1;
        self.visit(collection);
        self.visit(index);
    }

    fn visit_list(&mut self, items: &[Expression]) {
        if self.options.check_complexity {
            self.analysis.complexity_score += items.len() as u32;
        }
        self.analysis.node_count += 1;
        for item in items {
            self.visit(item);
        }
    }

    fn visit_map(&mut self, entries: &[(String, Expression)]) {
        if self.options.check_complexity {
            self.analysis.complexity_score += entries.len() as u32 * 2;
        }
        self.analysis.node_count += 1;
        for (_, value) in entries {
            self.visit(value);
        }
    }

    fn visit_list_comprehension(
        &mut self,
        _variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    ) {
        self.analysis.contains_subquery = true;
        if self.options.check_complexity {
            self.analysis.complexity_score += 30;
        }
        self.analysis.node_count += 1;
        self.visit(source);
        if let Some(f) = filter {
            self.visit(f);
        }
        if let Some(m) = map {
            self.visit(m);
        }
    }

    fn visit_predicate(&mut self, _func: &str, args: &[Expression]) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 15;
        }
        self.analysis.node_count += 1;
        for arg in args {
            self.visit(arg);
        }
    }

    fn visit_reduce(
        &mut self,
        _accumulator: &str,
        initial: &Expression,
        _variable: &str,
        source: &Expression,
        mapping: &Expression,
    ) {
        self.analysis.contains_subquery = true;
        if self.options.check_complexity {
            self.analysis.complexity_score += 25;
        }
        self.analysis.node_count += 1;
        self.visit(initial);
        self.visit(source);
        self.visit(mapping);
    }

    fn visit_path(&mut self, items: &[Expression]) {
        if self.options.check_complexity {
            self.analysis.complexity_score += items.len() as u32 * 3;
        }
        self.analysis.node_count += 1;
        for item in items {
            self.visit(item);
        }
    }

    fn visit_path_build(&mut self, items: &[Expression]) {
        if self.options.check_complexity {
            self.analysis.complexity_score += items.len() as u32 * 2;
        }
        self.analysis.node_count += 1;
        for item in items {
            self.visit(item);
        }
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 5;
        }
        self.analysis.node_count += 1;
        self.visit(collection);
        if let Some(s) = start {
            self.visit(s);
        }
        if let Some(e) = end {
            self.visit(e);
        }
    }

    fn visit_label(&mut self, _label: &str) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 3;
        }
        self.analysis.node_count += 1;
    }

    fn visit_label_tag_property(&mut self, tag: &Expression, _property: &str) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 5;
        }
        self.analysis.node_count += 1;
        self.visit(tag);
    }

    fn visit_tag_property(&mut self, _tag_name: &str, _property: &str) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 3;
        }
        self.analysis.node_count += 1;
    }

    fn visit_edge_property(&mut self, _edge_name: &str, _property: &str) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 3;
        }
        self.analysis.node_count += 1;
    }

    fn visit_parameter(&mut self, _name: &str) {
        if self.options.check_complexity {
            self.analysis.complexity_score += 1;
        }
        self.analysis.node_count += 1;
    }
}

impl Default for ExpressionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::query::validator::context::expression_context::ExpressionAnalysisContext;
    use std::sync::Arc;

    #[test]
    fn test_expression_analyzer_new() {
        let _analyzer = ExpressionAnalyzer::new();
        // 验证创建成功
    }

    #[test]
    fn test_literal_is_deterministic() {
        let analyzer = ExpressionAnalyzer::new();
        let expr = Expression::Literal(Value::Int(42));
        let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr_meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let expr_id = expr_ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(expr_id, expr_ctx);
        let analysis = analyzer.analyze(&ctx_expr);
        assert!(analysis.is_deterministic);
        assert_eq!(analysis.node_count, 1);
    }

    #[test]
    fn test_variable_extraction() {
        let analyzer = ExpressionAnalyzer::new();
        let expr = Expression::Variable("x".to_string());
        let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr_meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let expr_id = expr_ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(expr_id, expr_ctx);
        let analysis = analyzer.analyze(&ctx_expr);
        assert!(analysis.referenced_variables.contains(&"x".to_string()));
    }

    #[test]
    fn test_nondeterministic_function_detection() {
        let analyzer = ExpressionAnalyzer::new();
        let expr = Expression::Function {
            name: "rand".to_string(),
            args: vec![],
        };
        let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr_meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let expr_id = expr_ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(expr_id, expr_ctx);
        let analysis = analyzer.analyze(&ctx_expr);
        assert!(!analysis.is_deterministic);
    }

    #[test]
    fn test_deterministic_function() {
        let analyzer = ExpressionAnalyzer::new();
        let expr = Expression::Function {
            name: "abs".to_string(),
            args: vec![Expression::Literal(Value::Int(-5))],
        };
        let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr_meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let expr_id = expr_ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(expr_id, expr_ctx);
        let analysis = analyzer.analyze(&ctx_expr);
        assert!(analysis.is_deterministic);
    }

    #[test]
    fn test_property_extraction() {
        let analyzer = ExpressionAnalyzer::property_extractor();
        let expr = Expression::Property {
            object: Box::new(Expression::Variable("n".to_string())),
            property: "name".to_string(),
        };
        let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr_meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let expr_id = expr_ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(expr_id, expr_ctx);
        let analysis = analyzer.analyze(&ctx_expr);
        assert!(analysis.referenced_properties.contains(&"name".to_string()));
    }

    #[test]
    fn test_complexity_score() {
        let analyzer = ExpressionAnalyzer::new();
        // 简单表达式
        let simple = Expression::Literal(Value::Int(1));
        let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
        let simple_meta = crate::core::types::expr::ExpressionMeta::new(simple);
        let simple_id = expr_ctx.register_expression(simple_meta);
        let simple_ctx_expr =
            crate::core::types::ContextualExpression::new(simple_id, expr_ctx.clone());
        let simple_analysis = analyzer.analyze(&simple_ctx_expr);
        assert!(simple_analysis.complexity_score < 10);

        // 复杂表达式
        let complex = Expression::Function {
            name: "coalesce".to_string(),
            args: vec![
                Expression::Property {
                    object: Box::new(Expression::Variable("a".to_string())),
                    property: "x".to_string(),
                },
                Expression::Property {
                    object: Box::new(Expression::Variable("b".to_string())),
                    property: "y".to_string(),
                },
                Expression::Literal(Value::Null(crate::core::value::types::NullType::Null)),
            ],
        };
        let complex_meta = crate::core::types::expr::ExpressionMeta::new(complex);
        let complex_id = expr_ctx.register_expression(complex_meta);
        let complex_ctx_expr = crate::core::types::ContextualExpression::new(complex_id, expr_ctx);
        let complex_analysis = analyzer.analyze(&complex_ctx_expr);
        assert!(complex_analysis.complexity_score > simple_analysis.complexity_score);
    }
}
