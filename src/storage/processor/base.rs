use crate::core::error::{DBError, DBResult, StorageError};
use crate::query::context::runtime_context::RuntimeContext;
use crate::storage::StorageClient;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ProcessorCounters {
    pub num_calls: i64,
    pub num_errors: i64,
    pub latency_us: i64,
}

impl Default for ProcessorCounters {
    fn default() -> Self {
        Self {
            num_calls: 0,
            num_errors: 0,
            latency_us: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PartCode {
    pub code: DBError,
    pub part_id: u32,
}

pub trait StorageProcessor<RESP>: Send {
    fn context(&self) -> &Arc<RuntimeContext>;
    fn context_mut(&mut self) -> &mut RuntimeContext;

    fn execute(&mut self) -> DBResult<RESP>;

    fn on_finished(&mut self) -> DBResult<RESP> {
        self.execute()
    }

    fn on_error(&mut self, _error: DBError) -> DBResult<RESP> {
        self.execute()
    }
}

pub struct ProcessorBase<RESP, S>
where
    S: StorageClient,
{
    context: Arc<RuntimeContext>,
    resp: Option<RESP>,
    duration: Duration,
    codes: Vec<PartCode>,
    storage: Arc<Mutex<S>>,
}

impl<RESP, S> ProcessorBase<RESP, S>
where
    S: StorageClient,
{
    pub fn new(context: Arc<RuntimeContext>, storage: Arc<Mutex<S>>) -> Self {
        Self {
            context,
            resp: None,
            duration: Duration::ZERO,
            codes: Vec::new(),
            storage,
        }
    }

    pub fn push_code(&mut self, code: DBError, part_id: u32) {
        self.codes.push(PartCode { code, part_id });
    }

    pub fn is_memory_exceeded(&self) -> bool {
        false
    }

    pub fn storage(&self) -> &Arc<Mutex<S>> {
        &self.storage
    }

    pub fn context(&self) -> &Arc<RuntimeContext> {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut RuntimeContext {
        Arc::make_mut(&mut self.context)
    }

    pub fn failed_parts(&self) -> &[PartCode] {
        &self.codes
    }

    pub fn has_errors(&self) -> bool {
        !self.codes.is_empty()
    }

    pub fn take_response(&mut self) -> RESP {
        self.resp
            .take()
            .expect("response should be set before taking")
    }

    pub fn set_response(&mut self, resp: RESP) {
        self.resp = Some(resp);
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn start_timer(&mut self) {
        self.duration = Duration::ZERO;
    }

    pub fn stop_timer(&mut self) {
    }
}

pub struct ProcessorFuture<RESP> {
    resp: Option<DBResult<RESP>>,
}

impl<RESP> ProcessorFuture<RESP> {
    pub fn new(resp: DBResult<RESP>) -> Self {
        Self { resp: Some(resp) }
    }

    pub fn into_result(self) -> DBResult<RESP> {
        self.resp.unwrap_or(Err(DBError::Internal(
            "processor future was already consumed".to_string(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::runtime_context::{PlanContext, RuntimeContext, StorageEnv};
    use crate::storage::metadata::SchemaManager;
    use crate::storage::index::IndexManager;
    use std::sync::Mutex;

    struct DummySchemaManager;
    impl SchemaManager for DummySchemaManager {}

    struct DummyIndexManager;
    impl IndexManager for DummyIndexManager {}

    struct DummyStorage;
    impl StorageClient for DummyStorage {
        fn get(&self, _space: u32, _part: u32, _key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
            Ok(None)
        }

        fn put(&self, _space: u32, _part: u32, _key: &[u8], _value: &[u8]) -> StorageResult<()> {
            Ok(())
        }

        fn delete(&self, _space: u32, _part: u32, _key: &[u8]) -> StorageResult<()> {
            Ok(())
        }

        fn scan(
            &self,
            _space: u32,
            _part: u32,
            _start: &[u8],
            _end: &[u8],
        ) -> StorageResult<Box<dyn crate::storage::StorageIter>> {
            Ok(Box::new(crate::storage::DefaultIter::default()))
        }
    }

    fn create_test_runtime_context() -> Arc<RuntimeContext> {
        let storage_env = Arc::new(StorageEnv {
            storage_engine: Arc::new(DummyStorage),
            schema_manager: Arc::new(DummySchemaManager),
            index_manager: Arc::new(DummyIndexManager),
        });

        let plan_context = Arc::new(PlanContext {
            storage_env,
            space_id: 1,
            plan_id: 100,
            v_id_len: 8,
            is_int_id: true,
            is_edge: false,
        });

        Arc::new(RuntimeContext::new(plan_context))
    }

    #[test]
    fn test_processor_context_new() {
        let ctx = create_test_runtime_context();
        assert_eq!(ctx.space_id(), 1);
        assert_eq!(ctx.v_id_len(), 8);
    }

    #[test]
    fn test_processor_context_with_counters() {
        let counters = ProcessorCounters::default();
        let _ctx = create_test_runtime_context();
        assert!(true);
    }

    #[test]
    fn test_processor_base_new() {
        let ctx = create_test_runtime_context();
        let storage: Arc<Mutex<DummyStorage>> = Arc::new(Mutex::new(DummyStorage));

        let base = ProcessorBase::new(ctx, storage);

        assert!(!base.has_errors());
        assert!(base.failed_parts().is_empty());
        assert_eq!(base.duration(), Duration::ZERO);
    }

    #[test]
    fn test_processor_push_code() {
        let ctx = create_test_runtime_context();
        let storage: Arc<Mutex<DummyStorage>> = Arc::new(Mutex::new(DummyStorage));
        let mut base = ProcessorBase::new(ctx, storage);

        let error = DBError::Storage(StorageError::NotFound("test".to_string()));
        base.push_code(error.clone(), 100);

        assert!(base.has_errors());
        assert_eq!(base.failed_parts().len(), 1);
        assert_eq!(base.failed_parts()[0].part_id, 100);
    }
}
