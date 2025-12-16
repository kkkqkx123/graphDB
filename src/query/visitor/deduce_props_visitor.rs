//! DeducePropsVisitor - 用于推导表达式属性的访问器
//! 对应 NebulaGraph DeducePropsVisitor.h/.cpp 的功能

use crate::graph::expression::Expression;
use std::collections::{HashMap, HashSet};

/// 属性定义
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropDef {
    pub name: String,
    pub type_: String,
}

/// 节点信息 - 记录查询中涉及的节点及其属性
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub alias: String,
    pub props: HashSet<String>,
    pub vid: Option<Box<Expression>>,
    pub tags: Vec<String>, // 节点上的标签列表
}

/// 边信息 - 记录查询中涉及的边及其属性
#[derive(Debug, Clone)]
pub struct EdgeInfo {
    pub alias: String,
    pub props: HashSet<String>,
    pub type_name: String,
    pub src: Option<Box<Expression>>,
    pub dst: Option<Box<Expression>>,
    pub rank: Option<Box<Expression>>,
    pub steps: String, // "1" or "*"
}

/// 表达式属性集合 - 统一管理所有类型的属性
#[derive(Debug, Clone)]
pub struct ExpressionProps {
    /// 输入列属性 - 来自上一步的输出（$-）
    pub input_props: HashSet<String>,
    /// 变量属性 - 从变量中获取（$varName.prop）
    pub var_props: HashMap<String, HashSet<String>>,
    /// 源标签属性 - 起点节点属性（$^.tagName.prop）
    pub src_tag_props: HashMap<String, HashSet<String>>,
    /// 目标标签属性 - 终点节点属性（$$.tagName.prop）
    pub dst_tag_props: HashMap<String, HashSet<String>>,
    /// 普通标签属性 - 节点标签属性（tagName.prop）
    pub tag_props: HashMap<String, HashSet<String>>,
    /// 边属性 - 边的属性（edgeName.prop）
    pub edge_props: HashMap<String, HashSet<String>>,
    /// 标签名称映射 - tagName -> tagId
    pub tag_name_ids: HashMap<String, String>,
}

impl ExpressionProps {
    pub fn new() -> Self {
        Self {
            input_props: HashSet::new(),
            var_props: HashMap::new(),
            src_tag_props: HashMap::new(),
            dst_tag_props: HashMap::new(),
            tag_props: HashMap::new(),
            edge_props: HashMap::new(),
            tag_name_ids: HashMap::new(),
        }
    }

    /// 插入输入属性
    pub fn insert_input_prop(&mut self, prop: &str) {
        self.input_props.insert(prop.to_string());
    }

    /// 插入变量属性
    pub fn insert_var_prop(&mut self, var: &str, prop: &str) {
        self.var_props
            .entry(var.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop.to_string());
    }

    /// 插入源标签属性
    pub fn insert_src_tag_prop(&mut self, tag: &str, prop: &str) {
        self.src_tag_props
            .entry(tag.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop.to_string());
    }

    /// 插入目标标签属性
    pub fn insert_dst_tag_prop(&mut self, tag: &str, prop: &str) {
        self.dst_tag_props
            .entry(tag.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop.to_string());
    }

    /// 插入普通标签属性
    pub fn insert_tag_prop(&mut self, tag: &str, prop: &str) {
        self.tag_props
            .entry(tag.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop.to_string());
    }

    /// 插入边属性
    pub fn insert_edge_prop(&mut self, edge: &str, prop: &str) {
        self.edge_props
            .entry(edge.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop.to_string());
    }

    /// 记录标签名称到ID的映射
    pub fn insert_tag_name_id(&mut self, name: &str, id: &str) {
        self.tag_name_ids.insert(name.to_string(), id.to_string());
    }

    /// 检查是否有输入或变量属性
    pub fn has_input_var_property(&self) -> bool {
        !self.input_props.is_empty() || !self.var_props.is_empty()
    }

    /// 检查是否有源标签属性
    pub fn has_src_tag_property(&self) -> bool {
        !self.src_tag_props.is_empty()
    }

    /// 检查是否有源或目标标签属性
    pub fn has_src_dst_tag_property(&self) -> bool {
        !self.src_tag_props.is_empty() || !self.dst_tag_props.is_empty()
    }

    /// 检查所有属性集合是否为空
    pub fn is_all_props_empty(&self) -> bool {
        self.input_props.is_empty()
            && self.var_props.is_empty()
            && self.src_tag_props.is_empty()
            && self.dst_tag_props.is_empty()
            && self.tag_props.is_empty()
            && self.edge_props.is_empty()
    }

    /// 检查给定属性集合是否为输入属性的子集
    pub fn is_subset_of_input(&self, props: &HashSet<String>) -> bool {
        props.iter().all(|p| self.input_props.contains(p))
    }

