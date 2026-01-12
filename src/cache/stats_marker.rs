//! 统计功能编译时标记
//!
//! 使用泛型参数在编译时决定是否启用统计，消除运行时条件分发

use std::marker::PhantomData;

/// 统计模式标记 trait
///
/// 编译时标记，用于在编译时确定缓存是否启用统计功能
pub trait StatsMode: Send + Sync {
    /// 是否启用统计
    const ENABLED: bool;
}

/// 启用统计模式
///
/// 当缓存使用此标记时，会记录命中、未命中、驱逐等统计信息
#[derive(Debug, Clone, Copy)]
pub struct StatsEnabled;

impl StatsMode for StatsEnabled {
    const ENABLED: bool = true;
}

/// 禁用统计模式
///
/// 当缓存使用此标记时，不记录任何统计信息，零开销
#[derive(Debug, Clone, Copy)]
pub struct StatsDisabled;

impl StatsMode for StatsDisabled {
    const ENABLED: bool = false;
}

/// 统计模式标记助手
///
/// 用于在类型系统中传递统计模式信息
pub struct StatsModeMarker<S: StatsMode>(PhantomData<S>);

impl<S: StatsMode> StatsModeMarker<S> {
    /// 检查统计是否启用
    pub fn is_enabled() -> bool {
        S::ENABLED
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_enabled_constant() {
        assert!(StatsEnabled::ENABLED);
    }

    #[test]
    fn test_stats_disabled_constant() {
        assert!(!StatsDisabled::ENABLED);
    }

    #[test]
    fn test_stats_mode_marker() {
        assert!(StatsModeMarker::<StatsEnabled>::is_enabled());
        assert!(!StatsModeMarker::<StatsDisabled>::is_enabled());
    }
}
