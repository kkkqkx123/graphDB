//! 索引扫描相关的计划节点
//! 包含索引扫描等搜索相关操作

use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::types::graph_schema::OrderDirection;
use crate::define_plan_node;
use crate::query::planner::plan::core::node_id_generator::next_node_id;
use crate::query::planner::plan::core::nodes::base::plan_node_visitor::PlanNodeVisitor;

/// 排序项定义
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByItem {
    pub column: String,
    pub direction: OrderDirection,
}

impl OrderByItem {
    pub fn new(column: impl Into<String>, direction: OrderDirection) -> Self {
        Self {
            column: column.into(),
            direction,
        }
    }

    pub fn asc(column: impl Into<String>) -> Self {
        Self::new(column, OrderDirection::Asc)
    }

    pub fn desc(column: impl Into<String>) -> Self {
        Self::new(column, OrderDirection::Desc)
    }
}

/// 索引扫描类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScanType {
    /// 唯一匹配（等值查询）
    #[default]
    Unique,
    /// 前缀匹配
    Prefix,
    /// 范围查询
    Range,
    /// 全表扫描
    Full,
}

impl ScanType {
    /// 从字符串解析扫描类型
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "UNIQUE" => ScanType::Unique,
            "PREFIX" => ScanType::Prefix,
            "RANGE" => ScanType::Range,
            "FULL" => ScanType::Full,
            _ => ScanType::Range, // 默认使用范围扫描
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            ScanType::Unique => "UNIQUE",
            ScanType::Prefix => "PREFIX",
            ScanType::Range => "RANGE",
            ScanType::Full => "FULL",
        }
    }
}

/// 索引扫描限制条件
#[derive(Debug, Clone)]
pub struct IndexLimit {
    pub column: String,
    pub begin_value: Option<String>,
    pub end_value: Option<String>,
    /// 是否包含起始值
    pub include_begin: bool,
    /// 是否包含结束值
    pub include_end: bool,
    /// 扫描类型
    pub scan_type: ScanType,
}

impl IndexLimit {
    /// 创建等值查询限制
    pub fn equal(column: impl Into<String>, value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            column: column.into(),
            begin_value: Some(value.clone()),
            end_value: Some(value),
            include_begin: true,
            include_end: true,
            scan_type: ScanType::Unique,
        }
    }

    /// 创建范围查询限制
    pub fn range(
        column: impl Into<String>,
        begin: Option<impl Into<String>>,
        end: Option<impl Into<String>>,
        include_begin: bool,
        include_end: bool,
    ) -> Self {
        Self {
            column: column.into(),
            begin_value: begin.map(|v| v.into()),
            end_value: end.map(|v| v.into()),
            include_begin,
            include_end,
            scan_type: ScanType::Range,
        }
    }

    /// 创建前缀查询限制
    pub fn prefix(column: impl Into<String>, prefix: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            begin_value: Some(prefix.into()),
            end_value: None,
            include_begin: true,
            include_end: false,
            scan_type: ScanType::Prefix,
        }
    }
}

define_plan_node! {
    /// 索引扫描计划节点
    pub struct IndexScanNode {
        space_id: u64,
        tag_id: i32,
        index_id: i32,
        scan_type: ScanType,
        scan_limits: Vec<IndexLimit>,
        filter: Option<ContextualExpression>,
        return_columns: Vec<String>,
        limit: Option<i64>,
        order_by: Vec<OrderByItem>,
    }
    enum: IndexScan
    input: ZeroInputNode
}

impl IndexScanNode {
    pub fn new(space_id: u64, tag_id: i32, index_id: i32, scan_type: ScanType) -> Self {
        Self {
            id: next_node_id(),
            space_id,
            tag_id,
            index_id,
            scan_type,
            scan_limits: Vec::new(),
            filter: None,
            return_columns: Vec::new(),
            limit: None,
            order_by: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
        }
    }

    /// 从字符串创建新的 IndexScanNode
    pub fn new_with_str(
        space_id: u64,
        tag_id: i32,
        index_id: i32,
        scan_type: &str,
    ) -> Self {
        Self::new(
            space_id,
            tag_id,
            index_id,
            ScanType::from_str(scan_type),
        )
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn set_order_by(&mut self, order_by: Vec<OrderByItem>) {
        self.order_by = order_by;
    }

    pub fn has_effective_filter(&self) -> bool {
        self.filter.is_some() || !self.scan_limits.is_empty()
    }

    pub fn is_tag_scan(&self) -> bool {
        self.tag_id > 0
    }

    pub fn is_edge_scan(&self) -> bool {
        self.tag_id <= 0
    }

    pub fn index_name(&self) -> String {
        format!("index_{}", self.index_id)
    }

    pub fn space_id(&self) -> u64 {
        self.space_id
    }

    pub fn tag_id(&self) -> i32 {
        self.tag_id
    }

    pub fn index_id(&self) -> i32 {
        self.index_id
    }

    pub fn scan_type(&self) -> ScanType {
        self.scan_type
    }

    pub fn scan_limits(&self) -> &[IndexLimit] {
        &self.scan_limits
    }

    pub fn set_scan_limits(&mut self, limits: Vec<IndexLimit>) {
        self.scan_limits = limits;
    }

    pub fn filter(&self) -> Option<&ContextualExpression> {
        self.filter.as_ref()
    }

    pub fn set_filter(&mut self, filter: ContextualExpression) {
        self.filter = Some(filter);
    }

    pub fn return_columns(&self) -> &[String] {
        &self.return_columns
    }

    pub fn set_return_columns(&mut self, columns: Vec<String>) {
        self.return_columns = columns;
    }

    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    pub fn order_by(&self) -> &[OrderByItem] {
        &self.order_by
    }

    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: PlanNodeVisitor,
    {
        visitor.visit_index_scan(self)
    }
}