    /// 检查给定属性映射是否为变量属性的子集
    pub fn is_subset_of_var(&self, props: &HashMap<String, HashSet<String>>) -> bool {
        props.iter().all(|(var, var_props)| {
            self.var_props.get(var).map_or(false, |my_props| {
                var_props.iter().all(|p| my_props.contains(p))
            })
        })
    }

    /// 合并另一个ExpressionProps的属性
    pub fn union_props(&mut self, other: &ExpressionProps) {
        self.input_props.extend(other.input_props.iter().cloned());

        for (var, props) in &other.var_props {
            self.var_props
                .entry(var.clone())
                .or_insert_with(HashSet::new)
                .extend(props.iter().cloned());
        }

        for (tag, props) in &other.src_tag_props {
            self.src_tag_props
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .extend(props.iter().cloned());
        }

        for (tag, props) in &other.dst_tag_props {
            self.dst_tag_props
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .extend(props.iter().cloned());
        }

        for (tag, props) in &other.tag_props {
            self.tag_props
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .extend(props.iter().cloned());
        }

        for (edge, props) in &other.edge_props {
            self.edge_props
                .entry(edge.clone())
                .or_insert_with(HashSet::new)
                .extend(props.iter().cloned());
        }

        for (name, id) in &other.tag_name_ids {
            self.tag_name_ids.insert(name.clone(), id.clone());
        }
    }
}

/// 属性推导访问器
/// 用于递归遍历表达式树，收集所有涉及的属性信息
pub struct DeducePropsVisitor {
    /// 推导出的表达式属性集合
    props: ExpressionProps,
    /// 收集的节点信息
    node_info: Vec<NodeInfo>,
    /// 收集的边信息
    edge_info: Vec<EdgeInfo>,
    /// 用户定义的变量名称列表
    user_defined_vars: HashSet<String>,
    /// 错误状态
    error: Option<String>,
}

impl DeducePropsVisitor {
    pub fn new() -> Self {
        Self {
            props: ExpressionProps::new(),
            node_info: Vec::new(),
            edge_info: Vec::new(),
            user_defined_vars: HashSet::new(),
            error: None,
        }
    }

    /// 创建带有用户定义变量列表的访问器
    pub fn with_user_vars(user_defined_vars: HashSet<String>) -> Self {
        Self {
            props: ExpressionProps::new(),
            node_info: Vec::new(),
            edge_info: Vec::new(),
            user_defined_vars,
            error: None,
        }
    }

    /// 执行属性推导
    pub fn deduce(&mut self, expr: &Expression) -> Result<(), String> {
        self.visit(expr)
    }

