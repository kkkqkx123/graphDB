//! 访问者状态枚举实现
//!
//! 这个模块提供了零成本抽象的访问者状态管理，使用枚举替代动态分发

use std::collections::HashMap;

/// 访问者状态枚举 - 替代 dyn VisitorState
#[derive(Debug, Clone)]
pub enum VisitorStateEnum {
    /// 默认访问者状态
    Default(DefaultVisitorState),
}

/// 默认访问者状态实现
#[derive(Debug, Clone)]
pub struct DefaultVisitorState {
    /// 是否继续访问
    continue_visiting: bool,
    /// 访问深度
    depth: usize,
    /// 访问计数
    visit_count: usize,
    /// 自定义状态数据
    custom_data: HashMap<String, String>,
}

impl DefaultVisitorState {
    /// 创建新的默认状态
    pub fn new() -> Self {
        Self {
            continue_visiting: true,
            depth: 0,
            visit_count: 0,
            custom_data: HashMap::new(),
        }
    }

    /// 创建带初始深度的状态
    pub fn with_depth(depth: usize) -> Self {
        Self {
            continue_visiting: true,
            depth,
            visit_count: 0,
            custom_data: HashMap::new(),
        }
    }

    /// 获取是否继续访问
    pub fn continue_visiting(&self) -> bool {
        self.continue_visiting
    }

    /// 设置是否继续访问
    pub fn set_continue_visiting(&mut self, continue_visiting: bool) {
        self.continue_visiting = continue_visiting;
    }

    /// 获取访问深度
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// 设置访问深度
    pub fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    /// 获取访问计数
    pub fn visit_count(&self) -> usize {
        self.visit_count
    }

    /// 设置访问计数
    pub fn set_visit_count(&mut self, visit_count: usize) {
        self.visit_count = visit_count;
    }

    /// 获取自定义数据
    pub fn custom_data(&self) -> &HashMap<String, String> {
        &self.custom_data
    }

    /// 获取可变自定义数据
    pub fn custom_data_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.custom_data
    }
}

impl Default for DefaultVisitorState {
    fn default() -> Self {
        Self::new()
    }
}

impl VisitorStateEnum {
    /// 创建新的默认状态枚举
    pub fn new() -> Self {
        Self::Default(DefaultVisitorState::new())
    }

    /// 创建带初始深度的状态枚举
    pub fn with_depth(depth: usize) -> Self {
        Self::Default(DefaultVisitorState::with_depth(depth))
    }

    /// 重置状态
    pub fn reset(&mut self) {
        match self {
            VisitorStateEnum::Default(state) => {
                state.set_continue_visiting(true);
                state.set_depth(0);
                state.set_visit_count(0);
                state.custom_data_mut().clear();
            }
        }
    }

    /// 检查是否应该继续访问
    pub fn should_continue(&self) -> bool {
        match self {
            VisitorStateEnum::Default(state) => state.continue_visiting(),
        }
    }

    /// 停止访问
    pub fn stop(&mut self) {
        match self {
            VisitorStateEnum::Default(state) => {
                state.set_continue_visiting(false);
            }
        }
    }

    /// 获取访问深度
    pub fn depth(&self) -> usize {
        match self {
            VisitorStateEnum::Default(state) => state.depth(),
        }
    }

    /// 设置访问深度
    pub fn set_depth(&mut self, depth: usize) {
        match self {
            VisitorStateEnum::Default(state) => {
                state.set_depth(depth);
            }
        }
    }

    /// 增加访问深度
    pub fn inc_depth(&mut self) {
        match self {
            VisitorStateEnum::Default(state) => {
                state.set_depth(state.depth() + 1);
            }
        }
    }

    /// 减少访问深度
    pub fn dec_depth(&mut self) {
        match self {
            VisitorStateEnum::Default(state) => {
                let current_depth = state.depth();
                if current_depth > 0 {
                    state.set_depth(current_depth - 1);
                }
            }
        }
    }

    /// 获取访问计数
    pub fn visit_count(&self) -> usize {
        match self {
            VisitorStateEnum::Default(state) => state.visit_count(),
        }
    }

    /// 增加访问计数
    pub fn inc_visit_count(&mut self) {
        match self {
            VisitorStateEnum::Default(state) => {
                state.set_visit_count(state.visit_count() + 1);
            }
        }
    }

    /// 获取自定义状态数据
    pub fn get_custom_data(&self, key: &str) -> Option<&String> {
        match self {
            VisitorStateEnum::Default(state) => state.custom_data().get(key),
        }
    }

    /// 设置自定义状态数据
    pub fn set_custom_data(&mut self, key: String, value: String) {
        match self {
            VisitorStateEnum::Default(state) => {
                state.custom_data_mut().insert(key, value);
            }
        }
    }

    /// 移除自定义状态数据
    pub fn remove_custom_data(&mut self, key: &str) -> Option<String> {
        match self {
            VisitorStateEnum::Default(state) => state.custom_data_mut().remove(key),
        }
    }

    /// 获取状态类型
    pub fn state_type(&self) -> &'static str {
        match self {
            VisitorStateEnum::Default(_) => "Default",
        }
    }

    /// 转换为默认状态（如果可能）
    pub fn as_default(&self) -> Option<&DefaultVisitorState> {
        match self {
            VisitorStateEnum::Default(state) => Some(state),
        }
    }

    /// 转换为可变默认状态（如果可能）
    pub fn as_default_mut(&mut self) -> Option<&mut DefaultVisitorState> {
        match self {
            VisitorStateEnum::Default(state) => Some(state),
        }
    }
}

impl Default for VisitorStateEnum {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_visitor_state() {
        let mut state = DefaultVisitorState::new();

        assert_eq!(state.continue_visiting(), true);
        assert_eq!(state.depth(), 0);
        assert_eq!(state.visit_count(), 0);
        assert!(state.custom_data().is_empty());

        state.set_continue_visiting(false);
        state.set_depth(5);
        state.set_visit_count(10);
        state
            .custom_data_mut()
            .insert("key".to_string(), "value".to_string());

        assert_eq!(state.continue_visiting(), false);
        assert_eq!(state.depth(), 5);
        assert_eq!(state.visit_count(), 10);
        assert_eq!(state.custom_data().get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_visitor_state_enum() {
        let mut state = VisitorStateEnum::new();

        assert_eq!(state.should_continue(), true);
        assert_eq!(state.depth(), 0);
        assert_eq!(state.visit_count(), 0);
        assert_eq!(state.state_type(), "Default");

        state.inc_depth();
        state.inc_visit_count();
        state.set_custom_data("test".to_string(), "data".to_string());

        assert_eq!(state.depth(), 1);
        assert_eq!(state.visit_count(), 1);
        assert_eq!(state.get_custom_data("test"), Some(&"data".to_string()));

        state.stop();
        assert_eq!(state.should_continue(), false);

        state.reset();
        assert_eq!(state.should_continue(), true);
        assert_eq!(state.depth(), 0);
        assert_eq!(state.visit_count(), 0);
        assert!(state.get_custom_data("test").is_none());
    }

    #[test]
    fn test_visitor_state_enum_with_depth() {
        let state = VisitorStateEnum::with_depth(3);
        assert_eq!(state.depth(), 3);

        let mut state = state;
        state.dec_depth();
        assert_eq!(state.depth(), 2);

        state.dec_depth();
        state.dec_depth();
        assert_eq!(state.depth(), 0);

        // 深度不会变成负数
        state.dec_depth();
        assert_eq!(state.depth(), 0);
    }
}
