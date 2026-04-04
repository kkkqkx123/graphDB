//! Definition of Plan Node Classification
//!
//! Classify PlanNodes based on their functional characteristics to facilitate decision-making by the optimizer and to improve the organization of the code.

/// Plan node classification enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanNodeCategory {
    /// Access layer – Reads data from the storage layer
    Access,
    /// Operation Layer – Data Conversion and Filtering
    Operation,
    /// Connection layers – Multi-data stream connections
    Join,
    /// Traversal layer – Image traversal and expansion
    Traversal,
    /// Controlling the stratosphere – Managing the execution of processes
    ControlFlow,
    /// Data processing layer – Complex data operations
    DataProcessing,
    /// Algorithm Layer – Execution of Graph Algorithms
    Algorithm,
    /// Management/DDL Layer – Metadata Management
    Management,
    /// Data Access Layer – Full-text search and other data access operations
    DataAccess,
}

impl PlanNodeCategory {
    /// Obtain the category names
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
            PlanNodeCategory::DataAccess => "DataAccess",
        }
    }

    /// Please provide the text that needs to be translated into Chinese.
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
            PlanNodeCategory::DataAccess => "数据访问层 - 全文搜索等数据访问操作",
        }
    }

    /// Determine whether it is a leaf node (with no data dependencies).
    pub fn is_leaf(&self) -> bool {
        matches!(self, PlanNodeCategory::Access | PlanNodeCategory::DataAccess)
    }

    /// Determine whether it is a root node (with no downstream dependencies).
    pub fn is_root(&self) -> bool {
        matches!(
            self,
            PlanNodeCategory::ControlFlow | PlanNodeCategory::DataProcessing
        )
    }

    /// Determine whether parallel execution is supported.
    pub fn supports_parallelism(&self) -> bool {
        matches!(
            self,
            PlanNodeCategory::Operation
                | PlanNodeCategory::DataProcessing
                | PlanNodeCategory::Algorithm
        )
    }
}

impl std::fmt::Display for PlanNodeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
