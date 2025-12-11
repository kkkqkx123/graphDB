//! 搜索算法相关的计划节点
//! 包含索引扫描、全文索引扫描等搜索相关操作

use crate::query::planner::plan::core::{PlanNode as BasePlanNode, PlanNodeKind, PlanNodeVisitor, PlanNodeVisitError};
use crate::query::validator::Variable;
use crate::query::planner::plan::core::common::{TagProp, EdgeProp};

// 索引扫描的计划节点
#[derive(Debug)]
pub struct IndexScan {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub tag_id: i32,
    pub index_id: i32,
    pub scan_type: String,  // "RANGE", "PREFIX", "UNIQUE"等
    pub scan_limits: Vec<IndexLimit>,  // 索引扫描限制
    pub filter: Option<String>,
    pub return_columns: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IndexLimit {
    pub column: String,
    pub begin_value: Option<String>,
    pub end_value: Option<String>,
}

impl IndexScan {
    pub fn new(id: i64, space_id: i32, tag_id: i32, index_id: i32, scan_type: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::IndexScan,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            tag_id,
            index_id,
            scan_type: scan_type.to_string(),
            scan_limits: Vec::new(),
            filter: None,
            return_columns: Vec::new(),
        }
    }
}

impl Clone for IndexScan {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            tag_id: self.tag_id,
            index_id: self.index_id,
            scan_type: self.scan_type.clone(),
            scan_limits: self.scan_limits.clone(),
            filter: self.filter.clone(),
            return_columns: self.return_columns.clone(),
        }
    }
}

impl BasePlanNode for IndexScan {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_index_scan(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }
}

// 全文索引扫描的计划节点
#[derive(Debug)]
pub struct FulltextIndexScan {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub index_name: String,
    pub query: String,  // 全文检索查询
    pub limit: Option<i64>,
}

impl FulltextIndexScan {
    pub fn new(id: i64, space_id: i32, index_name: &str, query: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::FulltextIndexScan,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            space_id,
            index_name: index_name.to_string(),
            query: query.to_string(),
            limit: None,
        }
    }
}

impl Clone for FulltextIndexScan {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            space_id: self.space_id,
            index_name: self.index_name.clone(),
            query: self.query.clone(),
            limit: self.limit,
        }
    }
}

impl BasePlanNode for FulltextIndexScan {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_fulltext_index_scan(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }
}