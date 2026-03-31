//! AlterTagExecutor – The tag modification executor
//!
//! Responsible for modifying the attribute definitions of existing tags.

use parking_lot::Mutex;
use std::sync::Arc;

use crate::core::types::PropertyDef;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

/// Type of label modification operation
#[derive(Debug, Clone)]
pub enum AlterTagOp {
    Add,
    Drop,
    Change,
}

/// Tag modification items
#[derive(Debug, Clone)]
pub struct AlterTagItem {
    pub op: AlterTagOp,
    pub property: Option<PropertyDef>,
    pub property_name: Option<String>,
}

impl AlterTagItem {
    pub fn add_property(property: PropertyDef) -> Self {
        Self {
            op: AlterTagOp::Add,
            property: Some(property),
            property_name: None,
        }
    }

    pub fn drop_property(property_name: String) -> Self {
        Self {
            op: AlterTagOp::Drop,
            property: None,
            property_name: Some(property_name),
        }
    }
}

/// Tag modification information
#[derive(Debug, Clone)]
pub struct AlterTagInfo {
    pub space_name: String,
    pub tag_name: String,
    pub items: Vec<AlterTagItem>,
    pub comment: Option<String>,
}

impl AlterTagInfo {
    pub fn new(space_name: String, tag_name: String) -> Self {
        Self {
            space_name,
            tag_name,
            items: Vec::new(),
            comment: None,
        }
    }

    pub fn with_items(mut self, items: Vec<AlterTagItem>) -> Self {
        self.items = items;
        self
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

/// Modify the Tag Executor
///
/// This executor is responsible for modifying the attribute definitions of existing tags.
#[derive(Debug)]
pub struct AlterTagExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    alter_info: AlterTagInfo,
}

impl<S: StorageClient> AlterTagExecutor<S> {
    /// Create a new AlterTagExecutor
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        alter_info: AlterTagInfo,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "AlterTagExecutor".to_string(), storage, expr_context),
            alter_info,
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for AlterTagExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        let storage = self.get_storage();
        let mut storage_guard = storage.lock();

        let additions: Vec<crate::core::types::PropertyDef> = self
            .alter_info
            .items
            .iter()
            .filter_map(|item| match item.op {
                AlterTagOp::Add => item.property.clone(),
                _ => None,
            })
            .collect();

        let deletions: Vec<String> = self
            .alter_info
            .items
            .iter()
            .filter_map(|item| match item.op {
                AlterTagOp::Drop => item.property_name.clone(),
                AlterTagOp::Change => item.property_name.clone(),
                _ => None,
            })
            .collect();

        if !deletions.is_empty() {
            let tag_info =
                storage_guard.get_tag(&self.alter_info.space_name, &self.alter_info.tag_name);
            if let Ok(Some(tag)) = tag_info {
                for del_name in &deletions {
                    if !tag.properties.iter().any(|p| &p.name == del_name) {
                        return Ok(ExecutionResult::Error(format!(
                            "Property '{}' not found in tag '{}'",
                            del_name, self.alter_info.tag_name
                        )));
                    }
                }
            }
        }

        let result = storage_guard.alter_tag(
            &self.alter_info.space_name,
            &self.alter_info.tag_name,
            additions,
            deletions,
        );

        match result {
            Ok(true) => Ok(ExecutionResult::Success),
            Ok(false) => Ok(ExecutionResult::Error(format!(
                "Tag '{}' not found in space '{}'",
                self.alter_info.tag_name, self.alter_info.space_name
            ))),
            Err(e) => Ok(ExecutionResult::Error(format!(
                "Failed to alter tag: {}",
                e
            ))),
        }
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.open()
    }

    fn close(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.close()
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "AlterTagExecutor"
    }

    fn description(&self) -> &str {
        "Alters a tag"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> crate::query::executor::base::HasStorage<S> for AlterTagExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
