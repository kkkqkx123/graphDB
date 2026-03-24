//! 通用计划节点结构定义

// 标签属性结构
#[derive(Debug, Clone)]
pub struct TagProp {
    pub tag: String,
    pub props: Vec<String>,
}

impl TagProp {
    pub fn new(tag: &str, props: Vec<String>) -> Self {
        Self {
            tag: tag.to_string(),
            props,
        }
    }
}

// 边属性结构
#[derive(Debug, Clone)]
pub struct EdgeProp {
    pub edge_type: String,
    pub props: Vec<String>,
}

impl EdgeProp {
    pub fn new(edge_type: &str, props: Vec<String>) -> Self {
        Self {
            edge_type: edge_type.to_string(),
            props,
        }
    }
}
