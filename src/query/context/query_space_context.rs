//! 查询空间上下文
//!
//! 管理查询执行过程中的空间信息，包括当前空间、字符集等。

use crate::core::types::CharsetInfo;
use crate::core::types::SpaceInfo;

/// 查询空间上下文
///
/// 管理查询执行过程中的空间信息，包括：
/// - 当前空间信息
/// - 字符集信息
pub struct QuerySpaceContext {
    /// 当前空间信息
    space_info: Option<SpaceInfo>,

    /// 字符集信息
    charset_info: Option<Box<CharsetInfo>>,
}

impl QuerySpaceContext {
    /// 创建新的空间上下文
    pub fn new() -> Self {
        Self {
            space_info: None,
            charset_info: None,
        }
    }

    /// 获取当前空间信息
    pub fn space_info(&self) -> Option<&SpaceInfo> {
        self.space_info.as_ref()
    }

    /// 设置当前空间信息
    pub fn set_space_info(&mut self, space_info: SpaceInfo) {
        self.space_info = Some(space_info);
    }

    /// 获取当前空间的 ID
    pub fn space_id(&self) -> Option<u64> {
        self.space_info().map(|s| s.space_id)
    }

    /// 获取当前空间的名称
    pub fn space_name(&self) -> Option<String> {
        self.space_info().map(|s| s.space_name.clone())
    }

    /// 设置字符集信息
    pub fn set_charset_info(&mut self, charset_info: CharsetInfo) {
        self.charset_info = Some(Box::new(charset_info));
    }

    /// 获取字符集信息
    pub fn charset_info(&self) -> Option<&CharsetInfo> {
        self.charset_info.as_ref().map(|ci| ci.as_ref())
    }

    /// 重置空间上下文
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
