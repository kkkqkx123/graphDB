//! 重写规则宏定义
//!
//! 提供声明式宏用于简化重写规则的定义，减少样板代码。

// ==================== 基础规则宏 ====================

/// 定义基础重写规则
///
/// 自动生成规则结构体、Default实现、new()方法和RewriteRule trait实现
///
/// # 示例
/// ```rust
/// define_rewrite_rule! {
///     name: MyCustomRule,
///     pattern: Pattern::new_with_name("Filter"),
///     apply: |ctx, node| {
///         // 规则逻辑
///         Ok(None)
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_rewrite_rule {
    (
        $(#[$meta:meta])*
        name: $name:ident,
        pattern: $pattern:expr,
        apply: $apply_closure:expr
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name;

        impl $name {
            /// 创建规则实例
            pub fn new() -> Self {
                Self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::query::planner::rewrite::rule::RewriteRule for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn pattern(&self) -> $crate::query::planner::rewrite::pattern::Pattern {
                $pattern
            }

            fn apply(
                &self,
                ctx: &mut $crate::query::planner::rewrite::context::RewriteContext,
                node: &$crate::query::planner::plan::PlanNodeEnum,
            ) -> $crate::query::planner::rewrite::result::RewriteResult<Option<$crate::query::planner::rewrite::result::TransformResult>> {
                let apply_fn: fn(&mut _, &_) -> _ = $apply_closure;
                apply_fn(ctx, node)
            }
        }
    };
}

/// 定义带节点类型匹配的规则
///
/// 自动处理节点类型匹配和解包
///
/// # 示例
/// ```rust
/// define_typed_rewrite_rule! {
///     name: EliminateFilterRule,
///     pattern: Pattern::new_with_name("Filter"),
///     node_type: Filter,
///     apply: |ctx, filter_node| {
///         // filter_node 已经是解包后的 FilterNode 类型
///         Ok(None)
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_typed_rewrite_rule {
    (
        $(#[$meta:meta])*
        name: $name:ident,
        pattern: $pattern:expr,
        node_type: $node_type:ident,
        apply: $apply_closure:expr
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name;

        impl $name {
            /// 创建规则实例
            pub fn new() -> Self {
                Self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::query::planner::rewrite::rule::RewriteRule for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn pattern(&self) -> $crate::query::planner::rewrite::pattern::Pattern {
                $pattern
            }

            fn apply(
                &self,
                ctx: &mut $crate::query::planner::rewrite::context::RewriteContext,
                node: &$crate::query::planner::plan::PlanNodeEnum,
            ) -> $crate::query::planner::rewrite::result::RewriteResult<Option<$crate::query::planner::rewrite::result::TransformResult>> {
                use $crate::query::planner::plan::PlanNodeEnum;
                use $crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

                let typed_node = match node {
                    PlanNodeEnum::$node_type(n) => n,
                    _ => return Ok(None),
                };

                let apply_fn: fn(&mut _, &_) -> _ = $apply_closure;
                apply_fn(ctx, typed_node)
            }
        }
    };
}

// ==================== 下推规则宏 ====================

/// 定义下推规则
///
/// 自动生成 RewriteRule 和 PushDownRule trait 实现
///
/// # 示例
/// ```rust
/// define_rewrite_pushdown_rule! {
///     name: PushFilterDownGetNbrsRule,
///     parent_node: Filter,
///     child_node: GetNeighbors,
///     apply: |ctx, filter_node, get_neighbors_node| {
///         // 下推逻辑
///         Ok(None)
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_rewrite_pushdown_rule {
    (
        $(#[$meta:meta])*
        name: $name:ident,
        parent_node: $parent_type:ident,
        child_node: $child_type:ident,
        apply: $apply_closure:expr
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name;

        impl $name {
            /// 创建规则实例
            pub fn new() -> Self {
                Self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::query::planner::rewrite::rule::RewriteRule for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn pattern(&self) -> $crate::query::planner::rewrite::pattern::Pattern {
                $crate::query::planner::rewrite::pattern::Pattern::new_with_name(stringify!($parent_type))
                    .with_dependency_name(stringify!($child_type))
            }

            fn apply(
                &self,
                ctx: &mut $crate::query::planner::rewrite::context::RewriteContext,
                node: &$crate::query::planner::plan::PlanNodeEnum,
            ) -> $crate::query::planner::rewrite::result::RewriteResult<Option<$crate::query::planner::rewrite::result::TransformResult>> {
                use $crate::query::planner::plan::PlanNodeEnum;
                use $crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

                let parent_node = match node {
                    PlanNodeEnum::$parent_type(n) => n,
                    _ => return Ok(None),
                };

                let input = parent_node.input();
                let child_node = match input {
                    PlanNodeEnum::$child_type(n) => n,
                    _ => return Ok(None),
                };

                let apply_fn: fn(&mut _, &_, &_) -> _ = $apply_closure;
                apply_fn(ctx, parent_node, child_node)
            }
        }

        impl $crate::query::planner::rewrite::rule::PushDownRule for $name {
            fn can_push_down(
                &self,
                node: &$crate::query::planner::plan::PlanNodeEnum,
                target: &$crate::query::planner::plan::PlanNodeEnum,
            ) -> bool {
                use $crate::query::planner::plan::PlanNodeEnum;
                matches!(
                    (node, target),
                    (PlanNodeEnum::$parent_type(_), PlanNodeEnum::$child_type(_))
                )
            }

            fn push_down(
                &self,
                ctx: &mut $crate::query::planner::rewrite::context::RewriteContext,
                node: &$crate::query::planner::plan::PlanNodeEnum,
                _target: &$crate::query::planner::plan::PlanNodeEnum,
            ) -> $crate::query::planner::rewrite::result::RewriteResult<Option<$crate::query::planner::rewrite::result::TransformResult>> {
                use $crate::query::planner::rewrite::rule::RewriteRule;
                self.apply(ctx, node)
            }
        }
    };
}

// ==================== 消除规则宏 ====================

/// 定义消除规则
///
/// 自动生成 RewriteRule 和 EliminationRule trait 实现
///
/// # 示例
/// ```rust
/// define_rewrite_elimination_rule! {
///     name: EliminateFilterRule,
///     node_type: Filter,
///     can_eliminate: |filter_node| {
///         is_expression_tautology(filter_node.condition())
///     },
///     eliminate: |ctx, filter_node| {
///         let mut result = TransformResult::new();
///         result.erase_curr = true;
///         result.add_new_node(filter_node.input().clone());
///         Ok(Some(result))
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_rewrite_elimination_rule {
    (
        $(#[$meta:meta])*
        name: $name:ident,
        node_type: $node_type:ident,
        can_eliminate: $can_eliminate_closure:expr,
        eliminate: $eliminate_closure:expr
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name;

        impl $name {
            /// 创建规则实例
            pub fn new() -> Self {
                Self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::query::planner::rewrite::rule::RewriteRule for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn pattern(&self) -> $crate::query::planner::rewrite::pattern::Pattern {
                $crate::query::planner::rewrite::pattern::Pattern::new_with_name(stringify!($node_type))
            }

            fn apply(
                &self,
                ctx: &mut $crate::query::planner::rewrite::context::RewriteContext,
                node: &$crate::query::planner::plan::PlanNodeEnum,
            ) -> $crate::query::planner::rewrite::result::RewriteResult<Option<$crate::query::planner::rewrite::result::TransformResult>> {
                use $crate::query::planner::plan::PlanNodeEnum;

                let typed_node = match node {
                    PlanNodeEnum::$node_type(n) => n,
                    _ => return Ok(None),
                };

                let can_eliminate_fn: fn(&_) -> _ = $can_eliminate_closure;
                if !can_eliminate_fn(typed_node) {
                    return Ok(None);
                }

                let eliminate_fn: fn(&mut _, &_) -> _ = $eliminate_closure;
                eliminate_fn(ctx, typed_node)
            }
        }

        impl $crate::query::planner::rewrite::rule::EliminationRule for $name {
            fn can_eliminate(&self, node: &$crate::query::planner::plan::PlanNodeEnum) -> bool {
                use $crate::query::planner::plan::PlanNodeEnum;

                let typed_node = match node {
                    PlanNodeEnum::$node_type(n) => n,
                    _ => return false,
                };

                let can_eliminate_fn: fn(&_) -> _ = $can_eliminate_closure;
                can_eliminate_fn(typed_node)
            }

            fn eliminate(
                &self,
                ctx: &mut $crate::query::planner::rewrite::context::RewriteContext,
                node: &$crate::query::planner::plan::PlanNodeEnum,
            ) -> $crate::query::planner::rewrite::result::RewriteResult<Option<$crate::query::planner::rewrite::result::TransformResult>> {
                self.apply(ctx, node)
            }
        }
    };
}

/// 定义简单消除规则（仅删除当前节点）
///
/// # 示例
/// ```rust
/// define_simple_rewrite_elimination_rule! {
///     name: EliminateTrueFilterRule,
///     node_type: Filter,
///     condition: |filter_node: &FilterNode| is_expression_tautology(filter_node.condition())
/// }
/// ```
#[macro_export]
macro_rules! define_simple_rewrite_elimination_rule {
    (
        $(#[$meta:meta])*
        name: $name:ident,
        node_type: $node_type:ident,
        condition: $condition_closure:expr
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name;

        impl $name {
            /// 创建规则实例
            pub fn new() -> Self {
                Self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::query::planner::rewrite::rule::RewriteRule for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn pattern(&self) -> $crate::query::planner::rewrite::pattern::Pattern {
                $crate::query::planner::rewrite::pattern::Pattern::new_with_name(stringify!($node_type))
            }

            fn apply(
                &self,
                _ctx: &mut $crate::query::planner::rewrite::context::RewriteContext,
                node: &$crate::query::planner::plan::PlanNodeEnum,
            ) -> $crate::query::planner::rewrite::result::RewriteResult<Option<$crate::query::planner::rewrite::result::TransformResult>> {
                use $crate::query::planner::plan::PlanNodeEnum;
                use $crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

                let typed_node = match node {
                    PlanNodeEnum::$node_type(n) => n,
                    _ => return Ok(None),
                };

                let condition_fn: fn(&_) -> _ = $condition_closure;
                if !condition_fn(typed_node) {
                    return Ok(None);
                }

                let mut result = $crate::query::planner::rewrite::result::TransformResult::new();
                result.erase_curr = true;
                result.add_new_node(typed_node.input().clone());
                Ok(Some(result))
            }
        }

        impl $crate::query::planner::rewrite::rule::EliminationRule for $name {
            fn can_eliminate(&self, node: &$crate::query::planner::plan::PlanNodeEnum) -> bool {
                use $crate::query::planner::plan::PlanNodeEnum;

                let typed_node = match node {
                    PlanNodeEnum::$node_type(n) => n,
                    _ => return false,
                };

                let condition_fn: fn(&_) -> _ = $condition_closure;
                condition_fn(typed_node)
            }

            fn eliminate(
                &self,
                ctx: &mut $crate::query::planner::rewrite::context::RewriteContext,
                node: &$crate::query::planner::plan::PlanNodeEnum,
            ) -> $crate::query::planner::rewrite::result::RewriteResult<Option<$crate::query::planner::rewrite::result::TransformResult>> {
                use $crate::query::planner::rewrite::rule::RewriteRule;
                self.apply(ctx, node)
            }
        }
    };
}

// ==================== 合并规则宏 ====================

/// 定义合并规则
///
/// 自动生成 RewriteRule 和 MergeRule trait 实现
///
/// # 示例
/// ```rust
/// define_rewrite_merge_rule! {
///     name: CombineFilterRule,
///     parent_node: Filter,
///     child_node: Filter,
///     can_merge: |parent, child| true,
///     merge: |ctx, parent, child| {
///         // 合并逻辑
///         Ok(None)
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_rewrite_merge_rule {
    (
        $(#[$meta:meta])*
        name: $name:ident,
        parent_node: $parent_type:ident,
        child_node: $child_type:ident,
        can_merge: $can_merge_closure:expr,
        merge: $merge_closure:expr
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name;

        impl $name {
            /// 创建规则实例
            pub fn new() -> Self {
                Self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::query::planner::rewrite::rule::RewriteRule for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn pattern(&self) -> $crate::query::planner::rewrite::pattern::Pattern {
                $crate::query::planner::rewrite::pattern::Pattern::new_with_name(stringify!($parent_type))
                    .with_dependency_name(stringify!($child_type))
            }

            fn apply(
                &self,
                ctx: &mut $crate::query::planner::rewrite::context::RewriteContext,
                node: &$crate::query::planner::plan::PlanNodeEnum,
            ) -> $crate::query::planner::rewrite::result::RewriteResult<Option<$crate::query::planner::rewrite::result::TransformResult>> {
                use $crate::query::planner::plan::PlanNodeEnum;
                use $crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

                let parent_node = match node {
                    PlanNodeEnum::$parent_type(n) => n,
                    _ => return Ok(None),
                };

                let input = parent_node.input();
                let child_node = match input {
                    PlanNodeEnum::$child_type(n) => n,
                    _ => return Ok(None),
                };

                let can_merge_fn: fn(&_, &_) -> _ = $can_merge_closure;
                if !can_merge_fn(parent_node, child_node) {
                    return Ok(None);
                }

                let merge_fn: fn(&mut _, &_, &_) -> _ = $merge_closure;
                merge_fn(ctx, parent_node, child_node)
            }
        }

        impl $crate::query::planner::rewrite::rule::MergeRule for $name {
            fn can_merge(
                &self,
                parent: &$crate::query::planner::plan::PlanNodeEnum,
                child: &$crate::query::planner::plan::PlanNodeEnum,
            ) -> bool {
                use $crate::query::planner::plan::PlanNodeEnum;

                let parent_node = match parent {
                    PlanNodeEnum::$parent_type(n) => n,
                    _ => return false,
                };

                let child_node = match child {
                    PlanNodeEnum::$child_type(n) => n,
                    _ => return false,
                };

                let can_merge_fn: fn(&_, &_) -> _ = $can_merge_closure;
                can_merge_fn(parent_node, child_node)
            }

            fn create_merged_node(
                &self,
                ctx: &mut $crate::query::planner::rewrite::context::RewriteContext,
                parent: &$crate::query::planner::plan::PlanNodeEnum,
                child: &$crate::query::planner::plan::PlanNodeEnum,
            ) -> $crate::query::planner::rewrite::result::RewriteResult<Option<$crate::query::planner::rewrite::result::TransformResult>> {
                use $crate::query::planner::plan::PlanNodeEnum;

                let parent_node = match parent {
                    PlanNodeEnum::$parent_type(n) => n,
                    _ => return Ok(None),
                };

                let child_node = match child {
                    PlanNodeEnum::$child_type(n) => n,
                    _ => return Ok(None),
                };

                let merge_fn: fn(&mut _, &_, &_) -> _ = $merge_closure;
                merge_fn(ctx, parent_node, child_node)
            }
        }
    };
}

// ==================== 规则注册宏 ====================

/// 定义规则注册表
///
/// 自动生成 RuleRegistry 的 default 实现，包含所有规则的注册
///
/// # 示例
/// ```rust
/// define_rewrite_rule_registry! {
///     elimination: [
///         EliminateFilter,
///         RemoveNoopProject,
///     ],
///     merge: [
///         CombineFilter,
///         CollapseProject,
///     ],
/// }
/// ```
#[macro_export]
macro_rules! define_rewrite_rule_registry {
    (
        $(
            $category:ident: [
                $($rule_name:ident),* $(,)?
            ]
        ),* $(,)?
    ) => {
        impl Default for RuleRegistry {
            fn default() -> Self {
                let mut registry = Self::new();
                $(
                    $(
                        registry.add(RewriteRule::$rule_name(
                            $crate::query::planner::rewrite::$category::paste! {[<$rule_name Rule>]}::new()
                        ));
                    )*
                )*
                registry
            }
        }
    };
}

// 重新导出所有宏
pub use crate::define_rewrite_rule;
pub use crate::define_typed_rewrite_rule;
pub use crate::define_rewrite_pushdown_rule;
pub use crate::define_rewrite_elimination_rule;
pub use crate::define_simple_rewrite_elimination_rule;
pub use crate::define_rewrite_merge_rule;
pub use crate::define_rewrite_rule_registry;
