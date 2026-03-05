//! CREATE 数据语句规划器
//!
//! 处理 Cypher 风格 CREATE 语句的查询规划
//! 支持 CREATE (n:Label {props}) 和 CREATE (a)-[:Type]->(b) 语法

use crate::query::validator::context::ExpressionAnalysisContext;
use crate::core::types::ContextualExpression;
use crate::core::Value;
use crate::core::YieldColumn;
use crate::query::parser::ast::{CreateStmt, CreateTarget, Stmt};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{
        control_flow_node::PassThroughNode,
        insert_nodes::{
            EdgeInsertInfo, InsertEdgesNode, InsertVerticesNode, TagInsertSpec, VertexInsertInfo,
        },
        ArgumentNode, ProjectNode,
    },
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// CREATE 数据语句规划器
/// 负责将 Cypher 风格的 CREATE 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct CreatePlanner;

impl CreatePlanner {
    /// 创建新的 CREATE 规划器
    pub fn new() -> Self {
        Self
    }

    /// 判断是否为数据创建语句（而非 Schema 创建）
    fn is_data_create(stmt: &CreateStmt) -> bool {
        matches!(
            &stmt.target,
            CreateTarget::Node { .. } | CreateTarget::Edge { .. } | CreateTarget::Path { .. }
        )
    }

    /// 从 Stmt 提取 CreateStmt
    fn extract_create_stmt(&self, stmt: &Stmt) -> Result<CreateStmt, PlannerError> {
        match stmt {
            Stmt::Create(create_stmt) => Ok(create_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不包含 CREATE".to_string(),
            )),
        }
    }

    /// 构建顶点插入信息
    fn build_vertex_insert_info(
        &self,
        space_name: String,
        labels: &[String],
        properties: &[(String, ContextualExpression)],
        expr_context: &Arc<ExpressionAnalysisContext>,
    ) -> Result<VertexInsertInfo, PlannerError> {
        if labels.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "CREATE 节点必须指定至少一个 Label".to_string(),
            ));
        }

        let tag_specs: Vec<TagInsertSpec> = labels
            .iter()
            .map(|label| TagInsertSpec {
                tag_name: label.clone(),
                prop_names: properties.iter().map(|(k, _)| k.clone()).collect(),
            })
            .collect();

        let prop_values: Vec<ContextualExpression> =
            properties.iter().map(|(_, v)| v.clone()).collect();

        let vid_expr = {
            let expr_meta = crate::core::types::expression::ExpressionMeta::new(
                crate::core::Expression::literal(Value::Null(crate::core::NullType::default())),
            );
            let id = expr_context.register_expression(expr_meta);
            ContextualExpression::new(id, expr_context.clone())
        };

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
        src_vid: ContextualExpression,
        dst_vid: ContextualExpression,
        properties: &[(String, ContextualExpression)],
    ) -> EdgeInsertInfo {
        let prop_names: Vec<String> = properties.iter().map(|(k, _)| k.clone()).collect();
        let prop_values: Vec<ContextualExpression> =
            properties.iter().map(|(_, v)| v.clone()).collect();

        EdgeInsertInfo {
            space_name,
            edge_name: edge_type,
            prop_names,
            edges: vec![(src_vid, dst_vid, None, prop_values)],
        }
    }

    /// 创建结果投影列
    fn create_yield_columns(
        &self,
        count: usize,
        expr_context: &Arc<ExpressionAnalysisContext>,
    ) -> Vec<YieldColumn> {
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(
            crate::core::Expression::literal(Value::Int(count as i64)),
        );
        let id = expr_context.register_expression(expr_meta);
        let ctx_expr = ContextualExpression::new(id, expr_context.clone());

        vec![YieldColumn {
            expression: ctx_expr,
            alias: "created_count".to_string(),
            is_matched: false,
        }]
    }
}

