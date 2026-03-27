//! Memory estimation trait for plan nodes
//!
//! This trait provides a common interface for estimating memory usage
//! of different plan node types.

pub trait MemoryEstimatable {
    /// Estimate memory usage for this node (in bytes)
    /// This should only estimate the node's own fields, not child nodes
    fn estimate_memory(&self) -> usize;
}

/// Helper function to estimate String memory
pub fn estimate_string_memory(s: &str) -> usize {
    std::mem::size_of::<String>() + s.len()
}

/// Helper function to estimate Option<String> memory
pub fn estimate_option_string_memory(opt: &Option<String>) -> usize {
    std::mem::size_of::<Option<String>>()
        + opt
            .as_ref()
            .map(|s| std::mem::size_of::<String>() + s.len())
            .unwrap_or(0)
}

/// Helper function to estimate Vec<String> memory
pub fn estimate_vec_string_memory(vec: &[String]) -> usize {
    std::mem::size_of::<Vec<String>>()
        + vec
            .iter()
            .map(|s| std::mem::size_of::<String>() + s.len())
            .sum::<usize>()
}

/// Helper function to estimate Vec<T> memory
pub fn estimate_vec_memory<T>(vec: &[T]) -> usize {
    std::mem::size_of_val(vec)
}

/// Macro to implement a default estimate_memory for plan nodes
/// This macro estimates the base struct size and col_names vector
#[macro_export]
macro_rules! impl_default_estimate_memory {
    ($node_type:ty) => {
        impl $crate::query::planning::plan::core::nodes::base::memory_estimation::MemoryEstimatable for $node_type {
            fn estimate_memory(&self) -> usize {
                let base = std::mem::size_of::<$node_type>();

                // Estimate col_names vector
                let col_names_size = $crate::query::planning::plan::core::nodes::base::memory_estimation::estimate_vec_string_memory(&self.col_names());

                // Estimate output_var
                let output_var_size = std::mem::size_of::<Option<String>>() +
                    self.output_var()
                        .map(|s| std::mem::size_of::<String>() + s.capacity())
                        .unwrap_or(0);

                base + col_names_size + output_var_size
            }
        }
    };
}
