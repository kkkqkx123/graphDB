use graphdb::query::planner::plan::{
    PlanNode, PlanNodeKind, 
    GetVertices, GetEdges, Project, Filter, Dedup, ScanEdges,
    PlanNodeVisitor, PlanNodeVisitError, DefaultPlanNodeVisitor
};
use graphdb::query::validator::Variable;

// 用于测试的计数访问者
#[derive(Debug)]
struct CountingVisitor {
    get_vertices_count: usize,
    get_edges_count: usize,
    project_count: usize,
    filter_count: usize,
    scan_edges_count: usize,
}

impl CountingVisitor {
    fn new() -> Self {
        Self {
            get_vertices_count: 0,
            get_edges_count: 0,
            project_count: 0,
            filter_count: 0,
            scan_edges_count: 0,
        }
    }
}

impl PlanNodeVisitor for CountingVisitor {
    fn visit_get_vertices(&mut self, _node: &GetVertices) -> Result<(), PlanNodeVisitError> {
        self.get_vertices_count += 1;
        Ok(())
    }

    fn visit_get_edges(&mut self, _node: &GetEdges) -> Result<(), PlanNodeVisitError> {
        self.get_edges_count += 1;
        Ok(())
    }

    fn visit_project(&mut self, _node: &Project) -> Result<(), PlanNodeVisitError> {
        self.project_count += 1;
        Ok(())
    }

    fn visit_filter(&mut self, _node: &Filter) -> Result<(), PlanNodeVisitError> {
        self.filter_count += 1;
        Ok(())
    }

    fn visit_scan_edges(&mut self, _node: &ScanEdges) -> Result<(), PlanNodeVisitError> {
        self.scan_edges_count += 1;
        Ok(())
    }
}

#[test]
fn test_plan_node_creation() {
    // 测试创建不同的计划节点
    let mut get_vertices = GetVertices::new(1, 101, "vid");
    get_vertices.set_output_var(Variable::new("vertices"));
    get_vertices.set_col_names(vec!["id".to_string(), "name".to_string()]);
    get_vertices.set_cost(1.0);

    assert_eq!(get_vertices.id(), 1);
    assert_eq!(get_vertices.kind(), PlanNodeKind::GetVertices);
    assert_eq!(get_vertices.col_names(), &vec!["id".to_string(), "name".to_string()]);
    assert_eq!(get_vertices.cost(), 1.0);

    let mut get_edges = GetEdges::new(2, 101, "src", "edge1", "rank", "dst");
    get_edges.set_output_var(Variable::new("edges"));
    get_edges.set_col_names(vec!["src".to_string(), "dst".to_string()]);
    get_edges.set_cost(2.0);

    assert_eq!(get_edges.id(), 2);
    assert_eq!(get_edges.kind(), PlanNodeKind::GetEdges);
    assert_eq!(get_edges.col_names(), &vec!["src".to_string(), "dst".to_string()]);
    assert_eq!(get_edges.cost(), 2.0);
}

#[test]
fn test_plan_node_visitor() {
    // 创建节点并使用访问者
    let mut get_vertices = GetVertices::new(1, 101, "vid");
    get_vertices.set_output_var(Variable::new("vertices"));

    let mut visitor = CountingVisitor::new();
    let result = get_vertices.accept(&mut visitor);
    
    assert!(result.is_ok());
    assert_eq!(visitor.get_vertices_count, 1);
    assert_eq!(visitor.get_edges_count, 0);
}

#[test]
fn test_default_plan_node_visitor() {
    // 测试默认访问者
    let mut project = Project::new(3, "YIELD 1 AS value");
    project.set_output_var(Variable::new("project_result"));

    let mut visitor = DefaultPlanNodeVisitor;
    let result = project.accept(&mut visitor);
    
    assert!(result.is_ok());
}

#[test]
fn test_plan_node_cloning() {
    // 测试节点克隆功能
    let mut original = GetVertices::new(1, 101, "vid");
    original.set_output_var(Variable::new("vertices"));
    original.set_col_names(vec!["id".to_string()]);
    original.set_cost(5.0);

    let cloned_node = original.clone_plan_node();
    
    assert_eq!(cloned_node.id(), original.id());
    assert_eq!(cloned_node.kind(), original.kind());
    assert_eq!(cloned_node.col_names(), original.col_names());
    assert_eq!(cloned_node.cost(), original.cost());
}

#[test]
fn test_plan_node_dependencies() {
    // 测试节点依赖功能
    let mut get_vertices = GetVertices::new(1, 101, "vid");
    get_vertices.set_output_var(Variable::new("vertices"));
    
    let mut filter = Filter::new(2, "col > 10");
    filter.set_output_var(Variable::new("filtered"));
    
    // 为filter节点添加依赖
    let deps = vec![get_vertices.clone_plan_node()];
    filter.set_dependencies(deps);
    
    assert_eq!(filter.dependencies().len(), 1);
    assert_eq!(filter.dependencies()[0].id(), 1);
}