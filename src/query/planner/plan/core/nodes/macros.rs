//! 计划节点宏定义
//!
//! 提供宏来简化计划节点的定义，减少样板代码

/// 定义计划节点的宏
///
/// # 示例
/// ```
/// define_plan_node! {
///     pub struct GetVerticesNode {
///         space_id: i32,
///         src_vids: String,
///         tag_props: Vec<TagProp>,
///     }
///     input: ZeroInputNode
/// }
/// ```
#[macro_export]
macro_rules! define_plan_node {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            $($field:ident: $type:ty),* $(,)?
        }
        input: $input_trait:ty
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name {
            id: i64,
            $($field: $type,)*
            output_var: Option<crate::query::context::validate::types::Variable>,
            col_names: Vec<String>,
            cost: f64,
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                use crate::query::planner::plan::core::node_id_generator::next_node_id;
                Self {
                    id: next_node_id(),
                    $($field: self.$field.clone(),)*
                    output_var: self.output_var.clone(),
                    col_names: self.col_names.clone(),
                    cost: self.cost,
                }
            }
        }

        impl $name {
            pub fn id(&self) -> i64 {
                self.id
            }

            pub fn type_name(&self) -> &'static str {
                stringify!($name)
            }

            pub fn output_var(&self) -> Option<&crate::query::context::validate::types::Variable> {
                self.output_var.as_ref()
            }

            pub fn col_names(&self) -> &[String] {
                &self.col_names
            }

            pub fn cost(&self) -> f64 {
                self.cost
            }

            pub fn set_output_var(&mut self, var: crate::query::context::validate::types::Variable) {
                self.output_var = Some(var);
            }

            pub fn set_col_names(&mut self, names: Vec<String>) {
                self.col_names = names;
            }

            pub fn clone_plan_node(&self) -> crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum {
                use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
                PlanNodeEnum::$name(self.clone())
            }

            pub fn clone_with_new_id(&self, new_id: i64) -> crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum {
                let mut cloned = self.clone();
                cloned.id = new_id;
                use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
                PlanNodeEnum::$name(cloned)
            }
        }

        impl crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode for $name {
            fn id(&self) -> i64 {
                self.id()
            }

            fn name(&self) -> &'static str {
                self.type_name()
            }

            fn output_var(&self) -> Option<&crate::query::context::validate::types::Variable> {
                self.output_var()
            }

            fn col_names(&self) -> &[String] {
                self.col_names()
            }

            fn cost(&self) -> f64 {
                self.cost()
            }

            fn set_output_var(&mut self, var: crate::query::context::validate::types::Variable) {
                self.set_output_var(var);
            }

            fn set_col_names(&mut self, names: Vec<String>) {
                self.set_col_names(names);
            }

            fn into_enum(self) -> crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum {
                use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
                PlanNodeEnum::$name(self)
            }
        }

        impl crate::query::planner::plan::core::nodes::plan_node_traits::PlanNodeClonable for $name {
            fn clone_plan_node(&self) -> crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum {
                self.clone_plan_node()
            }

            fn clone_with_new_id(&self, new_id: i64) -> crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum {
                self.clone_with_new_id(new_id)
            }
        }

        impl $input_trait for $name {}
    };
}

/// 定义带依赖的计划节点宏
#[macro_export]
macro_rules! define_plan_node_with_deps {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            $($field:ident: $type:ty),* $(,)?
        }
        input: SingleInputNode
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name {
            id: i64,
            input: Option<Box<crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum>>,
            $($field: $type,)*
            output_var: Option<crate::query::context::validate::types::Variable>,
            col_names: Vec<String>,
            cost: f64,
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                use crate::query::planner::plan::core::node_id_generator::next_node_id;
                Self {
                    id: next_node_id(),
                    input: self.input.clone(),
                    $($field: self.$field.clone(),)*
                    output_var: self.output_var.clone(),
                    col_names: self.col_names.clone(),
                    cost: self.cost,
                }
            }
        }

        impl $name {
            pub fn id(&self) -> i64 {
                self.id
            }

            pub fn type_name(&self) -> &'static str {
                stringify!($name)
            }

            pub fn output_var(&self) -> Option<&crate::query::context::validate::types::Variable> {
                self.output_var.as_ref()
            }

            pub fn col_names(&self) -> &[String] {
                &self.col_names
            }

            pub fn cost(&self) -> f64 {
                self.cost
            }

            pub fn set_output_var(&mut self, var: crate::query::context::validate::types::Variable) {
                self.output_var = Some(var);
            }

            pub fn set_col_names(&mut self, names: Vec<String>) {
                self.col_names = names;
            }

            pub fn clone_plan_node(&self) -> crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum {
                use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
                PlanNodeEnum::$name(self.clone())
            }

            pub fn clone_with_new_id(&self, new_id: i64) -> crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum {
                let mut cloned = self.clone();
                cloned.id = new_id;
                use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
                PlanNodeEnum::$name(cloned)
            }
        }

        impl crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode for $name {
            fn id(&self) -> i64 { self.id() }
            fn name(&self) -> &'static str { self.type_name() }
            fn output_var(&self) -> Option<&crate::query::context::validate::types::Variable> { self.output_var() }
            fn col_names(&self) -> &[String] { self.col_names() }
            fn cost(&self) -> f64 { self.cost() }
            fn set_output_var(&mut self, var: crate::query::context::validate::types::Variable) { self.set_output_var(var); }
            fn set_col_names(&mut self, names: Vec<String>) { self.set_col_names(names); }
            fn into_enum(self) -> crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum {
                use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
                PlanNodeEnum::$name(self)
            }
        }

        impl crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode for $name {
            fn input(&self) -> &crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum {
                self.input.as_ref().expect("输入节点不存在")
            }

            fn set_input(&mut self, input: crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum) {
                self.input = Some(Box::new(input));
            }
        }

        impl crate::query::planner::plan::core::nodes::plan_node_traits::PlanNodeClonable for $name {
            fn clone_plan_node(&self) -> crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum {
                self.clone_plan_node()
            }
            fn clone_with_new_id(&self, new_id: i64) -> crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum {
                self.clone_with_new_id(new_id)
            }
        }
    };
}
