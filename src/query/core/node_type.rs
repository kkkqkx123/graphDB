//! 节点类型统一接口
//!
//! 提供 PlanNodeEnum 和 ExecutorEnum 的统一 trait 接口，
//! 用于确保两个枚举之间的一致性和可追踪性。

/// 节点分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeCategory {
    /// 扫描操作节点
    Scan,
    /// 连接操作节点
    Join,
    /// 过滤操作节点
    Filter,
    /// 投影操作节点
    Project,
    /// 聚合操作节点
    Aggregate,
    /// 排序操作节点
    Sort,
    /// 控制流节点
    Control,
    /// 数据收集节点
    DataCollect,
    /// 遍历操作节点
    Traversal,
    /// 集合操作节点
    SetOp,
    /// 路径算法节点
    Path,
    /// 管理操作节点
    Admin,
    /// 其他类型
    Other,
}

impl NodeCategory {
    /// 获取分类的名称
    pub fn name(&self) -> &'static str {
        match self {
            NodeCategory::Scan => "Scan",
            NodeCategory::Join => "Join",
            NodeCategory::Filter => "Filter",
            NodeCategory::Project => "Project",
            NodeCategory::Aggregate => "Aggregate",
            NodeCategory::Sort => "Sort",
            NodeCategory::Control => "Control",
            NodeCategory::DataCollect => "DataCollect",
            NodeCategory::Traversal => "Traversal",
            NodeCategory::SetOp => "SetOp",
            NodeCategory::Path => "Path",
            NodeCategory::Admin => "Admin",
            NodeCategory::Other => "Other",
        }
    }
}

/// 节点类型统一接口
///
/// 此 trait 用于统一 PlanNodeEnum 和 ExecutorEnum 的接口，
/// 确保两个枚举在语义上保持一致。
pub trait NodeType {
    /// 获取节点类型的唯一标识符
    ///
    /// 返回值应该是全局唯一的字符串标识符，
    /// 例如："cross_join", "index_scan" 等
    fn node_type_id(&self) -> &'static str;

    /// 获取节点类型的名称
    ///
    /// 返回值应该是人类可读的名称，
    /// 例如："Cross Join", "Index Scan" 等
    fn node_type_name(&self) -> &'static str;

    /// 获取节点所属的分类
    fn category(&self) -> NodeCategory;

    /// 判断是否为扫描节点
    fn is_scan(&self) -> bool {
        self.category() == NodeCategory::Scan
    }

    /// 判断是否为连接节点
    fn is_join(&self) -> bool {
        self.category() == NodeCategory::Join
    }

    /// 判断是否为过滤节点
    fn is_filter(&self) -> bool {
        self.category() == NodeCategory::Filter
    }

    /// 判断是否为投影节点
    fn is_project(&self) -> bool {
        self.category() == NodeCategory::Project
    }

    /// 判断是否为聚合节点
    fn is_aggregate(&self) -> bool {
        self.category() == NodeCategory::Aggregate
    }

    /// 判断是否为控制流节点
    fn is_control(&self) -> bool {
        self.category() == NodeCategory::Control
    }

    /// 判断是否为管理节点
    fn is_admin(&self) -> bool {
        self.category() == NodeCategory::Admin
    }
}

/// 节点类型注册表
///
/// 用于在编译期检查节点类型的一致性
pub struct NodeTypeRegistry {
    /// 注册的节点类型数量
    pub plan_node_count: usize,
    pub executor_count: usize,
}

impl NodeTypeRegistry {
    /// 创建新的注册表
    pub const fn new(plan_node_count: usize, executor_count: usize) -> Self {
        Self {
            plan_node_count,
            executor_count,
        }
    }

    /// 检查两个枚举的变体数量是否一致
    pub const fn check_consistency(&self) -> bool {
        self.plan_node_count == self.executor_count
    }
}

/// 编译期断言宏
///
/// 用于在编译期检查 PlanNodeEnum 和 ExecutorEnum 的变体数量是否一致
#[macro_export]
macro_rules! assert_enum_consistency {
    ($plan_node_count:expr, $executor_count:expr) => {
        const _: () = assert!(
            $plan_node_count == $executor_count,
            "PlanNodeEnum 和 ExecutorEnum 的变体数量不一致，可能导致某些节点没有对应的执行器"
        );
    };
}

/// 节点类型映射 trait
///
/// 用于将 PlanNodeEnum 映射到对应的 ExecutorEnum
pub trait NodeTypeMapping {
    /// 获取对应的执行器类型 ID
    fn corresponding_executor_type(&self) -> Option<&'static str>;
}
