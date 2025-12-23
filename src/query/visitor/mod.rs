//! 表达式访问器模块
//! 对应 NebulaGraph src/graph/visitor 的功能
//! 用于表达式分析和转换访问器

use crate::core::visitor::VisitorConfig;

mod deduce_props_visitor;
mod deduce_type_visitor;
mod evaluable_expr_visitor;
mod extract_filter_expr_visitor;
mod find_visitor;
mod fold_constant_expr_visitor;

pub use deduce_props_visitor::{DeducePropsVisitor, ExpressionProps};
pub use deduce_type_visitor::{DeduceTypeVisitor, TypeDeductionError};
pub use evaluable_expr_visitor::EvaluableExprVisitor;
pub use extract_filter_expr_visitor::ExtractFilterExprVisitor;
pub use find_visitor::{ExpressionType, FindVisitor};
pub use fold_constant_expr_visitor::FoldConstantExprVisitor;

/// 查询访问器基础trait
/// 提供查询特定的访问功能
pub trait QueryVisitor {
    /// 查询结果类型
    type QueryResult;

    /// 获取查询结果
    fn get_result(&self) -> Self::QueryResult;

    /// 重置访问器状态
    fn reset(&mut self);

    /// 检查访问是否成功
    fn is_success(&self) -> bool;
}

/// 查询访问器构建器
/// 用于构建不同类型的查询访问器
pub struct QueryVisitorBuilder {
    config: crate::core::visitor::VisitorConfig,
}

impl QueryVisitorBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: crate::core::visitor::VisitorConfig::new(),
        }
    }

    /// 设置最大访问深度
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.config = self.config.with_max_depth(depth);
        self
    }

    /// 设置是否启用缓存
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.config = self.config.with_cache(enable);
        self
    }

    /// 构建属性推导访问器
    pub fn build_deduce_props(self) -> DeducePropsVisitor {
        DeducePropsVisitor::with_config(self.config)
    }

    /// 构建类型推导访问器
    pub fn build_deduce_type<'a, S: crate::storage::StorageEngine>(
        self,
        storage: &'a S,
        validate_context: &'a crate::query::validator::ValidationContext,
        inputs: Vec<(String, crate::core::ValueTypeDef)>,
        space: String,
    ) -> DeduceTypeVisitor<'a, S> {
        DeduceTypeVisitor::new(storage, validate_context, inputs, space)
    }

    /// 构建可求值表达式访问器
    pub fn build_evaluable(self) -> EvaluableExprVisitor {
        EvaluableExprVisitor::new()
    }

    /// 构建过滤表达式提取访问器
    pub fn build_extract_filter(self, top_level_only: bool) -> ExtractFilterExprVisitor {
        ExtractFilterExprVisitor::new(top_level_only)
    }

    /// 构建查找访问器
    pub fn build_find(self) -> FindVisitor {
        FindVisitor::new()
    }

    /// 构建常量折叠访问器
    pub fn build_fold_constant(
        self,
        parameters: std::collections::HashMap<String, crate::core::Value>,
    ) -> FoldConstantExprVisitor {
        FoldConstantExprVisitor::new(parameters)
    }
}

impl Default for QueryVisitorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷宏：创建查询访问器
#[macro_export]
macro_rules! create_query_visitor {
    (deduce_props) => {
        QueryVisitorBuilder::new().build_deduce_props()
    };
    (deduce_type, $storage:expr, $validate_context:expr, $inputs:expr, $space:expr) => {
        QueryVisitorBuilder::new().build_deduce_type($storage, $validate_context, $inputs, $space)
    };
    (evaluable) => {
        QueryVisitorBuilder::new().build_evaluable()
    };
    (extract_filter, $top_level_only:expr) => {
        QueryVisitorBuilder::new().build_extract_filter($top_level_only)
    };
    (find) => {
        QueryVisitorBuilder::new().build_find()
    };
    (fold_constant, $parameters:expr) => {
        QueryVisitorBuilder::new().build_fold_constant($parameters)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_visitor_builder() {
        let visitor = QueryVisitorBuilder::new()
            .with_max_depth(10)
            .with_cache(true)
            .build_deduce_props();

        assert!(visitor.is_success());
    }

    #[test]
    fn test_create_query_visitor_macro() {
        let mut visitor = create_query_visitor!(deduce_props);
        assert!(visitor.is_success());

        let visitor = create_query_visitor!(evaluable);
        assert!(visitor.is_success());

        let visitor = create_query_visitor!(find);
        assert!(visitor.is_success());
    }
}
