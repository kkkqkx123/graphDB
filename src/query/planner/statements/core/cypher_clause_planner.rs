//! Cypher子句规划器接口
//! 架构重构：统一接口设计，优化数据流管理和上下文传递
//!
//! ## 重构理由
//!
//! ### 删除冗余方法
//! - `validate_input`, `can_start_flow`, `requires_input` 等方法功能重复，通过 `flow_direction()` 统一表达
//! - 现有实现中这些方法的逻辑存在冲突，增加了维护负担
//!
//! ### 简化类型系统
//! - 删除 `VariableRequirement` 和 `VariableProvider`：过度设计，实际使用中未被有效利用
//! - 使用 `VariableInfo` 替代：提供更准确的类型信息和生命周期管理
//!
//! ### 优化验证机制
//! - 删除复杂 `DataFlowValidator`：验证逻辑分散，违反单一职责原则
//! - 使用 `DataFlowManager`：内聚验证逻辑，提高可维护性
//!
//! ### 改进上下文管理
//! - 使用 `PlanningContext` 替代分散的上下文结构
//! - 通过 `ContextPropagator` 实现统一的上下文传播机制

use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use std::collections::HashMap;

/// 数据流方向
/// 定义子句在数据流中的角色，统一替代多个布尔方法
#[derive(Debug, Clone, PartialEq)]
pub enum FlowDirection {
    Source,    // 数据源：MATCH, LOOKUP - 不需要输入，开始数据流
    Transform, // 转换：WHERE, WITH, UNWIND - 需要输入，产生输出
    Output,    // 输出：RETURN, YIELD - 需要输入，结束数据流
}

/// 子句类型
/// 精确定义每种 Cypher 子句的类型和语义
#[derive(Debug, Clone, PartialEq)]
pub enum ClauseType {
    Match,   // MATCH子句：图模式匹配
    Where,   // WHERE子句：条件过滤
    Return,  // RETURN子句：结果输出
    With,    // WITH子句：管道传递
    OrderBy, // ORDER BY子句：结果排序
    Limit,   // LIMIT子句：结果限制
    Skip,    // SKIP子句：结果跳过
    Yield,   // YIELD子句：子查询输出
    Unwind,  // UNWIND子句：展开集合
}

impl ClauseType {
    /// 获取子句对应的数据流方向
    /// 统一替代 can_start_flow() 和 requires_input() 方法
    pub fn flow_direction(&self) -> FlowDirection {
        match self {
            ClauseType::Match => FlowDirection::Source,
            ClauseType::Where => FlowDirection::Transform,
            ClauseType::Return => FlowDirection::Output,
            ClauseType::With => FlowDirection::Transform,
            ClauseType::OrderBy => FlowDirection::Transform,
            ClauseType::Limit => FlowDirection::Transform,
            ClauseType::Skip => FlowDirection::Transform,
            ClauseType::Yield => FlowDirection::Output,
            ClauseType::Unwind => FlowDirection::Transform,
        }
    }
}

/// 变量信息
/// 替代 VariableRequirement 和 VariableProvider，提供完整的变量生命周期管理
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,              // 变量名称
    pub var_type: String,          // 变量类型（Vertex, Edge, Path, Property等）
    pub source_clause: ClauseType, // 产生该变量的子句类型
    pub is_output: bool,           // 是否为输出变量
}

/// 查询信息
/// 提供查询级别的元信息，用于上下文管理
#[derive(Debug, Clone)]
pub struct QueryInfo {
    pub query_id: String,       // 查询唯一标识
    pub statement_type: String, // 语句类型（MATCH, WHERE等）
}

/// 规划上下文
/// 统一的上下文管理，替代分散的上下文结构
#[derive(Debug, Clone)]
pub struct PlanningContext {
    pub query_info: QueryInfo,                    // 查询级别信息
    pub variables: HashMap<String, VariableInfo>, // 变量映射表
    pub types: HashMap<String, String>,           // 类型信息表
}

impl PlanningContext {
    pub fn new(query_info: QueryInfo) -> Self {
        Self {
            query_info,
            variables: HashMap::new(),
            types: HashMap::new(),
        }
    }

    /// 添加变量到上下文
    /// 替代原有的 add_variable 方法，提供更完整的变量信息
    pub fn add_variable(&mut self, variable: VariableInfo) {
        self.variables.insert(variable.name.clone(), variable);
    }

    /// 检查变量是否存在
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 获取变量信息
    pub fn get_variable(&self, name: &str) -> Option<&VariableInfo> {
        self.variables.get(name)
    }

    /// 获取所有变量
    pub fn get_variables(&self) -> &HashMap<String, VariableInfo> {
        &self.variables
    }

    /// 标记所有变量为输出变量
    /// 用于 RETURN 子句处理
    pub fn mark_output_variables(&mut self) {
        for (_, variable) in self.variables.iter_mut() {
            variable.is_output = true;
        }
    }

    /// 重置变量作用域
    /// 用于 WITH 子句处理，只保留输出变量
    pub fn reset_variable_scope(&mut self) {
        self.variables.retain(|_, variable| variable.is_output);
    }
}

/// 数据流节点特征
/// 定义数据流节点的基本行为，替代复杂的验证接口
pub trait DataFlowNode {
    /// 获取数据流方向
    /// 统一替代 can_start_flow() 和 requires_input() 方法
    fn flow_direction(&self) -> FlowDirection;

    /// 是否需要输入
    /// 基于 flow_direction() 的派生方法
    fn requires_input(&self) -> bool {
        !matches!(self.flow_direction(), FlowDirection::Source)
    }

