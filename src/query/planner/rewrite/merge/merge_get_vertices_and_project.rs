//! 合并获取顶点和投影操作的规则

use crate::query::optimizer::plan::Pattern;



crate::define_merge_rule! {
    /// 合并获取顶点和投影操作的规则
    ///
    /// # 转换示例
    ///
    /// Before:
    /// ```text
    ///   GetVertices
    ///       |
    ///   Project(col1, col2)
    ///       |
    ///   ScanVertices
    /// ```
    ///
    /// After:
    /// ```text
    ///   GetVertices
    ///       |
    ///   ScanVertices
    /// ```
    ///
    /// # 适用条件
    ///
    /// - 当前节点为GetVertices节点
    /// - 子节点为Project节点
    /// - 可以将投影操作合并到GetVertices中
    pub struct MergeGetVerticesAndProjectRule {
        parent: GetVertices,
        parent_check: is_get_vertices,
        child: Project,
        child_check: is_project,
        pattern: Pattern::new_with_name("GetVertices").with_dependency_name("Project")
    }
}
