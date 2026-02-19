//! CREATE 数据语句规划器
//!
//! 处理 Cypher 风格 CREATE 语句的查询规划
//! 支持 CREATE (n:Label {props}) 和 CREATE (a)-[:Type]->(b) 语法

use crate::query::context::ast::AstContext;
use crate::query::parser::ast::{CreateStmt, CreateTarget, Stmt};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{
        insert_nodes::{EdgeInsertInfo, InsertEdgesNode, InsertVerticesNode, VertexInsertInfo, TagInsertSpec},
        ArgumentNode, ProjectNode,
    },
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::validator::YieldColumn;
use crate::core::{Expression, Value};

/// CREATE 数据语句规划器
/// 负责将 Cypher 风格的 CREATE 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct CreatePlanner;

impl CreatePlanner {
    /// 创建新的 CREATE 规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查 AST 上下文是否匹配 CREATE 数据语句
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        matches!(ast_ctx.sentence(), Some(Stmt::Create(create_stmt)) if Self::is_data_create(create_stmt))
    }

    /// 判断是否为数据创建语句（而非 Schema 创建）
    fn is_data_create(stmt: &CreateStmt) -> bool {
        matches!(&stmt.target,
            CreateTarget::Node { .. } |
            CreateTarget::Edge { .. } |
            CreateTarget::Path { .. }
        )
    }

    /// 从 AstContext 提取 CreateStmt
    fn extract_create_stmt(&self, ast_ctx: &AstContext) -> Result<CreateStmt, PlannerError> {
        match ast_ctx.sentence() {
            Some(Stmt::Create(create_stmt)) => Ok(create_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "AST 上下文中不包含 CREATE 语句".to_string(),
            )),
        }
    }

    /// 构建顶点插入信息
    fn build_vertex_insert_info(
        &self,
        space_name: String,
        labels: &[String],
        properties: &[(String, Expression)],
    ) -> Result<VertexInsertInfo, PlannerError> {
        if labels.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "CREATE 节点必须指定至少一个 Label".to_string()
            ));
        }

        // 转换标签规范
        let tag_specs: Vec<TagInsertSpec> = labels
            .iter()
            .map(|label| TagInsertSpec {
                tag_name: label.clone(),
                prop_names: properties.iter().map(|(k, _)| k.clone()).collect(),
            })
            .collect();

        // 属性值
        let prop_values: Vec<Expression> = properties
            .iter()
            .map(|(_, v)| v.clone())
            .collect();

        // 注意：这里需要 VID，但在 Cypher 语法中通常不直接指定
        // 我们需要生成一个 VID 或使用变量引用
        // 暂时使用一个占位符，实际执行时会处理
        let vid_expr = Expression::literal(Value::Null(crate::core::NullType::default()));

        Ok(VertexInsertInfo {
            space_name,
            tags: tag_specs,
            values: vec![(vid_expr, vec![prop_values])],
        })
    }

    /// 构建边插入信息
    fn build_edge_insert_info(
        &self,
        space_name: String,
        edge_type: String,
        src_vid: Expression,
        dst_vid: Expression,
        properties: &[(String, Expression)],
    ) -> EdgeInsertInfo {
        let prop_names: Vec<String> = properties.iter().map(|(k, _)| k.clone()).collect();
        let prop_values: Vec<Expression> = properties.iter().map(|(_, v)| v.clone()).collect();

        EdgeInsertInfo {
            space_name,
            edge_name: edge_type,
            prop_names,
            edges: vec![(src_vid, dst_vid, None, prop_values)],
        }
    }

    /// 创建结果投影列
    fn create_yield_columns(&self, count: usize) -> Vec<YieldColumn> {
        vec![YieldColumn {
            expression: Expression::literal(Value::Int(count as i64)),
            alias: "created_count".to_string(),
            is_matched: false,
        }]
    }
}

