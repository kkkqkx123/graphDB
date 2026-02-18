//! 优化规则宏定义
//!
//! 提供宏来简化优化规则的定义，减少样板代码

/// 定义 LIMIT 下推规则的宏
///
/// # 示例
/// ```
/// define_limit_pushdown_rule! {
///     pub struct PushLimitDownGetEdgesRule {
///         target: GetEdges,
///         target_check: is_get_edges,
///         target_as: as_get_edges,
///         enum_variant: GetEdges,
///         pattern: PatternBuilder::with_dependency("Limit", "GetEdges")
///     }
/// }
#[macro_export]
macro_rules! define_limit_pushdown_rule {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            target: $target_type:ident,
            target_check: $target_check:ident,
            target_as: $target_as:ident,
            enum_variant: $enum_variant:ident,
            pattern: $pattern:expr
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name;

        impl $crate::query::optimizer::plan::OptRule for $name {
            fn name(&self) -> &str {
                stringify!($name)
            }

            fn apply(
                &self,
                ctx: &mut $crate::query::optimizer::plan::OptContext,
                group_node: &::std::rc::Rc<::std::cell::RefCell<$crate::query::optimizer::plan::OptGroupNode>>,
            ) -> Result<Option<$crate::query::optimizer::plan::TransformResult>, $crate::query::optimizer::OptimizerError> {
                let node_ref = group_node.borrow();
                
                if !node_ref.plan_node.is_limit() {
                    return Ok(None);
                }

                if node_ref.dependencies.len() != 1 {
                    return Ok(None);
                }

                let child_id = node_ref.dependencies[0];
                let child_node = match ctx.find_group_node_by_id(child_id) {
                    Some(node) => node,
                    None => return Ok(None),
                };

                let child_ref = child_node.borrow();
                
                if !child_ref.plan_node.$target_check() {
                    return Ok(None);
                }

                let limit_value = match node_ref.plan_node.as_limit() {
                    Some(limit) => limit.count(),
                    None => return Ok(None),
                };

                if let Some(target_node) = child_ref.plan_node.$target_as() {
                    let mut new_target = target_node.clone();
                    new_target.set_limit(limit_value);
                    
                    if let Some(output_var) = node_ref.plan_node.output_var() {
                        new_target.set_output_var(output_var.clone());
                    }

                    let mut new_node = child_ref.clone();
                    new_node.plan_node = $crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::$enum_variant(new_target);

                    let mut result = $crate::query::optimizer::plan::TransformResult::new();
                    result.add_new_group_node(::std::rc::Rc::new(::std::cell::RefCell::new(new_node)));
                    return Ok(Some(result));
                }

                Ok(None)
            }

            fn pattern(&self) -> $crate::query::optimizer::plan::Pattern {
                $pattern
            }
        }

        impl $crate::query::optimizer::rule_traits::BaseOptRule for $name {}
    };
}

