//! 会话级变更统计模块
//!
//! 提供查询执行的影响行数、最后插入ID等统计信息

use std::sync::atomic::{AtomicU64, Ordering};

/// 会话级变更统计
///
/// 记录会话中的查询执行情况，包括：
/// - 上次操作影响的行数
/// - 总会话变更数
/// - 最后插入的顶点ID
/// - 最后插入的边ID
#[derive(Debug)]
pub struct SessionStatistics {
    /// 上次操作影响的行数
    last_changes: AtomicU64,
    /// 总会话变更数
    total_changes: AtomicU64,
    /// 最后插入的顶点ID
    last_insert_vertex_id: AtomicU64,
    /// 最后插入的边ID
    last_insert_edge_id: AtomicU64,
    /// 是否有顶点ID（0 表示无效）
    has_vertex_id: AtomicU64,
    /// 是否有边ID（0 表示无效）
    has_edge_id: AtomicU64,
}

impl SessionStatistics {
    /// 创建新的统计实例
    pub fn new() -> Self {
        Self {
            last_changes: AtomicU64::new(0),
            total_changes: AtomicU64::new(0),
            last_insert_vertex_id: AtomicU64::new(0),
            last_insert_edge_id: AtomicU64::new(0),
            has_vertex_id: AtomicU64::new(0),
            has_edge_id: AtomicU64::new(0),
        }
    }

    /// 记录变更行数
    ///
    /// # 参数
    /// - `count` - 影响的行数
    pub fn record_changes(&self, count: u64) {
        self.last_changes.store(count, Ordering::SeqCst);
        self.total_changes.fetch_add(count, Ordering::SeqCst);
    }

