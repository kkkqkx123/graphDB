//! 测试新的规划器接口和注册机制

use graphdb::query::planner::match_planning::core::{
    CypherClausePlanner, ClauseType, PlanningContext, DataFlowValidator, FlowDirection
};
use graphdb::query::planner::match_planning::clauses::{
    ReturnClausePlannerV2, WhereClausePlannerV2, WithClausePlannerV2
};
use graphdb::query::planner::match_planning::MatchPlannerV2;
use graphdb::query::planner::planner_v2::{
    PlannerRegistry, SentenceKind, MatchAndInstantiate, SequentialPlannerV2
};
use graphdb::query::planner::plan::SubPlan;
use graphdb::query::context::ast::AstContext;

/// 测试用的 Cypher 子句规划器实现
#[derive(Debug)]
struct TestCypherClausePlanner {
    clause_type: ClauseType,
    should_fail: bool,
}

impl TestCypherClausePlanner {
    fn new(clause_type: ClauseType) -> Self {
        Self {
            clause_type,
            should_fail: false,
        }
    }
    
    fn new_with_failure(clause_type: ClauseType) -> Self {
        Self {
            clause_type,
            should_fail: true,
        }
    }
}

impl CypherClausePlanner for TestCypherClausePlanner {
    fn transform(
        &self,
        _clause_ctx: &graphdb::query::validator::structs::common_structs::CypherClauseContext,
        input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, graphdb::query::planner::planner::PlannerError> {
        if self.should_fail {
            return Err(graphdb::query::planner::planner::PlannerError::PlanGenerationFailed(
                "Test failure".to_string(),
            ));
        }
        
        // 验证输入要求
        self.validate_input(input_plan)?;
        
        // 创建一个简单的测试计划
        Ok(SubPlan::new(None, None))
    }
    
    fn validate_input(&self, input_plan: Option<&SubPlan>) -> Result<(), graphdb::query::planner::planner::PlannerError> {
        if self.requires_input() && input_plan.is_none() {
            return Err(graphdb::query::planner::planner::PlannerError::MissingInput(
                format!("{:?} clause requires input", self.clause_type())
            ));
        }
        Ok(())
    }
    
    fn clause_type(&self) -> ClauseType {
        self.clause_type.clone()
    }
    
    fn can_start_flow(&self) -> bool {
        matches!(self.clause_type, ClauseType::Source)
    }
    
    fn requires_input(&self) -> bool {
        !self.can_start_flow()
    }
}

/// 测试用的规划器实现
#[derive(Debug)]
struct TestPlanner {
    should_fail: bool,
}

impl TestPlanner {
    fn new() -> Self {
        Self { should_fail: false }
    }
    
    fn new_with_failure() -> Self {
        Self { should_fail: true }
    }
    
    fn make() -> Box<dyn graphdb::query::planner::planner_v2::Planner> {
        Box::new(Self::new())
    }
    
    fn match_ast_ctx(_ast_ctx: &AstContext) -> bool {
        true
    }
}

impl graphdb::query::planner::planner_v2::Planner for TestPlanner {
    fn transform(&mut self, _ast_ctx: &AstContext) -> Result<SubPlan, graphdb::query::planner::planner::PlannerError> {
        if self.should_fail {
            Err(graphdb::query::planner::planner::PlannerError::PlanGenerationFailed(
                "Test planner failure".to_string(),
            ))
        } else {
            Ok(SubPlan::new(None, None))
        }
    }
    