impl Planner for CreatePlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 获取空间名称
        let space_name = ast_ctx.space().space_name.clone();

        // 提取 CREATE 语句
        let create_stmt = self.extract_create_stmt(ast_ctx)?;

        // 创建参数节点
        let arg_node = ArgumentNode::new(next_node_id(), "create_args");

        // 根据 CREATE 目标类型创建相应的插入节点
        let (insert_node, created_count) = match &create_stmt.target {
            CreateTarget::Node { variable: _, labels, properties } => {
                // 解析属性
                let props = if let Some(expr) = properties {
                    Self::extract_properties(expr)?
                } else {
                    vec![]
                };

                let info = self.build_vertex_insert_info(
                    space_name,
                    labels,
                    &props,
                )?;

                (
                    PlanNodeEnum::InsertVertices(InsertVerticesNode::new(next_node_id(), info)),
                    1,
                )
            }
            CreateTarget::Edge { variable: _, edge_type, src, dst, properties, direction: _ } => {
                // 解析属性
                let props = if let Some(expr) = properties {
                    Self::extract_properties(expr)?
                } else {
                    vec![]
                };

                let info = self.build_edge_insert_info(
                    space_name,
                    edge_type.clone(),
                    src.clone(),
                    dst.clone(),
                    &props,
                );

                (
                    PlanNodeEnum::InsertEdges(InsertEdgesNode::new(next_node_id(), info)),
                    1,
                )
            }
            CreateTarget::Path { patterns: _ } => {
                // 路径创建需要更复杂的处理
                // 这里简化处理，返回错误提示
                return Err(PlannerError::PlanGenerationFailed(
                    "路径创建尚未完全实现".to_string()
                ));
            }
            _ => {
                return Err(PlannerError::PlanGenerationFailed(
                    "不支持的 CREATE 目标类型".to_string()
                ));
            }
        };

        // 创建投影节点来返回创建结果
        let yield_columns = self.create_yield_columns(created_count);

        let project_node = ProjectNode::new(insert_node, yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("创建 ProjectNode 失败: {}", e))
        })?;

        let final_node = PlanNodeEnum::Project(project_node);

        // 创建 SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl CreatePlanner {
    /// 从表达式中提取属性键值对
    fn extract_properties(expr: &Expression) -> Result<Vec<(String, Expression)>, PlannerError> {
        match expr {
            Expression::Map(map) => {
                let mut result = Vec::new();
                for (key, value) in map {
                    result.push((key.clone(), value.clone()));
                }
                Ok(result)
            }
            _ => Err(PlannerError::PlanGenerationFailed(
                "属性必须是 Map 表达式".to_string()
            )),
        }
    }
}

impl Default for CreatePlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::query::context::ast::base::AstContext;
    use crate::query::context::validate::types::SpaceInfo;
    use crate::query::parser::ast::{CreateStmt, CreateTarget, Span};

    // 辅助函数：创建常量表达式
    fn lit(val: Value) -> Expression {
        Expression::literal(val)
    }

    fn create_test_span() -> Span {
        use crate::core::types::span::Position;
        Span::new(
            Position::new(1, 1),
            Position::new(1, 10),
        )
    }

    #[test]
    fn test_create_planner_match() {
        // 测试匹配 CREATE 数据语句
        let create_stmt = CreateStmt {
            span: create_test_span(),
            target: CreateTarget::Node {
                variable: Some("n".to_string()),
                labels: vec!["Person".to_string()],
                properties: Some(Expression::Map(vec![
                    ("name".to_string(), lit(Value::String("Alice".to_string()))),
                ])),
            },
            if_not_exists: false,
        };

        let ast_ctx = AstContext::new(None, Some(Stmt::Create(create_stmt)));

        assert!(CreatePlanner::match_ast_ctx(&ast_ctx));
    }

    #[test]
    fn test_create_planner_not_match_ddl() {
        // 测试不匹配 DDL CREATE 语句
        let create_stmt = CreateStmt {
            span: create_test_span(),
            target: CreateTarget::Tag {
                name: "Person".to_string(),
                properties: vec![],
                ttl_duration: None,
                ttl_col: None,
            },
            if_not_exists: false,
        };

        let ast_ctx = AstContext::new(None, Some(Stmt::Create(create_stmt)));

        assert!(!CreatePlanner::match_ast_ctx(&ast_ctx));
    }
}
