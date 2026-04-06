//! 测试通用组件
//!
//! 提供测试所需的通用工具、固件和辅助函数

pub mod fixtures;

// 重新导出 fixtures 中的常用项
// 抑制未使用导入警告，这些固件供各个测试文件选择性使用
#[allow(unused_imports)]
pub use fixtures::*;
