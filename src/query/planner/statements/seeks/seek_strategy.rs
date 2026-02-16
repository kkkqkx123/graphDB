//! 查找策略模块
//!
//! 定义顶点查找策略和选择器，用于 MATCH 查询中确定起始顶点的查找方式

use crate::core::StorageError;
use crate::storage::StorageClient;

use super::edge_seek::{EdgeSeek, EdgePattern};
use super::index_seek::IndexSeek;
use super::prop_index_seek::PropIndexSeek;
use super::scan_seek::ScanSeek;
use super::seek_strategy_base::{
    SeekResult, SeekStrategyContext,
    SeekStrategySelector, SeekStrategyType,
};
use super::variable_prop_index_seek::VariablePropIndexSeek;
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
    PropIndexSeek(PropIndexSeek),
    VariablePropIndexSeek(VariablePropIndexSeek),
    EdgeSeek(EdgeSeek),
    ScanSeek(ScanSeek),
}

impl Clone for AnySeekStrategy {
    fn clone(&self) -> Self {
        match self {
            AnySeekStrategy::VertexSeek(v) => AnySeekStrategy::VertexSeek(v.clone()),
            AnySeekStrategy::IndexSeek(i) => AnySeekStrategy::IndexSeek(i.clone()),
            AnySeekStrategy::PropIndexSeek(p) => AnySeekStrategy::PropIndexSeek(p.clone()),
            AnySeekStrategy::VariablePropIndexSeek(v) => AnySeekStrategy::VariablePropIndexSeek(v.clone()),
            AnySeekStrategy::EdgeSeek(e) => AnySeekStrategy::EdgeSeek(EdgeSeek::new(EdgePattern {
                edge_types: e.edge_pattern.edge_types.clone(),
                direction: e.edge_pattern.direction,
                src_vid: e.edge_pattern.src_vid.clone(),
                dst_vid: e.edge_pattern.dst_vid.clone(),
                properties: e.edge_pattern.properties.clone(),
            })),
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
            AnySeekStrategy::PropIndexSeek(s) => s.execute(storage, context),
            AnySeekStrategy::VariablePropIndexSeek(s) => s.execute(storage, context),
            AnySeekStrategy::EdgeSeek(s) => s.execute(storage, context),
            AnySeekStrategy::ScanSeek(s) => s.execute(storage, context),
        }
    }

    fn estimated_cost(&self, context: &SeekStrategyContext) -> f64 {
        match self {
            AnySeekStrategy::VertexSeek(s) => s.estimated_cost(context),
            AnySeekStrategy::IndexSeek(s) => s.estimated_cost(context),
            AnySeekStrategy::PropIndexSeek(s) => s.estimated_cost(context),
            AnySeekStrategy::VariablePropIndexSeek(s) => s.estimated_cost(context),
            AnySeekStrategy::EdgeSeek(s) => s.estimated_cost(context),
            AnySeekStrategy::ScanSeek(s) => s.estimated_cost(context),
        }
    }

    fn supports(&self, context: &SeekStrategyContext) -> bool {
        match self {
            AnySeekStrategy::VertexSeek(s) => s.supports(context),
            AnySeekStrategy::IndexSeek(s) => s.supports(context),
            AnySeekStrategy::PropIndexSeek(s) => s.supports(context),
            AnySeekStrategy::VariablePropIndexSeek(s) => s.supports(context),
            AnySeekStrategy::EdgeSeek(s) => s.supports(context),
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
            SeekStrategyType::PropIndexSeek => {
                // PropIndexSeek 需要谓词，这里创建空实例
                AnySeekStrategy::PropIndexSeek(PropIndexSeek::new(vec![]))
            }
            SeekStrategyType::VariablePropIndexSeek => {
                // VariablePropIndexSeek 需要谓词，这里创建空实例
                AnySeekStrategy::VariablePropIndexSeek(VariablePropIndexSeek::new(vec![]))
            }
            SeekStrategyType::EdgeSeek => {
                // EdgeSeek 需要边模式，这里创建默认实例
                AnySeekStrategy::EdgeSeek(EdgeSeek::new(EdgePattern {
                    edge_types: vec![],
                    direction: super::edge_seek::EdgeDirection::Both,
                    src_vid: None,
                    dst_vid: None,
                    properties: vec![],
                }))
            }
            SeekStrategyType::ScanSeek => AnySeekStrategy::ScanSeek(ScanSeek::new()),
        }
    }

    /// 创建带参数的 PropIndexSeek 策略
    pub fn create_prop_index_strategy(
        &self,
        predicates: Vec<super::prop_index_seek::PropertyPredicate>,
    ) -> AnySeekStrategy {
        AnySeekStrategy::PropIndexSeek(PropIndexSeek::new(predicates))
    }

    /// 创建带参数的 VariablePropIndexSeek 策略
    pub fn create_variable_prop_index_strategy(
        &self,
        predicates: Vec<super::variable_prop_index_seek::VariablePropertyPredicate>,
    ) -> AnySeekStrategy {
        AnySeekStrategy::VariablePropIndexSeek(VariablePropIndexSeek::new(predicates))
    }

    /// 创建带参数的 EdgeSeek 策略
    pub fn create_edge_strategy(
        &self,
        edge_pattern: EdgePattern,
    ) -> AnySeekStrategy {
        AnySeekStrategy::EdgeSeek(EdgeSeek::new(edge_pattern))
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
