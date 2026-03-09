//! 执行器工厂模块
//!
//! 负责根据执行计划创建对应的执行器实例
//! 采用模块化设计，将职责拆分为多个子模块：
//! - parsers: 解析器，负责解析顶点ID、边方向、权重配置等
//! - validators: 验证器，负责验证计划节点、递归检测、安全验证
//! - builders: 构建器，负责创建各种类型的执行器
//! - executors: 执行器执行，负责执行执行计划

pub mod executor_factory;
pub mod builders;
pub mod parsers;
pub mod validators;
pub mod executors;

// 重新导出主要类型
pub use executor_factory::ExecutorFactory;
pub use executors::PlanExecutor;
pub use validators::{RecursionDetector, SafetyValidator};

// 从子模块重新导出安全配置
pub use validators::safety_validator::ExecutorSafetyConfig;
