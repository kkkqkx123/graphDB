//! 测试固件模块
//!
//! 提供测试所需的所有数据固件

pub mod documents;
pub mod helpers;

// 重新导出常用固件
// 抑制未使用导入警告，这些固件供各个测试文件选择性使用
#[allow(unused_imports)]
pub use documents::*;
#[allow(unused_imports)]
pub use helpers::*;
