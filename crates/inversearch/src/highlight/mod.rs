pub mod core;
pub mod boundary;
pub mod matcher;
pub mod processor;
pub mod types;

#[cfg(test)]
mod tests;

pub use types::*;
pub use core::{highlight_document, highlight_single_document};
pub use processor::{highlight_fields, HighlightProcessor};
pub use boundary::{apply_advanced_boundary, BoundaryTerm, BoundaryState};