    fn match_planner(&self, _ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(_ast_ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clause_type_enum() {
        assert!(ClauseType::Source.is_source());
        assert!(ClauseType::Output.is_output());
        assert!(ClauseType::Transform.is_transform());
        assert!(ClauseType::Modifier.is_modifier());
        
        assert!(!ClauseType::Source.is_output());
        assert!(!ClauseType::Output.is_source());
    }
    
    #[test]
    fn test_cypher_clause_planner_interface() {
        let source_planner = TestCypherClausePlanner::new(ClauseType::Source);
        assert_eq!(source_planner.clause_type(), ClauseType::Source);
        assert!(source_planner.can_start_flow());
        assert!(!source_planner.requires_input());
        
        let output_planner = TestCypherClausePlanner::new(ClauseType::Output);
        assert_eq!(output_planner.clause_type(), ClauseType::Output);
        assert!(!output_planner.can_start_flow());
        assert!(output_planner.requires_input());
    }
    
    #[test]
    fn test_cypher_clause_planner_validation() {
        let source_planner = TestCypherClausePlanner::new(ClauseType::Source);
        let output_planner = TestCypherClausePlanner::new(ClauseType::Output);
        
        // 源子句不需要输入
        assert!(source_planner.validate_input(None).is_ok());
        
        // 输出子句需要输入
        assert!(output_planner.validate_input(None).is_err());
        assert!(output_planner.validate_input(Some(&SubPlan::new(None, None))).is_ok());
    }
    
    #[test]
    fn test_sentence_kind_from_str() {
        assert_eq!(SentenceKind::from_str("MATCH").unwrap(), SentenceKind::Match);
        assert_eq!(SentenceKind::from_str("match").unwrap(), SentenceKind::Match);
        assert_eq!(SentenceKind::from_str("GO").unwrap(), SentenceKind::Go);
        assert_eq!(SentenceKind::from_str("FETCH VERTICES").unwrap(), SentenceKind::FetchVertices);
        
        assert!(SentenceKind::from_str("INVALID").is_err());
    }
    
    #[test]
    fn test_planner_registry() {
        let mut registry = PlannerRegistry::new();
        assert_eq!(registry.planner_count(), 0);
        
        // 注册测试规划器
        registry.register_planner(
            SentenceKind::Match,
            TestPlanner::match_ast_ctx,
            TestPlanner::make,
            100,
        );
        
        assert_eq!(registry.planner_count(), 1);
        assert!(registry.has_planners_for(&SentenceKind::Match));
        assert_eq!(registry.planner_count_for(&SentenceKind::Match), 1);
        assert!(!registry.has_planners_for(&SentenceKind::Go));
        assert_eq!(registry.planner_count_for(&SentenceKind::Go), 0);
    }
    
    #[test]
    fn test_planner_registry_priority() {
        let mut registry = PlannerRegistry::new();
        
        // 注册不同优先级的规划器
        registry.register_planner(
            SentenceKind::Match,
            TestPlanner::match_ast_ctx,
            TestPlanner::make,
            50,
        );
        
        registry.register_planner(
            SentenceKind::Match,
            TestPlanner::match_ast_ctx,
            TestPlanner::make,
            100,
        );
        
        assert_eq!(registry.planner_count_for(&SentenceKind::Match), 2);
        
        // 验证规划器按优先级排序（高优先级在前）
        let planners = registry.planners.get(&SentenceKind::Match).unwrap();
        assert_eq!(planners[0].priority, 100);
        assert_eq!(planners[1].priority, 50);
    }
    
    #[test]
    fn test_sequential_planner_v2() {
        let planner = SequentialPlannerV2::new();
        assert!(SequentialPlannerV2::match_ast_ctx(&AstContext::new("test", "test")));
    }
    
    #[test]
    fn test_data_flow_validator() {
        // 测试数据流转换验证
        assert!(DataFlowValidator::is_valid_flow_transition(
            FlowDirection::Source,
            FlowDirection::Transform
        ));
        
        assert!(DataFlowValidator::is_valid_flow_transition(
            FlowDirection::Transform,
            FlowDirection::Output
        ));
        
        assert!(!DataFlowValidator::is_valid_flow_transition(
            FlowDirection::Output,
            FlowDirection::Transform
        ));
    }
    
    #[test]
    fn test_return_clause_planner_v2() {
        let planner = ReturnClausePlannerV2::new();
        assert_eq!(planner.clause_type(), ClauseType::Output);
        assert!(!planner.can_start_flow());
        assert!(planner.requires_input());
        
        // 测试输入验证
        assert!(planner.validate_input(None).is_err());
        assert!(planner.validate_input(Some(&SubPlan::new(None, None))).is_ok());
    }
    
    #[test]
    fn test_planning_context() {
        let query_ctx = AstContext::new("test", "test");
        let mut context = PlanningContext::new(query_ctx);
        
        assert!(!context.has_variable("test"));
        
        context.add_variable("test".to_string());
        assert!(context.has_variable("test"));
        assert!(context.get_available_variables().contains("test"));
    }
    
    #[test]
    fn test_error_types() {
        let missing_input_error = graphdb::query::planner::planner::PlannerError::MissingInput("Test input".to_string());
        assert!(matches!(missing_input_error, graphdb::query::planner::planner::PlannerError::PlanGenerationFailed(_)));
        
        let missing_var_error = graphdb::query::planner::planner::PlannerError::MissingVariable("Test var".to_string());
        assert!(matches!(missing_var_error, graphdb::query::planner::planner::PlannerError::PlanGenerationFailed(_)));
    }
    
    #[test]
    fn test_where_clause_planner_v2() {
        let planner = WhereClausePlannerV2::new(false);
        assert_eq!(planner.clause_type(), ClauseType::Transform);
        assert!(!planner.can_start_flow());
        assert!(planner.requires_input());
        
        // 测试输入验证
        assert!(planner.validate_input(None).is_err());
        assert!(planner.validate_input(Some(&SubPlan::new(None, None))).is_ok());
    }
    
    #[test]
    fn test_with_clause_planner_v2() {
        let planner = WithClausePlannerV2::new();
        assert_eq!(planner.clause_type(), ClauseType::Transform);
        assert!(!planner.can_start_flow());
        assert!(planner.requires_input());
        
        // 测试输入验证
        assert!(planner.validate_input(None).is_err());
        assert!(planner.validate_input(Some(&SubPlan::new(None, None))).is_ok());
    }
    
    #[test]
    fn test_match_planner_v2() {
        let planner = MatchPlannerV2::make();
        assert!(planner.match_planner(&AstContext::new("MATCH", "MATCH (n)")));
        assert!(!planner.match_planner(&AstContext::new("GO", "GO 1 TO 2")));
    }
    
    #[test]
    fn test_planner_registry_with_match_planner() {
        let mut registry = PlannerRegistry::new();
        registry.register_match_planners();
        
        assert_eq!(registry.planner_count(), 1);
        assert!(registry.has_planners_for(&SentenceKind::Match));
        assert_eq!(registry.planner_count_for(&SentenceKind::Match), 1);
    }
}