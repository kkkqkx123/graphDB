pub mod boundary;
pub mod core;
pub mod matcher;
pub mod processor;
pub mod types;

#[cfg(test)]
mod tests;

pub use boundary::{apply_advanced_boundary, BoundaryState, BoundaryTerm};
pub use core::{
    highlight_document, highlight_document_structured, highlight_single_document,
    highlight_single_document_structured,
};
pub use processor::{
    highlight_fields, highlight_results, highlight_results_with_complete, HighlightProcessor,
};
pub use types::*;
