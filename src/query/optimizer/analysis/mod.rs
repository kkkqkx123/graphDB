//! 计划分析模块
//!
//! 提供查询计划分析功能，支持优化决策：
//! - 引用计数分析：识别被多次引用的子计划
//! - 表达式分析：分析表达式特性（确定性、复杂度等）
//! - 指纹计算：计算计划节点的结构指纹
//!
//! # 模块位置说明
//!
//! 本模块位于 `src/query/optimizer/analysis/`，与 `cost` 模块同级。
//! 放在 optimizer 而非 planner 的原因：
//! 1. 职责分离：planner 负责生成计划，optimizer 负责优化计划
//! 2. 按需计算：仅在优化阶段根据需要进行分析
//! 3. 依赖关系：optimizer 已依赖 planner，不会引入循环依赖
//!
//! # 使用示例
//!
//! ```rust
//! use crate::query::optimizer::analysis::{
//!     ReferenceCountAnalyzer,
//!     ExpressionAnalyzer,
//! };
//!
//! // 引用计数分析
//! let ref_analyzer = ReferenceCountAnalyzer::new();
//! let ref_analysis = ref_analyzer.analyze(plan.root());
//!
//! // 表达式分析
//! let expr_analyzer = ExpressionAnalyzer::new();
//! let expr_analysis = expr_analyzer.analyze(condition);
//! ```

pub mod expression;
pub mod fingerprint;
pub mod reference_count;

// 重新导出主要类型
pub use expression::{
    AnalysisOptions, ExpressionAnalysis, ExpressionAnalyzer, NondeterministicChecker,
};
pub use fingerprint::{FingerprintCalculator, PlanFingerprint};
pub use reference_count::{
    ReferenceCountAnalysis, ReferenceCountAnalyzer, SubplanId, SubplanReferenceInfo,
};