    /// 递归访问表达式树
    fn visit(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Literal(_) => {
                // 常量表达式不包含属性
                Ok(())
            }
            Expression::Variable(name) => {
                // 处理属性表达式 - 作为输入属性
                self.props.insert_input_prop(name);
                Ok(())
            }
            Expression::Unary { op: _, operand } => {
                // 一元操作符，递归访问操作数
                self.visit(operand)
            }
            Expression::Binary { left, op: _, right } => {
                // 二元操作符，递归访问左右操作数
                self.visit(left)?;
                self.visit(right)
            }
            Expression::Function { name: _, args } => {
                // 函数调用，递归访问所有参数
                for arg in args {
                    self.visit(arg)?;
                }
                Ok(())
            }
            Expression::TagProperty { tag, prop } => {
                // 处理标签属性表达式（tagName.prop）
                self.props.insert_tag_name_id(tag, tag);
                self.props.insert_tag_prop(tag, prop);
                Ok(())
            }
            Expression::EdgeProperty { edge, prop } => {
                // 处理边属性表达式（edgeName.prop）
                self.props.insert_edge_prop(edge, prop);
                Ok(())
            }
            Expression::InputProperty(prop) => {
                // 处理输入属性表达式（$-.prop）
                self.props.insert_input_prop(prop);
                Ok(())
            }
            Expression::VariableProperty { var, prop } => {
                // 处理变量属性表达式（$var.prop）
                if !var.is_empty() {
                    self.props.insert_var_prop(var, prop);
                    self.user_defined_vars.insert(var.clone());
                }
                Ok(())
            }
            Expression::SourceProperty { tag, prop } => {
                // 处理源属性表达式（$^.tag.prop）
                self.props.insert_tag_name_id(tag, tag);
                self.props.insert_src_tag_prop(tag, prop);
                Ok(())
            }
            Expression::DestinationProperty { tag, prop } => {
                // 处理目标属性表达式（$$.tag.prop）
                self.props.insert_tag_name_id(tag, tag);
                self.props.insert_dst_tag_prop(tag, prop);
                Ok(())
            }
            Expression::UnaryPlus(operand) => self.visit(operand),
            Expression::UnaryNegate(operand) => self.visit(operand),
            Expression::UnaryNot(operand) => self.visit(operand),
            Expression::UnaryIncr(operand) => self.visit(operand),
            Expression::UnaryDecr(operand) => self.visit(operand),
            Expression::IsNull(operand) => self.visit(operand),
            Expression::IsNotNull(operand) => self.visit(operand),
            Expression::IsEmpty(operand) => self.visit(operand),
            Expression::IsNotEmpty(operand) => self.visit(operand),
            Expression::List(items) => {
                for item in items {
                    self.visit(item)?;
                }
                Ok(())
            }
            Expression::Map(items) => {
                for (_, value) in items {
                    self.visit(value)?;
                }
                Ok(())
            }
            Expression::TypeCasting { expr, .. } => self.visit(expr),
            Expression::Case {
                conditions,
                default,
            } => {
                for (condition, value) in conditions {
                    self.visit(condition)?;
                    self.visit(value)?;
                }
                if let Some(default_expr) = default {
                    self.visit(default_expr)?;
                }
                Ok(())
            }
            Expression::Aggregate { arg, .. } => {
                self.visit(arg.as_ref())?;
                Ok(())
            }
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                self.visit(generator)?;
                if let Some(condition_expr) = condition {
                    self.visit(condition_expr)?;
                }
                Ok(())
            }
            Expression::Predicate { list, condition } => {
                self.visit(list)?;
                self.visit(condition)?;
                Ok(())
            }
            Expression::Reduce {
                list,
                initial,
                expr,
                ..
            } => {
                self.visit(list)?;
                self.visit(initial)?;
                self.visit(expr)?;
                Ok(())
            }
            Expression::PathBuild(items) => {
                for item in items {
                    self.visit(item)?;
                }
                Ok(())
            }
            Expression::ESQuery(_) => {
                // 文本搜索表达式不需要处理属性
                Ok(())
            }
            Expression::UUID => {
                // UUID表达式不需要处理属性
                Ok(())
            }
            Expression::Subscript { collection, index } => {
                self.visit(collection)?;
                self.visit(index)?;
                Ok(())
            }
            Expression::SubscriptRange {
                collection,
                start,
                end,
            } => {
                self.visit(collection)?;
                if let Some(start_expr) = start {
                    self.visit(start_expr)?;
                }
                if let Some(end_expr) = end {
                    self.visit(end_expr)?;
                }
                Ok(())
            }
            Expression::Label(name) => {
                // 标签表达式
                if !name.is_empty() {
                    self.user_defined_vars.insert(name.clone());
                }
                Ok(())
            }
            Expression::MatchPathPattern { patterns, .. } => {
                for pattern in patterns {
                    self.visit(pattern)?;
                }
                Ok(())
            }
            Expression::Property { .. } => {
                // 属性表达式已在其他地方处理
                Ok(())
            }
            Expression::TypeCast { expr, .. } => {
                // 类型转换表达式
                self.visit(expr)
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                // 范围访问
                self.visit(collection)?;
                if let Some(start_expr) = start {
                    self.visit(start_expr)?;
                }
                if let Some(end_expr) = end {
                    self.visit(end_expr)?;
                }
                Ok(())
            }
            Expression::Path(items) => {
                // 路径表达式
                for item in items {
                    self.visit(item)?;
                }
                Ok(())
            }
        }
    }

    /// 获取推导出的表达式属性
    pub fn get_props(&self) -> &ExpressionProps {
        &self.props
    }

    /// 获取可变的表达式属性引用
    pub fn get_props_mut(&mut self) -> &mut ExpressionProps {
        &mut self.props
    }

    /// 获取收集的节点信息
    pub fn get_node_info(&self) -> &[NodeInfo] {
        &self.node_info
    }

    /// 获取收集的边信息
    pub fn get_edge_info(&self) -> &[EdgeInfo] {
        &self.edge_info
    }

    /// 获取用户定义变量列表
    pub fn get_user_defined_vars(&self) -> &HashSet<String> {
        &self.user_defined_vars
    }

    /// 获取错误信息
    pub fn get_error(&self) -> &Option<String> {
        &self.error
    }

    /// 是否推导成功
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }

    /// 设置错误信息
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// 向节点信息列表添加节点
    pub fn add_node_info(&mut self, node: NodeInfo) {
        self.node_info.push(node);
    }

    /// 向边信息列表添加边
    pub fn add_edge_info(&mut self, edge: EdgeInfo) {
        self.edge_info.push(edge);
    }
}

impl Default for DeducePropsVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ExpressionProps {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_props_input() {
        let mut props = ExpressionProps::new();
        props.insert_input_prop("name");
        props.insert_input_prop("age");

