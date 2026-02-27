//! 优化决策模块
//!
//! 提供从 AST 到物理执行计划的中间表示——优化决策，
//! 以及基于决策的缓存机制。
//!
//! ## 设计目标
//!
//! 1. **内存效率**：缓存决策而非完整计划树，减少内存占用
//! 2. **版本感知**：支持统计信息和索引版本检查，智能失效
//! 3. **可适应性**：基于决策重新构建计划，适应数据变化
//!
//! ## 使用示例
//!
//! ```rust
//! use graphdb::query::optimizer::decision::{
//!     DecisionCache, DecisionCacheConfig, DecisionCacheKey,
//!     OptimizationDecision, TraversalStartDecision, AccessPath, EntityType,
//! };
//!
//! // 创建缓存
//! let cache = DecisionCache::with_default_config()?;
//!
//! // 创建缓存键
//! let key = DecisionCacheKey::new(
//!     DecisionCacheKey::hash_template("MATCH (n:Person) WHERE n.age > $1 RETURN n"),
//!     Some(1),
//!     SentenceKind::Match,
//!     None,
//! );
//!
//! // 获取或计算决策
//! let decision = cache.get_or_compute(
//!     key,
//!     stats.version(),
//!     index.version(),
//!     || compute_decision(),
//! )?;
//! ```

// 类型定义
pub mod types;

// 缓存实现
pub mod cache;

// 重新导出主要类型
pub use types::{
    AccessPath,
    EntityType,
    EntityIndexChoice,
    IndexChoice,
    IndexSelectionDecision,
    JoinAlgorithm,
    JoinOrderDecision,
    OptimizationDecision,
    RewriteRuleId,
    TraversalStartDecision,
};

pub use cache::{
    CachedDecision,
    DecisionCache,
    DecisionCacheConfig,
    DecisionCacheError,
    DecisionCacheKey,
    DecisionCacheStats,
};
