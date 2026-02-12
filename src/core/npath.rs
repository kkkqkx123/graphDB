//! NPath - 链表结构的路径表示
//!
//! 参考nebula-graph的NPath设计，使用共享所有权实现前缀共享。
//! 适用于图遍历中需要频繁扩展路径的场景。
//!
//! # 核心优势
//!
//! 1. **共享前缀**：多条路径共享相同的前缀部分，节省内存
//! 2. **O(1)扩展**：新路径只需创建一个新节点，指向父路径
//! 3. **快速拼接**：双向BFS路径拼接时，只需找到交汇点
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use graphdb::core::npath::NPath;
//! use graphdb::core::{Vertex, Edge, Value};
//!
//! // 创建起点
//! let start_vertex = Arc::new(Vertex::new(Value::Int(1), vec![]));
//! let start = Arc::new(NPath::new(start_vertex));
//!
//! // 扩展路径
//! let edge = Arc::new(Edge::new("friend", Value::Int(1), Value::Int(2)));
//! let next_vertex = Arc::new(Vertex::new(Value::Int(2), vec![]));
//! let extended = Arc::new(NPath::extend(start, edge, next_vertex));
//! ```

use std::sync::Arc;
use std::collections::HashSet;

use crate::core::{Vertex, Edge, Path, Value};
use crate::core::vertex_edge_path::Step;

/// NPath - 链表结构的路径表示
///
/// 使用不可变数据结构，通过Arc实现共享所有权。
/// 每个节点包含一个顶点和到达该顶点的边（起点除外）。
#[derive(Debug, Clone)]
pub struct NPath {
    /// 父路径节点（None表示起点）
    parent: Option<Arc<NPath>>,
    /// 当前顶点
    vertex: Arc<Vertex>,
    /// 到达当前顶点的边（起点为None）
    edge: Option<Arc<Edge>>,
    /// 路径长度（缓存，避免递归计算）
    length: usize,
    /// 路径哈希（缓存，用于快速比较）
    hash: u64,
}

impl NPath {
    /// 创建起点路径
    ///
    /// # 参数
    ///
    /// * `vertex` - 起始顶点
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let start = Arc::new(NPath::new(Arc::new(vertex)));
    /// ```
    pub fn new(vertex: Arc<Vertex>) -> Self {
        let hash = Self::compute_hash(&vertex, None, None);
        Self {
            parent: None,
            vertex,
            edge: None,
            length: 0,
            hash,
        }
    }

    /// 扩展路径 - O(1)操作
    ///
    /// 创建新路径节点，指向父路径，实现前缀共享。
    ///
    /// # 参数
    ///
    /// * `parent` - 父路径
    /// * `edge` - 到达新顶点的边
    /// * `vertex` - 新顶点
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let extended = Arc::new(NPath::extend(parent, edge, vertex));
    /// ```
    pub fn extend(parent: Arc<NPath>, edge: Arc<Edge>, vertex: Arc<Vertex>) -> Self {
        let length = parent.length + 1;
        let hash = Self::compute_hash(&vertex, Some(&edge), Some(parent.hash));
        Self {
            parent: Some(parent),
            vertex,
            edge: Some(edge),
            length,
            hash,
        }
    }

    /// 扩展路径并检查环路 - O(1)平均时间
    ///
    /// 使用HashSet快速检测是否形成环路，适用于DFS探索中的提前剪枝。
    ///
    /// # 参数
    ///
    /// * `parent` - 父路径
    /// * `edge` - 到达新顶点的边
    /// * `vertex` - 新顶点
    /// * `seen_vertices` - 已访问顶点集合
    ///
    /// # 返回
    ///
    /// * `Some(NPath)` - 扩展成功
    /// * `None` - 检测到环路
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let mut seen = HashSet::new();
    /// seen.insert(parent.vertex.vid.as_ref().clone());
    /// if let Some(extended) = NPath::extend_with_set(parent, edge, vertex, &mut seen) {
    ///     // 继续探索
    /// }
    /// ```
    pub fn extend_with_set(
        parent: Arc<NPath>,
        edge: Arc<Edge>,
        vertex: Arc<Vertex>,
        seen_vertices: &mut HashSet<Value>,
    ) -> Option<Self> {
        // 检查是否形成环路
        if seen_vertices.contains(&vertex.vid.as_ref().clone()) {
            return None;
        }

        let new_path = Self::extend(parent, edge, vertex);
        seen_vertices.insert(new_path.vertex.vid.as_ref().clone());
        Some(new_path)
    }