impl Planner for CreatePlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        // 使用验证信息进行优化规划
        let validation_info = &validated.validation_info;

        // 检查语义信息
        let referenced_tags = &validation_info.semantic_info.referenced_tags;
        if !referenced_tags.is_empty() {
            log::debug!("CREATE 引用的标签: {:?}", referenced_tags);
        }

        let referenced_edges = &validation_info.semantic_info.referenced_edges;
        if !referenced_edges.is_empty() {
            log::debug!("CREATE 引用的边类型: {:?}", referenced_edges);
        }

        // 获取空间名称
        let space_name = qctx
            .rctx()
            .space_name
            .clone()
            .unwrap_or_else(|| "default".to_string());

        // 提取 CREATE 语句
        let create_stmt = self.extract_create_stmt(validated.stmt())?;

        // 创建参数节点
        let arg_node = ArgumentNode::new(next_node_id(), "create_args");

        // 根据 CREATE 目标类型创建相应的插入节点
        let (insert_node, created_count) = match &create_stmt.target {
            CreateTarget::Node {
                variable: _,
                labels,
                properties,
            } => {
                // 解析属性
                let props = if let Some(expr) = properties {
                    Self::extract_properties(expr, validated.expr_context())?
                } else {
                    vec![]
                };

                let info = self.build_vertex_insert_info(space_name, labels, &props, validated.expr_context())?;

                (
                    PlanNodeEnum::InsertVertices(InsertVerticesNode::new(next_node_id(), info)),
                    1,
                )
            }
            CreateTarget::Edge {
                variable: _,
                edge_type,
                src,
                dst,
                properties,
                direction: _,
            } => {
                // 解析属性
                let props = if let Some(expr) = properties {
                    Self::extract_properties(expr, validated.expr_context())?
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
            CreateTarget::Path { patterns } => {
                let mut vertex_infos = Vec::new();
                let mut edge_infos = Vec::new();
                let mut created_count = 0;

                for pattern in patterns {
                    match pattern {
                        crate::query::parser::ast::pattern::Pattern::Path(path) => {
                            let (mut vertices, mut edges) =
                                self.process_path_pattern(path, &space_name, validated.expr_context())?;
                            vertex_infos.append(&mut vertices);
                            edge_infos.append(&mut edges);
                            created_count += 1;
                        }
                        crate::query::parser::ast::pattern::Pattern::Node(node) => {
                            let info = self.process_node_pattern(node, &space_name, validated.expr_context())?;
                            vertex_infos.push(info);
                            created_count += 1;
                        }
                        _ => {
                            return Err(PlannerError::PlanGenerationFailed(
                                "路径创建只支持节点和路径模式".to_string(),
                            ));
                        }
                    }
                }

                if vertex_infos.is_empty() && edge_infos.is_empty() {
                    return Err(PlannerError::PlanGenerationFailed(
                        "路径创建必须包含至少一个节点或边".to_string(),
                    ));
                }

                let mut insert_nodes = Vec::new();

                for info in vertex_infos {
                    insert_nodes.push(PlanNodeEnum::InsertVertices(InsertVerticesNode::new(
                        next_node_id(),
                        info,
                    )));
                }

                for info in edge_infos {
                    insert_nodes.push(PlanNodeEnum::InsertEdges(InsertEdgesNode::new(
                        next_node_id(),
                        info,
                    )));
                }

                if insert_nodes.len() == 1 {
                    (insert_nodes.into_iter().next().unwrap(), created_count)
                } else {
                    let combined = self.combine_insert_nodes(insert_nodes)?;
                    (PlanNodeEnum::PassThrough(combined), created_count)
                }
            }
            _ => {
                return Err(PlannerError::PlanGenerationFailed(
                    "不支持的 CREATE 目标类型".to_string(),
                ));
            }
        };

        // 创建投影节点来返回创建结果
        let yield_columns = self.create_yield_columns(created_count, validated.expr_context());

        let project_node = ProjectNode::new(insert_node, yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("创建 ProjectNode 失败: {}", e))
        })?;

        let final_node = PlanNodeEnum::Project(project_node);

        // 创建 SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Create(create_stmt) => Self::is_data_create(create_stmt),
            _ => false,
        }
    }
}

