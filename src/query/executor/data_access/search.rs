//! Search for actuators
//!
//! Includes search-related executables such as index scanning

use std::sync::Arc;

use super::super::base::{BaseExecutor, ExecutorConfig, IndexScanConfig};
use crate::core::error::DBError;
use crate::core::{NullType, Value};
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::executor::expression::evaluator::traits::ExpressionContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// IndexScanExecutor - Index Scan Executor
///
/// Used to perform index-based scanning operations, supporting complex index queries
pub struct IndexScanExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    space_id: u64,
    tag_id: i32,
    index_id: i32,
    scan_type: String,
    scan_limits: Vec<crate::query::planning::plan::core::nodes::access::IndexLimit>,
    filter: Option<crate::core::Expression>,
    return_columns: Vec<String>,
    limit: Option<usize>,
    is_edge: bool,
}

impl<S: StorageClient> IndexScanExecutor<S> {
    pub fn new(base_config: ExecutorConfig<S>, scan_config: IndexScanConfig) -> Self {
        Self {
            base: BaseExecutor::new(
                base_config.id,
                "IndexScanExecutor".to_string(),
                base_config.storage,
                base_config.expr_context,
            ),
            space_id: scan_config.space_id,
            tag_id: scan_config.tag_id,
            index_id: scan_config.index_id,
            scan_type: scan_config.scan_type,
            scan_limits: scan_config.scan_limits,
            filter: scan_config.filter,
            return_columns: scan_config.return_columns,
            limit: scan_config.limit,
            is_edge: scan_config.is_edge,
        }
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

    pub fn scan_type(&self) -> &str {
        &self.scan_type
    }

    pub fn scan_limits(&self) -> &[crate::query::planning::plan::core::nodes::access::IndexLimit] {
        &self.scan_limits
    }

    pub fn return_columns(&self) -> &[String] {
        &self.return_columns
    }

    pub fn is_edge(&self) -> bool {
        self.is_edge
    }

    /// Get space name
    fn get_space_name(&self, storage: &S) -> DBResult<String> {
        if let Ok(Some(space_info)) = storage.get_space_by_id(self.space_id) {
            Ok(space_info.space_name)
        } else {
            Ok("default".to_string())
        }
    }

    /// Get the schema name (tag or edge type name)
    fn get_schema_name(&self, storage: &S) -> DBResult<String> {
        let space_name = self.get_space_name(storage)?;

        if self.is_edge {
            let edge_types = storage
                .list_edge_types(&space_name)
                .map_err(DBError::Storage)?;
            if let Some(edge_type_info) = edge_types.iter().find(|e| e.edge_type_id == self.tag_id)
            {
                Ok(edge_type_info.edge_type_name.clone())
            } else {
                Ok(format!("edge_type_{}", self.tag_id.abs()))
            }
        } else {
            let tags = storage.list_tags(&space_name).map_err(DBError::Storage)?;
            if let Some(tag_info) = tags.iter().find(|t| t.tag_id == self.tag_id) {
                Ok(tag_info.tag_name.clone())
            } else {
                Ok(format!("tag_{}", self.tag_id))
            }
        }
    }

    /// Perform an index lookup
    fn lookup_by_index(&self, storage: &S) -> DBResult<Vec<Value>> {
        let space_name = self.get_space_name(storage)?;
        let index_name = format!("index_{}", self.index_id);

        // Using the storage tier's index lookup function
        // Select different lookup strategies based on scan_type
        match self.scan_type.as_str() {
            "UNIQUE" => {
                // Unique Index Lookup
                if let Some(first_limit) = self.scan_limits.first() {
                    let value = first_limit
                        .begin_value
                        .as_ref()
                        .map(|v| Value::String(v.clone()))
                        .unwrap_or(Value::Null(NullType::Null));
                    storage
                        .lookup_index(&space_name, &index_name, &value)
                        .map_err(DBError::Storage)
                } else {
                    Ok(Vec::new())
                }
            }
            "PREFIX" => {
                // prefix index lookup
                if let Some(first_limit) = self.scan_limits.first() {
                    let prefix = first_limit
                        .begin_value
                        .as_ref()
                        .map(|v| Value::String(v.clone()))
                        .unwrap_or(Value::Null(NullType::Null));
                    storage
                        .lookup_index(&space_name, &index_name, &prefix)
                        .map_err(DBError::Storage)
                } else {
                    Ok(Vec::new())
                }
            }
            "RANGE" => {
                // Range Index Lookup
                if let Some(first_limit) = self.scan_limits.first() {
                    let column_name = &first_limit.column;
                    let include_begin = first_limit.include_begin;
                    let include_end = first_limit.include_end;

                    // Get start and end values
                    let start_value = first_limit
                        .begin_value
                        .as_ref()
                        .map(|v| Value::String(v.clone()));
                    let end_value = first_limit
                        .end_value
                        .as_ref()
                        .map(|v| Value::String(v.clone()));

                    // Returns null if there is no starting value
                    let start_val = match start_value {
                        Some(v) => v,
                        None => return Ok(Vec::new()),
                    };

                    // Prefix lookup using start value to get candidate results
                    let candidates = storage
                        .lookup_index(&space_name, &index_name, &start_val)
                        .map_err(DBError::Storage)?;

                    // Range filtering if there is an end value
                    if let Some(end_val) = end_value {
                        let filtered: Vec<Value> = candidates
                            .into_iter()
                            .filter(|id| {
                                // Getting the value of an entity's attribute for comparison
                                match self.get_entity_property_for_filter(storage, id, column_name)
                                {
                                    Some(prop_value) => {
                                        // Compare attribute values to see if they are in range, consider boundary inclusion control
                                        Self::value_in_range(
                                            &prop_value,
                                            &start_val,
                                            &end_val,
                                            include_begin,
                                            include_end,
                                        )
                                    }
                                    None => false,
                                }
                            })
                            .collect();
                        Ok(filtered)
                    } else {
                        // No end value, returns all candidate results (from start value to infinity)
                        // However, the starting boundary still needs to be checked
                        if include_begin {
                            Ok(candidates)
                        } else {
                            // does not contain a starting value and needs to be filtered out equal to the starting value of the
                            let filtered: Vec<Value> = candidates
                                .into_iter()
                                .filter(|id| {
                                    match self.get_entity_property_for_filter(
                                        storage,
                                        id,
                                        column_name,
                                    ) {
                                        Some(prop_value) => {
                                            !Self::values_equal(&prop_value, &start_val)
                                        }
                                        None => false,
                                    }
                                })
                                .collect();
                            Ok(filtered)
                        }
                    }
                } else {
                    Ok(Vec::new())
                }
            }
            _ => {
                // Default scanning of all
                Ok(Vec::new())
            }
        }
    }

    /// Getting the value of an attribute of an entity for range filtering
    fn get_entity_property_for_filter(
        &self,
        storage: &S,
        id: &Value,
        column_name: &str,
    ) -> Option<Value> {
        let space_name = match self.get_space_name(storage) {
            Ok(name) => name,
            Err(_) => return None,
        };

        if self.is_edge {
            // Edge type: ID format should be src:dst:ranking
            if let Value::String(edge_key) = id {
                let parts: Vec<&str> = edge_key.split(':').collect();
                if parts.len() >= 2 {
                    let src = Value::String(parts[0].to_string());
                    let dst = Value::String(parts[1].to_string());
                    let schema_name = match self.get_schema_name(storage) {
                        Ok(name) => name,
                        Err(_) => return None,
                    };

                    if let Ok(Some(edge)) = storage.get_edge(&space_name, &src, &dst, &schema_name, 0)
                    {
                        // Find from the properties of the edge
                        if let Some(value) = edge.props.get(column_name) {
                            return Some(value.clone());
                        }
                        // special field
                        match column_name {
                            "src" => return Some((*edge.src).clone()),
                            "dst" => return Some((*edge.dst).clone()),
                            "edge_type" => return Some(Value::String(edge.edge_type.clone())),
                            "ranking" => return Some(Value::Int(edge.ranking)),
                            _ => return None,
                        }
                    }
                }
            }
        } else {
            // Vertex Type
            if let Ok(Some(vertex)) = storage.get_vertex(&space_name, id) {
                // Find from vertex's attributes
                if let Some(value) = vertex.properties.get(column_name) {
                    return Some(value.clone());
                }
                // Find from tag's attributes
                for tag in &vertex.tags {
                    if let Some(value) = tag.properties.get(column_name) {
                        return Some(value.clone());
                    }
                }
                // special field
                match column_name {
                    "vid" => return Some((*vertex.vid).clone()),
                    "id" => return Some(Value::Int(vertex.id)),
                    _ => return None,
                }
            }
        }

        None
    }

    /// Get the complete vertex or edge based on the ID list
    fn fetch_entities(&self, storage: &S, ids: Vec<Value>) -> DBResult<Vec<Value>> {
        let space_name = self.get_space_name(storage)?;
        let schema_name = self.get_schema_name(storage)?;

        let mut results = Vec::new();

        for id in ids {
            if self.is_edge {
                // Edge type: ID format should be src_dst_ranking
                if let Value::String(edge_key) = &id {
                    let parts: Vec<&str> = edge_key.split(':').collect();
                    if parts.len() >= 2 {
                        let src = Value::String(parts[0].to_string());
                        let dst = Value::String(parts[1].to_string());
                        if let Some(edge) = storage
                            .get_edge(&space_name, &src, &dst, &schema_name, 0)
                            .map_err(DBError::Storage)?
                        {
                            results.push(Value::Edge(edge));
                        }
                    }
                }
            } else {
                // Vertex Type
                if let Some(vertex) = storage
                    .get_vertex(&space_name, &id)
                    .map_err(DBError::Storage)?
                {
                    results.push(Value::Vertex(Box::new(vertex)));
                }
            }
        }

        Ok(results)
    }

    /// Application filters
    fn apply_filter(&self, entities: Vec<Value>) -> Vec<Value> {
        if let Some(ref filter_expr) = self.filter {
            let mut context = crate::query::executor::expression::DefaultExpressionContext::new();
            entities
                .into_iter()
                .filter(|entity| {
                    context.set_variable("entity".to_string(), entity.clone());
                    match crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator::evaluate(filter_expr, &mut context) {
                        Ok(value) => match &value {
                            Value::Bool(true) => true,
                            Value::Int(i) => *i != 0,
                            Value::Float(f) => *f != 0.0,
                            Value::String(s) => !s.is_empty(),
                            Value::List(l) => !l.is_empty(),
                            Value::Map(m) => !m.is_empty(),
                            _ => false,
                        },
                        Err(_) => true,
                    }
                })
                .collect()
        } else {
            entities
        }
    }

    /// Projected return columns
    fn project_columns(&self, entities: Vec<Value>) -> Vec<Value> {
        if self.return_columns.is_empty() || self.return_columns.contains(&"*".to_string()) {
            return entities;
        }

        entities
            .into_iter()
            .map(|entity| match entity {
                Value::Vertex(vertex) => {
                    let mut props = std::collections::HashMap::new();
                    for col in &self.return_columns {
                        match col.as_str() {
                            "vid" => {
                                props.insert(col.clone(), (*vertex.vid).clone());
                            }
                            "id" => {
                                props.insert(col.clone(), Value::Int(vertex.id));
                            }
                            "*" => {
                                for (k, v) in &vertex.properties {
                                    props.insert(k.clone(), v.clone());
                                }
                            }
                            _ => {
                                if let Some(v) = vertex.properties.get(col) {
                                    props.insert(col.clone(), v.clone());
                                }
                            }
                        }
                    }
                    Value::Map(props)
                }
                Value::Edge(edge) => {
                    let mut props = std::collections::HashMap::new();
                    for col in &self.return_columns {
                        match col.as_str() {
                            "src" => {
                                props.insert(col.clone(), (*edge.src).clone());
                            }
                            "dst" => {
                                props.insert(col.clone(), (*edge.dst).clone());
                            }
                            "edge_type" => {
                                props.insert(col.clone(), Value::String(edge.edge_type.clone()));
                            }
                            "ranking" => {
                                props.insert(col.clone(), Value::Int(edge.ranking));
                            }
                            "*" => {
                                for (k, v) in &edge.props {
                                    props.insert(k.clone(), v.clone());
                                }
                            }
                            _ => {
                                if let Some(v) = edge.props.get(col) {
                                    props.insert(col.clone(), v.clone());
                                }
                            }
                        }
                    }
                    Value::Map(props)
                }
                _ => entity,
            })
            .collect()
    }

    /// Checks if the value is within the specified range
    fn value_in_range(
        value: &Value,
        start: &Value,
        end: &Value,
        include_begin: bool,
        include_end: bool,
    ) -> bool {
        use std::cmp::Ordering;

        // Compare Starting Boundaries
        let pass_start = match Self::compare_values(value, start) {
            Some(Ordering::Greater) => true,
            Some(Ordering::Equal) => include_begin,
            Some(Ordering::Less) => false,
            None => false,
        };

        if !pass_start {
            return false;
        }

        // Comparison End Boundary
        match Self::compare_values(value, end) {
            Some(Ordering::Less) => true,
            Some(Ordering::Equal) => include_end,
            Some(Ordering::Greater) => false,
            None => false,
        }
    }

    /// Comparing two values
    fn compare_values(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
        match (a, b) {
            (Value::Int(a_i), Value::Int(b_i)) => Some(a_i.cmp(b_i)),
            (Value::Float(a_f), Value::Float(b_f)) => a_f.partial_cmp(b_f),
            (Value::Int(a_i), Value::Float(b_f)) => (*a_i as f64).partial_cmp(b_f),
            (Value::Float(a_f), Value::Int(b_i)) => a_f.partial_cmp(&(*b_i as f64)),
            (Value::String(a_s), Value::String(b_s)) => Some(a_s.cmp(b_s)),
            _ => None,
        }
    }

    /// Check if two values are equal
    fn values_equal(a: &Value, b: &Value) -> bool {
        matches!(Self::compare_values(a, b), Some(std::cmp::Ordering::Equal))
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for IndexScanExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = self.get_storage().lock();

        // 1. Use index lookup to get a list of IDs
        let index_results = self.lookup_by_index(&storage)?;

        // 2. Obtaining the full entity by ID
        let entities = self.fetch_entities(&storage, index_results)?;

        // 3. Application filters
        let filtered = self.apply_filter(entities);

        // 4. Projected return columns
        let projected = self.project_columns(filtered);

        // 5. Application limitations
        let limited: Vec<Value> = if let Some(limit) = self.limit {
            projected.into_iter().take(limit).collect()
        } else {
            projected
        };

        // 6. Constructing return results
        let rows: Vec<Vec<Value>> = limited.into_iter().map(|v| vec![v]).collect();

        Ok(ExecutionResult::Values(
            rows.into_iter().flatten().collect(),
        ))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Index scan executor - scans vertices using index"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for IndexScanExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
