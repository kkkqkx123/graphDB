use std::sync::Arc;
use std::time::Instant;

use super::super::base::{BaseExecutor, ExecutorStats};
use crate::core::Value;
use crate::query::DataSet;
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;

#[derive(Debug)]
pub struct GetPropExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    vertex_ids: Option<Vec<Value>>,
    edge_ids: Option<Vec<Value>>,
    prop_names: Vec<String>,
}

impl<S: StorageClient> GetPropExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_ids: Option<Vec<Value>>,
        edge_ids: Option<Vec<Value>>,
        prop_names: Vec<String>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "GetPropExecutor".to_string(), storage, expr_context),
            vertex_ids,
            edge_ids,
            prop_names,
        }
    }
}

impl<S: StorageClient> Executor<S> for GetPropExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(values) => {
                let dataset = DataSet::from_rows(
                    values.into_iter().map(|v| vec![v]).collect(),
                    vec!["value".to_string()],
                );
                Ok(ExecutionResult::DataSet(dataset))
            }
            Err(e) => Err(e),
        }
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
        "GetPropExecutor"
    }

    fn description(&self) -> &str {
        "Get property executor - retrieves properties from vertices or edges"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for GetPropExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient> GetPropExecutor<S> {
    fn do_execute(&mut self) -> DBResult<Vec<Value>> {
        let storage = self.get_storage().lock();

        let mut props = Vec::new();

        if let Some(ref vertex_ids) = self.vertex_ids {
            let total_props = vertex_ids.len() * self.prop_names.len();
            props.reserve(total_props);

            for vertex_id in vertex_ids {
                if let Some(vertex) = storage.get_vertex("default", vertex_id)? {
                    for prop_name in &self.prop_names {
                        if let Some(value) = vertex.get_property_any(prop_name) {
                            props.push(value.clone());
                        } else {
                            props
                                .push(crate::core::Value::Null(crate::core::value::NullType::Null));
                        }
                    }
                }
            }
        }

        if let Some(ref edge_ids) = self.edge_ids {
            let total_props = edge_ids.len() * self.prop_names.len();
            props.reserve(total_props);

            for edge_id in edge_ids {
                if let crate::core::Value::Edge(edge) = edge_id {
                    for prop_name in &self.prop_names {
                        if let Some(value) = edge.get_property(prop_name) {
                            props.push(value.clone());
                        } else {
                            props
                                .push(crate::core::Value::Null(crate::core::value::NullType::Null));
                        }
                    }
                }
            }
        }

        Ok(props)
    }
}
