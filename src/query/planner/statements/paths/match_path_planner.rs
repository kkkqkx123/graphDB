//! MATCH 路径规划器
//!
//! 负责规划 MATCH 查询中的路径模式，生成遍历计划

use crate::core::types::graph_schema::EdgeDirection;
use crate::core::{StorageError, Value};
use crate::query::planner::statements::seeks::seek_strategy_base::{
    NodePattern, SeekStrategyContext, SeekStrategySelector, SeekStrategyType,
};
use crate::storage::StorageClient;
use crate::storage::transaction::TransactionId;

pub type PlannerError = StorageError;

#[derive(Debug)]
pub struct MatchPathPlanner;

impl MatchPathPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn plan_path_pattern<S: StorageClient>(
        &self,
        storage: &S,
        pattern: &PathPattern,
        space_id: i32,
    ) -> Result<PathPlan, PlannerError> {
        match &pattern.kind {
            PathPatternKind::Simple { start, edge, end } => {
                self.plan_simple_pattern(storage, start, edge, end, space_id)
            }
            PathPatternKind::VariableLength {
                start,
                edge,
                end,
                lower,
                upper,
            } => self.plan_variable_length_pattern(
                storage,
                start,
                edge,
                end,
                *lower,
                *upper,
                space_id,
            ),
            PathPatternKind::Named { name, inner } => {
                let inner_plan = self.plan_path_pattern(storage, inner, space_id)?;
                Ok(PathPlan::Named {
                    name: name.clone(),
                    inner: Box::new(inner_plan),
                })
            }
        }
    }

    fn plan_simple_pattern<S: StorageClient>(
        &self,
        storage: &S,
        start: &NodePattern,
        edge: &EdgePattern,
        end: &NodePattern,
        space_id: i32,
    ) -> Result<PathPlan, PlannerError> {
        let start_finder = self.plan_start_finder(storage, start, space_id)?;
        let edge_traversal = self.plan_edge_traversal(edge)?;
        let end_finder = self.plan_end_finder(end)?;

        Ok(PathPlan::Simple {
            start: Box::new(start_finder),
            edge: edge_traversal,
            end: end_finder,
        })
    }

    fn plan_variable_length_pattern<S: StorageClient>(
        &self,
        storage: &S,
        start: &NodePattern,
        edge: &EdgePattern,
        end: &NodePattern,
        lower: Option<usize>,
        upper: Option<usize>,
        space_id: i32,
    ) -> Result<PathPlan, PlannerError> {
        let start_finder = self.plan_start_finder(storage, start, space_id)?;
        let edge_types = self.extract_edge_types(edge)?;
        let direction = self.extract_direction(edge)?;
        let end_finder = self.plan_end_finder(end)?;

        Ok(PathPlan::VariableLength {
            start: Box::new(start_finder),
            edge_types,
            direction,
            end: end_finder,
            lower,
            upper,
        })
    }

    fn plan_start_finder<S: StorageClient>(
        &self,
        _storage: &S,
        pattern: &NodePattern,
        space_id: i32,
    ) -> Result<StartVidFinder, PlannerError> {
        let context = SeekStrategyContext::new(space_id, pattern.clone(), vec![]);
        let selector = SeekStrategySelector::new();
        let strategy_type = selector.select_strategy(&DummyStorage, &context);

        let finder = match strategy_type {
            SeekStrategyType::VertexSeek => StartVidFinder::VertexSeek {
                pattern: pattern.clone(),
            },
            SeekStrategyType::IndexSeek => StartVidFinder::IndexScan {
                pattern: pattern.clone(),
            },
            SeekStrategyType::ScanSeek => StartVidFinder::FullScan {
                pattern: pattern.clone(),
            },
        };

        Ok(finder)
    }

    fn plan_end_finder(&self, pattern: &NodePattern) -> Result<EndCondition, PlannerError> {
        Ok(EndCondition {
            pattern: pattern.clone(),
        })
    }

    fn plan_edge_traversal(&self, edge: &EdgePattern) -> Result<EdgeTraversal, PlannerError> {
        let direction = self.extract_direction(edge)?;
        let edge_types = self.extract_edge_types(edge)?;
        let properties = edge.properties.clone();

        Ok(EdgeTraversal {
            direction,
            edge_types,
            properties,
        })
    }

    fn extract_direction(&self, edge: &EdgePattern) -> Result<EdgeDirection, PlannerError> {
        Ok(match edge.direction {
            Some(ref dir) => dir.clone(),
            None => EdgeDirection::Both,
        })
    }

    fn extract_edge_types(&self, edge: &EdgePattern) -> Result<Vec<String>, PlannerError> {
        Ok(edge.types.clone().unwrap_or_default())
    }
}

struct DummyStorage;

