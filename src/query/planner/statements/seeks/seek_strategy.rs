//! 查找策略模块
//!
//! 定义顶点查找策略和选择器，用于 MATCH 查询中确定起始顶点的查找方式

use crate::core::StorageError;
use crate::storage::StorageClient;

use super::index_seek::IndexSeek;
use super::scan_seek::ScanSeek;
use super::seek_strategy_base::{
    SeekResult, SeekStrategyContext,
    SeekStrategySelector, SeekStrategyType,
};
use super::vertex_seek::VertexSeek;

pub type SeekStrategyTraitObject = dyn SeekStrategy + Send + Sync;

pub trait SeekStrategy: Send + Sync {
    fn execute(
        &self,
        storage: &dyn StorageClient,
        context: &SeekStrategyContext,
    ) -> Result<SeekResult, StorageError>;

    fn estimated_cost(&self, context: &SeekStrategyContext) -> f64;

    fn supports(&self, context: &SeekStrategyContext) -> bool;
}

pub enum AnySeekStrategy {
    VertexSeek(VertexSeek),
    IndexSeek(IndexSeek),
    ScanSeek(ScanSeek),
}

impl Clone for AnySeekStrategy {
    fn clone(&self) -> Self {
        match self {
            AnySeekStrategy::VertexSeek(v) => AnySeekStrategy::VertexSeek(v.clone()),
            AnySeekStrategy::IndexSeek(i) => AnySeekStrategy::IndexSeek(i.clone()),
            AnySeekStrategy::ScanSeek(s) => AnySeekStrategy::ScanSeek(s.clone()),
        }
    }
}

impl SeekStrategy for AnySeekStrategy {
    fn execute(
        &self,
        storage: &dyn StorageClient,
        context: &SeekStrategyContext,
    ) -> Result<SeekResult, StorageError> {
        match self {
            AnySeekStrategy::VertexSeek(s) => s.execute(storage, context),
            AnySeekStrategy::IndexSeek(s) => s.execute(storage, context),
            AnySeekStrategy::ScanSeek(s) => s.execute(storage, context),
        }
    }

    fn estimated_cost(&self, context: &SeekStrategyContext) -> f64 {
        match self {
            AnySeekStrategy::VertexSeek(s) => s.estimated_cost(context),
            AnySeekStrategy::IndexSeek(s) => s.estimated_cost(context),
            AnySeekStrategy::ScanSeek(s) => s.estimated_cost(context),
        }
    }

    fn supports(&self, context: &SeekStrategyContext) -> bool {
        match self {
            AnySeekStrategy::VertexSeek(s) => s.supports(context),
            AnySeekStrategy::IndexSeek(s) => s.supports(context),
            AnySeekStrategy::ScanSeek(s) => s.supports(context),
        }
    }
}

impl SeekStrategySelector {
    pub fn create_strategy(
        &self,
        strategy_type: SeekStrategyType,
    ) -> AnySeekStrategy {
        match strategy_type {
            SeekStrategyType::VertexSeek => AnySeekStrategy::VertexSeek(VertexSeek::new()),
            SeekStrategyType::IndexSeek => AnySeekStrategy::IndexSeek(IndexSeek::new()),
            SeekStrategyType::ScanSeek => AnySeekStrategy::ScanSeek(ScanSeek::new()),
        }
    }

    pub fn find(
        &self,
        storage: &dyn StorageClient,
        context: &SeekStrategyContext,
    ) -> Result<SeekResult, StorageError> {
        let strategy_type = self.select_strategy(storage, context);
        let strategy = self.create_strategy(strategy_type);
        strategy.execute(storage, context)
    }
}
