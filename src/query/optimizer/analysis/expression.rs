//! 表达式分析模块
//!
//! 提供表达式特性分析功能，包括：
//! - 确定性检查（是否包含非确定性函数）
//! - 复杂度评分
//! - 属性/变量/函数提取

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
    /// 表达式深度
    pub depth: u32,
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

    /// 添加属性引用
    fn add_property(&mut self, property: String) {
        if !self.referenced_properties.contains(&property) {
            self.referenced_properties.push(property);
        }
    }

    /// 添加变量引用
    fn add_variable(&mut self, variable: String) {
        if !self.referenced_variables.contains(&variable) {
            self.referenced_variables.push(variable);
        }
    }

    /// 添加函数调用
    fn add_function(&mut self, function: String) {
        if !self.called_functions.contains(&function) {
            self.called_functions.push(function);
        }
    }
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
/// 分析表达式的各种特性，支持按需分析（通过AnalysisOptions配置）。
#[derive(Debug, Clone)]
pub struct ExpressionAnalyzer {
    /// 分析选项
    options: AnalysisOptions,
}

impl ExpressionAnalyzer {
    /// 创建默认的表达式分析器
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
        Self::with_options(AnalysisOptions {
            check_deterministic: true,
            check_complexity: false,
            extract_properties: false,
            extract_variables: false,
            count_functions: false,
        })
    }

    /// 创建只提取属性引用的分析器
    pub fn property_extractor() -> Self {
        Self::with_options(AnalysisOptions {
            check_deterministic: false,
            check_complexity: false,
            extract_properties: true,
            extract_variables: false,
            count_functions: false,
        })
    }

    /// 分析表达式
    ///
    /// # 参数
    /// - `expr`: 要分析的表达式
    ///
    /// # 返回
    /// 表达式的分析结果
    pub fn analyze(&self, expr: &Expression) -> ExpressionAnalysis {
        let mut result = ExpressionAnalysis::new();
        self.analyze_recursive(expr, &mut result, 0);
        result
    }

    /// 快速检查表达式是否确定性
    pub fn is_deterministic(&self, expr: &Expression) -> bool {
        let analysis = self.analyze(expr);
        analysis.is_deterministic
    }

    /// 快速提取表达式引用的属性
    pub fn extract_properties(&self, expr: &Expression) -> Vec<String> {
        let analysis = self.analyze(expr);
        analysis.referenced_properties
    }

    /// 快速提取表达式引用的变量
    pub fn extract_variables(&self, expr: &Expression) -> Vec<String> {
        let analysis = self.analyze(expr);
        analysis.referenced_variables
    }

    /// 递归分析表达式
    fn analyze_recursive(
        &self,
        expr: &Expression,
        result: &mut ExpressionAnalysis,
        depth: u32,
    ) {
        // 更新深度和节点计数
        result.depth = result.depth.max(depth);
        result.node_count += 1;

        match expr {
            Expression::Literal(_) => {
                // 字面量是确定性的
                if self.options.check_complexity {
                    result.complexity_score += 1;
                }
            }

            Expression::Variable(var) => {
                if self.options.extract_variables {
                    result.add_variable(var.clone());
                }
                if self.options.check_complexity {
                    result.complexity_score += 2;
                }
            }

            Expression::Property { object, property } => {
                if self.options.extract_properties {
                    result.add_property(property.clone());
                }
                if self.options.check_complexity {
                    result.complexity_score += 5;
                }
                // 递归分析对象
                self.analyze_recursive(object, result, depth + 1);
            }

            Expression::Binary { left, op, right } => {
                if self.options.check_complexity {
                    // 二元运算基础复杂度
                    result.complexity_score += 2;
                    // 某些操作符增加额外复杂度
                    use crate::core::types::BinaryOperator;
                    match op {
                        BinaryOperator::Like => {
                            // LIKE 操作符是确定性的
                            result.complexity_score += 5;
                        }
                        _ => {}
                    }
                }
                self.analyze_recursive(left, result, depth + 1);
                self.analyze_recursive(right, result, depth + 1);
            }

            Expression::Unary { op: _, operand } => {
                if self.options.check_complexity {
                    result.complexity_score += 1;
                }
                self.analyze_recursive(operand, result, depth + 1);
            }

            Expression::Function { name, args } => {
                if self.options.count_functions {
                    result.add_function(name.clone());
                }

                // 检查是否非确定性
                if self.options.check_deterministic {
                    if NondeterministicChecker::is_nondeterministic(name) {
                        result.is_deterministic = false;
                    }
                }

                // 函数调用增加复杂度
                if self.options.check_complexity {
                    result.complexity_score += 10;
                    // 参数数量也影响复杂度
                    result.complexity_score += args.len() as u32 * 2;
                }

                // 递归分析参数
                for arg in args {
                    self.analyze_recursive(arg, result, depth + 1);
                }
            }

            Expression::Aggregate { func, arg, .. } => {
                result.contains_aggregate = true;
                if self.options.count_functions {
                    result.add_function(format!("{:?}", func));
                }
                if self.options.check_complexity {
                    result.complexity_score += 20;
                }
                self.analyze_recursive(arg, result, depth + 1);
            }

            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                if self.options.check_complexity {
                    // CASE表达式基础复杂度
                    result.complexity_score += 5;
                    // 每个条件增加复杂度
                    result.complexity_score += conditions.len() as u32 * 5;
                }

                // 分析测试表达式
                if let Some(test) = test_expr {
                    self.analyze_recursive(test, result, depth + 1);
                }

                // 分析条件和结果
                for (when, then) in conditions {
                    self.analyze_recursive(when, result, depth + 1);
                    self.analyze_recursive(then, result, depth + 1);
                }

                // 分析默认值
                if let Some(default_expr) = default {
                    self.analyze_recursive(default_expr, result, depth + 1);
                }
            }

            Expression::TypeCast { expression, .. } => {
                if self.options.check_complexity {
                    result.complexity_score += 3;
                }
                self.analyze_recursive(expression, result, depth + 1);
            }

            Expression::Subscript { collection, index } => {
                if self.options.check_complexity {
                    result.complexity_score += 4;
                }
                self.analyze_recursive(collection, result, depth + 1);
                self.analyze_recursive(index, result, depth + 1);
            }

            Expression::List(expressions) => {
                if self.options.check_complexity {
                    result.complexity_score += expressions.len() as u32;
                }
                for expr in expressions {
                    self.analyze_recursive(expr, result, depth + 1);
                }
            }

            Expression::Map(entries) => {
                if self.options.check_complexity {
                    result.complexity_score += entries.len() as u32 * 2;
                }
                for (_, value) in entries {
                    self.analyze_recursive(value, result, depth + 1);
                }
            }

            Expression::ListComprehension { .. } => {
                result.contains_subquery = true;
                if self.options.check_complexity {
                    result.complexity_score += 30;
                }
                // 列表推导式比较复杂，这里简化处理
            }

            Expression::Predicate { func, args } => {
                if self.options.count_functions {
                    result.add_function(func.clone());
                }
                if self.options.check_complexity {
                    result.complexity_score += 15;
                }
                for arg in args {
                    self.analyze_recursive(arg, result, depth + 1);
                }
            }

            Expression::Reduce {
                initial,
                source,
                mapping,
                ..
            } => {
                result.contains_subquery = true;
                if self.options.check_complexity {
                    result.complexity_score += 25;
                }
                self.analyze_recursive(initial, result, depth + 1);
                self.analyze_recursive(source, result, depth + 1);
                self.analyze_recursive(mapping, result, depth + 1);
            }

            Expression::Path(expressions) => {
                if self.options.check_complexity {
                    result.complexity_score += expressions.len() as u32 * 3;
                }
                for expr in expressions {
                    self.analyze_recursive(expr, result, depth + 1);
                }
            }

            Expression::PathBuild(expressions) => {
                if self.options.check_complexity {
                    result.complexity_score += expressions.len() as u32 * 2;
                }
                for expr in expressions {
                    self.analyze_recursive(expr, result, depth + 1);
                }
            }

            Expression::Range { collection, start, end } => {
                if self.options.check_complexity {
                    result.complexity_score += 5;
                }
                self.analyze_recursive(collection, result, depth + 1);
                if let Some(s) = start {
                    self.analyze_recursive(s, result, depth + 1);
                }
                if let Some(e) = end {
                    self.analyze_recursive(e, result, depth + 1);
                }
            }

            Expression::Label(_) | Expression::TagProperty { .. } | Expression::EdgeProperty { .. } => {
                if self.options.check_complexity {
                    result.complexity_score += 3;
                }
            }

            Expression::LabelTagProperty { tag, .. } => {
                if self.options.check_complexity {
                    result.complexity_score += 5;
                }
                self.analyze_recursive(tag, result, depth + 1);
            }

            Expression::Parameter(_) => {
                // 参数是确定性的（在查询执行时绑定）
                if self.options.check_complexity {
                    result.complexity_score += 1;
                }
            }
        }

        // 限制复杂度分数在0-100范围内
        if self.options.check_complexity {
            result.complexity_score = result.complexity_score.min(100);
        }
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

    #[test]
    fn test_expression_analyzer_new() {
        let analyzer = ExpressionAnalyzer::new();
        // 验证创建成功
    }

    #[test]
    fn test_literal_is_deterministic() {
        let analyzer = ExpressionAnalyzer::new();
        let expr = Expression::Literal(Value::Int(42));
        let analysis = analyzer.analyze(&expr);
        assert!(analysis.is_deterministic);
        assert_eq!(analysis.node_count, 1);
    }

    #[test]
    fn test_variable_extraction() {
        let analyzer = ExpressionAnalyzer::new();
        let expr = Expression::Variable("x".to_string());
        let analysis = analyzer.analyze(&expr);
        assert!(analysis.referenced_variables.contains(&"x".to_string()));
    }

    #[test]
    fn test_nondeterministic_function_detection() {
        let analyzer = ExpressionAnalyzer::new();
        let expr = Expression::Function {
            name: "rand".to_string(),
            args: vec![],
        };
        let analysis = analyzer.analyze(&expr);
        assert!(!analysis.is_deterministic);
    }

    #[test]
    fn test_deterministic_function() {
        let analyzer = ExpressionAnalyzer::new();
        let expr = Expression::Function {
            name: "abs".to_string(),
            args: vec![Expression::Literal(Value::Int(-5))],
        };
        let analysis = analyzer.analyze(&expr);
        assert!(analysis.is_deterministic);
    }

    #[test]
    fn test_property_extraction() {
        let analyzer = ExpressionAnalyzer::property_extractor();
        let expr = Expression::Property {
            object: Box::new(Expression::Variable("n".to_string())),
            property: "name".to_string(),
        };
        let analysis = analyzer.analyze(&expr);
        assert!(analysis.referenced_properties.contains(&"name".to_string()));
    }

    #[test]
    fn test_complexity_score() {
        let analyzer = ExpressionAnalyzer::new();
        // 简单表达式
        let simple = Expression::Literal(Value::Int(1));
        let simple_analysis = analyzer.analyze(&simple);
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
        let complex_analysis = analyzer.analyze(&complex);
        assert!(complex_analysis.complexity_score > simple_analysis.complexity_score);
    }
}
