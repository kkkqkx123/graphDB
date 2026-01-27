//! 核心操作类型枚举
//!
//! 此模块定义了查询系统的核心操作类型枚举 `CoreOperationKind`。
//! 统一了查询系统中的所有操作类型，贯穿 Parser、Validator、Planner、Optimizer 和 Executor 五个模块。
//! 通过统一的类型定义，减少了各模块之间的类型映射复杂性，提高了代码的可维护性。

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreOperationKind {
    // ==================== 数据查询操作 ====================
    
    /// MATCH 查询 - 图模式匹配查询
    Match,
    
    /// GO 查询 - 简单的图遍历查询
    Go,
    
    /// LOOKUP 查询 - 基于索引的查找查询
    Lookup,
    
    /// FIND PATH 查询 - 查找两点之间的路径
    FindPath,
    
    /// GET SUBGRAPH 查询 - 获取子图
    GetSubgraph,
    
    // ==================== 数据访问操作 ====================
    
    /// 扫描所有顶点
    ScanVertices,
    
    /// 扫描所有边
    ScanEdges,
    
    /// 获取指定顶点
    GetVertices,
    
    /// 获取指定边
    GetEdges,
    
    /// 获取邻居节点
    GetNeighbors,
    
    // ==================== 数据转换操作 ====================
    
    /// 项目操作 - 选择输出列
    Project,
    
    /// 过滤操作 - 根据条件筛选行
    Filter,
    
    /// 排序操作 - 对结果排序
    Sort,
    
    /// 限制操作 - 限制返回行数
    Limit,
    
    /// TopN 操作 - 获取前 N 行
    TopN,
    
    /// 采样操作 - 随机采样
    Sample,
    
    /// 展开操作 - 将数组展开为行
    Unwind,
    
    // ==================== 数据聚合操作 ====================
    
    /// 聚合操作 - 分组聚合
    Aggregate,
    
    /// 分组操作 - GROUP BY
    GroupBy,
    
    /// HAVING 操作 - 分组后过滤
    Having,
    
    /// 去重操作 - 去除重复行
    Dedup,
    
    // ==================== 连接操作 ====================
    
    /// 内连接 - INNER JOIN
    InnerJoin,
    
    /// 左连接 - LEFT JOIN
    LeftJoin,
    
    /// 交叉连接 - CROSS JOIN
    CrossJoin,
    
    /// 哈希连接 - HASH JOIN
    HashJoin,
    
    // ==================== 图遍历操作 ====================
    
    /// 遍历操作 - 广度优先遍历
    Traverse,
    
    /// 扩展操作 - 扩展到邻居节点
    Expand,
    
    /// 全扩展操作 - 扩展到所有层级的邻居
    ExpandAll,
    
    /// 最短路径 - 单源最短路径
    ShortestPath,
    
    /// 所有路径 - 查找所有路径
    AllPaths,
    
    /// 多源最短路径
    MultiShortestPath,
    
    /// BFS 最短路径
    BFSShortest,
    
    // ==================== 数据修改操作 ====================
    
    /// 插入操作 - INSERT
    Insert,
    
    /// 更新操作 - UPDATE
    Update,
    
    /// 删除操作 - DELETE
    Delete,
    
    /// 合并操作 - MERGE
    Merge,
    
    // ==================== 模式匹配操作 ====================
    
    /// 模式应用 - PATTERN APPLY
    PatternApply,
    
    /// 卷起应用 - ROLL UP APPLY
    RollUpApply,
    
    // ==================== 循环控制操作 ====================
    
    /// 循环 - LOOP
    Loop,
    
    /// FOR 循环 - FOR LOOP
    ForLoop,
    
    /// WHILE 循环 - WHILE LOOP
    WhileLoop,
    
    // ==================== 空间管理操作 ====================
    
    /// 创建空间 - CREATE SPACE
    CreateSpace,
    
    /// 删除空间 - DROP SPACE
    DropSpace,
    
    /// 描述空间 - DESCRIBE SPACE
    DescribeSpace,
    
    /// 使用空间 - USE SPACE
    UseSpace,
    
    /// 显示空间 - SHOW SPACES
    ShowSpaces,
    
    // ==================== 标签管理操作 ====================
    
    /// 创建标签 - CREATE TAG
    CreateTag,
    
    /// 修改标签 - ALTER TAG
    AlterTag,
    
    /// 删除标签 - DROP TAG
    DropTag,
    
    /// 描述标签 - DESCRIBE TAG
    DescribeTag,
    
    /// 显示标签 - SHOW TAGS
    ShowTags,
    
    // ==================== 边类型管理操作 ====================
    
    /// 创建边类型 - CREATE EDGE
    CreateEdge,
    
    /// 修改边类型 - ALTER EDGE
    AlterEdge,
    
    /// 删除边类型 - DROP EDGE
    DropEdge,
    
    /// 描述边类型 - DESCRIBE EDGE
    DescribeEdge,
    
    /// 显示边类型 - SHOW EDGES
    ShowEdges,
    
    // ==================== 索引管理操作 ====================
    
    /// 创建索引 - CREATE INDEX
    CreateIndex,
    
    /// 删除索引 - DROP INDEX
    DropIndex,
    
    /// 描述索引 - DESCRIBE INDEX
    DescribeIndex,
    
    /// 重建索引 - REBUILD INDEX
    RebuildIndex,
    
    /// 全文索引扫描 - FULLTEXT INDEX SCAN
    FulltextIndexScan,
    
    /// 索引扫描 - INDEX SCAN
    IndexScan,
    
    // ==================== 用户管理操作 ====================
    
    /// 创建用户 - CREATE USER
    CreateUser,
    
    /// 修改用户 - ALTER USER
    AlterUser,
    
    /// 删除用户 - DROP USER
    DropUser,
    
    /// 修改密码 - CHANGE PASSWORD
    ChangePassword,
    
    // ==================== 作业控制操作 ====================
    
    /// 设置操作 - SET
    Set,
    
    /// 分配操作 - ASSIGNMENT
    Assignment,
    
    /// 管道操作 - PIPE
    Pipe,
    
    /// 解释执行 - EXPLAIN
    Explain,
    
    /// 显示操作 - SHOW
    Show,
    
    /// 顺序执行 - SEQUENTIAL
    Sequential,
    
    // ==================== 结果处理操作 ====================
    
    /// 参数传递 - ARGUMENT
    Argument,
    
    /// 直通 - PASS THROUGH
    PassThrough,
    
    /// 选择 - SELECT
    Select,
    
    /// 数据收集 - DATA COLLECT
    DataCollect,
    
    /// 求差 - MINUS
    Minus,
    
    /// 交集 - INTERSECT
    Intersect,
    
    /// 并集 - UNION
    Union,
    
    /// 全并集 - UNION ALL
    UnionAll,
    
    /// 追加顶点 - APPEND VERTICES
    AppendVertices,
    
    /// 赋值 - ASSIGN
    Assign,
    
    /// 移除 - REMOVE
    Remove,
}