    /// 从Path创建NPath
    ///
    /// 便于兼容已有接口，将传统的Path转换为NPath。
    ///
    /// # 参数
    ///
    /// * `path` - 传统Path结构
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let npath = NPath::from_path(&path);
    /// ```
    pub fn from_path(path: &Path) -> Arc<Self> {
        let start_vertex = Arc::new((*path.src).clone());
        let mut current = Arc::new(Self::new(start_vertex));

        for step in &path.steps {
            let edge = Arc::new((*step.edge).clone());
            let vertex = Arc::new((*step.dst).clone());
            current = Arc::new(Self::extend(current, edge, vertex));
        }

        current
    }

    /// 获取路径长度（边数）
    pub fn len(&self) -> usize {
        self.length
    }

    /// 检查路径是否为空（仅包含起点）
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// 获取当前顶点
    pub fn vertex(&self) -> &Arc<Vertex> {
        &self.vertex
    }

    /// 获取到达当前顶点的边
    pub fn edge(&self) -> Option<&Arc<Edge>> {
        self.edge.as_ref()
    }

    /// 获取父路径
    pub fn parent(&self) -> Option<&Arc<NPath>> {
        self.parent.as_ref()
    }

    /// 获取起点顶点
    pub fn start_vertex(&self) -> &Arc<Vertex> {
        let mut current = self;
        while let Some(ref parent) = current.parent {
            current = parent;
        }
        &current.vertex
    }

    /// 获取终点顶点（当前顶点）
    pub fn end_vertex(&self) -> &Arc<Vertex> {
        &self.vertex
    }

    /// 转换为Path（需要时再进行转换）
    ///
    /// 时间复杂度：O(n)，n为路径长度
    pub fn to_path(&self) -> Path {
        let mut steps = Vec::with_capacity(self.length);
        let mut current = self;

        // 收集所有步骤（从终点到起点）
        while let Some(ref parent) = current.parent {
            if let Some(ref edge) = current.edge {
                steps.push(Step {
                    dst: Box::new((*current.vertex).clone()),
                    edge: Box::new((**edge).clone()),
                });
            }
            current = parent;
        }

        // 反转步骤（从起点到终点）
        steps.reverse();

        Path {
            src: Box::new((*current.vertex).clone()),
            steps,
        }
    }

    /// 检查是否包含某个顶点（用于noLoop检查）
    ///
    /// 时间复杂度：O(n)，n为路径长度
    pub fn contains_vertex(&self, vid: &Value) -> bool {
        if self.vertex.vid.as_ref() == vid {
            return true;
        }
        if let Some(ref parent) = self.parent {
            return parent.contains_vertex(vid);
        }
        false
    }

    /// 检查是否包含某条边（去重检查）
    ///
    /// 时间复杂度：O(n)，n为路径长度
    pub fn contains_edge(&self, edge_key: &(Value, Value, String)) -> bool {
        if let Some(ref edge) = self.edge {
            let key = (
                (*edge.src).clone(),
                (*edge.dst).clone(),
                edge.edge_type.clone(),
            );
            if &key == edge_key {
                return true;
            }
        }
        if let Some(ref parent) = self.parent {
            return parent.contains_edge(edge_key);
        }
        false
    }

    /// 检查与另一条路径是否有共同顶点（用于双向BFS路径拼接检查）
    ///
    /// 时间复杂度：O(n*m)，建议先收集顶点再比较
    pub fn has_common_vertices(&self, other: &NPath) -> bool {
        let self_vertices: HashSet<_> = self.iter_vertices().map(|v| v.vid.as_ref()).collect();
        other.iter_vertices().any(|v| self_vertices.contains(v.vid.as_ref()))
    }

    /// 收集所有顶点ID
    pub fn collect_vertex_ids(&self) -> Vec<Value> {
        self.iter_vertices().map(|v| (*v.vid).clone()).collect()
    }

    /// 收集所有边
    pub fn collect_edges(&self) -> Vec<Arc<Edge>> {
        self.iter_edges().cloned().collect()
    }

