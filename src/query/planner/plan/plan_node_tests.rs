#[cfg(test)]
mod plan_node_tests {
    use crate::query::planner::plan::{
        PlanNode, PlanNodeKind, 
        GetVertices, GetEdges, Project, Filter, Dedup, 
        ScanEdges, ScanVertices, IndexScan, FulltextIndexScan,
        PlanNodeVisitor, PlanNodeVisitError
    };
    use crate::query::validator::Variable;

    // 简单的访问者实现用于测试
    #[derive(Debug)]
    struct TestVisitor {
        visited_nodes: Vec<String>,
    }

    impl TestVisitor {
        fn new() -> Self {
            Self {
                visited_nodes: Vec::new(),
            }
        }
    }

    impl PlanNodeVisitor for TestVisitor {
        fn pre_visit(&mut self) -> Result<(), PlanNodeVisitError> {
            self.visited_nodes.push("pre_visit".to_string());
            Ok(())
        }

        fn visit_get_vertices(&mut self, _node: &GetVertices) -> Result<(), PlanNodeVisitError> {
            self.visited_nodes.push("GetVertices".to_string());
            Ok(())
        }

        fn visit_get_edges(&mut self, _node: &GetEdges) -> Result<(), PlanNodeVisitError> {
            self.visited_nodes.push("GetEdges".to_string());
            Ok(())
        }

        fn visit_project(&mut self, _node: &Project) -> Result<(), PlanNodeVisitError> {
            self.visited_nodes.push("Project".to_string());
            Ok(())
        }

        fn visit_filter(&mut self, _node: &Filter) -> Result<(), PlanNodeVisitError> {
            self.visited_nodes.push("Filter".to_string());
            Ok(())
        }

        fn visit_dedup(&mut self, _node: &Dedup) -> Result<(), PlanNodeVisitError> {
            self.visited_nodes.push("Dedup".to_string());
            Ok(())
        }

        fn visit_scan_edges(&mut self, _node: &ScanEdges) -> Result<(), PlanNodeVisitError> {
            self.visited_nodes.push("ScanEdges".to_string());
            Ok(())
        }

        fn visit_scan_vertices(&mut self, _node: &ScanVertices) -> Result<(), PlanNodeVisitError> {
            self.visited_nodes.push("ScanVertices".to_string());
            Ok(())
        }

        fn visit_index_scan(&mut self, _node: &IndexScan) -> Result<(), PlanNodeVisitError> {
            self.visited_nodes.push("IndexScan".to_string());
            Ok(())
        }

        fn visit_fulltext_index_scan(&mut self, _node: &FulltextIndexScan) -> Result<(), PlanNodeVisitError> {
            self.visited_nodes.push("FulltextIndexScan".to_string());
            Ok(())
        }

        fn post_visit(&mut self) -> Result<(), PlanNodeVisitError> {
            self.visited_nodes.push("post_visit".to_string());
            Ok(())
        }
    }

    #[test]
    fn test_get_vertices_node() {
        let mut node = GetVertices::new(1, 101, "var1");
        node.set_output_var(Variable::new("result"));
        
        let mut visitor = TestVisitor::new();
        let result = node.accept(&mut visitor);
        
        assert!(result.is_ok());
        assert!(visitor.visited_nodes.contains(&"pre_visit".to_string()));
        assert!(visitor.visited_nodes.contains(&"GetVertices".to_string()));
        assert!(visitor.visited_nodes.contains(&"post_visit".to_string()));
    }

    #[test]
    fn test_get_edges_node() {
        let mut node = GetEdges::new(2, 101, "src", "edge_type", "rank", "dst");
        node.set_output_var(Variable::new("edges"));
        
        let mut visitor = TestVisitor::new();
        let result = node.accept(&mut visitor);
        
        assert!(result.is_ok());
        assert!(visitor.visited_nodes.contains(&"pre_visit".to_string()));
        assert!(visitor.visited_nodes.contains(&"GetEdges".to_string()));
        assert!(visitor.visited_nodes.contains(&"post_visit".to_string()));
    }

    #[test]
    fn test_project_node() {
        let mut node = Project::new(3, "YIELD 1 AS col1");
        node.set_output_var(Variable::new("project_result"));
        
        let mut visitor = TestVisitor::new();
        let result = node.accept(&mut visitor);
        
        assert!(result.is_ok());
        assert!(visitor.visited_nodes.contains(&"pre_visit".to_string()));
        assert!(visitor.visited_nodes.contains(&"Project".to_string()));
        assert!(visitor.visited_nodes.contains(&"post_visit".to_string()));
    }

    #[test]
    fn test_filter_node() {
        let mut node = Filter::new(4, "col1 > 10");
        node.set_output_var(Variable::new("filter_result"));
        
        let mut visitor = TestVisitor::new();
        let result = node.accept(&mut visitor);
        
        assert!(result.is_ok());
        assert!(visitor.visited_nodes.contains(&"pre_visit".to_string()));
        assert!(visitor.visited_nodes.contains(&"Filter".to_string()));
        assert!(visitor.visited_nodes.contains(&"post_visit".to_string()));
    }

    #[test]
    fn test_scan_edges_node() {
        let mut node = ScanEdges::new(5, 101, "edge_type");
        node.set_output_var(Variable::new("scan_edges_result"));
        
        let mut visitor = TestVisitor::new();
        let result = node.accept(&mut visitor);
        
        assert!(result.is_ok());
        assert!(visitor.visited_nodes.contains(&"pre_visit".to_string()));
        assert!(visitor.visited_nodes.contains(&"ScanEdges".to_string()));
        assert!(visitor.visited_nodes.contains(&"post_visit".to_string()));
    }

    #[test]
    fn test_node_properties() {
        let mut node = GetVertices::new(6, 101, "var1");
        node.set_col_names(vec!["col1".to_string(), "col2".to_string()]);
        node.set_cost(10.5);
        node.set_output_var(Variable::new("test_var"));
        
        assert_eq!(node.id(), 6);
        assert_eq!(node.kind(), PlanNodeKind::GetVertices);
        assert_eq!(node.col_names(), &vec!["col1".to_string(), "col2".to_string()]);
        assert_eq!(node.cost(), 10.5);
        assert_eq!(node.output_var().as_ref().unwrap().name, "test_var");
    }
}