//! 数据更新操作相关的计划节点
//! 包括更新顶点和边的操作

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt,
        PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use std::sync::Arc;

/// 更新顶点计划节点
#[derive(Debug)]
pub struct UpdateVertex {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub tag_id: i32,
    pub vid: String,
    pub updated_props: Vec<(String, String)>, // 属性名和新值
    pub insertable: bool,                     // 如果顶点不存在是否插入
    pub return_props: Vec<String>,            // 返回的属性列表
    pub condition: Option<String>,            // 更新条件
}

impl UpdateVertex {
    pub fn new(
        id: i64,
        space_id: i32,
        tag_id: i32,
        vid: &str,
        updated_props: Vec<(String, String)>,
        insertable: bool,
        return_props: Vec<String>,
        condition: Option<String>,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::UpdateVertex,
            deps: Vec::new(),
            output_var: None,
            col_names: return_props.clone(),
            cost: 0.0,
            space_id,
            tag_id,
            vid: vid.to_string(),
            updated_props,
            insertable,
            return_props,
            condition,
        }
    }

    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    pub fn tag_id(&self) -> i32 {
        self.tag_id
    }

    pub fn vid(&self) -> &str {
        &self.vid
    }

    pub fn updated_props(&self) -> &[(String, String)] {
        &self.updated_props
    }

    pub fn insertable(&self) -> bool {
        self.insertable
    }

    pub fn return_props(&self) -> &[String] {
        &self.return_props
    }

    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }
}

impl Clone for UpdateVertex {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            tag_id: self.tag_id,
            vid: self.vid.clone(),
            updated_props: self.updated_props.clone(),
            insertable: self.insertable,
            return_props: self.return_props.clone(),
            condition: self.condition.clone(),
        }
    }
}

impl PlanNodeIdentifiable for UpdateVertex {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for UpdateVertex {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for UpdateVertex {
    fn dependencies(&self) -> Vec<PlanNodeEnum> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for UpdateVertex {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[PlanNodeEnum]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for UpdateVertex {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for UpdateVertex {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for UpdateVertex {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_update_vertex(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for UpdateVertex {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 更新边计划节点
#[derive(Debug)]
pub struct UpdateEdge {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub edge_type_id: i32,
    pub src_id: String,
    pub dst_id: String,
    pub rank: i64,
    pub updated_props: Vec<(String, String)>, // 属性名和新值
    pub insertable: bool,                     // 如果边不存在是否插入
    pub return_props: Vec<String>,            // 返回的属性列表
    pub condition: Option<String>,            // 更新条件
}

impl UpdateEdge {
    pub fn new(
        id: i64,
        space_id: i32,
        edge_type_id: i32,
        src_id: &str,
        dst_id: &str,
        rank: i64,
        updated_props: Vec<(String, String)>,
        insertable: bool,
        return_props: Vec<String>,
        condition: Option<String>,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::UpdateEdge,
            deps: Vec::new(),
            output_var: None,
            col_names: return_props.clone(),
            cost: 0.0,
            space_id,
            edge_type_id,
            src_id: src_id.to_string(),
            dst_id: dst_id.to_string(),
            rank,
            updated_props,
            insertable,
            return_props,
            condition,
        }
    }

    pub fn space_id(&self) -> i32 {
        self.space_id
    }

    pub fn edge_type_id(&self) -> i32 {
        self.edge_type_id
    }

    pub fn src_id(&self) -> &str {
        &self.src_id
    }

    pub fn dst_id(&self) -> &str {
        &self.dst_id
    }

    pub fn rank(&self) -> i64 {
        self.rank
    }

    pub fn updated_props(&self) -> &[(String, String)] {
        &self.updated_props
    }

    pub fn insertable(&self) -> bool {
        self.insertable
    }

    pub fn return_props(&self) -> &[String] {
        &self.return_props
    }

    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }
}

impl Clone for UpdateEdge {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            edge_type_id: self.edge_type_id,
            src_id: self.src_id.clone(),
            dst_id: self.dst_id.clone(),
            rank: self.rank,
            updated_props: self.updated_props.clone(),
            insertable: self.insertable,
            return_props: self.return_props.clone(),
            condition: self.condition.clone(),
        }
    }
}

impl PlanNodeIdentifiable for UpdateEdge {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for UpdateEdge {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for UpdateEdge {
    fn dependencies(&self) -> Vec<PlanNodeEnum> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for UpdateEdge {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[PlanNodeEnum]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for UpdateEdge {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for UpdateEdge {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for UpdateEdge {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_update_edge(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for UpdateEdge {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
