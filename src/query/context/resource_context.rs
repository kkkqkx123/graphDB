//! Query the resource context.
//!
//! Manage the resources required during the execution of queries, including ID generators, etc.

use crate::utils::IdGenerator;

/// Query the resource context.
///
/// Managing the resources required during the execution of queries, including:
/// ID Generator (used for generating unique IDs)
pub struct QueryResourceContext {
    /// ID Generator
    id_gen: IdGenerator,
}

impl QueryResourceContext {
    /// Create a new resource context.
    pub fn new() -> Self {
        Self {
            id_gen: IdGenerator::new(0),
        }
    }

    /// Create a resource context with custom configurations.
    pub fn with_config(start_id: i64) -> Self {
        Self {
            id_gen: IdGenerator::new(start_id),
        }
    }

    /// Generate an ID.
    pub fn gen_id(&self) -> i64 {
        self.id_gen.id()
    }

    /// Retrieve the current ID value (without incrementing it).
    pub fn current_id(&self) -> i64 {
        self.id_gen.current_value()
    }

    /// Reset the resource context
    pub fn reset(&mut self) {
        self.id_gen.reset(0);
        log::info!("Query resource context has been reset");
    }
}

impl Default for QueryResourceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for QueryResourceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryResourceContext")
            .field("current_id", &self.current_id())
            .finish()
    }
}
