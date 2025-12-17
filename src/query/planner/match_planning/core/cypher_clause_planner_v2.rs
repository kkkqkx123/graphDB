//! 新的 Cypher子句规划器接口
//! 参考 Nebula-Graph 的简洁设计，增加输入依赖支持

use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use std::collections::HashSet;

/// 子句类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ClauseType {
    Source,      // 数据源子句：MATCH, LOOKUP
    Transform,   // 转换子句：WHERE, WITH, UNWIND
    Output,      // 输出子句：RETURN, YIELD
    Modifier,    // 修饰子句：ORDER BY, LIMIT, SKIP
}

impl std::fmt::Display for ClauseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClauseType::Source => write!(f, "Source"),
            ClauseType::Transform => write!(f, "Transform"),
            ClauseType::Output => write!(f, "Output"),
            ClauseType::Modifier => write!(f, "Modifier"),
        }
    }
}

impl ClauseType {
    pub fn is_source(&self) -> bool {
        matches!(self, ClauseType::Source)
    }
    
    pub fn is_transform(&self) -> bool {
        matches!(self, ClauseType::Transform)
    }
    
    pub fn is_output(&self) -> bool {
        matches!(self, ClauseType::Output)
    }
    
    pub fn is_modifier(&self) -> bool {
        matches!(self, ClauseType::Modifier)
    }
}

/// 变量要求
#[derive(Debug, Clone)]
pub struct VariableRequirement {
    pub name: String,
    pub var_type: VariableType,
    pub required: bool,  // 是否必需
}

/// 变量类型
#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Vertex,
    Edge,
    Path,
    Property,
    Any,
}

/// 变量提供
#[derive(Debug, Clone)]
pub struct VariableProvider {
    pub name: String,
    pub var_type: VariableType,
    pub nullable: bool,  // 是否可为空
}

/// 规划上下文
#[derive(Debug)]
pub struct PlanningContext {
    query_context: crate::query::context::ast::AstContext,
    available_variables: HashSet<String>,
    generated_variables: HashSet<String>,
}

impl PlanningContext {
    pub fn new(query_context: crate::query::context::ast::AstContext) -> Self {
        Self {
            query_context,
            available_variables: HashSet::new(),
            generated_variables: HashSet::new(),
        }
    }
    
    pub fn query_context(&self) -> &crate::query::context::ast::AstContext {
        &self.query_context
    }
    
    pub fn add_variable(&mut self, name: String) {
        self.generated_variables.insert(name.clone());
        self.available_variables.insert(name);
    }
    
    pub fn has_variable(&self, name: &str) -> bool {
        self.available_variables.contains(name)
    }
    
    pub fn get_available_variables(&self) -> &HashSet<String> {
        &self.available_variables
    }
}

/// 新的子句规划器接口
/// 参考 Nebula-Graph 的简洁设计，增加输入依赖支持
pub trait CypherClausePlanner {
    /// 转换子句上下文为执行计划
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, crate::query::planner::planner::PlannerError>;
    
    /// 验证输入计划是否满足要求
    fn validate_input(&self, _input_plan: Option<&SubPlan>) -> Result<(), crate::query::planner::planner::PlannerError> {
        // 默认实现：任何输入都可以接受
        Ok(())
    }
    
    /// 获取子句类型
    fn clause_type(&self) -> ClauseType;
    
    /// 是否可以开始数据流
    fn can_start_flow(&self) -> bool {
        matches!(self.clause_type(), ClauseType::Source)
    }
    
    /// 是否需要输入
    fn requires_input(&self) -> bool {
        !self.can_start_flow()
    }
    
    /// 输入要求
    fn input_requirements(&self) -> Vec<VariableRequirement> {
        Vec::new()
    }
    
    /// 输出提供
    fn output_provides(&self) -> Vec<VariableProvider> {
        Vec::new()
    }
    
    /// 数据流方向
    fn flow_direction(&self) -> FlowDirection {
        match self.clause_type() {
            ClauseType::Source => FlowDirection::Source,
            ClauseType::Transform => FlowDirection::Transform,
            ClauseType::Output => FlowDirection::Output,
            ClauseType::Modifier => FlowDirection::Transform,
        }
    }
    
    /// 验证数据流
    fn validate_data_flow(&self, input: Option<&SubPlan>) -> Result<(), crate::query::planner::planner::PlannerError> {
        // 验证输入要求
        if self.requires_input() && input.is_none() {
            return Err(PlannerError::missing_input(
                format!("{:?} clause requires input", self.clause_type())
            ));
        }
        
        // 验证数据流方向
        if self.can_start_flow() && input.is_some() {
            // 起始子句不应该有输入
            return Err(crate::query::planner::planner::PlannerError::PlanGenerationFailed(
                format!("{:?} clause should not have input", self.clause_type())
            ));
        }
        
        Ok(())
    }
}

