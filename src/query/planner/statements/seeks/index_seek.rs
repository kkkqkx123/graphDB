//! 索引查找策略

#[derive(Debug)]
pub struct IndexSeek;

impl IndexSeek {
    pub fn new() -> Self {
        Self
    }
}

pub type IndexScanMetadata = ();
pub type IndexSeekType = ();