    /// 记录顶点插入
    ///
    /// # 参数
    /// - `id` - 插入的顶点ID
    pub fn record_vertex_insert(&self, id: i64) {
        if id > 0 {
            self.last_insert_vertex_id
                .store(id as u64, Ordering::SeqCst);
            self.has_vertex_id.store(1, Ordering::SeqCst);
            self.last_changes.store(1, Ordering::SeqCst);
            self.total_changes.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// 记录边插入
    ///
    /// # 参数
    /// - `id` - 插入的边ID
    pub fn record_edge_insert(&self, id: i64) {
        if id > 0 {
            self.last_insert_edge_id.store(id as u64, Ordering::SeqCst);
            self.has_edge_id.store(1, Ordering::SeqCst);
            self.last_changes.store(1, Ordering::SeqCst);
            self.total_changes.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// 获取上次操作影响的行数
    pub fn last_changes(&self) -> u64 {
        self.last_changes.load(Ordering::SeqCst)
    }

    /// 获取总会话变更数
    pub fn total_changes(&self) -> u64 {
        self.total_changes.load(Ordering::SeqCst)
    }

    /// 获取最后插入的顶点ID
    ///
    /// 返回 None 表示没有记录
    pub fn last_insert_vertex_id(&self) -> Option<i64> {
        if self.has_vertex_id.load(Ordering::SeqCst) != 0 {
            Some(self.last_insert_vertex_id.load(Ordering::SeqCst) as i64)
        } else {
            None
        }
    }

    /// 获取最后插入的边ID
    ///
    /// 返回 None 表示没有记录
    pub fn last_insert_edge_id(&self) -> Option<i64> {
        if self.has_edge_id.load(Ordering::SeqCst) != 0 {
            Some(self.last_insert_edge_id.load(Ordering::SeqCst) as i64)
        } else {
            None
        }
    }

    /// 重置上次变更记录
    ///
    /// 通常在执行新查询前调用
    pub fn reset_last(&self) {
        self.last_changes.store(0, Ordering::SeqCst);
        self.has_vertex_id.store(0, Ordering::SeqCst);
        self.has_edge_id.store(0, Ordering::SeqCst);
    }

    /// 重置所有统计
    pub fn reset_all(&self) {
        self.last_changes.store(0, Ordering::SeqCst);
        self.total_changes.store(0, Ordering::SeqCst);
        self.last_insert_vertex_id.store(0, Ordering::SeqCst);
        self.last_insert_edge_id.store(0, Ordering::SeqCst);
        self.has_vertex_id.store(0, Ordering::SeqCst);
        self.has_edge_id.store(0, Ordering::SeqCst);
    }
}

impl Default for SessionStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SessionStatistics {
    fn clone(&self) -> Self {
        Self {
            last_changes: AtomicU64::new(self.last_changes.load(Ordering::SeqCst)),
            total_changes: AtomicU64::new(self.total_changes.load(Ordering::SeqCst)),
            last_insert_vertex_id: AtomicU64::new(
                self.last_insert_vertex_id.load(Ordering::SeqCst),
            ),
            last_insert_edge_id: AtomicU64::new(self.last_insert_edge_id.load(Ordering::SeqCst)),
            has_vertex_id: AtomicU64::new(self.has_vertex_id.load(Ordering::SeqCst)),
            has_edge_id: AtomicU64::new(self.has_edge_id.load(Ordering::SeqCst)),
        }
    }
}

/// 查询结果统计信息
///
/// 从查询结果中提取的统计信息
#[derive(Debug, Clone, Default)]
pub struct QueryStatistics {
    /// 影响的行数
    pub rows_affected: u64,
    /// 返回的行数
    pub rows_returned: u64,
    /// 插入的顶点ID列表
    pub inserted_vertex_ids: Vec<i64>,
    /// 插入的边ID列表
    pub inserted_edge_ids: Vec<i64>,
    /// 更新的顶点数
    pub vertices_updated: u64,
    /// 更新的边数
    pub edges_updated: u64,
    /// 删除的顶点数
    pub vertices_deleted: u64,
    /// 删除的边数
    pub edges_deleted: u64,
}

impl QueryStatistics {
    /// 创建空的统计信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 从查询结果元数据创建
    ///
    /// # 参数
    /// - `metadata` - 查询结果元数据
    pub fn from_metadata(metadata: &crate::api::core::ExecutionMetadata) -> Self {
        Self {
            rows_affected: metadata.rows_returned,
            rows_returned: metadata.rows_returned as u64,
            ..Default::default()
        }
    }

    /// 合并另一个统计信息
    pub fn merge(&mut self, other: &QueryStatistics) {
        self.rows_affected += other.rows_affected;
        self.rows_returned += other.rows_returned;
        self.inserted_vertex_ids
            .extend_from_slice(&other.inserted_vertex_ids);
        self.inserted_edge_ids
            .extend_from_slice(&other.inserted_edge_ids);
        self.vertices_updated += other.vertices_updated;
        self.edges_updated += other.edges_updated;
        self.vertices_deleted += other.vertices_deleted;
        self.edges_deleted += other.edges_deleted;
    }

    /// 获取总变更数
    pub fn total_changes(&self) -> u64 {
        self.rows_affected
            + self.vertices_updated
            + self.edges_updated
            + self.vertices_deleted
            + self.edges_deleted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_statistics_basic() {
        let stats = SessionStatistics::new();

        assert_eq!(stats.last_changes(), 0);
        assert_eq!(stats.total_changes(), 0);
        assert_eq!(stats.last_insert_vertex_id(), None);
        assert_eq!(stats.last_insert_edge_id(), None);
    }

    #[test]
    fn test_record_changes() {
        let stats = SessionStatistics::new();

        stats.record_changes(5);
        assert_eq!(stats.last_changes(), 5);
        assert_eq!(stats.total_changes(), 5);

        stats.record_changes(3);
        assert_eq!(stats.last_changes(), 3);
        assert_eq!(stats.total_changes(), 8);
    }

    #[test]
    fn test_record_vertex_insert() {
        let stats = SessionStatistics::new();

        stats.record_vertex_insert(100);
        assert_eq!(stats.last_insert_vertex_id(), Some(100));
        assert_eq!(stats.last_changes(), 1);
        assert_eq!(stats.total_changes(), 1);

        // 无效ID不应该记录
        stats.record_vertex_insert(0);
        assert_eq!(stats.last_insert_vertex_id(), Some(100)); // 保持不变
    }

    #[test]
    fn test_record_edge_insert() {
        let stats = SessionStatistics::new();

        stats.record_edge_insert(200);
        assert_eq!(stats.last_insert_edge_id(), Some(200));
        assert_eq!(stats.last_changes(), 1);
        assert_eq!(stats.total_changes(), 1);
    }

    #[test]
    fn test_reset() {
        let stats = SessionStatistics::new();

        stats.record_changes(5);
        stats.record_vertex_insert(100);

        stats.reset_last();
        assert_eq!(stats.last_changes(), 0);
        assert_eq!(stats.last_insert_vertex_id(), None);
        assert_eq!(stats.total_changes(), 6); // 总数不变

        stats.reset_all();
        assert_eq!(stats.total_changes(), 0);
    }

    #[test]
    fn test_query_statistics() {
        let mut stats = QueryStatistics::new();
        stats.rows_affected = 10;
        stats.rows_returned = 5;
        stats.inserted_vertex_ids = vec![1, 2, 3];

        assert_eq!(stats.total_changes(), 10);

        let mut other = QueryStatistics::new();
        other.rows_affected = 5;
        other.inserted_vertex_ids = vec![4, 5];

        stats.merge(&other);
        assert_eq!(stats.rows_affected, 15);
        assert_eq!(stats.inserted_vertex_ids.len(), 5);
    }
}
