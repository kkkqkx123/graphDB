//! 查找策略

#[derive(Debug)]
pub struct SeekStrategy;

impl SeekStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct SeekStrategySelector;

impl SeekStrategySelector {
    pub fn new() -> Self {
        Self
    }
}

pub type SeekStrategyType = ();
