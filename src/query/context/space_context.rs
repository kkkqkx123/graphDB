//! Query space context
//!
//! Manage spatial information during query execution, including current space, character set, etc.

use crate::core::types::CharsetInfo;
use crate::core::types::SpaceInfo;

/// Query space context
///
/// Manage spatial information during query execution, including:
/// - Current space information
/// - character set information
pub struct QuerySpaceContext {
    /// Current space information
    space_info: Option<SpaceInfo>,

    /// character set information
    charset_info: Option<Box<CharsetInfo>>,
}

impl QuerySpaceContext {
    /// Creating a new spatial context
    pub fn new() -> Self {
        Self {
            space_info: None,
            charset_info: None,
        }
    }

    /// Get current space information
    pub fn space_info(&self) -> Option<&SpaceInfo> {
        self.space_info.as_ref()
    }

    /// Setting current space information
    pub fn set_space_info(&mut self, space_info: SpaceInfo) {
        self.space_info = Some(space_info);
    }

    /// Get the ID of the current space
    pub fn space_id(&self) -> Option<u64> {
        self.space_info().map(|s| s.space_id)
    }

    /// Get the name of the current space
    pub fn space_name(&self) -> Option<String> {
        self.space_info().map(|s| s.space_name.clone())
    }

    /// Setting Character Set Information
    pub fn set_charset_info(&mut self, charset_info: CharsetInfo) {
        self.charset_info = Some(Box::new(charset_info));
    }

    /// Getting Character Set Information
    pub fn charset_info(&self) -> Option<&CharsetInfo> {
        self.charset_info.as_ref().map(|ci| ci.as_ref())
    }

    /// Reset the spatial context
    pub fn reset(&mut self) {
        self.space_info = None;
        self.charset_info = None;
        log::info!("查询空间上下文已重置");
    }
}

impl Default for QuerySpaceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for QuerySpaceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuerySpaceContext")
            .field("space_id", &self.space_id())
            .field("space_name", &self.space_name())
            .finish()
    }
}