        assert!(props.input_props.contains("name"));
        assert!(props.input_props.contains("age"));
        assert_eq!(props.input_props.len(), 2);
    }

    #[test]
    fn test_expression_props_var() {
        let mut props = ExpressionProps::new();
        props.insert_var_prop("x", "value");
        props.insert_var_prop("x", "name");
        props.insert_var_prop("y", "age");

        assert_eq!(props.var_props.get("x").map(|p| p.len()), Some(2));
        assert_eq!(props.var_props.get("y").map(|p| p.len()), Some(1));
    }

    #[test]
    fn test_expression_props_tag() {
        let mut props = ExpressionProps::new();
        props.insert_tag_prop("Person", "name");
        props.insert_tag_prop("Person", "age");
        props.insert_src_tag_prop("Person", "id");
        props.insert_dst_tag_prop("Animal", "type");

        assert_eq!(props.tag_props.get("Person").map(|p| p.len()), Some(2));
        assert_eq!(props.src_tag_props.get("Person").map(|p| p.len()), Some(1));
        assert_eq!(props.dst_tag_props.get("Animal").map(|p| p.len()), Some(1));
    }

    #[test]
    fn test_expression_props_edge() {
        let mut props = ExpressionProps::new();
        props.insert_edge_prop("follow", "weight");
        props.insert_edge_prop("follow", "time");
        props.insert_edge_prop("like", "score");

        assert_eq!(props.edge_props.get("follow").map(|p| p.len()), Some(2));
        assert_eq!(props.edge_props.get("like").map(|p| p.len()), Some(1));
    }

    #[test]
    fn test_deduce_visitor_constant() {
        let mut visitor = DeducePropsVisitor::new();
        let expr = Expression::Literal(crate::graph::expression::LiteralValue::Int(42));

        assert!(visitor.deduce(&expr).is_ok());
        assert!(visitor.get_props().is_all_props_empty());
    }

    #[test]
    fn test_deduce_visitor_property() {
        let mut visitor = DeducePropsVisitor::new();
        let expr = Expression::Variable("name".to_string());

        assert!(visitor.deduce(&expr).is_ok());
        assert!(visitor.get_props().input_props.contains("name"));
    }

    #[test]
    fn test_deduce_visitor_tag_property() {
        let mut visitor = DeducePropsVisitor::new();
        let expr = Expression::TagProperty {
            tag: "Person".to_string(),
            prop: "name".to_string(),
        };

        assert!(visitor.deduce(&expr).is_ok());
        assert!(visitor
            .get_props()
            .tag_props
            .get("Person")
            .map_or(false, |p| p.contains("name")));
    }

    #[test]
    fn test_deduce_visitor_source_property() {
        let mut visitor = DeducePropsVisitor::new();
        let expr = Expression::SourceProperty {
            tag: "Person".to_string(),
            prop: "id".to_string(),
        };

        assert!(visitor.deduce(&expr).is_ok());
        assert!(visitor
            .get_props()
            .src_tag_props
            .get("Person")
            .map_or(false, |p| p.contains("id")));
    }

    #[test]
    fn test_deduce_visitor_edge_property() {
        let mut visitor = DeducePropsVisitor::new();
        let expr = Expression::EdgeProperty {
            edge: "follow".to_string(),
            prop: "weight".to_string(),
        };

        assert!(visitor.deduce(&expr).is_ok());
        assert!(visitor
            .get_props()
            .edge_props
            .get("follow")
            .map_or(false, |p| p.contains("weight")));
    }

    #[test]
    fn test_deduce_visitor_binary_op() {
        let mut visitor = DeducePropsVisitor::new();
        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("age".to_string())),
            op: crate::graph::expression::BinaryOperator::Add,
            right: Box::new(Expression::Variable("bonus".to_string())),
        };

        assert!(visitor.deduce(&expr).is_ok());
        let props = visitor.get_props();
        assert!(props.input_props.contains("age"));
        assert!(props.input_props.contains("bonus"));
    }

    #[test]
    fn test_deduce_visitor_union() {
        let mut props1 = ExpressionProps::new();
        props1.insert_input_prop("name");
        props1.insert_tag_prop("Person", "age");

        let mut props2 = ExpressionProps::new();
        props2.insert_input_prop("email");
        props2.insert_tag_prop("Person", "city");
        props2.insert_edge_prop("follow", "weight");

        props1.union_props(&props2);

        assert_eq!(props1.input_props.len(), 2);
        assert!(props1
            .tag_props
            .get("Person")
            .map_or(false, |p| p.len() == 2));
        assert!(props1
            .edge_props
            .get("follow")
            .map_or(false, |p| p.len() == 1));
    }
}
