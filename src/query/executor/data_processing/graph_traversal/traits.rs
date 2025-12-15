/// 图遍历执行器的通用特征
pub trait GraphTraversalExecutor<S: crate::storage::StorageEngine> {
    /// 设置边方向
    fn set_edge_direction(&mut self, direction: crate::query::executor::base::EdgeDirection);

    /// 设置边类型过滤
    fn set_edge_types(&mut self, edge_types: Option<Vec<String>>);

    /// 设置最大深度
    fn set_max_depth(&mut self, max_depth: Option<usize>);

    /// 获取当前边方向
    fn get_edge_direction(&self) -> &crate::query::executor::base::EdgeDirection;

    /// 获取当前边类型过滤
    fn get_edge_types(&self) -> &Option<Vec<String>>;

    /// 获取当前最大深度
    fn get_max_depth(&self) -> &Option<usize>;
}