impl CoreOperationKind {
    /// 获取操作类别的名称
    pub fn category(&self) -> &'static str {
        match self {
            // 数据查询
            Self::Match | Self::Go | Self::Lookup | Self::FindPath | Self::GetSubgraph => "DATA_QUERY",
            
            // 数据访问
            Self::ScanVertices | Self::ScanEdges | Self::GetVertices | Self::GetEdges | Self::GetNeighbors => "DATA_ACCESS",
            
            // 数据转换
            Self::Project | Self::Filter | Self::Sort | Self::Limit | Self::TopN | Self::Sample | Self::Unwind => "DATA_TRANSFORMATION",
            
            // 数据聚合
            Self::Aggregate | Self::GroupBy | Self::Having | Self::Dedup => "DATA_AGGREGATION",
            
            // 连接操作
            Self::InnerJoin | Self::LeftJoin | Self::CrossJoin | Self::HashJoin => "JOIN",
            
            // 图遍历
            Self::Traverse | Self::Expand | Self::ExpandAll | Self::ShortestPath | Self::AllPaths | Self::MultiShortestPath | Self::BFSShortest => "GRAPH_TRAVERSAL",
            
            // 数据修改
            Self::Insert | Self::Update | Self::Delete | Self::Merge => "DATA_MODIFICATION",
            
            // 模式匹配
            Self::PatternApply | Self::RollUpApply => "PATTERN_MATCHING",
            
            // 循环控制
            Self::Loop | Self::ForLoop | Self::WhileLoop => "LOOP_CONTROL",
            
            // 空间管理
            Self::CreateSpace | Self::DropSpace | Self::DescribeSpace | Self::UseSpace | Self::ShowSpaces => "SPACE_MANAGEMENT",
            
            // 标签管理
            Self::CreateTag | Self::AlterTag | Self::DropTag | Self::DescribeTag | Self::ShowTags => "TAG_MANAGEMENT",
            
            // 边类型管理
            Self::CreateEdge | Self::AlterEdge | Self::DropEdge | Self::DescribeEdge | Self::ShowEdges => "EDGE_MANAGEMENT",
            
            // 索引管理
            Self::CreateIndex | Self::DropIndex | Self::DescribeIndex | Self::RebuildIndex | Self::FulltextIndexScan | Self::IndexScan => "INDEX_MANAGEMENT",
            
            // 用户管理
            Self::CreateUser | Self::AlterUser | Self::DropUser | Self::ChangePassword => "USER_MANAGEMENT",
            
            // 作业控制
            Self::Set | Self::Assignment | Self::Pipe | Self::Explain | Self::Show | Self::Sequential => "CONTROL",
            
            // 结果处理
            Self::Argument | Self::PassThrough | Self::Select | Self::DataCollect => "RESULT_PROCESSING",
            Self::Minus | Self::Intersect | Self::Union | Self::UnionAll | Self::AppendVertices | Self::Assign | Self::Remove => "SET_OPERATION",
        }
    }
    
    /// 判断是否为只读操作
    pub fn is_read_only(&self) -> bool {
        matches!(
            self,
            Self::Match | Self::Go | Self::Lookup | Self::FindPath | Self::GetSubgraph
                | Self::ScanVertices | Self::ScanEdges | Self::GetVertices | Self::GetEdges | Self::GetNeighbors
                | Self::Project | Self::Filter | Self::Sort | Self::Limit | Self::TopN | Self::Sample | Self::Unwind
                | Self::Aggregate | Self::GroupBy | Self::Having | Self::Dedup
                | Self::InnerJoin | Self::LeftJoin | Self::CrossJoin | Self::HashJoin
                | Self::Traverse | Self::Expand | Self::ExpandAll | Self::ShortestPath | Self::AllPaths | Self::MultiShortestPath | Self::BFSShortest
                | Self::DescribeSpace | Self::DescribeTag | Self::DescribeEdge | Self::DescribeIndex
                | Self::Show | Self::Explain | Self::ShowSpaces | Self::ShowTags | Self::ShowEdges
                | Self::Argument | Self::PassThrough | Self::Select | Self::DataCollect
                | Self::IndexScan | Self::FulltextIndexScan
        )
    }
    
    /// 判断是否为元数据操作
    pub fn is_metadata_operation(&self) -> bool {
        matches!(
            self,
            Self::CreateSpace | Self::DropSpace | Self::DescribeSpace | Self::UseSpace | Self::ShowSpaces
                | Self::CreateTag | Self::AlterTag | Self::DropTag | Self::DescribeTag | Self::ShowTags
                | Self::CreateEdge | Self::AlterEdge | Self::DropEdge | Self::DescribeEdge | Self::ShowEdges
                | Self::CreateIndex | Self::DropIndex | Self::DescribeIndex | Self::RebuildIndex
                | Self::CreateUser | Self::AlterUser | Self::DropUser | Self::ChangePassword
                | Self::Show | Self::Explain
        )
    }
    
    /// 判断是否为 DML 操作
    pub fn is_dml(&self) -> bool {
        matches!(
            self,
            Self::Insert | Self::Update | Self::Delete | Self::Merge
        )
    }
    
    /// 判断是否为 DDL 操作
    pub fn is_ddl(&self) -> bool {
        matches!(
            self,
            Self::CreateSpace | Self::DropSpace | Self::CreateTag | Self::AlterTag | Self::DropTag
                | Self::CreateEdge | Self::AlterEdge | Self::DropEdge
                | Self::CreateIndex | Self::DropIndex | Self::RebuildIndex
        )
    }
    
    /// 获取操作的字符串表示
    pub fn name(&self) -> &'static str {
        match self {
            Self::Match => "MATCH",
            Self::Go => "GO",
            Self::Lookup => "LOOKUP",
            Self::FindPath => "FIND_PATH",
            Self::GetSubgraph => "GET_SUBGRAPH",
            Self::ScanVertices => "SCAN_VERTICES",
            Self::ScanEdges => "SCAN_EDGES",
            Self::GetVertices => "GET_VERTICES",
            Self::GetEdges => "GET_EDGES",
            Self::GetNeighbors => "GET_NEIGHBORS",
            Self::Project => "PROJECT",
            Self::Filter => "FILTER",
            Self::Sort => "SORT",
            Self::Limit => "LIMIT",
            Self::TopN => "TOPN",
            Self::Sample => "SAMPLE",
            Self::Unwind => "UNWIND",
            Self::Aggregate => "AGGREGATE",
            Self::GroupBy => "GROUP_BY",
            Self::Having => "HAVING",
            Self::Dedup => "DEDUP",
            Self::InnerJoin => "INNER_JOIN",
            Self::LeftJoin => "LEFT_JOIN",
            Self::CrossJoin => "CROSS_JOIN",
            Self::HashJoin => "HASH_JOIN",
            Self::Traverse => "TRAVERSE",
            Self::Expand => "EXPAND",
            Self::ExpandAll => "EXPAND_ALL",
            Self::ShortestPath => "SHORTEST_PATH",
            Self::AllPaths => "ALL_PATHS",
            Self::MultiShortestPath => "MULTI_SHORTEST_PATH",
            Self::BFSShortest => "BFS_SHORTEST_PATH",
            Self::Insert => "INSERT",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
            Self::Merge => "MERGE",
            Self::PatternApply => "PATTERN_APPLY",
            Self::RollUpApply => "ROLLUP_APPLY",
            Self::Loop => "LOOP",
            Self::ForLoop => "FOR_LOOP",
            Self::WhileLoop => "WHILE_LOOP",
            Self::CreateSpace => "CREATE_SPACE",
            Self::DropSpace => "DROP_SPACE",
            Self::DescribeSpace => "DESCRIBE_SPACE",
            Self::UseSpace => "USE_SPACE",
            Self::ShowSpaces => "SHOW_SPACES",
            Self::CreateTag => "CREATE_TAG",
            Self::AlterTag => "ALTER_TAG",
            Self::DropTag => "DROP_TAG",
            Self::DescribeTag => "DESCRIBE_TAG",
            Self::ShowTags => "SHOW_TAGS",
            Self::CreateEdge => "CREATE_EDGE",
            Self::AlterEdge => "ALTER_EDGE",
            Self::DropEdge => "DROP_EDGE",
            Self::DescribeEdge => "DESCRIBE_EDGE",
            Self::ShowEdges => "SHOW_EDGES",
            Self::CreateIndex => "CREATE_INDEX",
            Self::DropIndex => "DROP_INDEX",
            Self::DescribeIndex => "DESCRIBE_INDEX",
            Self::RebuildIndex => "REBUILD_INDEX",
            Self::FulltextIndexScan => "FULLTEXT_INDEX_SCAN",
            Self::IndexScan => "INDEX_SCAN",
            Self::CreateUser => "CREATE_USER",
            Self::AlterUser => "ALTER_USER",
            Self::DropUser => "DROP_USER",
            Self::ChangePassword => "CHANGE_PASSWORD",
            Self::Set => "SET",
            Self::Assignment => "ASSIGNMENT",
            Self::Pipe => "PIPE",
            Self::Explain => "EXPLAIN",
            Self::Show => "SHOW",
            Self::Sequential => "SEQUENTIAL",
            Self::Argument => "ARGUMENT",
            Self::PassThrough => "PASS_THROUGH",
            Self::Select => "SELECT",
            Self::DataCollect => "DATA_COLLECT",
            Self::Minus => "MINUS",
            Self::Intersect => "INTERSECT",
            Self::Union => "UNION",
            Self::UnionAll => "UNION_ALL",
            Self::AppendVertices => "APPEND_VERTICES",
            Self::Assign => "ASSIGN",
            Self::Remove => "REMOVE",
        }
    }
}

impl fmt::Display for CoreOperationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