    /// 验证数据流
    /// 简化的验证逻辑，内聚到接口中
    fn validate_flow(&self, input: Option<&SubPlan>) -> Result<(), PlannerError> {
        if self.requires_input() {
            if input.is_none() {
                return Err(PlannerError::PlanGenerationFailed(format!(
                    "{:?} clause requires input",
                    self.flow_direction()
                )));
            }
        } else {
            if input.is_some() {
                return Err(PlannerError::PlanGenerationFailed(format!(
                    "{:?} clause does not require input",
                    self.flow_direction()
                )));
            }
        }
        Ok(())
    }
}

/// 子句规划器接口
/// 重构后的统一接口，删除冗余方法，保留核心功能
pub trait CypherClausePlanner: DataFlowNode {
    /// 转换子句为执行计划
    /// 核心方法：将子句上下文转换为可执行的 SubPlan
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError>;

    /// 获取子句类型
    /// 必需方法：用于数据流验证和上下文传播
    fn clause_type(&self) -> ClauseType;

    /// 实现数据流方向
    /// 基于 clause_type() 的默认实现
    fn flow_direction(&self) -> FlowDirection {
        self.clause_type().flow_direction()
    }
}

/// 数据流管理器
/// 替代复杂的 DataFlowValidator，提供内聚的数据流管理功能
pub struct DataFlowManager;

impl DataFlowManager {
    /// 验证子句序列的数据流
    /// 简化的验证逻辑，专注于核心数据流规则
    pub fn validate_clause_sequence(
        clauses: &[&dyn CypherClausePlanner],
    ) -> Result<(), PlannerError> {
        if clauses.is_empty() {
            return Ok(());
        }

        // 第一个子句必须是数据源
        if !matches!(
            DataFlowNode::flow_direction(clauses[0]),
            FlowDirection::Source
        ) {
            return Err(PlannerError::PlanGenerationFailed(
                "First clause must be a data source".to_string(),
            ));
        }

        // 验证后续子句的数据流
        for clause in clauses {
            clause.validate_flow(None)?;
        }

        // 最后一个子句应该是输出
        let last_clause = clauses.last().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("Clause list should not be empty".to_string())
        })?;
        if !matches!(
            DataFlowNode::flow_direction(*last_clause),
            FlowDirection::Output
        ) {
            return Err(PlannerError::PlanGenerationFailed(
                "Last clause should be an output clause".to_string(),
            ));
        }

        Ok(())
    }
}

/// 上下文传播器
/// 统一的上下文传播机制，替代分散的上下文管理逻辑
pub struct ContextPropagator;

impl ContextPropagator {
    /// 传播上下文到子句
    /// 根据子句类型调整上下文状态
    pub fn propagate_to_clause(
        &self,
        source_context: &PlanningContext,
        clause_type: ClauseType,
    ) -> PlanningContext {
        let mut clause_context = source_context.clone();

        match clause_type {
            ClauseType::Match => {
                // MATCH子句不需要特殊处理
            }
            ClauseType::Where => {
                // WHERE子句继承所有变量
            }
            ClauseType::Return => {
                // RETURN子句标记输出变量
                clause_context.mark_output_variables();
            }
            ClauseType::With => {
                // WITH子句重置变量作用域
                clause_context.reset_variable_scope();
            }
            _ => {
                // 其他子句类型保持默认行为
            }
        }

        clause_context
    }

    /// 合并上下文
    /// 用于处理复杂的查询场景，如子查询和联合查询
    pub fn merge_contexts(
        &self,
        contexts: &[&PlanningContext],
    ) -> Result<PlanningContext, PlannerError> {
        if contexts.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "Cannot merge empty contexts".to_string(),
            ));
        }

        let mut merged = contexts[0].clone();

        for context in &contexts[1..] {
            Self::merge_variables(&mut merged, context)?;
            Self::merge_types(&mut merged, context)?;
        }

        Ok(merged)
    }

    /// 合并变量信息
    /// 检查类型兼容性并合并变量映射
    fn merge_variables(
        target: &mut PlanningContext,
        source: &PlanningContext,
    ) -> Result<(), PlannerError> {
        for (name, variable) in &source.variables {
            if let Some(existing) = target.variables.get(name) {
                // 检查类型兼容性
                if existing.var_type != variable.var_type {
                    return Err(PlannerError::PlanGenerationFailed(format!(
                        "Variable {} has incompatible types",
                        name
                    )));
                }
            } else {
                target.variables.insert(name.clone(), variable.clone());
            }
        }
        Ok(())
    }

    /// 合并类型信息
    /// 合并类型映射表
    fn merge_types(
        target: &mut PlanningContext,
        source: &PlanningContext,
    ) -> Result<(), PlannerError> {
        for (name, type_info) in &source.types {
            target.types.insert(name.clone(), type_info.clone());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clause_type_flow_direction() {
        assert_eq!(ClauseType::Match.flow_direction(), FlowDirection::Source);
        assert_eq!(ClauseType::Where.flow_direction(), FlowDirection::Transform);
        assert_eq!(ClauseType::Return.flow_direction(), FlowDirection::Output);
        assert_eq!(ClauseType::With.flow_direction(), FlowDirection::Transform);
    }

    #[test]
    fn test_planning_context() {
        let query_info = QueryInfo {
            query_id: "test".to_string(),
            statement_type: "MATCH".to_string(),
        };
        let mut context = PlanningContext::new(query_info);

        assert!(!context.has_variable("test"));

        let variable = VariableInfo {
            name: "test".to_string(),
            var_type: "Vertex".to_string(),
            source_clause: ClauseType::Match,
            is_output: false,
        };

        context.add_variable(variable);
        assert!(context.has_variable("test"));
    }

    #[test]
    fn test_data_flow_manager() {
        // 这个测试需要具体的规划器实现
        // 在实际实现中添加
    }
}
