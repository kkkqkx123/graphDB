// 图相关类型定义
//
// 包含图数据库中图结构相关的核心类型定义

/// 边的方向类型
///
/// 用于表示边的遍历方向，支持出边、入边和双向遍历
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeDirection {
    /// 出边：从源节点指向目标节点
    Outgoing,
    /// 入边：从目标节点指向源节点
    Incoming,
    /// 双向：同时包含出边和入边
    Both,
}

impl EdgeDirection {
    /// 判断是否包含出边
    pub fn is_outgoing(&self) -> bool {
        matches!(self, EdgeDirection::Outgoing | EdgeDirection::Both)
    }

    /// 判断是否包含入边
    pub fn is_incoming(&self) -> bool {
        matches!(self, EdgeDirection::Incoming | EdgeDirection::Both)
    }

    /// 获取反向方向
    pub fn reverse(&self) -> Self {
        match self {
            EdgeDirection::Outgoing => EdgeDirection::Incoming,
            EdgeDirection::Incoming => EdgeDirection::Outgoing,
            EdgeDirection::Both => EdgeDirection::Both,
        }
    }
}

impl From<&str> for EdgeDirection {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "out" | "outgoing" => EdgeDirection::Outgoing,
            "in" | "incoming" => EdgeDirection::Incoming,
            "both" | "bidirectional" => EdgeDirection::Both,
            _ => EdgeDirection::Both,
        }
    }
}

impl From<String> for EdgeDirection {
    fn from(s: String) -> Self {
        EdgeDirection::from(s.as_str())
    }
}
