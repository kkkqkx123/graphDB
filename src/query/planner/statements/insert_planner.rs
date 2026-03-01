//! 插入操作规划器
//!
//! 处理 INSERT VERTEX 和 INSERT EDGE 语句的查询规划

use std::sync::Arc;
use crate::query::QueryContext;
use crate::query::parser::ast::{InsertStmt, InsertTarget, Stmt, VertexRow};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{
        insert_nodes::{EdgeInsertInfo, InsertEdgesNode, InsertVerticesNode, VertexInsertInfo, TagInsertSpec},
        ArgumentNode, ProjectNode,
    },
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};
use crate::core::YieldColumn;
use crate::query::parser::ast::utils::ExprFactory;
use crate::core::types::expression::contextual::ContextualExpression;

/// 插入操作规划器
/// 负责将 INSERT 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct InsertPlanner;

impl InsertPlanner {
    /// 创建新的插入规划器
    pub fn new() -> Self {
        Self
    }

    /// 检查语句是否匹配插入操作
    pub fn match_stmt(stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Insert(_))
    }

    /// 从 Stmt 提取 InsertStmt
    fn extract_insert_stmt(&self, stmt: &Stmt) -> Result<InsertStmt, PlannerError> {
        match stmt {
            Stmt::Insert(insert_stmt) => Ok(insert_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不是 INSERT 语句".to_string(),
            )),
        }
    }

    /// 构建顶点插入信息
    /// 支持多标签插入
    fn build_vertex_insert_info(
        &self,
        space_name: String,
        tags: Vec<crate::query::parser::ast::TagInsertSpec>,
        values: Vec<VertexRow>,
    ) -> Result<VertexInsertInfo, PlannerError> {
        // 转换标签规范
        let tag_specs: Vec<TagInsertSpec> = tags
            .into_iter()
            .map(|tag| TagInsertSpec {
                tag_name: tag.tag_name,
                prop_names: tag.prop_names,
            })
            .collect();

        // 将 VertexRow 转换为 (vid, Vec<Vec<Expression>>) 格式
        // 每个标签对应一个属性值列表
        let converted_values: Vec<(ContextualExpression, Vec<Vec<ContextualExpression>>)> = values
            .into_iter()
            .map(|row| {
                (row.vid, row.tag_values)
            })
            .collect();
        
        Ok(VertexInsertInfo {
            space_name,
            tags: tag_specs,
            values: converted_values,
        })
    }

    /// 构建边插入信息
    fn build_edge_insert_info(
        &self,
        space_name: String,
        edge_name: String,
        prop_names: Vec<String>,
        edges: Vec<(ContextualExpression, ContextualExpression, Option<ContextualExpression>, Vec<ContextualExpression>)>,
    ) -> EdgeInsertInfo {
        EdgeInsertInfo {
            space_name,
            edge_name,
            prop_names,
            edges,
        }
    }

    /// 创建插入结果投影列
    fn create_yield_columns(&self, count: usize, qctx: Arc<QueryContext>) -> Vec<YieldColumn> {
        let expr = ExprFactory::constant(
            crate::core::Value::Int(count as i64),
            qctx.expr_context_clone(),
        );
        vec![YieldColumn::new(expr, "inserted_count".to_string())]
    }
}

impl Planner for InsertPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        // 获取空间名称
        let space_name = qctx.rctx().space_name.clone().unwrap_or_else(|| "default".to_string());

        // 提取 INSERT 语句
        let insert_stmt = self.extract_insert_stmt(&validated.stmt)?;

        // 创建参数节点
        let arg_node = ArgumentNode::new(next_node_id(), "insert_args");

        // 根据 INSERT 目标类型创建相应的插入节点
        let (insert_node, inserted_count) = match &insert_stmt.target {
            InsertTarget::Vertices { tags, values } => {
                let count = values.len();
                // 支持多标签插入
                if tags.is_empty() {
                    return Err(PlannerError::PlanGenerationFailed(
                        "INSERT VERTEX must specify at least one tag".to_string()
                    ));
                }
                let info = self.build_vertex_insert_info(
                    space_name,
                    tags.clone(),
                    values.clone(),
                )?;
                (
                    PlanNodeEnum::InsertVertices(InsertVerticesNode::new(next_node_id(), info)),
                    count,
                )
            }
            InsertTarget::Edge {
                edge_name,
                prop_names,
                edges,
            } => {
                let count = edges.len();
                let info = self.build_edge_insert_info(space_name, edge_name.clone(), prop_names.clone(), edges.clone());
                (
                    PlanNodeEnum::InsertEdges(InsertEdgesNode::new(next_node_id(), info)),
                    count,
                )
            }
        };

        // 创建投影节点来返回插入结果
        let yield_columns = self.create_yield_columns(inserted_count, qctx.clone());

        let project_node = ProjectNode::new(insert_node, yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("创建 ProjectNode 失败: {}", e))
        })?;

        let final_node = PlanNodeEnum::Project(project_node);

        // 创建 SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        Self::match_stmt(stmt)
    }
}