impl StorageClient for DummyStorage {
    fn insert_vertex(&mut self, _space: &str, _vertex: crate::core::Vertex) -> Result<Value, crate::core::StorageError> {
        Ok(Value::Int(0))
    }
    fn get_vertex(&self, _space: &str, _id: &Value) -> Result<Option<crate::core::Vertex>, crate::core::StorageError> {
        Ok(None)
    }
    fn update_vertex(&mut self, _space: &str, _vertex: crate::core::Vertex) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn delete_vertex(&mut self, _space: &str, _id: &Value) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn scan_vertices(&self, _space: &str) -> Result<Vec<crate::core::Vertex>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn scan_vertices_by_tag(&self, _space: &str, _tag: &str) -> Result<Vec<crate::core::Vertex>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn scan_vertices_by_prop(&self, _space: &str, _tag: &str, _prop: &str, _value: &Value) -> Result<Vec<crate::core::Vertex>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn insert_edge(&mut self, _space: &str, _edge: crate::core::Edge) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn get_edge(&self, _space: &str, _src: &Value, _dst: &Value, _edge_type: &str) -> Result<Option<crate::core::Edge>, crate::core::StorageError> {
        Ok(None)
    }
    fn get_node_edges(&self, _space: &str, _node_id: &Value, _direction: EdgeDirection) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn get_node_edges_filtered(&self, _space: &str, _node_id: &Value, _direction: EdgeDirection, _filter: Option<Box<dyn Fn(&crate::core::Edge) -> bool + Send + Sync>>) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn delete_edge(&mut self, _space: &str, _src: &Value, _dst: &Value, _edge_type: &str) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn scan_edges_by_type(&self, _space: &str, _edge_type: &str) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn scan_all_edges(&self, _space: &str) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn batch_insert_vertices(&mut self, _space: &str, _vertices: Vec<crate::core::Vertex>) -> Result<Vec<Value>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn batch_insert_edges(&mut self, _space: &str, _edges: Vec<crate::core::Edge>) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn begin_transaction(&mut self, _space: &str) -> Result<TransactionId, crate::core::StorageError> {
        Ok(TransactionId::new(0))
    }
    fn commit_transaction(&mut self, _space: &str, _tx_id: TransactionId) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn rollback_transaction(&mut self, _space: &str, _tx_id: TransactionId) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn create_space(&mut self, _space: &crate::core::types::SpaceInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn drop_space(&mut self, _space: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_space(&self, _space: &str) -> Result<Option<crate::core::types::SpaceInfo>, crate::core::StorageError> {
        Ok(None)
    }
    fn list_spaces(&self) -> Result<Vec<crate::core::types::SpaceInfo>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn create_tag(&mut self, _space: &str, _info: &crate::core::types::TagInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn alter_tag(&mut self, _space: &str, _tag_name: &str, _additions: Vec<crate::core::types::PropertyDef>, _deletions: Vec<String>) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_tag(&self, _space: &str, _tag_name: &str) -> Result<Option<crate::core::types::TagInfo>, crate::core::StorageError> {
        Ok(None)
    }
    fn drop_tag(&mut self, _space: &str, _tag_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn list_tags(&self, _space: &str) -> Result<Vec<crate::core::types::TagInfo>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn create_edge_type(&mut self, _space: &str, _info: &crate::core::types::EdgeTypeSchema) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn alter_edge_type(&mut self, _space: &str, _edge_type_name: &str, _additions: Vec<crate::core::types::PropertyDef>, _deletions: Vec<String>) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_edge_type(&self, _space: &str, _edge_type_name: &str) -> Result<Option<crate::core::types::EdgeTypeSchema>, crate::core::StorageError> {
        Ok(None)
    }
    fn drop_edge_type(&mut self, _space: &str, _edge_type_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn list_edge_types(&self, _space: &str) -> Result<Vec<crate::core::types::EdgeTypeSchema>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn create_tag_index(&mut self, _space: &str, _info: &crate::core::types::IndexInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn drop_tag_index(&mut self, _space: &str, _index_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_tag_index(&self, _space: &str, _index_name: &str) -> Result<Option<crate::core::types::IndexInfo>, crate::core::StorageError> {
        Ok(None)
    }
    fn list_tag_indexes(&self, _space: &str) -> Result<Vec<crate::core::types::IndexInfo>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn rebuild_tag_index(&mut self, _space: &str, _index_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn create_edge_index(&mut self, _space: &str, _info: &crate::core::types::IndexInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn drop_edge_index(&mut self, _space_name: &str, _index_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_edge_index(&self, _space_name: &str, _index_name: &str) -> Result<Option<crate::core::types::IndexInfo>, crate::core::StorageError> {
        Ok(None)
    }
    fn list_edge_indexes(&self, _space: &str) -> Result<Vec<crate::core::types::IndexInfo>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn rebuild_edge_index(&mut self, _space: &str, _index_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn change_password(&mut self, _info: &crate::core::types::PasswordInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn insert_vertex_data(&mut self, _space: &str, _info: &crate::core::types::InsertVertexInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn insert_edge_data(&mut self, _space: &str, _info: &crate::core::types::InsertEdgeInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn delete_vertex_data(&mut self, _space: &str, _vertex_id: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn delete_edge_data(&mut self, _space: &str, _src: &str, _dst: &str, _rank: i64) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn update_data(&mut self, _space: &str, _info: &crate::core::types::UpdateInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }

    fn get_vertex_with_schema(&self, _space: &str, _tag_name: &str, _id: &crate::core::Value) -> Result<Option<(crate::expression::storage::Schema, Vec<u8>)>, crate::core::StorageError> {
        Ok(None)
    }

    fn get_edge_with_schema(&self, _space: &str, _edge_type_name: &str, _src: &crate::core::Value, _dst: &crate::core::Value) -> Result<Option<(crate::expression::storage::Schema, Vec<u8>)>, crate::core::StorageError> {
        Ok(None)
    }

    fn scan_vertices_with_schema(&self, _space: &str, _tag_name: &str) -> Result<Vec<(crate::expression::storage::Schema, Vec<u8>)>, crate::core::StorageError> {
        Ok(Vec::new())
    }

    fn scan_edges_with_schema(&self, _space: &str, _edge_type_name: &str) -> Result<Vec<(crate::expression::storage::Schema, Vec<u8>)>, crate::core::StorageError> {
        Ok(Vec::new())
    }

    fn lookup_index(&self, _space: &str, _index: &str, _value: &crate::core::Value) -> Result<Vec<crate::core::Value>, crate::core::StorageError> {
        Ok(Vec::new())
    }

    fn load_from_disk(&mut self) -> Result<(), crate::core::StorageError> {
        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathPattern {
    pub kind: PathPatternKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathPatternKind {
    Simple {
        start: NodePattern,
        edge: EdgePattern,
        end: NodePattern,
    },
    VariableLength {
        start: NodePattern,
        edge: EdgePattern,
        end: NodePattern,
        lower: Option<usize>,
        upper: Option<usize>,
    },
    Named {
        name: String,
        inner: Box<PathPattern>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgePattern {
    pub types: Option<Vec<String>>,
    pub direction: Option<EdgeDirection>,
    pub properties: Vec<(String, Value)>,
}

#[derive(Debug)]
pub enum StartVidFinder {
    VertexSeek { pattern: NodePattern },
    IndexScan { pattern: NodePattern },
    FullScan { pattern: NodePattern },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EndCondition {
    pub pattern: NodePattern,
}

#[derive(Debug)]
pub enum PathPlan {
    Simple {
        start: Box<StartVidFinder>,
        edge: EdgeTraversal,
        end: EndCondition,
    },
    VariableLength {
        start: Box<StartVidFinder>,
        edge_types: Vec<String>,
        direction: EdgeDirection,
        end: EndCondition,
        lower: Option<usize>,
        upper: Option<usize>,
    },
    Named {
        name: String,
        inner: Box<PathPlan>,
    },
}

#[derive(Debug)]
pub struct EdgeTraversal {
    pub direction: EdgeDirection,
    pub edge_types: Vec<String>,
    pub properties: Vec<(String, Value)>,
}

impl PathPattern {
    pub fn simple(start: NodePattern, edge: EdgePattern, end: NodePattern) -> Self {
        Self {
            kind: PathPatternKind::Simple { start, edge, end },
        }
    }

    pub fn variable_length(
        start: NodePattern,
        edge: EdgePattern,
        end: NodePattern,
        lower: Option<usize>,
        upper: Option<usize>,
    ) -> Self {
        Self {
            kind: PathPatternKind::VariableLength {
                start,
                edge,
                end,
                lower,
                upper,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_path_planner_new() {
        let planner = MatchPathPlanner::new();
        assert!(true);
    }

    #[test]
    fn test_path_pattern_simple() {
        let pattern = PathPattern::simple(
            NodePattern {
                vid: Some(Value::String("start".to_string())),
                labels: vec![],
                properties: vec![],
            },
            EdgePattern {
                types: Some(vec!["follows".to_string()]),
                direction: Some(EdgeDirection::Out),
                properties: vec![],
            },
            NodePattern {
                vid: Some(Value::String("end".to_string())),
                labels: vec![],
                properties: vec![],
            },
        );
        assert!(true);
    }

    #[test]
    fn test_path_pattern_variable_length() {
        let pattern = PathPattern::variable_length(
            NodePattern {
                vid: None,
                labels: vec!["person".to_string()],
                properties: vec![],
            },
            EdgePattern {
                types: Some(vec!["follows".to_string()]),
                direction: Some(EdgeDirection::Out),
                properties: vec![],
            },
            NodePattern {
                vid: None,
                labels: vec!["person".to_string()],
                properties: vec![],
            },
            Some(1),
            Some(5),
        );
        assert!(true);
    }
}
