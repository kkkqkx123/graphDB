//! 计划节点模块 - 定义计划节点的引用和类型

/// 计划节点引用
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlanNodeRef {
    pub id: String,
    pub node_type: String,
}

impl PlanNodeRef {
    pub fn new(id: String, node_type: String) -> Self {
        Self { id, node_type }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn node_type(&self) -> &str {
        &self.node_type
    }
}

/// 计划节点类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlanNodeType {
    Scan,
    Filter,
    Project,
    Join,
    Aggregate,
    Sort,
    Limit,
    Union,
    Intersect,
    Except,
    Unknown(String),
}

impl PlanNodeType {
    pub fn as_str(&self) -> &str {
        match self {
            PlanNodeType::Scan => "Scan",
            PlanNodeType::Filter => "Filter",
            PlanNodeType::Project => "Project",
            PlanNodeType::Join => "Join",
            PlanNodeType::Aggregate => "Aggregate",
            PlanNodeType::Sort => "Sort",
            PlanNodeType::Limit => "Limit",
            PlanNodeType::Union => "Union",
            PlanNodeType::Intersect => "Intersect",
            PlanNodeType::Except => "Except",
            PlanNodeType::Unknown(name) => name,
        }
    }
}

impl From<&str> for PlanNodeType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "scan" => PlanNodeType::Scan,
            "filter" => PlanNodeType::Filter,
            "project" => PlanNodeType::Project,
            "join" => PlanNodeType::Join,
            "aggregate" => PlanNodeType::Aggregate,
            "sort" => PlanNodeType::Sort,
            "limit" => PlanNodeType::Limit,
            "union" => PlanNodeType::Union,
            "intersect" => PlanNodeType::Intersect,
            "except" => PlanNodeType::Except,
            _ => PlanNodeType::Unknown(s.to_string()),
        }
    }
}

impl From<&String> for PlanNodeType {
    fn from(s: &String) -> Self {
        PlanNodeType::from(s.as_str())
    }
}

impl PlanNodeRef {
    /// 从节点类型创建引用
    pub fn from_type(id: String, node_type: PlanNodeType) -> Self {
        Self {
            id,
            node_type: node_type.as_str().to_string(),
        }
    }

    /// 获取节点类型枚举
    pub fn get_node_type_enum(&self) -> PlanNodeType {
        PlanNodeType::from(&self.node_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_node_ref() {
        let node = PlanNodeRef::new("node_1".to_string(), "Scan".to_string());
        assert_eq!(node.id(), "node_1");
        assert_eq!(node.node_type(), "Scan");
    }

    #[test]
    fn test_plan_node_type() {
        assert_eq!(PlanNodeType::Scan.as_str(), "Scan");
        assert_eq!(PlanNodeType::Filter.as_str(), "Filter");
        assert_eq!(
            PlanNodeType::Unknown("Custom".to_string()).as_str(),
            "Custom"
        );
    }

    #[test]
    fn test_plan_node_type_conversion() {
        let scan_type: PlanNodeType = "scan".into();
        assert_eq!(scan_type, PlanNodeType::Scan);

        let unknown_type: PlanNodeType = "custom".into();
        assert_eq!(unknown_type, PlanNodeType::Unknown("custom".to_string()));
    }

    #[test]
    fn test_plan_node_ref_from_type() {
        let node = PlanNodeRef::from_type("node_1".to_string(), PlanNodeType::Project);
        assert_eq!(node.id(), "node_1");
        assert_eq!(node.node_type(), "Project");
        assert_eq!(node.get_node_type_enum(), PlanNodeType::Project);
    }
}