/// 数据流方向枚举
#[derive(Debug, Clone, PartialEq)]
pub enum FlowDirection {
    Source,     // 数据源：MATCH, LOOKUP
    Transform,  // 转换：WHERE, WITH, UNWIND
    Output,     // 输出：RETURN, YIELD
    Combine,    // 组合：UNION, JOIN
}

/// 数据流验证器
pub struct DataFlowValidator;

impl DataFlowValidator {
    /// 验证子句规划器的数据流
    pub fn validate_clause_flow(
        planner: &dyn CypherClausePlanner,
        input_plan: Option<&SubPlan>,
        context: &PlanningContext,
    ) -> Result<(), crate::query::planner::planner::PlannerError> {
        // 验证输入要求
        if planner.requires_input() && input_plan.is_none() {
            return Err(PlannerError::missing_input(
                format!("{:?} clause requires input", planner.clause_type())
            ));
        }
        
        // 验证数据流方向
        if planner.can_start_flow() && input_plan.is_some() {
            // 起始子句不应该有输入
            return Err(crate::query::planner::planner::PlannerError::PlanGenerationFailed(
                format!("{:?} clause should not have input", planner.clause_type())
            ));
        }
        
        // 验证变量依赖
        if let Some(input) = input_plan {
            Self::validate_variable_dependencies(planner, input, context)?;
        }
        
        Ok(())
    }
    
    /// 验证变量依赖
    fn validate_variable_dependencies(
        planner: &dyn CypherClausePlanner,
        input_plan: &SubPlan,
        _context: &PlanningContext,
    ) -> Result<(), crate::query::planner::planner::PlannerError> {
        let requirements = planner.input_requirements();
        
        for requirement in &requirements {
            if requirement.required {
                // 检查输入计划是否提供所需变量
                let input_vars: HashSet<String> = input_plan.root
                    .as_ref()
                    .map(|node| node.col_names().iter().cloned().collect())
                    .unwrap_or_default();
                
                if !input_vars.contains(&requirement.name) {
                    return Err(PlannerError::missing_variable(
                        format!("Required variable '{}' not found in input", requirement.name)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// 验证查询的数据流
    pub fn validate_query_flow(
        clauses: &[&dyn CypherClausePlanner],
        _context: &PlanningContext,
    ) -> Result<(), crate::query::planner::planner::PlannerError> {
        if clauses.is_empty() {
            return Ok(());
        }
        
        // 第一个子句必须是数据源
        if !clauses[0].can_start_flow() {
            return Err(crate::query::planner::planner::PlannerError::PlanGenerationFailed(
                "First clause must be a data source".to_string()
            ));
        }
        
        // 验证后续子句的数据流
        for i in 1..clauses.len() {
            let prev_clause = &clauses[i - 1];
            let current_clause = &clauses[i];
            
            // 检查数据流方向是否合理
            if !Self::is_valid_flow_transition(
                prev_clause.flow_direction(),
                current_clause.flow_direction(),
            ) {
                return Err(crate::query::planner::planner::PlannerError::PlanGenerationFailed(
                    format!(
                        "Invalid flow transition from {:?} to {:?}",
                        prev_clause.flow_direction(),
                        current_clause.flow_direction()
                    )
                ));
            }
        }
        
        // 最后一个子句应该是输出
        let last_clause = clauses.last().unwrap();
        if !matches!(last_clause.flow_direction(), FlowDirection::Output) {
            return Err(crate::query::planner::planner::PlannerError::PlanGenerationFailed(
                "Last clause should be an output clause".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// 检查数据流转换是否有效
    fn is_valid_flow_transition(
        from: FlowDirection,
        to: FlowDirection,
    ) -> bool {
        match (from, to) {
            // 数据源可以转换为任何类型
            (FlowDirection::Source, _) => true,
            // 转换可以转换为转换或输出
            (FlowDirection::Transform, FlowDirection::Transform) => true,
            (FlowDirection::Transform, FlowDirection::Output) => true,
            // 输出不能转换为其他类型（应该是最后一个）
            (FlowDirection::Output, _) => false,
            // 组合可以转换为转换或输出
            (FlowDirection::Combine, FlowDirection::Transform) => true,
            (FlowDirection::Combine, FlowDirection::Output) => true,
            // 其他情况无效
            _ => false,
        }
    }
}

// 为 PlannerError 添加新的错误类型
impl crate::query::planner::planner::PlannerError {
    pub fn missing_input(message: String) -> Self {
        crate::query::planner::planner::PlannerError::PlanGenerationFailed(format!("Missing input: {}", message))
    }
    
    pub fn missing_variable(message: String) -> Self {
        crate::query::planner::planner::PlannerError::PlanGenerationFailed(format!("Missing variable: {}", message))
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
    fn test_planning_context() {
        let query_ctx = crate::query::context::ast::AstContext::new("test", "test");
        let mut context = PlanningContext::new(query_ctx);
        
        assert!(!context.has_variable("test"));
        
        context.add_variable("test".to_string());
        assert!(context.has_variable("test"));
        assert!(context.get_available_variables().contains("test"));
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
}