    /// 迭代器：从起点到当前节点的所有顶点
    pub fn iter_vertices(&self) -> NPathVertexIter {
        NPathVertexIter::new(self)
    }

    /// 迭代器：从起点到当前节点的所有边
    pub fn iter_edges(&self) -> NPathEdgeIter {
        NPathEdgeIter::new(self)
    }

    /// 迭代器：从起点到当前节点的所有节点
    pub fn iter(&self) -> NPathIter {
        NPathIter::new(self)
    }

    /// 计算路径哈希
    fn compute_hash(vertex: &Arc<Vertex>, edge: Option<&Arc<Edge>>, parent_hash: Option<u64>) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        if let Some(ph) = parent_hash {
            ph.hash(&mut hasher);
        }

        vertex.vid.hash(&mut hasher);

        if let Some(e) = edge {
            e.edge_type.hash(&mut hasher);
            e.src.hash(&mut hasher);
            e.dst.hash(&mut hasher);
        }

        hasher.finish()
    }

    /// 获取路径哈希
    pub fn hash(&self) -> u64 {
        self.hash
    }
}

impl PartialEq for NPath {
    fn eq(&self, other: &Self) -> bool {
        if self.hash != other.hash || self.length != other.length {
            return false;
        }

        // 哈希相同，进一步比较内容
        self.vertex.vid == other.vertex.vid
            && self.edge == other.edge
            && self.parent == other.parent
    }
}

impl Eq for NPath {}

/// NPath迭代器 - 遍历所有节点（从终点到起点）
///
/// 使用惰性求值，每次.next()向上跳一步，避免预分配Vec
pub struct NPathIter<'a> {
    current: Option<&'a NPath>,
}

impl<'a> NPathIter<'a> {
    fn new(path: &'a NPath) -> Self {
        Self { current: Some(path) }
    }
}

impl<'a> Iterator for NPathIter<'a> {
    type Item = &'a NPath;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.current?;
        self.current = curr.parent.as_deref();
        Some(curr)
    }
}

/// NPath顶点迭代器 - 惰性遍历所有顶点
///
/// 优化：不预分配Vec，每次.next()向上跳一步
pub struct NPathVertexIter<'a> {
    current: Option<&'a NPath>,
}

impl<'a> NPathVertexIter<'a> {
    fn new(path: &'a NPath) -> Self {
        Self { current: Some(path) }
    }
}

impl<'a> Iterator for NPathVertexIter<'a> {
    type Item = &'a Arc<Vertex>;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.current?;
        let vertex = &curr.vertex;
        self.current = curr.parent.as_deref();
        Some(vertex)
    }
}

/// NPath边迭代器 - 惰性遍历所有边
///
/// 优化：不预分配Vec，每次.next()向上跳一步
pub struct NPathEdgeIter<'a> {
    current: Option<&'a NPath>,
}

impl<'a> NPathEdgeIter<'a> {
    fn new(path: &'a NPath) -> Self {
        Self { current: Some(path) }
    }
}

impl<'a> Iterator for NPathEdgeIter<'a> {
    type Item = &'a Arc<Edge>;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.current?;
        let edge = curr.edge.as_ref()?;
        self.current = curr.parent.as_deref();
        Some(edge)
    }
}

/// NPath工具函数
pub mod utils {
    use super::*;

    /// 拼接两条路径（用于双向BFS）
    ///
    /// 左路径从起点到中间，右路径从终点到中间
    /// 结果路径从起点到终点
    pub fn combine_paths(left: &Arc<NPath>, right: &Arc<NPath>) -> Option<Path> {
        // 检查两条路径是否在同一个顶点交汇
        if left.vertex.vid != right.vertex.vid {
            return None;
        }

        // 构建从左起点到交汇点的路径
        let left_path = left.to_path();

        // 构建从右起点到交汇点的路径，然后反转
        let mut right_path = right.to_path();
        right_path.reverse();

        // 合并两条路径
        let mut combined = left_path;
        combined.steps.extend(right_path.steps);

        Some(combined)
    }

    /// 批量将NPath转换为Path
    pub fn batch_to_paths(npaths: &[Arc<NPath>]) -> Vec<Path> {
        npaths.iter().map(|np| np.to_path()).collect()
    }

