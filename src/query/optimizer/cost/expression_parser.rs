//! 表达式解析器
//!
//! 提供表达式解析功能，用于：
//! - 估算 Unwind 节点的列表大小
//! - 估算 Loop 节点的迭代次数
//! - 解析各种表达式模式
//! - 表达式常量折叠优化

use crate::core::types::{BinaryOperator, Expression, UnaryOperator};
use crate::core::value::Value;
use crate::query::optimizer::cost::config::CostModelConfig;

/// 表达式解析器
///
/// 用于从表达式字符串中解析出有用的信息，如列表大小、迭代次数等
#[derive(Debug, Clone)]
pub struct ExpressionParser {
    /// 配置（内部使用）
    config: CostModelConfig,
}

impl ExpressionParser {
    /// 获取配置
    pub fn config(&self) -> &CostModelConfig {
        &self.config
    }
}

impl ExpressionParser {
    /// 创建新的表达式解析器
    pub fn new(config: CostModelConfig) -> Self {
        Self { config }
    }

    /// 尝试从表达式字符串解析列表大小
    ///
    /// 支持以下模式：
    /// - 数组字面量：[a, b, c] -> 3
    /// - range 函数：range(1, 10) -> 9, range(1, 10, 2) -> 5
    /// - 范围表达式：1..10 -> 9, 0..=5 -> 6
    /// - 集合函数：keys(map), values(map), nodes(path), relationships(path)
    /// - 字符串分割：split(str, ",")（估算）
    /// - 集合操作：collect(set)（估算）
    pub fn parse_list_size(&self, expr: &str) -> Option<f64> {
        let expr = expr.trim();

        // 尝试解析数组字面量 [a, b, c]
        if expr.starts_with('[') && expr.ends_with(']') {
            return self.parse_array_literal(expr);
        }

        // 尝试解析 range(start, end) 或 range(start, end, step)
        if expr.starts_with("range(") && expr.ends_with(')') {
            return self.parse_range_function(expr);
        }

        // 尝试解析范围表达式：1..10 或 0..=5
        if expr.contains("..") {
            return self.parse_range_expression(expr);
        }

        // 尝试解析集合函数：keys(), values(), nodes(), relationships()
        if let Some(size) = self.parse_collection_function(expr) {
            return Some(size);
        }

        // 尝试解析字符串分割函数
        if let Some(size) = self.parse_split_function(expr) {
            return Some(size);
        }

        // 尝试解析 collect 函数（通常用于聚合）
        if expr.starts_with("collect(") {
            // collect 函数的结果大小取决于输入数据，使用保守估计
            return Some(self.config.default_unwind_list_size * 2.0);
        }

        None
    }

    // ==================== 常量折叠优化 ====================

