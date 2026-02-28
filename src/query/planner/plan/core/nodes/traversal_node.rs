//! 图遍历节点实现
//!
//! 包含Expand、ExpandAll、Traverse等图遍历相关的计划节点

use std::sync::Arc;

use super::super::common::{EdgeProp, TagProp};
use crate::core::types::{EdgeDirection, ContextualExpression, SerializableExpression, ExpressionContext};
use crate::core::Expression;
use crate::define_plan_node;
use crate::define_plan_node_with_deps;
use crate::query::planner::plan::core::node_id_generator::next_node_id;

define_plan_node! {
    pub struct ExpandNode {
        space_id: u64,
        edge_types: Vec<String>,
        direction: EdgeDirection,
        step_limit: Option<u32>,
        filter: Option<ContextualExpression>,
        filter_serializable: Option<SerializableExpression>,
    }
    enum: Expand
    input: MultipleInputNode
}

impl ExpandNode {
    pub fn new(space_id: u64, edge_types: Vec<String>, direction: EdgeDirection) -> Self {
        Self {
            id: next_node_id(),
            deps: Vec::new(),
            space_id,
            edge_types,
            direction,
            step_limit: None,
            filter: None,
            filter_serializable: None,
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn direction(&self) -> EdgeDirection {
        self.direction
    }

    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    pub fn step_limit(&self) -> Option<u32> {
        self.step_limit
    }

    pub fn filter(&self) -> Option<&ContextualExpression> {
        self.filter.as_ref()
    }

    pub fn set_filter(&mut self, filter: ContextualExpression) {
        self.filter = Some(filter);
        self.filter_serializable = None;
    }
    
    pub fn set_filter_string(&mut self, filter: String, ctx: Arc<ExpressionContext>) {
        let expr = crate::core::types::expression::ExpressionMeta::new(
            crate::core::Expression::Variable(filter)
        );
        let id = ctx.register_expression(expr);
        self.filter = Some(ContextualExpression::new(id, ctx));
        self.filter_serializable = None;
    }
    
    pub fn prepare_for_serialization(&mut self) {
        if let Some(ref ctx_expr) = self.filter {
            self.filter_serializable = Some(SerializableExpression::from_contextual(ctx_expr));
        }
    }
    
    pub fn after_deserialization(&mut self, ctx: Arc<ExpressionContext>) {
        if let Some(ref ser_expr) = self.filter_serializable {
            self.filter = Some(ser_expr.clone().to_contextual(ctx));
        }
    }
}

define_plan_node! {
    pub struct ExpandAllNode {
        space_id: u64,
        edge_types: Vec<String>,
        direction: String,
        step_limit: Option<u32>,
        step_limits: Option<Vec<u32>>,
        join_input: bool,
        sample: bool,
        edge_props: Vec<EdgeProp>,
        vertex_props: Vec<TagProp>,
        filter: Option<ContextualExpression>,
        filter_serializable: Option<SerializableExpression>,
    }
    enum: ExpandAll
    input: MultipleInputNode
}

impl ExpandAllNode {
    pub fn new(space_id: u64, edge_types: Vec<String>, direction: &str) -> Self {
        Self {
            id: next_node_id(),
            deps: Vec::new(),
            space_id,
            edge_types,
            direction: direction.to_string(),
            step_limit: None,
            step_limits: None,
            join_input: false,
            sample: false,
            edge_props: Vec::new(),
            vertex_props: Vec::new(),
            filter: None,
            filter_serializable: None,
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn step_limits(&self) -> Option<&Vec<u32>> {
        self.step_limits.as_ref()
    }

    pub fn set_step_limits(&mut self, limits: Vec<u32>) {
        self.step_limits = Some(limits);
    }

    pub fn join_input(&self) -> bool {
        self.join_input
    }

    pub fn set_join_input(&mut self, join: bool) {
        self.join_input = join;
    }

    pub fn sample(&self) -> bool {
        self.sample
    }

    pub fn set_sample(&mut self, sample: bool) {
        self.sample = sample;
    }

    pub fn edge_props(&self) -> &[EdgeProp] {
        &self.edge_props
    }

    pub fn set_edge_props(&mut self, props: Vec<EdgeProp>) {
        self.edge_props = props;
    }

    pub fn vertex_props(&self) -> &[TagProp] {
        &self.vertex_props
    }

    pub fn set_vertex_props(&mut self, props: Vec<TagProp>) {
        self.vertex_props = props;
    }

    pub fn step_limit(&self) -> Option<u32> {
        self.step_limit
    }

    pub fn set_step_limit(&mut self, limit: u32) {
        self.step_limit = Some(limit);
    }

    pub fn direction(&self) -> &str {
        &self.direction
    }

    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    pub fn filter(&self) -> Option<&ContextualExpression> {
        self.filter.as_ref()
    }

    pub fn set_filter(&mut self, filter: ContextualExpression) {
        self.filter = Some(filter);
        self.filter_serializable = None;
    }
    
    pub fn set_filter_string(&mut self, filter: String, ctx: Arc<ExpressionContext>) {
        let expr = crate::core::types::expression::ExpressionMeta::new(
            crate::core::Expression::Variable(filter)
        );
        let id = ctx.register_expression(expr);
        self.filter = Some(ContextualExpression::new(id, ctx));
        self.filter_serializable = None;
    }
    
    pub fn prepare_for_serialization(&mut self) {
        if let Some(ref ctx_expr) = self.filter {
            self.filter_serializable = Some(SerializableExpression::from_contextual(ctx_expr));
        }
    }
    
    pub fn after_deserialization(&mut self, ctx: Arc<ExpressionContext>) {
        if let Some(ref ser_expr) = self.filter_serializable {
            self.filter = Some(ser_expr.clone().to_contextual(ctx));
        }
    }
}

define_plan_node_with_deps! {
    pub struct TraverseNode {
        space_id: u64,
        start_vids: String,
        end_vids: Option<String>,
        edge_types: Vec<String>,
        direction: EdgeDirection,
        min_steps: u32,
        max_steps: u32,
        edge_alias: Option<String>,
        vertex_alias: Option<String>,
        e_filter: Option<ContextualExpression>,
        e_filter_serializable: Option<SerializableExpression>,
        v_filter: Option<ContextualExpression>,
        v_filter_serializable: Option<SerializableExpression>,
        first_step_filter: Option<ContextualExpression>,
        first_step_filter_serializable: Option<SerializableExpression>,
    }
    enum: Traverse
    input: SingleInputNode
}

impl TraverseNode {
    pub fn new(space_id: u64, start_vids: &str, min_steps: u32, max_steps: u32) -> Self {
        Self {
            id: next_node_id(),
            input: None,
            deps: Vec::new(),
            space_id,
            start_vids: start_vids.to_string(),
            end_vids: None,
            edge_types: Vec::new(),
            direction: EdgeDirection::Both,
            min_steps,
            max_steps,
            edge_alias: None,
            vertex_alias: None,
            e_filter: None,
            e_filter_serializable: None,
            v_filter: None,
            v_filter_serializable: None,
            first_step_filter: None,
            first_step_filter_serializable: None,
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn set_end_vids(&mut self, end_vids: &str) {
        self.end_vids = Some(end_vids.to_string());
    }

    pub fn set_edge_types(&mut self, edge_types: Vec<String>) {
        self.edge_types = edge_types;
    }

    pub fn set_direction(&mut self, direction: EdgeDirection) {
        self.direction = direction;
    }

    pub fn start_vids(&self) -> &str {
        &self.start_vids
    }

    pub fn end_vids(&self) -> Option<&String> {
        self.end_vids.as_ref()
    }

    pub fn edge_types(&self) -> &[String] {
        &self.edge_types
    }

    pub fn direction(&self) -> EdgeDirection {
        self.direction
    }

    pub fn min_steps(&self) -> u32 {
        self.min_steps
    }

    pub fn max_steps(&self) -> u32 {
        self.max_steps
    }

    pub fn step_limit(&self) -> Option<u32> {
        Some(self.max_steps)
    }

    pub fn filter(&self) -> Option<&String> {
        None
    }

    pub fn is_one_step(&self) -> bool {
        self.min_steps == 1 && self.max_steps == 1
    }

    pub fn edge_alias(&self) -> Option<&String> {
        self.edge_alias.as_ref()
    }

    pub fn vertex_alias(&self) -> Option<&String> {
        self.vertex_alias.as_ref()
    }

    pub fn e_filter(&self) -> Option<&ContextualExpression> {
        self.e_filter.as_ref()
    }

    pub fn set_e_filter(&mut self, filter: ContextualExpression) {
        self.e_filter = Some(filter);
        self.e_filter_serializable = None;
    }
    
    pub fn set_e_filter_expression(&mut self, filter: Expression, ctx: Arc<ExpressionContext>) {
        let expr = crate::core::types::expression::ExpressionMeta::new(filter);
        let id = ctx.register_expression(expr);
        self.e_filter = Some(ContextualExpression::new(id, ctx));
        self.e_filter_serializable = None;
    }

    pub fn v_filter(&self) -> Option<&ContextualExpression> {
        self.v_filter.as_ref()
    }

    pub fn set_v_filter(&mut self, filter: ContextualExpression) {
        self.v_filter = Some(filter);
        self.v_filter_serializable = None;
    }
    
    pub fn set_v_filter_expression(&mut self, filter: Expression, ctx: Arc<ExpressionContext>) {
        let expr = crate::core::types::expression::ExpressionMeta::new(filter);
        let id = ctx.register_expression(expr);
        self.v_filter = Some(ContextualExpression::new(id, ctx));
        self.v_filter_serializable = None;
    }

    pub fn first_step_filter(&self) -> Option<&ContextualExpression> {
        self.first_step_filter.as_ref()
    }

    pub fn set_first_step_filter(&mut self, filter: ContextualExpression) {
        self.first_step_filter = Some(filter);
        self.first_step_filter_serializable = None;
    }
    
    pub fn set_first_step_filter_expression(&mut self, filter: Expression, ctx: Arc<ExpressionContext>) {
        let expr = crate::core::types::expression::ExpressionMeta::new(filter);
        let id = ctx.register_expression(expr);
        self.first_step_filter = Some(ContextualExpression::new(id, ctx));
        self.first_step_filter_serializable = None;
    }
    
    pub fn prepare_for_serialization(&mut self) {
        if let Some(ref ctx_expr) = self.e_filter {
            self.e_filter_serializable = Some(SerializableExpression::from_contextual(ctx_expr));
        }
        if let Some(ref ctx_expr) = self.v_filter {
            self.v_filter_serializable = Some(SerializableExpression::from_contextual(ctx_expr));
        }
        if let Some(ref ctx_expr) = self.first_step_filter {
            self.first_step_filter_serializable = Some(SerializableExpression::from_contextual(ctx_expr));
        }
    }
    
    pub fn after_deserialization(&mut self, ctx: Arc<ExpressionContext>) {
        if let Some(ref ser_expr) = self.e_filter_serializable {
            self.e_filter = Some(ser_expr.clone().to_contextual(ctx.clone()));
        }
        if let Some(ref ser_expr) = self.v_filter_serializable {
            self.v_filter = Some(ser_expr.clone().to_contextual(ctx.clone()));
        }
        if let Some(ref ser_expr) = self.first_step_filter_serializable {
            self.first_step_filter = Some(ser_expr.clone().to_contextual(ctx));
        }
    }
}

define_plan_node! {
    pub struct AppendVerticesNode {
        space_id: u64,
        vertex_tag: String,
        vertex_props: Vec<TagProp>,
        filter: Option<ContextualExpression>,
        filter_serializable: Option<SerializableExpression>,
        input_var: Option<String>,
        src_expression: Option<ContextualExpression>,
        src_expression_serializable: Option<SerializableExpression>,
        dedup: bool,
        track_prev_path: bool,
        need_fetch_prop: bool,
        vids: Vec<String>,
        tag_ids: Vec<i32>,
        v_filter: Option<ContextualExpression>,
        v_filter_serializable: Option<SerializableExpression>,
        node_alias: Option<String>,
    }
    enum: AppendVertices
    input: MultipleInputNode
}

impl AppendVerticesNode {
    pub fn new(space_id: u64, vertex_tag: &str) -> Self {
        Self {
            id: next_node_id(),
            deps: Vec::new(),
            space_id,
            vertex_tag: vertex_tag.to_string(),
            vertex_props: Vec::new(),
            filter: None,
            filter_serializable: None,
            input_var: None,
            src_expression: None,
            src_expression_serializable: None,
            dedup: false,
            track_prev_path: false,
            need_fetch_prop: false,
            vids: Vec::new(),
            tag_ids: Vec::new(),
            v_filter: None,
            v_filter_serializable: None,
            node_alias: None,
            output_var: None,
            col_names: Vec::new(),
        }
    }

    pub fn node_alias(&self) -> Option<&String> {
        self.node_alias.as_ref()
    }

    pub fn set_node_alias(&mut self, alias: String) {
        self.node_alias = Some(alias);
    }

    pub fn vertex_tag(&self) -> &str {
        &self.vertex_tag
    }

    pub fn vertex_props(&self) -> &[TagProp] {
        &self.vertex_props
    }

    pub fn set_vertex_props(&mut self, props: Vec<TagProp>) {
        self.vertex_props = props;
    }

    pub fn filter(&self) -> Option<&ContextualExpression> {
        self.filter.as_ref()
    }

    pub fn set_filter(&mut self, filter: ContextualExpression) {
        self.filter = Some(filter);
        self.filter_serializable = None;
    }
    
    pub fn set_filter_string(&mut self, filter: String, ctx: Arc<ExpressionContext>) {
        let expr = crate::core::types::expression::ExpressionMeta::new(
            crate::core::Expression::Variable(filter)
        );
        let id = ctx.register_expression(expr);
        self.filter = Some(ContextualExpression::new(id, ctx));
        self.filter_serializable = None;
    }

    pub fn input_var(&self) -> Option<&str> {
        self.input_var.as_deref()
    }

    pub fn src_expression(&self) -> Option<&ContextualExpression> {
        self.src_expression.as_ref()
    }
    
    pub fn set_src_expression(&mut self, expr: ContextualExpression) {
        self.src_expression = Some(expr);
        self.src_expression_serializable = None;
    }
    
    pub fn set_src_expression_expression(&mut self, expr: Expression, ctx: Arc<ExpressionContext>) {
        let meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        self.src_expression = Some(ContextualExpression::new(id, ctx));
        self.src_expression_serializable = None;
    }

    pub fn v_filter(&self) -> Option<&ContextualExpression> {
        self.v_filter.as_ref()
    }

    pub fn set_v_filter(&mut self, filter: ContextualExpression) {
        self.v_filter = Some(filter);
        self.v_filter_serializable = None;
    }
    
    pub fn set_v_filter_expression(&mut self, filter: Expression, ctx: Arc<ExpressionContext>) {
        let expr = crate::core::types::expression::ExpressionMeta::new(filter);
        let id = ctx.register_expression(expr);
        self.v_filter = Some(ContextualExpression::new(id, ctx));
        self.v_filter_serializable = None;
    }
    
    pub fn prepare_for_serialization(&mut self) {
        if let Some(ref ctx_expr) = self.filter {
            self.filter_serializable = Some(SerializableExpression::from_contextual(ctx_expr));
        }
        if let Some(ref ctx_expr) = self.src_expression {
            self.src_expression_serializable = Some(SerializableExpression::from_contextual(ctx_expr));
        }
        if let Some(ref ctx_expr) = self.v_filter {
            self.v_filter_serializable = Some(SerializableExpression::from_contextual(ctx_expr));
        }
    }
    
    pub fn after_deserialization(&mut self, ctx: Arc<ExpressionContext>) {
        if let Some(ref ser_expr) = self.filter_serializable {
            self.filter = Some(ser_expr.clone().to_contextual(ctx.clone()));
        }
        if let Some(ref ser_expr) = self.src_expression_serializable {
            self.src_expression = Some(ser_expr.clone().to_contextual(ctx.clone()));
        }
        if let Some(ref ser_expr) = self.v_filter_serializable {
            self.v_filter = Some(ser_expr.clone().to_contextual(ctx));
        }
    }

    pub fn dedup(&self) -> bool {
        self.dedup
    }

    pub fn track_prev_path(&self) -> bool {
        self.track_prev_path
    }

    pub fn need_fetch_prop(&self) -> bool {
        self.need_fetch_prop
    }

    pub fn vids(&self) -> &[String] {
        &self.vids
    }

    pub fn tag_ids(&self) -> &[i32] {
        &self.tag_ids
    }
}
