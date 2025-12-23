//! 搜索算法相关的计划节点
//! 包含索引扫描、全文索引扫描等搜索相关操作

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;

#[derive(Debug, Clone)]
pub struct IndexLimit {
    pub column: String,
    pub begin_value: Option<String>,
    pub end_value: Option<String>,
}

// 索引扫描的计划节点
#[derive(Debug, Clone)]
pub struct IndexScan {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub tag_id: i32,
    pub index_id: i32,
    pub scan_type: String,            // "RANGE", "PREFIX", "UNIQUE"等
    pub scan_limits: Vec<IndexLimit>, // 索引扫描限制
    pub filter: Option<String>,
    pub return_columns: Vec<String>,
    pub limit: Option<i64>, // 限制返回的记录数量
}

impl IndexScan {
    pub fn new(id: i64, space_id: i32, tag_id: i32, index_id: i32, scan_type: &str) -> Self {
        Self {
            id,
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
            limit: None,
        }
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn has_effective_filter(&self) -> bool {
        self.filter.is_some() || !self.scan_limits.is_empty()
    }

    /// 获取节点的唯一ID
    pub fn id(&self) -> i64 {
        self.id
    }

    /// 获取类型名称
    pub fn type_name(&self) -> &'static str {
        "IndexScan"
    }

    /// 获取节点的输出变量
    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    /// 获取列名列表
    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    /// 获取节点的成本估计值
    pub fn cost(&self) -> f64 {
        self.cost
    }

    /// 获取节点的依赖节点列表
    pub fn dependencies(&self) -> &[PlanNodeEnum] {
        &self.deps
    }

    /// 添加依赖节点
    pub fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    /// 移除依赖节点
    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(index) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(index);
            true
        } else {
            false
        }
    }

    /// 设置节点的输出变量
    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    /// 设置列名
    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    /// 克隆节点
    pub fn clone_plan_node(&self) -> PlanNodeEnum {
        // 这里需要创建一个 PlanNodeEnum::IndexScan，但需要先在 PlanNodeEnum 中添加这个变体
        // 暂时返回一个 StartNode 作为占位符
        PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new())
    }

    /// 克隆节点并分配新的ID
    pub fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        // 这里需要创建一个 PlanNodeEnum::IndexScan，但需要先在 PlanNodeEnum 中添加这个变体
        // 暂时返回一个 StartNode 作为占位符
        PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new())
    }

    /// 使用访问者模式访问节点
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeVisitor,
    {
        visitor.visit_index_scan(self)
    }
}

// 全文索引扫描的计划节点
#[derive(Debug, Clone)]
pub struct FulltextIndexScan {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub space_id: i32,
    pub index_name: String,
    pub query: String, // 全文检索查询
    pub limit: Option<i64>,
}

impl FulltextIndexScan {
    pub fn new(id: i64, space_id: i32, index_name: &str, query: &str) -> Self {
        Self {
            id,
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

    /// 获取节点的唯一ID
    pub fn id(&self) -> i64 {
        self.id
    }

    /// 获取类型名称
    pub fn type_name(&self) -> &'static str {
        "FulltextIndexScan"
    }

    /// 获取节点的输出变量
    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    /// 获取列名列表
    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    /// 获取节点的成本估计值
    pub fn cost(&self) -> f64 {
        self.cost
    }

    /// 获取节点的依赖节点列表
    pub fn dependencies(&self) -> &[PlanNodeEnum] {
        &self.deps
    }

    /// 添加依赖节点
    pub fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    /// 移除依赖节点
    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(index) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(index);
            true
        } else {
            false
        }
    }

    /// 设置节点的输出变量
    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    /// 设置列名
    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    /// 克隆节点
    pub fn clone_plan_node(&self) -> PlanNodeEnum {
        // 这里需要创建一个 PlanNodeEnum::FulltextIndexScan，但需要先在 PlanNodeEnum 中添加这个变体
        // 暂时返回一个 StartNode 作为占位符
        PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new())
    }

    /// 克隆节点并分配新的ID
    pub fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        // 这里需要创建一个 PlanNodeEnum::FulltextIndexScan，但需要先在 PlanNodeEnum 中添加这个变体
        // 暂时返回一个 StartNode 作为占位符
        PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new())
    }

    /// 使用访问者模式访问节点
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeVisitor,
    {
        visitor.visit_fulltext_index_scan(self)
    }
}