    /// 尝试折叠表达式中的常量
    ///
    /// 对表达式进行递归遍历，将所有可以计算的常量表达式替换为字面量
    /// 例如：1 + 2 -> 3, "hello" + "world" -> "helloworld"
    ///
    /// # 参数
    /// - `expr`: 输入表达式
    ///
    /// # 返回值
    /// 折叠后的表达式
    pub fn fold_constants(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Binary { left, op, right } => {
                let folded_left = self.fold_constants(left);
                let folded_right = self.fold_constants(right);

                // 如果两边都是常量，直接计算结果
                if let (Expression::Literal(l), Expression::Literal(r)) = (&folded_left, &folded_right)
                {
                    if let Some(result) = self.evaluate_binary_op(op, l, r) {
                        return Expression::Literal(result);
                    }
                }

                Expression::Binary {
                    left: Box::new(folded_left),
                    op: op.clone(),
                    right: Box::new(folded_right),
                }
            }
            Expression::Unary { op, operand } => {
                let folded_operand = self.fold_constants(operand);

                // 如果操作数是常量，直接计算结果
                if let Expression::Literal(v) = &folded_operand {
                    if let Some(result) = self.evaluate_unary_op(op, v) {
                        return Expression::Literal(result);
                    }
                }

                Expression::Unary {
                    op: op.clone(),
                    operand: Box::new(folded_operand),
                }
            }
            Expression::Function { name, args } => {
                let folded_args: Vec<Expression> =
                    args.iter().map(|arg| self.fold_constants(arg)).collect();

                // 如果所有参数都是常量，尝试计算函数
                if folded_args.iter().all(|arg| matches!(arg, Expression::Literal(_))) {
                    let arg_values: Vec<&Value> = folded_args
                        .iter()
                        .filter_map(|arg| match arg {
                            Expression::Literal(v) => Some(v),
                            _ => None,
                        })
                        .collect();

                    if let Some(result) = self.evaluate_function(name, &arg_values) {
                        return Expression::Literal(result);
                    }
                }

                Expression::Function {
                    name: name.clone(),
                    args: folded_args,
                }
            }
            Expression::List(items) => {
                let folded_items: Vec<Expression> =
                    items.iter().map(|item| self.fold_constants(item)).collect();
                Expression::List(folded_items)
            }
            Expression::Map(entries) => {
                let folded_entries: Vec<(String, Expression)> = entries
                    .iter()
                    .map(|(k, v)| (k.clone(), self.fold_constants(v)))
                    .collect();
                Expression::Map(folded_entries)
            }
            // 其他表达式类型保持不变
            _ => expr.clone(),
        }
    }

    /// 评估二元操作
    fn evaluate_binary_op(&self, op: &BinaryOperator, left: &Value, right: &Value) -> Option<Value> {
        match op {
            BinaryOperator::Add => self.add_values(left, right),
            BinaryOperator::Subtract => self.subtract_values(left, right),
            BinaryOperator::Multiply => self.multiply_values(left, right),
            BinaryOperator::Divide => self.divide_values(left, right),
            BinaryOperator::Modulo => self.modulo_values(left, right),
            BinaryOperator::Equal => Some(Value::Bool(self.compare_values(left, right) == Some(std::cmp::Ordering::Equal))),
            BinaryOperator::NotEqual => Some(Value::Bool(self.compare_values(left, right) != Some(std::cmp::Ordering::Equal))),
            BinaryOperator::LessThan => Some(Value::Bool(self.compare_values(left, right) == Some(std::cmp::Ordering::Less))),
            BinaryOperator::GreaterThan => Some(Value::Bool(self.compare_values(left, right) == Some(std::cmp::Ordering::Greater))),
            BinaryOperator::LessThanOrEqual => {
                let cmp = self.compare_values(left, right);
                Some(Value::Bool(cmp == Some(std::cmp::Ordering::Less) || cmp == Some(std::cmp::Ordering::Equal)))
            }
            BinaryOperator::GreaterThanOrEqual => {
                let cmp = self.compare_values(left, right);
                Some(Value::Bool(cmp == Some(std::cmp::Ordering::Greater) || cmp == Some(std::cmp::Ordering::Equal)))
            }
            BinaryOperator::And => self.logical_and(left, right),
            BinaryOperator::Or => self.logical_or(left, right),
            BinaryOperator::StringConcat => self.concat_values(left, right),
            _ => None,
        }
    }

    /// 评估一元操作
    fn evaluate_unary_op(&self, op: &UnaryOperator, operand: &Value) -> Option<Value> {
        match op {
            UnaryOperator::Not => match operand {
                Value::Bool(b) => Some(Value::Bool(!b)),
                _ => None,
            },
            UnaryOperator::Minus => match operand {
                Value::Int(i) => Some(Value::Int(-i)),
                Value::Float(f) => Some(Value::Float(-f)),
                _ => None,
            },
            UnaryOperator::IsNull => Some(Value::Bool(operand.is_null())),
            UnaryOperator::IsNotNull => Some(Value::Bool(!operand.is_null())),
            _ => None,
        }
    }

    /// 评估函数
    fn evaluate_function(&self, name: &str, args: &[&Value]) -> Option<Value> {
        let name_lower = name.to_lowercase();

        match name_lower.as_str() {
            "abs" if args.len() == 1 => match args[0] {
                Value::Int(i) => Some(Value::Int(i.abs())),
                Value::Float(f) => Some(Value::Float(f.abs())),
                _ => None,
            },
            "length" | "size" if args.len() == 1 => match args[0] {
                Value::String(s) => Some(Value::Int(s.len() as i64)),
                Value::List(list) => Some(Value::Int(list.len() as i64)),
                _ => None,
            },
            "upper" | "toupper" if args.len() == 1 => match args[0] {
                Value::String(s) => Some(Value::String(s.to_uppercase())),
                _ => None,
            },
            "lower" | "tolower" if args.len() == 1 => match args[0] {
                Value::String(s) => Some(Value::String(s.to_lowercase())),
                _ => None,
            },
            "substring" | "substr" if args.len() >= 2 => {
                match (args[0], args.get(1).copied()) {
                    (Value::String(s), Some(Value::Int(start))) => {
                        let start_idx = if *start >= 0 { *start as usize } else { 0 };
                        let end_idx = if args.len() >= 3 {
                            match args[2] {
                                Value::Int(len) => start_idx + (*len as usize),
                                _ => s.len(),
                            }
                        } else {
                            s.len()
                        };
                        Some(Value::String(s.chars().skip(start_idx).take(end_idx - start_idx).collect()))
                    }
                    _ => None,
                }
            }
            "trim" if args.len() == 1 => match args[0] {
                Value::String(s) => Some(Value::String(s.trim().to_string())),
                _ => None,
            },
            _ => None,
        }
    }

    // ==================== 值操作辅助方法 ====================

    fn add_values(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Some(Value::Int(l + r)),
            (Value::Float(l), Value::Float(r)) => Some(Value::Float(l + r)),
            (Value::Int(l), Value::Float(r)) => Some(Value::Float(*l as f64 + r)),
            (Value::Float(l), Value::Int(r)) => Some(Value::Float(l + *r as f64)),
            (Value::String(l), Value::String(r)) => Some(Value::String(format!("{}{}", l, r))),
            _ => None,
        }
    }

    fn subtract_values(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Some(Value::Int(l - r)),
            (Value::Float(l), Value::Float(r)) => Some(Value::Float(l - r)),
            (Value::Int(l), Value::Float(r)) => Some(Value::Float(*l as f64 - r)),
            (Value::Float(l), Value::Int(r)) => Some(Value::Float(l - *r as f64)),
            _ => None,
        }
    }

    fn multiply_values(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Some(Value::Int(l * r)),
            (Value::Float(l), Value::Float(r)) => Some(Value::Float(l * r)),
            (Value::Int(l), Value::Float(r)) => Some(Value::Float(*l as f64 * r)),
            (Value::Float(l), Value::Int(r)) => Some(Value::Float(l * *r as f64)),
            _ => None,
        }
    }

    fn divide_values(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) if *r != 0 => Some(Value::Int(l / r)),
            (Value::Float(l), Value::Float(r)) if *r != 0.0 => Some(Value::Float(l / r)),
            (Value::Int(l), Value::Float(r)) if *r != 0.0 => Some(Value::Float(*l as f64 / r)),
            (Value::Float(l), Value::Int(r)) if *r != 0 => Some(Value::Float(l / *r as f64)),
            _ => None,
        }
    }

    fn modulo_values(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) if *r != 0 => Some(Value::Int(l % r)),
            _ => None,
        }
    }

    fn compare_values(&self, left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Some(l.cmp(r)),
            (Value::Float(l), Value::Float(r)) => l.partial_cmp(r),
            (Value::Int(l), Value::Float(r)) => (*l as f64).partial_cmp(r),
            (Value::Float(l), Value::Int(r)) => l.partial_cmp(&(*r as f64)),
            (Value::String(l), Value::String(r)) => Some(l.cmp(r)),
            (Value::Bool(l), Value::Bool(r)) => Some(l.cmp(r)),
            _ => None,
        }
    }

    fn logical_and(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => Some(Value::Bool(*l && *r)),
            _ => None,
        }
    }

    fn logical_or(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => Some(Value::Bool(*l || *r)),
            _ => None,
        }
    }

    fn concat_values(&self, left: &Value, right: &Value) -> Option<Value> {
        match (left, right) {
            (Value::String(l), Value::String(r)) => Some(Value::String(format!("{}{}", l, r))),
            _ => None,
        }
    }

    /// 估算 Loop 节点的迭代次数
    ///
    /// 尝试从条件字符串中解析迭代次数，支持以下模式：
    /// - 数字直接量："10" -> 10
    /// - 范围表达式："1..10" 或 "range(1, 10)" -> 9
    /// - 比较表达式："i < 10", "i <= 10" -> 10
    /// - 集合大小："items" -> 使用集合大小估算
    ///
    /// 如果无法解析，则使用配置默认值
    pub fn parse_loop_iterations(&self, condition: &str) -> Option<u32> {
        let condition = condition.trim();

        // 尝试直接解析数字
        if let Ok(num) = condition.parse::<u32>() {
            return Some(num.max(1));
        }

        // 尝试解析范围表达式：1..10 或 1..=10
        if condition.contains("..") {
            return self.parse_range_expression_u32(condition);
        }

        // 尝试解析 range(start, end) 或 range(start, end, step)
        if condition.starts_with("range(") && condition.ends_with(")") {
            return self.parse_range_function_u32(condition);
        }

        // 尝试解析比较表达式：i < 10, i <= 10, count > 5 等
        if let Some(iterations) = self.parse_comparison_expression(condition) {
            return Some(iterations);
        }

        // 尝试解析列表/集合大小：[a,b,c] 或 {a,b,c}
        if condition.starts_with('[') && condition.ends_with(']') {
            return Some(self.parse_collection_size(condition));
        }

        None
    }

    /// 解析数组字面量
    fn parse_array_literal(&self, expr: &str) -> Option<f64> {
        let inner = &expr[1..expr.len() - 1];
        if inner.trim().is_empty() {
            return Some(0.0);
        }
        // 处理嵌套数组和复杂表达式
        let count = self.count_top_level_commas(inner) as f64;
        Some(count)
    }

    /// 计算顶层逗号数量（用于处理嵌套结构）
    fn count_top_level_commas(&self, s: &str) -> usize {
        let mut count = 0;
        let mut depth = 0;
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '[' | '(' | '{' => depth += 1,
                ']' | ')' | '}' => depth -= 1,
                ',' if depth == 0 => count += 1,
                _ => {}
            }
        }

        count + 1 // 逗号数量 + 1 = 元素数量
    }

    /// 解析 range 函数
    fn parse_range_function(&self, expr: &str) -> Option<f64> {
        let args_str = &expr[6..expr.len() - 1];
        let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();

        if args.len() >= 2 {
            let start: i64 = args[0].parse().ok()?;
            let end: i64 = args[1].parse().ok()?;
            let step: i64 = if args.len() >= 3 {
                args[2].parse().ok()?
            } else {
                1
            };

            if step != 0 {
                let count = ((end - start) / step).abs() as f64;
                return Some(count.max(0.0));
            }
        }
        None
    }

    /// 解析 range 函数（返回 u32）
    fn parse_range_function_u32(&self, expr: &str) -> Option<u32> {
        let inner = &expr[6..expr.len() - 1]; // 去掉 "range(" 和 ")"
        let args: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

        if args.len() >= 2 {
            let start: i64 = args[0].parse().ok()?;
            let end: i64 = args[1].parse().ok()?;
            let step: i64 = if args.len() >= 3 {
                args[2].parse().ok()?
            } else {
                1
            };

            if step != 0 && end > start {
                let count = (end - start) / step;
                return Some(count.max(1) as u32);
            }
        }
        None
    }

    /// 解析范围表达式
    fn parse_range_expression(&self, expr: &str) -> Option<f64> {
        if let Some(pos) = expr.find("..") {
            let start_str = expr[..pos].trim();
            let end_part = &expr[pos + 2..];

            let (end_str, inclusive) = if end_part.starts_with('=') {
                (&end_part[1..], true)
            } else {
                (end_part, false)
            };

            if let (Ok(start), Ok(end)) = (start_str.parse::<i64>(), end_str.trim().parse::<i64>())
            {
                if end > start {
                    let count = if inclusive {
                        end - start + 1
                    } else {
                        end - start
                    };
                    return Some(count.max(0) as f64);
                }
            }
        }
        None
    }

    /// 解析范围表达式（返回 u32）
    fn parse_range_expression_u32(&self, expr: &str) -> Option<u32> {
        // 处理 1..10 格式（不包含结束）
        if let Some(pos) = expr.find("..") {
            let start_str = expr[..pos].trim();
            let end_part = &expr[pos + 2..];

            // 检查是否包含等号（1..=10 表示包含结束）
            let (end_str, inclusive) = if end_part.starts_with('=') {
                (&end_part[1..], true)
            } else {
                (end_part, false)
            };

            if let (Ok(start), Ok(end)) = (start_str.parse::<i64>(), end_str.trim().parse::<i64>())
            {
                if end > start {
                    let count = if inclusive {
                        end - start + 1
                    } else {
                        end - start
                    };
                    return Some(count.max(1) as u32);
                }
            }
        }
        None
    }

    /// 解析比较表达式（如 "i < 10", "count <= 100"）
    fn parse_comparison_expression(&self, expr: &str) -> Option<u32> {
        // 匹配模式：var < num, var <= num, var > num, var >= num
        let operators = [("<", 0u32), ("<=", 0u32), (">", 0u32), (">=", 0u32)];

        for (op, _) in &operators {
            if let Some(pos) = expr.find(op) {
                let right_side = &expr[pos + op.len()..];
                if let Ok(num) = right_side.trim().parse::<u32>() {
                    // 对于 < 操作符，实际迭代次数是 num（如果 num > 0）
                    let iterations = if *op == "<" && num > 0 {
                        num
                    } else if *op == "<=" {
                        num
                    } else if *op == ">" {
                        // 无法确定起始值，使用保守估计
                        num + 10
                    } else if *op == ">=" {
                        num + 10
                    } else {
                        num
                    };
                    return Some(iterations.max(1));
                }
            }
        }
        None
    }

    /// 解析集合函数（keys, values, nodes, relationships）
    fn parse_collection_function(&self, expr: &str) -> Option<f64> {
        let expr_lower = expr.to_lowercase();

        // keys(map) 或 map.keys() - 返回 map 的键列表
        if expr_lower.starts_with("keys(") || expr_lower.contains(".keys()") {
            // 无法确定 map 大小，使用默认估计
            return Some(self.config.default_unwind_list_size);
        }

        // values(map) 或 map.values()
        if expr_lower.starts_with("values(") || expr_lower.contains(".values()") {
            return Some(self.config.default_unwind_list_size);
        }

        // nodes(path) - 返回路径中的节点列表
        if expr_lower.starts_with("nodes(") {
            // 路径长度未知，使用默认估计
            return Some(self.config.default_unwind_list_size);
        }

        // relationships(path) 或 rels(path) - 返回路径中的关系列表
        if expr_lower.starts_with("relationships(") || expr_lower.starts_with("rels(") {
            return Some(self.config.default_unwind_list_size - 1.0); // 关系数通常比节点数少1
        }

        // labels(node) - 返回标签列表（通常很小）
        if expr_lower.starts_with("labels(") {
            return Some(1.0); // 通常一个节点只有1-2个标签
        }

        None
    }

    /// 解析字符串分割函数
    fn parse_split_function(&self, expr: &str) -> Option<f64> {
        let expr_lower = expr.to_lowercase();

        // split(string, delimiter) 或 string.split(delimiter)
        if expr_lower.starts_with("split(") || expr_lower.contains(".split(") {
            // 无法确定分割后的数量，基于字符串长度估算
            // 假设平均每个元素长度为5个字符
            return Some(self.config.default_unwind_list_size);
        }

        None
    }

    /// 解析集合大小（如 "[a, b, c]"）
    fn parse_collection_size(&self, expr: &str) -> u32 {
        let inner = &expr[1..expr.len() - 1];
        if inner.trim().is_empty() {
            return 0;
        }
        // 简单计算逗号数量 + 1
        let count = inner.split(',').count() as u32;
        count.max(1)
    }
}