impl CreatePlanner {
    /// 从表达式中提取属性键值对
    fn extract_properties(
        expr: &ContextualExpression,
        expr_context: &Arc<ExpressionAnalysisContext>,
    ) -> Result<Vec<(String, ContextualExpression)>, PlannerError> {
        if let Some(expr_meta) = expr.expression() {
            if let crate::core::Expression::Map(map) = expr_meta.inner() {
                let mut result = Vec::new();
                for (key, value_expr) in map {
                    let value_meta =
                        crate::core::types::expression::ExpressionMeta::new(value_expr.clone());
                    let id = expr_context.register_expression(value_meta);
                    let ctx_expr = ContextualExpression::new(id, expr_context.clone());
                    result.push((key.clone(), ctx_expr));
                }
                Ok(result)
            } else {
                Err(PlannerError::PlanGenerationFailed(
                    "属性必须是 Map 表达式".to_string(),
                ))
            }
        } else {
            Err(PlannerError::PlanGenerationFailed("表达式无效".to_string()))
        }
    }

    /// 处理节点模式
    fn process_node_pattern(
        &self,
        node: &crate::query::parser::ast::pattern::NodePattern,
        space_name: &str,
        expr_context: &Arc<ExpressionAnalysisContext>,
    ) -> Result<VertexInsertInfo, PlannerError> {
        let props = if let Some(ref expr) = node.properties {
            Self::extract_properties(expr, expr_context)?
        } else {
            vec![]
        };

        self.build_vertex_insert_info(space_name.to_string(), &node.labels, &props, expr_context)
    }

    /// 处理路径模式
    fn process_path_pattern(
        &self,
        path: &crate::query::parser::ast::pattern::PathPattern,
        space_name: &str,
        expr_context: &Arc<ExpressionAnalysisContext>,
    ) -> Result<(Vec<VertexInsertInfo>, Vec<EdgeInsertInfo>), PlannerError> {
        let mut vertex_infos = Vec::new();
        let mut edge_infos = Vec::new();
        let mut prev_vertex: Option<VertexInsertInfo> = None;

        for element in &path.elements {
            match element {
                crate::query::parser::ast::pattern::PathElement::Node(node) => {
                    let vertex_info = self.process_node_pattern(node, space_name, expr_context)?;
                    prev_vertex = Some(vertex_info.clone());
                    vertex_infos.push(vertex_info);
                }
                crate::query::parser::ast::pattern::PathElement::Edge(edge) => {
                    if prev_vertex.is_none() {
                        return Err(PlannerError::PlanGenerationFailed(
                            "边模式前必须有节点模式".to_string(),
                        ));
                    }

                    let props = if let Some(ref expr) = edge.properties {
                        Self::extract_properties(expr, expr_context)?
                    } else {
                        vec![]
                    };

                    if edge.edge_types.is_empty() {
                        return Err(PlannerError::PlanGenerationFailed(
                            "边模式必须指定边类型".to_string(),
                        ));
                    }

                    let edge_type = edge.edge_types[0].clone();

                    let src_vid = {
                        let expr_meta = crate::core::types::expression::ExpressionMeta::new(
                            crate::core::Expression::literal(Value::Null(
                                crate::core::NullType::default(),
                            )),
                        );
                        let id = expr_context.register_expression(expr_meta);
                        ContextualExpression::new(id, expr_context.clone())
                    };
                    let dst_vid = {
                        let expr_meta = crate::core::types::expression::ExpressionMeta::new(
                            crate::core::Expression::literal(Value::Null(
                                crate::core::NullType::default(),
                            )),
                        );
                        let id = expr_context.register_expression(expr_meta);
                        ContextualExpression::new(id, expr_context.clone())
                    };

                    let edge_info = EdgeInsertInfo {
                        space_name: space_name.to_string(),
                        edge_name: edge_type,
                        prop_names: props.iter().map(|(k, _)| k.clone()).collect(),
                        edges: vec![(
                            src_vid,
                            dst_vid,
                            None,
                            props.iter().map(|(_, v)| v.clone()).collect(),
                        )],
                    };

                    edge_infos.push(edge_info);
                }
                _ => {
                    return Err(PlannerError::PlanGenerationFailed(
                        "路径创建不支持 Alternative、Optional 或 Repeated 模式".to_string(),
                    ));
                }
            }
        }

        Ok((vertex_infos, edge_infos))
    }

