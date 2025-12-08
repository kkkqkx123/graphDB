pub mod value;
pub mod base;
pub mod operations;
pub mod function_call;
pub mod container;
pub mod property_access;
pub mod eval;
pub mod eval_context;
pub mod tests;
pub mod agg;

// Import the expression utilities functionality
mod expression_utils {
    use std::collections::{HashMap, HashSet};
    use crate::expressions::value::{Expression, ExpressionKind};
    use crate::core::Value;

    // 常量定义
    pub const MAX_EXPRESSION_DEPTH: i32 = 1000; // 对应原代码中的FLAGS_max_expression_depth

    /// 表达式类型检查相关函数
    pub mod expr_check {
        use super::*;

        /// 检查表达式类型是否为预期类型之一
        pub fn is_kind_of(expr: &Expression, expected: &HashSet<ExpressionKind>) -> bool {
            expected.contains(&expr.kind())
        }

        /// 检查表达式是否为属性表达式
        pub fn is_property_expr(expr: &Expression) -> bool {
            matches!(expr.kind(),
                ExpressionKind::Property
            )
        }

        /// 检查表达式是否可求值
        pub fn is_evaluable_expr(expr: &Expression) -> bool {
            // 这里实现表达式可求值性检查逻辑
            // 例如检查是否包含参数表达式等
            true // 简化实现
        }

        /// 检查表达式深度是否超过最大限制
        pub fn check_expr_depth(expr: &Expression) -> bool {
            check_expr_depth_recursive(expr, 0) <= MAX_EXPRESSION_DEPTH as usize
        }

        fn check_expr_depth_recursive(expr: &Expression, current_depth: usize) -> usize {
            if current_depth > MAX_EXPRESSION_DEPTH as usize {
                return current_depth; // 提前终止
            }

            let mut max_depth = current_depth;
            for child in expr.children() {
                let child_depth = check_expr_depth_recursive(child, current_depth + 1);
                max_depth = max_depth.max(child_depth);
            }
            max_depth
        }
    }

    /// 表达式重写相关函数
    pub mod expr_rewrite {
        use super::*;
        use std::collections::HashMap;

        // AliasType 枚举复制过来
        #[derive(Debug, Clone, PartialEq)]
        pub enum AliasType {
            Vertex,
            Edge,
            Path,
            Variable,
        }

        /// 将Attribute表达式重写为LabelTagProp
        pub fn rewrite_attr_to_label_tag_prop(
            expr: &Expression,
            alias_type_map: &HashMap<String, AliasType>
        ) -> Expression {
            // 实现Attribute到LabelTagProp的重写逻辑
            expr.clone() // 简化实现
        }

        /// 将边属性函数重写为标签属性 (如 rank(e) -> e._rank)
        pub fn rewrite_edge_prop_func_to_label_attr(
            expr: &Expression,
            alias_type_map: &HashMap<String, AliasType>
        ) -> Expression {
            // 实现边属性函数到标签属性的重写逻辑
            expr.clone() // 简化实现
        }

        /// 将参数表达式重写为常量表达式
        pub fn rewrite_parameter(expr: &Expression, _params: &HashMap<String, Value>) -> Expression {
            // 实现参数到常量的重写逻辑
            expr.clone() // 简化实现
        }

        /// 简化逻辑表达式
        pub fn simplify_logical_expr(logical_expr: &Expression) -> Expression {
            // 实现逻辑表达式简化 (A and true => A, A or false => A等)
            logical_expr.clone() // 简化实现
        }

        /// 常量折叠
        pub fn fold_constant_expr(expr: &Expression) -> Result<Expression, String> {
            // 实现常量折叠逻辑 (v.age > 40 + 1 => v.age > 41)
            Ok(expr.clone()) // 简化实现
        }
    }

    /// 表达式转换相关函数
    pub mod expr_transform {
        use super::*;

        /// 表达式过滤器转换
        pub fn filter_transform(expr: &Expression) -> Result<Expression, String> {
            // 实现过滤器转换逻辑
            // 1. 重写关系表达式，使常量在右侧
            // 2. 常量折叠
            // 3. 减少非表达式
            Ok(expr.clone()) // 简化实现
        }

        /// 拉平内部逻辑AND表达式
        pub fn flatten_inner_logical_and_expr(expr: &Expression) -> Expression {
            // 实现AND表达式拉平逻辑
            expr.clone() // 简化实现
        }

        /// 拉平内部逻辑OR表达式
        pub fn flatten_inner_logical_or_expr(expr: &Expression) -> Expression {
            // 实现OR表达式拉平逻辑
            expr.clone() // 简化实现
        }

        /// 拉平内部逻辑表达式
        pub fn flatten_inner_logical_expr(expr: &Expression) -> Expression {
            // 先执行AND表达式拉平，再执行OR表达式拉平
            let expr_after_and = flatten_inner_logical_and_expr(expr);
            flatten_inner_logical_or_expr(&expr_after_and)
        }
    }

    /// 表达式收集相关函数
    pub mod expr_collect {
        use super::*;

        /// 递归查找表达式中匹配预期类型的子表达式
        pub fn find_any<'a>(expr: &'a Expression, expected: &HashSet<ExpressionKind>) -> Option<&'a Expression> {
            if expected.contains(&expr.kind()) {
                return Some(expr);
            }

            for child in expr.children() {
                if let Some(found) = find_any(child, expected) {
                    return Some(found);
                }
            }
            None
        }

        /// 递归收集表达式中所有匹配预期类型的子表达式
        pub fn collect_all<'a>(expr: &'a Expression, expected: &HashSet<ExpressionKind>) -> Vec<&'a Expression> {
            let mut result = Vec::new();
            collect_all_recursive(expr, expected, &mut result);
            result
        }

        fn collect_all_recursive<'a>(
            expr: &'a Expression,
            expected: &HashSet<ExpressionKind>,
            result: &mut Vec<&'a Expression>
        ) {
            if expected.contains(&expr.kind()) {
                result.push(expr);
            }

            for child in expr.children() {
                collect_all_recursive(child, expected, result);
            }
        }
    }
}

pub use value::*;
pub use base::*;
pub use operations::*;
pub use function_call::*;
pub use container::*;
pub use property_access::*;
pub use eval::*;
pub use eval_context::*;
pub use agg::*;
pub use expression_utils::*;

// Explicitly re-export core items for external access
pub use value::{Expression, ExpressionKind};