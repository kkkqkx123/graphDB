//! LOOKUP语句规划器
//! 处理Nebula LOOKUP查询的规划

use crate::core::types::expression::Expression;
use crate::core::types::operators::BinaryOperator;
use crate::query::context::ast::{AstContext, LookupContext};
use crate::query::planner::plan::core::{
    ArgumentNode, DedupNode, FilterNode, GetEdgesNode, GetVerticesNode, HashInnerJoinNode,
    ProjectNode,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

/// LOOKUP查询规划器
/// 负责将LOOKUP语句转换为执行计划
#[derive(Debug)]
pub struct LookupPlanner;

impl LookupPlanner {
    /// 创建新的LOOKUP规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配LOOKUP查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "LOOKUP"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
            priority: 100,
        }
    }
}

impl Planner for LookupPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let lookup_ctx = LookupContext::new(ast_ctx.clone());
        let space_id = ast_ctx.space().space_id
            .ok_or_else(|| PlannerError::PlanGenerationFailed("Space ID is required for LOOKUP query".to_string()))?;
        
        if space_id == 0 {
            return Err(PlannerError::PlanGenerationFailed("Invalid space ID: 0".to_string()));
        }

        if lookup_ctx.schema_id == 0 {
            return Err(PlannerError::PlanGenerationFailed("Invalid schema ID: 0".to_string()));
        }

        let mut sub_plan = SubPlan {
            root: None,
            tail: None,
        };

        let index_scan_node = if lookup_ctx.is_fulltext_index {
            if lookup_ctx.is_edge {
                let edge_index_scan = crate::query::planner::plan::algorithms::IndexScan::new(
                    -1,
                    space_id as i32,
                    lookup_ctx.schema_id,
                    lookup_ctx.schema_id,
                    "FULLTEXT",
                );
                crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::IndexScan(
                    edge_index_scan,
                )
            } else {
                let tag_index_scan = crate::query::planner::plan::algorithms::IndexScan::new(
                    -1,
                    space_id as i32,
                    lookup_ctx.schema_id,
                    lookup_ctx.schema_id,
                    "FULLTEXT",
                );
                crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::IndexScan(
                    tag_index_scan,
                )
            }
        } else {
            if lookup_ctx.is_edge {
                let edge_index_scan = crate::query::planner::plan::algorithms::IndexScan::new(
                    -1,
                    space_id as i32,
                    lookup_ctx.schema_id,
                    lookup_ctx.schema_id,
                    "RANGE",
                );
                crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::IndexScan(
                    edge_index_scan,
                )
            } else {
                let tag_index_scan = crate::query::planner::plan::algorithms::IndexScan::new(
                    -1,
                    space_id as i32,
                    lookup_ctx.schema_id,
                    lookup_ctx.schema_id,
                    "RANGE",
                );
                crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::IndexScan(
                    tag_index_scan,
                )
            }
        };

        sub_plan.tail = Some(index_scan_node.clone());
        let mut current_node = index_scan_node;

        if lookup_ctx.is_fulltext_index && lookup_ctx.has_score {
            let id_expr = Expression::Variable("id".to_string());
            let score_expr = Expression::Variable("_score".to_string());

            let get_node = if lookup_ctx.is_edge {
                let get_edges = GetEdgesNode::new(
                    space_id as i32,
                    "",
                    "",
                    "",
                    "",
                );
                crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::GetEdges(
                    get_edges,
                )
            } else {
                let get_vertices = GetVerticesNode::new(
                    space_id as i32,
                    "",
                );
                crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::GetVertices(
                    get_vertices,
                )
            };

            let argument_node = ArgumentNode::new(-1, "id");
            let argument_enum = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Argument(argument_node);

            let hash_join = HashInnerJoinNode::new(
                get_node,
                argument_enum,
                vec![id_expr.clone()],
                vec![id_expr],
            )
            .map_err(|e| PlannerError::PlanGenerationFailed(format!("Failed to create HashInnerJoinNode: {}", e)))?;

            current_node = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::HashInnerJoin(hash_join);
        } else if lookup_ctx.is_fulltext_index {
            let get_node = if lookup_ctx.is_edge {
                let get_edges = GetEdgesNode::new(
                    space_id as i32,
                    "",
                    "",
                    "",
                    "",
                );
                crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::GetEdges(
                    get_edges,
                )
            } else {
                let get_vertices = GetVerticesNode::new(
                    space_id as i32,
                    "",
                );
                crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::GetVertices(
                    get_vertices,
                )
            };

            current_node = get_node;
        }

        if let Some(ref condition) = lookup_ctx.filter {
            let expr = parse_filter_expression(condition)
                .map_err(|e| PlannerError::PlanGenerationFailed(format!("Failed to parse filter expression: {}", e)))?;
            let filter_node = FilterNode::new(current_node, expr)
                .map_err(|e| PlannerError::PlanGenerationFailed(format!("Failed to create FilterNode: {}", e)))?;
            current_node = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Filter(
                filter_node,
            );
        }

        sub_plan.root = Some(current_node.clone());

        use crate::query::validator::YieldColumn;

        let yield_columns = if let Some(ref yield_expr) = lookup_ctx.yield_expr {
            yield_expr
                .columns
                .iter()
                .map(|col| {
                    let expr = parse_yield_expression(col.name(), lookup_ctx.is_edge)
                        .map_err(|e| PlannerError::PlanGenerationFailed(format!("Failed to parse yield expression: {}", e)))?;
                    Ok(YieldColumn {
                        expr,
                        alias: col.alias.clone(),
                        is_matched: false,
                    })
                })
                .collect::<Result<Vec<_>, PlannerError>>()?
        } else {
            vec![YieldColumn {
                expr: Expression::Variable("*".to_string()),
                alias: "result".to_string(),
                is_matched: false,
            }]
        };

        let project_node = ProjectNode::new(current_node, yield_columns)
            .map_err(|e| PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e)))?;
        current_node = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Project(
            project_node,
        );
        sub_plan.root = Some(current_node.clone());

        if lookup_ctx.dedup {
            let dedup_node = DedupNode::new(current_node)
                .map_err(|e| PlannerError::PlanGenerationFailed(format!("Failed to create DedupNode: {}", e)))?;
            current_node = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Dedup(
                dedup_node,
            );
            sub_plan.root = Some(current_node);
        }

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for LookupPlanner {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析过滤条件表达式
fn parse_filter_expression(condition: &str) -> Result<Expression, String> {
    if condition.contains("=") {
        let parts: Vec<&str> = condition.split("=").collect();
        if parts.len() == 2 {
            return Ok(Expression::Binary {
                left: Box::new(Expression::Variable(parts[0].trim().to_string())),
                op: BinaryOperator::Equal,
                right: Box::new(Expression::Variable(parts[1].trim().to_string())),
            });
        }
    }

    if condition.contains(">") {
        let parts: Vec<&str> = condition.split(">").collect();
        if parts.len() == 2 {
            return Ok(Expression::Binary {
                left: Box::new(Expression::Variable(parts[0].trim().to_string())),
                op: BinaryOperator::GreaterThan,
                right: Box::new(Expression::Variable(parts[1].trim().to_string())),
            });
        }
    }

    if condition.contains("<") {
        let parts: Vec<&str> = condition.split("<").collect();
        if parts.len() == 2 {
            return Ok(Expression::Binary {
                left: Box::new(Expression::Variable(parts[0].trim().to_string())),
                op: BinaryOperator::LessThan,
                right: Box::new(Expression::Variable(parts[1].trim().to_string())),
            });
        }
    }

    Ok(Expression::Variable(condition.trim().to_string()))
}

/// 解析 yield 表达式
fn parse_yield_expression(expr_str: &str, is_edge: bool) -> Result<Expression, PlannerError> {
    if expr_str.starts_with("src(") {
        return Ok(Expression::Function {
            name: "src".to_string(),
            args: vec![],
        });
    }

    if expr_str.starts_with("dst(") {
        return Ok(Expression::Function {
            name: "dst".to_string(),
            args: vec![],
        });
    }

    if expr_str.starts_with("rank(") {
        return Ok(Expression::Function {
            name: "rank".to_string(),
            args: vec![],
        });
    }

    if expr_str.starts_with("id(") {
        return Ok(Expression::Function {
            name: "id".to_string(),
            args: vec![],
        });
    }

    if expr_str == "*" {
        return Ok(Expression::Variable("*".to_string()));
    }

    if expr_str.contains(".") {
        let parts: Vec<&str> = expr_str.split(".").collect();
        if parts.len() == 2 {
            return Ok(Expression::Property {
                object: Box::new(Expression::Variable(parts[0].to_string())),
                property: parts[1].to_string(),
            });
        }
    }

    Ok(Expression::Variable(expr_str.to_string()))
}
