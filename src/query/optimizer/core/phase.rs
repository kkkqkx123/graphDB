//! 优化阶段定义
//! 定义优化过程的各个阶段和规则优先级

#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationPhase {
    LogicalOptimization,
    PhysicalOptimization,
    PostOptimization,
}

impl Default for OptimizationPhase {
    fn default() -> Self {
        Self::LogicalOptimization
    }
}

#[derive(Debug)]
pub struct RulePriority {
    pub phase: OptimizationPhase,
    pub priority: u32,
    pub rule_name: String,
}

impl RulePriority {
    pub fn new(phase: OptimizationPhase, priority: u32, rule_name: String) -> Self {
        Self {
            phase,
            priority,
            rule_name,
        }
    }
}
