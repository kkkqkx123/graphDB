//! 批量操作
//!
//! 提供高效的批量文档添加、更新、删除操作
//!
//! # 使用示例
//!
//! ```rust
//! use inversearch::{Document, Batch};
//!
//! let mut batch = Batch::new(1000); // 批量大小 1000
//!
//! // 添加操作
//! batch.add(1, &json!({"title": "Doc 1"}));
//! batch.add(2, &json!({"title": "Doc 2"}));
//!
//! // 执行批量操作
//! document.execute_batch(&mut batch)?;
//! ```

use serde_json::{Value, json};
use crate::DocId;

/// 批量操作类型
#[derive(Debug, Clone)]
pub enum BatchOperation<'a> {
    Add(DocId, &'a Value),
    Update(DocId, &'a Value),
    Remove(DocId),
}

/// 批量操作缓冲
#[derive(Debug, Clone)]
pub struct Batch<'a> {
    operations: Vec<BatchOperation<'a>>,
    max_size: usize,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Batch<'a> {
    /// 创建新的批量操作
    pub fn new(max_size: usize) -> Self {
        Batch {
            operations: Vec::with_capacity(max_size),
            max_size,
            _phantom: std::marker::PhantomData,
        }
    }

    /// 添加文档
    pub fn add(&mut self, id: DocId, content: &'a Value) {
        self.operations.push(BatchOperation::Add(id, content));
    }

    /// 更新文档
    pub fn update(&mut self, id: DocId, content: &'a Value) {
        self.operations.push(BatchOperation::Update(id, content));
    }

    /// 删除文档
    pub fn remove(&mut self, id: DocId) {
        self.operations.push(BatchOperation::Remove(id));
    }

    /// 检查是否需要刷新
    pub fn should_flush(&self) -> bool {
        self.operations.len() >= self.max_size
    }

    /// 获取操作数量
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// 清空操作队列
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// 取出所有操作
    pub fn drain(&mut self) -> Vec<BatchOperation<'a>> {
        self.operations.drain(..).collect()
    }

    /// 获取操作引用
    pub fn operations(&self) -> &[BatchOperation<'a>] {
        &self.operations
    }
}

/// 批量操作执行器
pub struct BatchExecutor<'a, A, U, R>
where
    A: FnMut(DocId, &Value) -> Result<(), crate::error::InversearchError>,
    U: FnMut(DocId, &Value) -> Result<(), crate::error::InversearchError>,
    R: FnMut(DocId, &Value) -> Result<(), crate::error::InversearchError>,
{
    add_fn: A,
    update_fn: U,
    remove_fn: R,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, A, U, R> BatchExecutor<'a, A, U, R>
where
    A: FnMut(DocId, &Value) -> Result<(), crate::error::InversearchError>,
    U: FnMut(DocId, &Value) -> Result<(), crate::error::InversearchError>,
    R: FnMut(DocId, &Value) -> Result<(), crate::error::InversearchError>,
{
    /// 创建新的执行器
    pub fn new(
        add_fn: A,
        update_fn: U,
        remove_fn: R,
    ) -> Self {
        BatchExecutor {
            add_fn,
            update_fn,
            remove_fn,
            _phantom: std::marker::PhantomData,
        }
    }

    /// 执行单个操作
    pub fn execute(&mut self, op: &BatchOperation) -> Result<(), crate::error::InversearchError> {
        match op {
            BatchOperation::Add(id, content) => (self.add_fn)(*id, content),
            BatchOperation::Update(id, content) => (self.update_fn)(*id, content),
            BatchOperation::Remove(id) => (self.remove_fn)(*id, &json!({})),
        }
    }

    /// 执行批量操作
    pub fn execute_batch(
        &mut self,
        batch: &Batch,
    ) -> Result<(), crate::error::InversearchError> {
        for op in &batch.operations {
            self.execute(op)?;
        }
        Ok(())
    }

    /// 执行并清空
    pub fn execute_and_clear(
        &mut self,
        batch: &mut Batch,
    ) -> Result<(), crate::error::InversearchError> {
        self.execute_batch(batch)?;
        batch.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_batch_add() {
        let mut batch = Batch::new(100);
        
        let doc1 = json!({"title": "Doc 1"});
        let doc2 = json!({"title": "Doc 2"});
        batch.add(1, &doc1);
        batch.add(2, &doc2);
        
        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_batch_update() {
        let mut batch = Batch::new(100);
        
        let doc = json!({"title": "Updated"});
        batch.update(1, &doc);
        
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_batch_remove() {
        let mut batch = Batch::new(100);
        
        batch.remove(1);
        batch.remove(2);
        
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_batch_mixed_operations() {
        let mut batch = Batch::new(100);
        
        let doc1 = json!({"title": "New"});
        let doc2 = json!({"title": "Updated"});
        batch.add(1, &doc1);
        batch.update(2, &doc2);
        batch.remove(3);
        
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_batch_clear() {
        let mut batch = Batch::new(100);
        
        let doc1 = json!({"title": "Doc 1"});
        let doc2 = json!({"title": "Doc 2"});
        batch.add(1, &doc1);
        batch.add(2, &doc2);
        
        batch.clear();
        
        assert!(batch.is_empty());
    }

    #[test]
    fn test_batch_drain() {
        let mut batch = Batch::new(100);
        
        let doc1 = json!({"title": "Doc 1"});
        let doc2 = json!({"title": "Doc 2"});
        batch.add(1, &doc1);
        batch.add(2, &doc2);
        
        let ops = batch.drain();
        
        assert!(batch.is_empty());
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn test_batch_executor() {
        let mut batch = Batch::new(100);
        let doc1 = json!({"title": "Doc 1"});
        let doc2 = json!({"title": "Doc 2"});
        batch.add(1, &doc1);
        batch.add(2, &doc2);
        
        let mut add_count = 0;
        let mut update_count = 0;
        let mut remove_count = 0;
        
        let mut executor = BatchExecutor::new(
            |_, _| { add_count += 1; Ok(()) },
            |_, _| { update_count += 1; Ok(()) },
            |_, _| { remove_count += 1; Ok(()) },
        );
        
        executor.execute_batch(&batch).unwrap();
        
        assert_eq!(add_count, 2);
        assert_eq!(update_count, 0);
        assert_eq!(remove_count, 0);
    }
}