impl Default for InsertPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::core::Value;
    use crate::query::QueryContext;
    use crate::query::parser::ast::{InsertStmt, InsertTarget, Span, TagInsertSpec, VertexRow, Stmt};
    use crate::query::planner::planner::{Planner, ValidatedStatement};
    use crate::query::validator::ValidationInfo;
    use crate::query::parser::ast::utils::ExprFactory;
    use crate::core::types::expression::contextual::ContextualExpression;

    fn create_test_span() -> Span {
        use crate::core::types::span::Position;
        Span::new(Position::new(1, 1), Position::new(1, 1))
    }

    fn create_test_stmt_with_insert(target: InsertTarget) -> Stmt {
        let insert_stmt = InsertStmt {
            span: create_test_span(),
            target,
            if_not_exists: false,
        };
        Stmt::Insert(insert_stmt)
    }

    fn create_test_qctx() -> Arc<QueryContext> {
        Arc::new(QueryContext::default())
    }

    // 辅助函数：创建常量表达式
    fn lit(val: Value) -> ContextualExpression {
        let qctx = create_test_qctx();
        ExprFactory::constant(val, qctx.expr_context_clone())
    }

    #[test]
    fn test_insert_planner_new() {
        let planner = InsertPlanner::new();
        let stmt = create_test_stmt_with_insert(InsertTarget::Vertices {
            tags: vec![
                TagInsertSpec {
                    tag_name: "person".to_string(),
                    prop_names: vec!["name".to_string(), "age".to_string()],
                    is_default_props: false,
                },
            ],
            values: vec![
                VertexRow {
                    vid: lit(Value::Int(1)),
                    tag_values: vec![
                        vec![
                            lit(Value::String("Alice".to_string())),
                            lit(Value::Int(30)),
                        ],
                    ],
                },
            ],
        });
        assert!(planner.match_planner(&stmt));
    }

    #[test]
    fn test_match_stmt_with_insert() {
        let stmt = create_test_stmt_with_insert(InsertTarget::Vertices {
            tags: vec![
                TagInsertSpec {
                    tag_name: "person".to_string(),
                    prop_names: vec![],
                    is_default_props: true,
                },
            ],
            values: vec![],
        });
        assert!(InsertPlanner::match_stmt(&stmt));
    }

    #[test]
    fn test_match_stmt_without_insert() {
        let stmt = Stmt::Use(crate::query::parser::ast::UseStmt {
            span: create_test_span(),
            space: "test_space".to_string(),
        });
        assert!(!InsertPlanner::match_stmt(&stmt));
    }

    #[test]
    fn test_extract_insert_stmt_success() {
        let planner = InsertPlanner::new();
        let target = InsertTarget::Vertices {
            tags: vec![
                TagInsertSpec {
                    tag_name: "person".to_string(),
                    prop_names: vec!["name".to_string()],
                    is_default_props: false,
                },
            ],
            values: vec![],
        };
        let stmt = create_test_stmt_with_insert(target.clone());
        let result = planner.extract_insert_stmt(&stmt).expect("Failed to extract insert statement");
        assert_eq!(result.target, target);
    }

    #[test]
    fn test_extract_insert_stmt_failure() {
        let planner = InsertPlanner::new();
        let stmt = Stmt::Use(crate::query::parser::ast::UseStmt {
            span: create_test_span(),
            space: "test_space".to_string(),
        });
        let result = planner.extract_insert_stmt(&stmt);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("不是 INSERT 语句"));
    }

    #[test]
    fn test_build_vertex_insert_info() {
        let planner = InsertPlanner::new();
        let info = planner.build_vertex_insert_info(
            "test_space".to_string(),
            vec![
                TagInsertSpec {
                    tag_name: "person".to_string(),
                    prop_names: vec!["name".to_string(), "age".to_string()],
                    is_default_props: false,
                },
            ],
            vec![
                VertexRow {
                    vid: lit(Value::Int(1)),
                    tag_values: vec![
                        vec![
                            lit(Value::String("Alice".to_string())),
                            lit(Value::Int(30)),
                        ],
                    ],
                },
            ],
        ).expect("Failed to build vertex insert info");
        assert_eq!(info.space_name, "test_space");
        assert_eq!(info.tags.len(), 1);
        assert_eq!(info.tags[0].tag_name, "person");
        assert_eq!(info.tags[0].prop_names.len(), 2);
        assert_eq!(info.values.len(), 1);
    }

    #[test]
    fn test_build_edge_insert_info() {
        let planner = InsertPlanner::new();
        let info = planner.build_edge_insert_info(
            "test_space".to_string(),
            "follow".to_string(),
            vec!["since".to_string()],
            vec![(
                lit(Value::Int(1)),
                lit(Value::Int(2)),
                Some(lit(Value::Int(0))),
                vec![lit(Value::String("2023".to_string()))],
            )],
        );
        assert_eq!(info.space_name, "test_space");
        assert_eq!(info.edge_name, "follow");
        assert_eq!(info.prop_names.len(), 1);
        assert_eq!(info.edges.len(), 1);
    }

    #[test]
    fn test_create_yield_columns() {
        let planner = InsertPlanner::new();
        let columns = planner.create_yield_columns(5);
        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0].alias, "inserted_count");
    }

    #[test]
    fn test_transform_insert_vertices() {
        let mut planner = InsertPlanner::new();
        let target = InsertTarget::Vertices {
            tags: vec![
                TagInsertSpec {
                    tag_name: "person".to_string(),
                    prop_names: vec!["name".to_string()],
                    is_default_props: false,
                },
            ],
            values: vec![
                VertexRow {
                    vid: lit(Value::Int(1)),
                    tag_values: vec![vec![lit(Value::String("Alice".to_string()))]],
                },
                VertexRow {
                    vid: lit(Value::Int(2)),
                    tag_values: vec![vec![lit(Value::String("Bob".to_string()))]],
                },
            ],
        };
        let stmt = create_test_stmt_with_insert(target);
        let qctx = create_test_qctx();

        // 创建验证后的语句
        let validation_info = ValidationInfo::new();
        let validated = ValidatedStatement::new(stmt, validation_info);

        let result = planner.transform(&validated, qctx);
        assert!(result.is_ok());
        let sub_plan = result.expect("Failed to transform insert statement");
        assert!(sub_plan.root.is_some());
    }

    #[test]
    fn test_transform_insert_edge() {
        let mut planner = InsertPlanner::new();
        let target = InsertTarget::Edge {
            edge_name: "follow".to_string(),
            prop_names: vec!["since".to_string()],
            edges: vec![(
                lit(Value::Int(1)),
                lit(Value::Int(2)),
                Some(lit(Value::Int(0))),
                vec![lit(Value::String("2023".to_string()))],
            )],
        };
        let stmt = create_test_stmt_with_insert(target);
        let qctx = create_test_qctx();

        // 创建验证后的语句
        let validation_info = ValidationInfo::new();
        let validated = ValidatedStatement::new(stmt, validation_info);

        let result = planner.transform(&validated, qctx);
        assert!(result.is_ok());
        let sub_plan = result.expect("Failed to transform insert statement");
        assert!(sub_plan.root.is_some());
    }

    #[test]
    fn test_transform_without_insert_stmt() {
        let mut planner = InsertPlanner::new();
        let stmt = Stmt::Use(crate::query::parser::ast::UseStmt {
            span: create_test_span(),
            space: "test_space".to_string(),
        });
        let qctx = create_test_qctx();

        // 创建验证后的语句
        let validation_info = ValidationInfo::new();
        let validated = ValidatedStatement::new(stmt, validation_info);

        let result = planner.transform(&validated, qctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_impl() {
        let planner: InsertPlanner = Default::default();
        let stmt = create_test_stmt_with_insert(InsertTarget::Vertices {
            tags: vec![
                TagInsertSpec {
                    tag_name: "test".to_string(),
                    prop_names: vec![],
                    is_default_props: true,
                },
            ],
            values: vec![],
        });
        assert!(planner.match_planner(&stmt));
    }
}
