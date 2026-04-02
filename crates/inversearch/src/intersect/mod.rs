//! 交集模块的统一导出
//! 
//! 提供所有交集相关功能的统一接口

pub mod core;
pub mod scoring;
pub mod suggestion;
pub mod compat;

// 重新导出核心函数
pub use core::{
    intersect,
    union,
    intersect_union,
    intersect_simple,
    union_simple,
};

// 重新导出评分函数
pub use scoring::{
    TfIdfScorer,
    Bm25Scorer,
    ScoreConfig,
    ScoredId,
};

// 重新导出建议函数
pub use suggestion::{
    SuggestionConfig,
    SuggestionEngine,
};

// 重新导出兼容函数
pub use compat::{
    intersect_compatible,
    union_compatible,
    intersect_union_compatible,
    convert_old_to_new,
    convert_new_to_old,
    flatten_intermediate,
    rebuild_intermediate,
};

/// 兼容的交集函数（旧接口）
pub fn intersect_old(
    arrays: &crate::r#type::IntermediateSearchResults,
    resolution: usize,
    limit: usize,
    offset: usize,
    suggest: bool,
    boost: i32,
    resolve: bool,
) -> crate::r#type::IntermediateSearchResults {
    compat::intersect_compatible(arrays, resolution, limit, offset, suggest, boost, resolve)
}

/// 兼容的并集函数（旧接口）
pub fn union_old(
    arrays: &crate::r#type::IntermediateSearchResults,
    limit: usize,
    offset: usize,
    sort_by_score: bool,
    boost: i32,
) -> crate::r#type::IntermediateSearchResults {
    compat::union_compatible(arrays, limit, offset, sort_by_score, boost)
}

/// 兼容的交集并集函数（旧接口）
pub fn intersect_union_old(
    arrays: &crate::r#type::IntermediateSearchResults,
    mandatory: &crate::r#type::IntermediateSearchResults,
    limit: usize,
    offset: usize,
    sort_by_score: bool,
    boost: i32,
) -> crate::r#type::SearchResults {
    compat::intersect_union_compatible(arrays, mandatory, limit, offset, sort_by_score, boost)
}