impl Default for ExpressionParser {
    fn default() -> Self {
        Self::new(CostModelConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_array_literal() {
        let parser = ExpressionParser::default();
        assert_eq!(parser.parse_list_size("[1, 2, 3]"), Some(3.0));
        assert_eq!(parser.parse_list_size("[]"), Some(0.0));
        assert_eq!(parser.parse_list_size("[a]"), Some(1.0));
    }

    #[test]
    fn test_parse_range_function() {
        let parser = ExpressionParser::default();
        assert_eq!(parser.parse_list_size("range(1, 10)"), Some(9.0));
        assert_eq!(parser.parse_list_size("range(1, 10, 2)"), Some(4.0));
    }

    #[test]
    fn test_parse_range_expression() {
        let parser = ExpressionParser::default();
        assert_eq!(parser.parse_list_size("1..10"), Some(9.0));
        assert_eq!(parser.parse_list_size("0..=5"), Some(6.0));
    }

    #[test]
    fn test_parse_loop_iterations_number() {
        let parser = ExpressionParser::default();
        assert_eq!(parser.parse_loop_iterations("10"), Some(10));
        assert_eq!(parser.parse_loop_iterations("0"), Some(1)); // 至少1次
    }

    #[test]
    fn test_parse_loop_iterations_comparison() {
        let parser = ExpressionParser::default();
        assert_eq!(parser.parse_loop_iterations("i < 10"), Some(10));
        assert_eq!(parser.parse_loop_iterations("i <= 5"), Some(5));
    }
}
