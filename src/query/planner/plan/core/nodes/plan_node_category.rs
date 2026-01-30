//! 计划节点分类定义
//!
//! 根据功能特性对 PlanNode 进行分类，便于优化器决策和代码组织

/// 计划节点分类枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanNodeCategory {
    /// 访问层 - 从存储层读取数据
    Access,
    /// 操作层 - 数据转换和过滤
    Operation,
    /// 连接层 - 多数据流连接
    Join,
    /// 遍历层 - 图遍历和扩展
    Traversal,
    /// 控制流层 - 执行流程控制
    ControlFlow,
    /// 数据处理层 - 复杂数据操作
    DataProcessing,
    /// 算法层 - 图算法执行
    Algorithm,
    /// 管理/DDL层 - 元数据管理
    Management,
}

impl PlanNodeCategory {
    /// 获取分类名称
    pub fn name(&self) -> &'static str {
        match self {
            PlanNodeCategory::Access => "Access",
            PlanNodeCategory::Operation => "Operation",
            PlanNodeCategory::Join => "Join",
            PlanNodeCategory::Traversal => "Traversal",
            PlanNodeCategory::ControlFlow => "ControlFlow",
            PlanNodeCategory::DataProcessing => "DataProcessing",
            PlanNodeCategory::Algorithm => "Algorithm",
            PlanNodeCategory::Management => "Management",
        }
    }

    /// 获取中文描述
    pub fn description(&self) -> &'static str {
        match self {
            PlanNodeCategory::Access => "访问层 - 从存储层读取数据",
            PlanNodeCategory::Operation => "操作层 - 数据转换和过滤",
            PlanNodeCategory::Join => "连接层 - 多数据流连接",
            PlanNodeCategory::Traversal => "遍历层 - 图遍历和扩展",
            PlanNodeCategory::ControlFlow => "控制流层 - 执行流程控制",
            PlanNodeCategory::DataProcessing => "数据处理层 - 复杂数据操作",
            PlanNodeCategory::Algorithm => "算法层 - 图算法执行",
            PlanNodeCategory::Management => "管理/DDL层 - 元数据管理",
        }
    }

    /// 判断是否为叶子节点（无数据依赖）
    pub fn is_leaf(&self) -> bool {
        matches!(self, PlanNodeCategory::Access)
    }

    /// 判断是否为根节点（无下游依赖）
    pub fn is_root(&self) -> bool {
        matches!(self, PlanNodeCategory::ControlFlow | PlanNodeCategory::DataProcessing)
    }

    /// 判断是否支持并行执行
    pub fn supports_parallelism(&self) -> bool {
        matches!(
            self,
            PlanNodeCategory::Operation | PlanNodeCategory::DataProcessing | PlanNodeCategory::Algorithm
        )
    }
}

impl std::fmt::Display for PlanNodeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