    /// 组合多个插入节点
    fn combine_insert_nodes(
        &self,
        nodes: Vec<PlanNodeEnum>,
    ) -> Result<PassThroughNode, PlannerError> {
        if nodes.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "无法组合空的节点列表".to_string(),
            ));
        }

        Ok(PassThroughNode::new(next_node_id()))
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
    use crate::query::parser::parser::Parser;
    use crate::query::planner::planner::{Planner, ValidatedStatement};
    use crate::query::validator::ValidationInfo;
    use crate::query::QueryContext;
    use std::sync::Arc;

    #[test]
    fn test_create_path_simple() {
        let sql = "CREATE (a:Person {name: 'Alice'})-[:FRIEND]->(b:Person {name: 'Bob'})";
        let mut parser = Parser::new(sql);
        let parser_result = parser.parse().expect("解析失败");

        let mut planner = CreatePlanner::new();
        let qctx = Arc::new(QueryContext::default());

        // 创建验证后的语句
        let validation_info = ValidationInfo::new();
        let validated = ValidatedStatement::new(parser_result.ast, validation_info);

        let result = planner.transform(&validated, qctx);
        assert!(
            result.is_ok(),
            "CREATE PATH 应该成功，但得到错误: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_create_path_with_properties() {
        let sql = "CREATE (a:Person {name: 'Alice', age: 30})-[:FRIEND {since: 2020}]->(b:Person {name: 'Bob', age: 25})";
        let mut parser = Parser::new(sql);
        let parser_result = parser.parse().expect("解析失败");

        let mut planner = CreatePlanner::new();
        let qctx = Arc::new(QueryContext::default());

        // 创建验证后的语句
        let validation_info = ValidationInfo::new();
        let validated = ValidatedStatement::new(parser_result.ast, validation_info);

        let result = planner.transform(&validated, qctx);
        assert!(result.is_ok(), "带属性的 CREATE PATH 应该成功");
    }

    #[test]
    fn test_create_path_multiple_edges() {
        let sql = "CREATE (a:Person)-[:FRIEND]->(b:Person)-[:FRIEND]->(c:Person)";
        let mut parser = Parser::new(sql);
        let parser_result = parser.parse().expect("解析失败");

        let mut planner = CreatePlanner::new();
        let qctx = Arc::new(QueryContext::default());

        // 创建验证后的语句
        let validation_info = ValidationInfo::new();
        let validated = ValidatedStatement::new(parser_result.ast, validation_info);

        let result = planner.transform(&validated, qctx);
        assert!(result.is_ok(), "多边 CREATE PATH 应该成功");
    }

    #[test]
    fn test_create_path_single_node() {
        let sql = "CREATE (a:Person {name: 'Alice'})";
        let mut parser = Parser::new(sql);
        let parser_result = parser.parse().expect("解析失败");

        let mut planner = CreatePlanner::new();
        let qctx = Arc::new(QueryContext::default());

        // 创建验证后的语句
        let validation_info = ValidationInfo::new();
        let validated = ValidatedStatement::new(parser_result.ast, validation_info);

        let result = planner.transform(&validated, qctx);
        assert!(result.is_ok(), "单节点 CREATE 应该成功");
    }

    #[test]
    fn test_create_path_without_labels() {
        let sql = "CREATE (a)-[:FRIEND]->(b)";
        let mut parser = Parser::new(sql);
        let parser_result = parser.parse().expect("解析失败");

        let mut planner = CreatePlanner::new();
        let qctx = Arc::new(QueryContext::default());

        // 创建验证后的语句
        let validation_info = ValidationInfo::new();
        let validated = ValidatedStatement::new(parser_result.ast, validation_info);

        let result = planner.transform(&validated, qctx);
        assert!(result.is_err(), "没有标签的 CREATE PATH 应该失败");
    }

    #[test]
    fn test_create_path_bidirectional_edge() {
        let sql = "CREATE (a:Person)-[:FRIEND]-(b:Person)";
        let mut parser = Parser::new(sql);
        let parser_result = parser.parse().expect("解析应该成功");

        let mut planner = CreatePlanner::new();
        let qctx = Arc::new(QueryContext::default());

        // 创建验证后的语句
        let validation_info = ValidationInfo::new();
        let validated = ValidatedStatement::new(parser_result.ast, validation_info);

        let result = planner.transform(&validated, qctx);
        assert!(result.is_ok(), "双向边 CREATE PATH 应该成功");
    }
}