/// 定义合并规则的宏
///
/// # 示例
/// ```
/// define_merge_rule! {
///     pub struct MergeGetVerticesAndDedupRule {
///         parent: GetVertices,
///         parent_check: is_get_vertices,
///         child: Dedup,
///         child_check: is_dedup,
///         pattern: PatternBuilder::with_dependency("GetVertices", "Dedup")
///     }
/// }
#[macro_export]
macro_rules! define_merge_rule {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            parent: $parent_type:ident,
            parent_check: $parent_check:ident,
            child: $child_type:ident,
            child_check: $child_check:ident,
            pattern: $pattern:expr
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name;

        impl $crate::query::optimizer::plan::OptRule for $name {
            fn name(&self) -> &str {
                stringify!($name)
            }

            fn apply(
                &self,
                ctx: &mut $crate::query::optimizer::plan::OptContext,
                group_node: &::std::rc::Rc<::std::cell::RefCell<$crate::query::optimizer::plan::OptGroupNode>>,
            ) -> Result<Option<$crate::query::optimizer::plan::TransformResult>, $crate::query::optimizer::OptimizerError> {
                let node_ref = group_node.borrow();
                if !node_ref.plan_node.$parent_check() {
                    return Ok(None);
                }

                if let Some(matched) = self.match_pattern(ctx, group_node)? {
                    if matched.dependencies.len() >= 1 {
                        let child = &matched.dependencies[0];

                        if child.borrow().plan_node.$child_check() {
                            drop(node_ref);
                            let mut result = $crate::query::optimizer::plan::TransformResult::new();
                            result.add_new_group_node(group_node.clone());
                            return Ok(Some(result));
                        }
                    }
                }
                Ok(None)
            }

            fn pattern(&self) -> $crate::query::optimizer::plan::Pattern {
                $pattern
            }
        }

        impl $crate::query::optimizer::rule_traits::BaseOptRule for $name {}

        impl $crate::query::optimizer::rule_traits::MergeRule for $name {
            fn can_merge(
                &self,
                group_node: &::std::rc::Rc<::std::cell::RefCell<$crate::query::optimizer::plan::OptGroupNode>>,
                child: &$crate::query::optimizer::plan::OptGroupNode,
            ) -> bool {
                let node_ref = group_node.borrow();
                node_ref.plan_node.$parent_check() && child.plan_node.$child_check()
            }

            fn create_merged_node(
                &self,
                _ctx: &mut $crate::query::optimizer::plan::OptContext,
                group_node: &::std::rc::Rc<::std::cell::RefCell<$crate::query::optimizer::plan::OptGroupNode>>,
                _child: &$crate::query::optimizer::plan::OptGroupNode,
            ) -> Result<Option<$crate::query::optimizer::plan::TransformResult>, $crate::query::optimizer::OptimizerError> {
                let _node_ref = group_node.borrow();
                let mut result = $crate::query::optimizer::plan::TransformResult::new();
                result.add_new_group_node(group_node.clone());
                Ok(Some(result))
            }
        }
    };
}

/// 定义消除规则的宏
///
/// # 示例
/// ```
/// define_elimination_rule! {
///     pub struct DedupEliminationRule {
///         target: Dedup,
///         target_check: is_dedup,
///         pattern: PatternBuilder::dedup()
///     }
///     visitor: DedupEliminationVisitor
/// }
#[macro_export]
macro_rules! define_elimination_rule {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            target: $target_type:ident,
            target_check: $target_check:ident,
            pattern: $pattern:expr
        }
        visitor: $visitor:ident
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name;

        impl $crate::query::optimizer::plan::OptRule for $name {
            fn name(&self) -> &str {
                stringify!($name)
            }

            fn apply(
                &self,
                ctx: &mut $crate::query::optimizer::plan::OptContext,
                group_node: &::std::rc::Rc<::std::cell::RefCell<$crate::query::optimizer::plan::OptGroupNode>>,
            ) -> Result<Option<$crate::query::optimizer::plan::TransformResult>, $crate::query::optimizer::OptimizerError> {
                let node_ref = group_node.borrow();
                let mut visitor = $visitor {
                    ctx,
                    is_eliminated: false,
                    eliminated_node: None,
                };

                let result = visitor.visit(&node_ref.plan_node);
                drop(node_ref);

                if result.is_eliminated {
                    if let Some(new_node) = result.eliminated_node {
                        let mut transform_result = $crate::query::optimizer::plan::TransformResult::new();
                        transform_result.add_new_group_node(::std::rc::Rc::new(::std::cell::RefCell::new(new_node)));
                        return Ok(Some(transform_result));
                    }
                }
                Ok(None)
            }

            fn pattern(&self) -> $crate::query::optimizer::plan::Pattern {
                $pattern
            }
        }

        impl $crate::query::optimizer::rule_traits::BaseOptRule for $name {}
    };
}
