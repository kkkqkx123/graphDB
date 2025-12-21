use super::nodes::{
    AggregateNode, AppendVerticesNode, ArgumentNode, DataCollectNode, DedupNode, ExpandAllNode,
    ExpandNode, FilterNode, GetEdgesNode, GetNeighborsNode, GetVerticesNode, InnerJoinNode,
    LeftJoinNode, LimitNode, LoopNode, PassThroughNode, PatternApplyNode, PlaceholderNode,
    ProjectNode, RollUpApplyNode, ScanEdgesNode, ScanVerticesNode, SelectNode, SortNode, StartNode,
    TopNNode, TraverseNode, UnionNode, UnwindNode,
};
use super::plan_node_traits::PlanNode as BasePlanNode;
use crate::core::error::{DBError, DBResult};
use crate::core::visitor::{
    VisitorConfig, VisitorContext, VisitorCore, VisitorResult, VisitorState,
};
use crate::query::planner::plan::algorithms::{FulltextIndexScan, IndexScan};
use crate::query::planner::plan::management::dml::{
    DeleteEdges, DeleteTags, DeleteVertices, InsertEdges, InsertVertices, NewEdge, NewProp, NewTag,
    NewVertex, UpdateEdge, UpdateVertex,
};
use std::fmt;

/// 统一的计划节点访问者基础trait
/// 现在继承自VisitorCore，使用统一的基础设施
pub trait PlanNodeVisitor: VisitorCore<Result = ()> + std::fmt::Debug {
    /// 计划节点特定的预访问钩子
    /// 注意：VisitorCore的pre_visit会在accept方法中自动调用
    fn plan_pre_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 计划节点特定的后访问钩子
    /// 注意：VisitorCore的post_visit会在accept方法中自动调用
    fn plan_post_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_plan_node(&mut self, _node: &dyn BasePlanNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_neighbors(&mut self, _node: &GetNeighborsNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_vertices(&mut self, _node: &GetVerticesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_edges(&mut self, _node: &GetEdgesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_traverse(&mut self, _node: &TraverseNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_append_vertices(
        &mut self,
        _node: &AppendVerticesNode,
    ) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_filter(&mut self, _node: &FilterNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_project(&mut self, _node: &ProjectNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_union(&mut self, _node: &UnionNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_unwind(&mut self, _node: &UnwindNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_aggregate(&mut self, _node: &AggregateNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_sort(&mut self, _node: &SortNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_limit(&mut self, _node: &LimitNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_topn(&mut self, _node: &TopNNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_data_collect(&mut self, _node: &DataCollectNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_vertices(&mut self, _node: &ScanVerticesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_index_scan(&mut self, _node: &IndexScan) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_fulltext_index_scan(
        &mut self,
        _node: &FulltextIndexScan,
    ) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_expand(&mut self, _node: &ExpandNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_expand_all(&mut self, _node: &ExpandAllNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_start(&mut self, _node: &StartNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_start_node(&mut self, _node: &StartNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_argument(&mut self, _node: &ArgumentNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_argument_node(&mut self, _node: &ArgumentNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_placeholder(&mut self, _node: &PlaceholderNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_select(&mut self, _node: &SelectNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_loop(&mut self, _node: &LoopNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_pass_through(&mut self, _node: &PassThroughNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_hash_left_join(&mut self, _node: &InnerJoinNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_left_join(&mut self, _node: &LeftJoinNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_hash_inner_join(&mut self, _node: &InnerJoinNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_roll_up_apply(&mut self, _node: &RollUpApplyNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_pattern_apply(&mut self, _node: &PatternApplyNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_dedup(&mut self, _node: &DedupNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_edges(&mut self, _node: &ScanEdgesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_edges_node(&mut self, _node: &ScanEdgesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问InsertVertices节点
    fn visit_insert_vertices(&mut self, _node: &InsertVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问InsertEdges节点
    fn visit_insert_edges(&mut self, _node: &InsertEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问UpdateVertex节点
    fn visit_update_vertex(&mut self, _node: &UpdateVertex) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问UpdateEdge节点
    fn visit_update_edge(&mut self, _node: &UpdateEdge) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问DeleteVertices节点
    fn visit_delete_vertices(&mut self, _node: &DeleteVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问DeleteEdges节点
    fn visit_delete_edges(&mut self, _node: &DeleteEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问DeleteTags节点
    fn visit_delete_tags(&mut self, _node: &DeleteTags) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问NewVertex节点
    fn visit_new_vertex(&mut self, _node: &NewVertex) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问NewTag节点
    fn visit_new_tag(&mut self, _node: &NewTag) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问NewProp节点
    fn visit_new_prop(&mut self, _node: &NewProp) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问NewEdge节点
    fn visit_new_edge(&mut self, _node: &NewEdge) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }
}

/// 统一的计划节点访问者基础实现
/// 为所有PlanNodeVisitor提供统一的基础设施支持
#[derive(Debug)]
pub struct UnifiedPlanNodeVisitor {
    context: VisitorContext,
    state: Box<dyn VisitorState>,
    // 计划节点特定的状态可以在这里添加
    visit_count: usize,
    node_stack: Vec<String>,
}

impl UnifiedPlanNodeVisitor {
    pub fn new() -> Self {
        Self {
            context: VisitorContext::new(VisitorConfig::default()),
            state: Box::new(crate::core::visitor::DefaultVisitorState::new()),
            visit_count: 0,
            node_stack: Vec::new(),
        }
    }

    pub fn with_config(config: VisitorConfig) -> Self {
        Self {
            context: VisitorContext::new(config),
            state: Box::new(crate::core::visitor::DefaultVisitorState::new()),
            visit_count: 0,
            node_stack: Vec::new(),
        }
    }

    /// 获取访问计数
    pub fn visit_count(&self) -> usize {
        self.visit_count
    }

    /// 获取当前节点栈深度
    pub fn stack_depth(&self) -> usize {
        self.node_stack.len()
    }

    /// 推入节点到栈中
    pub fn push_node(&mut self, node_name: String) {
        self.node_stack.push(node_name);
    }

    /// 从栈中弹出节点
    pub fn pop_node(&mut self) -> Option<String> {
        self.node_stack.pop()
    }

    /// 获取当前节点路径
    pub fn current_path(&self) -> String {
        self.node_stack.join(" -> ")
    }

    /// 增加访问计数
    fn inc_visit_count(&mut self) {
        self.visit_count += 1;
    }
}

impl VisitorCore for UnifiedPlanNodeVisitor {
    type Result = ();

    fn context(&self) -> &VisitorContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut VisitorContext {
        &mut self.context
    }

    fn state(&self) -> &dyn VisitorState {
        self.state.as_ref()
    }

    fn state_mut(&mut self) -> &mut dyn VisitorState {
        self.state.as_mut()
    }

    fn pre_visit(&mut self) -> VisitorResult<()> {
        self.inc_visit_count();
        self.state.inc_visit_count();
        if self.state.depth() > self.context.config().max_depth {
            return Err(DBError::Validation(format!(
                "访问深度超过限制: {}",
                self.context.config().max_depth
            )));
        }
        Ok(())
    }

    fn post_visit(&mut self) -> VisitorResult<()> {
        Ok(())
    }
}

impl Default for UnifiedPlanNodeVisitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 计划节点访问错误
#[derive(Debug, Clone)]
pub enum PlanNodeVisitError {
    /// 访问错误
    VisitError(String),

    /// 遍历错误
    TraversalError(String),

    /// 验证错误
    ValidationError(String),
}

impl fmt::Display for PlanNodeVisitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanNodeVisitError::VisitError(msg) => write!(f, "访问错误: {}", msg),
            PlanNodeVisitError::TraversalError(msg) => write!(f, "遍历错误: {}", msg),
            PlanNodeVisitError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
        }
    }
}

impl std::error::Error for PlanNodeVisitError {}

/// 实现从 DBError 到 PlanNodeVisitError 的转换
impl From<crate::core::error::DBError> for PlanNodeVisitError {
    fn from(err: crate::core::error::DBError) -> Self {
        match err {
            crate::core::error::DBError::Storage(e) => {
                PlanNodeVisitError::VisitError(format!("存储错误: {}", e))
            }
            crate::core::error::DBError::Query(e) => {
                PlanNodeVisitError::VisitError(format!("查询错误: {}", e))
            }
            crate::core::error::DBError::Expression(e) => {
                PlanNodeVisitError::VisitError(format!("表达式错误: {}", e))
            }
            crate::core::error::DBError::Plan(e) => {
                PlanNodeVisitError::VisitError(format!("计划错误: {}", e))
            }
            crate::core::error::DBError::Lock(e) => {
                PlanNodeVisitError::VisitError(format!("锁错误: {}", e))
            }
            crate::core::error::DBError::Validation(msg) => {
                PlanNodeVisitError::ValidationError(msg)
            }
            crate::core::error::DBError::Io(e) => {
                PlanNodeVisitError::VisitError(format!("IO错误: {}", e))
            }
            crate::core::error::DBError::TypeDeduction(msg) => {
                PlanNodeVisitError::VisitError(format!("类型推导错误: {}", msg))
            }
            crate::core::error::DBError::Serialization(msg) => {
                PlanNodeVisitError::VisitError(format!("序列化错误: {}", msg))
            }
            crate::core::error::DBError::Internal(msg) => {
                PlanNodeVisitError::VisitError(format!("内部错误: {}", msg))
            }
        }
    }
}

/// 具体的计划节点访问者实现示例
/// 现在基于UnifiedPlanNodeVisitor
#[derive(Debug)]
pub struct DefaultPlanNodeVisitor {
    base: UnifiedPlanNodeVisitor,
}

impl DefaultPlanNodeVisitor {
    pub fn new() -> Self {
        Self {
            base: UnifiedPlanNodeVisitor::new(),
        }
    }

    pub fn with_config(config: VisitorConfig) -> Self {
        Self {
            base: UnifiedPlanNodeVisitor::with_config(config),
        }
    }

    /// 获取基础访问者的引用
    pub fn base(&self) -> &UnifiedPlanNodeVisitor {
        &self.base
    }

    /// 获取基础访问者的可变引用
    pub fn base_mut(&mut self) -> &mut UnifiedPlanNodeVisitor {
        &mut self.base
    }
}

impl Default for DefaultPlanNodeVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl VisitorCore for DefaultPlanNodeVisitor {
    type Result = ();

    fn context(&self) -> &VisitorContext {
        self.base.context()
    }

    fn context_mut(&mut self) -> &mut VisitorContext {
        self.base.context_mut()
    }

    fn state(&self) -> &dyn VisitorState {
        self.base.state()
    }

    fn state_mut(&mut self) -> &mut dyn VisitorState {
        self.base.state_mut()
    }

    fn pre_visit(&mut self) -> VisitorResult<()> {
        self.base.pre_visit()
    }

    fn post_visit(&mut self) -> VisitorResult<()> {
        self.base.post_visit()
    }
}

impl PlanNodeVisitor for DefaultPlanNodeVisitor {
    fn plan_pre_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        VisitorCore::pre_visit(self).map_err(|e| PlanNodeVisitError::VisitError(e.to_string()))
    }

    fn visit_plan_node(&mut self, _node: &dyn BasePlanNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_neighbors(&mut self, _node: &GetNeighborsNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_vertices(&mut self, _node: &GetVerticesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_get_edges(&mut self, _node: &GetEdgesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_traverse(&mut self, _node: &TraverseNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_append_vertices(
        &mut self,
        _node: &AppendVerticesNode,
    ) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_filter(&mut self, _node: &FilterNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_project(&mut self, _node: &ProjectNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_union(&mut self, _node: &UnionNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_unwind(&mut self, _node: &UnwindNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_aggregate(&mut self, _node: &AggregateNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_dedup(&mut self, _node: &DedupNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_edges(&mut self, _node: &ScanEdgesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_edges_node(&mut self, _node: &ScanEdgesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_roll_up_apply(&mut self, _node: &RollUpApplyNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_pattern_apply(&mut self, _node: &PatternApplyNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_sort(&mut self, _node: &SortNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_limit(&mut self, _node: &LimitNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_topn(&mut self, _node: &TopNNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_data_collect(&mut self, _node: &DataCollectNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_scan_vertices(&mut self, _node: &ScanVerticesNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_index_scan(&mut self, _node: &IndexScan) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_fulltext_index_scan(
        &mut self,
        _node: &FulltextIndexScan,
    ) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_expand(&mut self, _node: &ExpandNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_expand_all(&mut self, _node: &ExpandAllNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_start(&mut self, _node: &StartNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_start_node(&mut self, _node: &StartNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_argument(&mut self, _node: &ArgumentNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_argument_node(&mut self, _node: &ArgumentNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_placeholder(&mut self, _node: &PlaceholderNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_select(&mut self, _node: &SelectNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_loop(&mut self, _node: &LoopNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_pass_through(&mut self, _node: &PassThroughNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_hash_left_join(&mut self, _node: &InnerJoinNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_left_join(&mut self, _node: &LeftJoinNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_hash_inner_join(&mut self, _node: &InnerJoinNode) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_insert_vertices(&mut self, _node: &InsertVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_insert_edges(&mut self, _node: &InsertEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_update_vertex(&mut self, _node: &UpdateVertex) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_update_edge(&mut self, _node: &UpdateEdge) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_delete_vertices(&mut self, _node: &DeleteVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_delete_edges(&mut self, _node: &DeleteEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_delete_tags(&mut self, _node: &DeleteTags) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_new_vertex(&mut self, _node: &NewVertex) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_new_tag(&mut self, _node: &NewTag) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_new_prop(&mut self, _node: &NewProp) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn visit_new_edge(&mut self, _node: &NewEdge) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn plan_post_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        VisitorCore::post_visit(self).map_err(|e| PlanNodeVisitError::VisitError(e.to_string()))
    }
}

/// 便捷宏：创建基于UnifiedPlanNodeVisitor的访问者
#[macro_export]
macro_rules! create_plan_visitor {
    ($visitor_type:ty) => {
        <$visitor_type>::new()
    };
    ($visitor_type:ty, $config:expr) => {
        <$visitor_type>::with_config($config)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::visitor::VisitorConfig;

    #[test]
    fn test_unified_plan_node_visitor() {
        let mut visitor = UnifiedPlanNodeVisitor::new();

        // 测试VisitorCore方法
        assert!(visitor.should_continue());
        assert_eq!(visitor.visit_count(), 0);
        assert_eq!(visitor.stack_depth(), 0);

        // 测试节点栈操作
        visitor.push_node("StartNode".to_string());
        visitor.push_node("FilterNode".to_string());
        assert_eq!(visitor.stack_depth(), 2);
        assert_eq!(visitor.current_path(), "StartNode -> FilterNode");

        let popped = visitor.pop_node();
        assert_eq!(popped, Some("FilterNode".to_string()));
        assert_eq!(visitor.stack_depth(), 1);
    }

    #[test]
    fn test_default_plan_node_visitor() {
        let config = VisitorConfig::new().with_max_depth(5);
        let mut visitor = DefaultPlanNodeVisitor::with_config(config);

        // 测试继承的VisitorCore功能
        assert!(visitor.should_continue());
        assert_eq!(visitor.base().visit_count(), 0);

        // 测试pre_visit和post_visit
        assert!(visitor.pre_visit().is_ok());
        assert!(visitor.post_visit().is_ok());
    }

    #[test]
    fn test_plan_visitor_macro() {
        let visitor = create_plan_visitor!(DefaultPlanNodeVisitor);
        assert!(visitor.should_continue());

        let config = VisitorConfig::new().with_max_depth(3);
        let visitor_with_config = create_plan_visitor!(DefaultPlanNodeVisitor, config);
        assert_eq!(visitor_with_config.context().config().max_depth, 3);
    }
}
