//! 表达式解析器
//!
//! 提供表达式解析功能，用于：
//! - 估算 Unwind 节点的列表大小
//! - 估算 Loop 节点的迭代次数
//! - 解析各种表达式模式

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

            if let (Ok(start), Ok(end)) = (start_str.parse::<i64>(), end_str.trim().parse::<i64>()) {
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

            if let (Ok(start), Ok(end)) = (start_str.parse::<i64>(), end_str.trim().parse::<i64>()) {
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