    /// 检查路径集合中是否有重复
    pub fn has_duplicates(npaths: &[Arc<NPath>]) -> bool {
        let mut seen = HashSet::new();
        for np in npaths {
            if !seen.insert(np.hash()) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    fn create_test_vertex(id: i64) -> Arc<Vertex> {
        Arc::new(Vertex::new(Value::Int(id), vec![]))
    }

    fn create_test_edge(src_id: i64, dst_id: i64, edge_type: &str) -> Arc<Edge> {
        use std::collections::HashMap;
        Arc::new(Edge::new(
            Value::Int(src_id),
            Value::Int(dst_id),
            edge_type.to_string(),
            0,
            HashMap::new(),
        ))
    }

    #[test]
    fn test_npath_new() {
        let v = create_test_vertex(1);
        let path = NPath::new(v.clone());

        assert_eq!(path.len(), 0);
        assert!(path.is_empty());
        assert_eq!(path.vertex().vid.as_ref(), &Value::Int(1));
        assert!(path.parent().is_none());
        assert!(path.edge().is_none());
    }

    #[test]
    fn test_npath_extend() {
        let v1 = create_test_vertex(1);
        let v2 = create_test_vertex(2);
        let e = create_test_edge(1, 2, "friend");

        let start = Arc::new(NPath::new(v1));
        let extended = NPath::extend(start, e, v2);

        assert_eq!(extended.len(), 1);
        assert!(!extended.is_empty());
        assert_eq!(extended.vertex().vid.as_ref(), &Value::Int(2));
        assert!(extended.parent().is_some());
        assert!(extended.edge().is_some());
    }

    #[test]
    fn test_npath_to_path() {
        let v1 = create_test_vertex(1);
        let v2 = create_test_vertex(2);
        let v3 = create_test_vertex(3);
        let e1 = create_test_edge(1, 2, "friend");
        let e2 = create_test_edge(2, 3, "friend");

        let start = Arc::new(NPath::new(v1));
        let p2 = Arc::new(NPath::extend(start, e1, v2));
        let p3 = Arc::new(NPath::extend(p2, e2, v3));

        let path = p3.to_path();

        assert_eq!(path.len(), 2);
        assert_eq!(path.src.vid.as_ref(), &Value::Int(1));
    }

    #[test]
    fn test_npath_contains_vertex() {
        let v1 = create_test_vertex(1);
        let v2 = create_test_vertex(2);
        let v3 = create_test_vertex(3);
        let e1 = create_test_edge(1, 2, "friend");
        let e2 = create_test_edge(2, 3, "friend");

        let start = Arc::new(NPath::new(v1));
        let p2 = Arc::new(NPath::extend(start, e1, v2));
        let p3 = Arc::new(NPath::extend(p2, e2, v3));

        assert!(p3.contains_vertex(&Value::Int(1)));
        assert!(p3.contains_vertex(&Value::Int(2)));
        assert!(p3.contains_vertex(&Value::Int(3)));
        assert!(!p3.contains_vertex(&Value::Int(4)));
    }

    #[test]
    fn test_npath_iter_vertices() {
        let v1 = create_test_vertex(1);
        let v2 = create_test_vertex(2);
        let v3 = create_test_vertex(3);
        let e1 = create_test_edge(1, 2, "friend");
        let e2 = create_test_edge(2, 3, "friend");

        let start = Arc::new(NPath::new(v1));
        let p2 = Arc::new(NPath::extend(start, e1, v2));
        let p3 = Arc::new(NPath::extend(p2, e2, v3));

        let vertices: Vec<_> = p3.iter_vertices().collect();
        assert_eq!(vertices.len(), 3);
    }

    #[test]
    fn test_npath_equality() {
        let v1 = create_test_vertex(1);
        let v2 = create_test_vertex(2);
        let e = create_test_edge(1, 2, "friend");

        let start1 = Arc::new(NPath::new(v1.clone()));
        let path1 = Arc::new(NPath::extend(start1, e.clone(), v2.clone()));

        let start2 = Arc::new(NPath::new(v1));
        let path2 = Arc::new(NPath::extend(start2, e, v2));

        assert_eq!(path1.hash(), path2.hash());
    }
}
