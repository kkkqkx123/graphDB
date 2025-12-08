//! 验证上下文模块 - 管理查询验证阶段的上下文信息
//! 对应原C++中的ValidateContext.h

use std::collections::HashMap;
use crate::core::Value;

/// 验证上下文
/// 
/// 验证阶段的上下文，包含验证所需的信息
/// 对应原C++中的ValidateContext类
pub struct ValidateContext {
    // 命名空间映射
    namespaces: HashMap<String, String>,
    
    // 变量类型映射
    var_types: HashMap<String, String>,
    
    // 参数映射
    parameters: HashMap<String, Value>,
    
    // 当前选择的space
    current_space: Option<String>,
    
    // 是否已经选择了space
    space_chosen: bool,
}

impl ValidateContext {
    /// 创建新的验证上下文
    pub fn new() -> Self {
        Self {
            namespaces: HashMap::new(),
            var_types: HashMap::new(),
            parameters: HashMap::new(),
            current_space: None,
            space_chosen: false,
        }
    }

    /// 添加命名空间映射
    pub fn add_namespace(&mut self, alias: String, name: String) {
        self.namespaces.insert(alias, name);
    }

    /// 获取命名空间
    pub fn get_namespace(&self, alias: &str) -> Option<&String> {
        self.namespaces.get(alias)
    }

    /// 设置变量类型
    pub fn set_var_type(&mut self, var: String, var_type: String) {
        self.var_types.insert(var, var_type);
    }

    /// 获取变量类型
    pub fn get_var_type(&self, var: &str) -> Option<&String> {
        self.var_types.get(var)
    }

    /// 设置参数
    pub fn set_parameter(&mut self, name: String, value: Value) {
        self.parameters.insert(name, value);
    }

    /// 获取参数
    pub fn get_parameter(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name)
    }

    /// 获取所有参数
    pub fn get_parameters(&self) -> &HashMap<String, Value> {
        &self.parameters
    }

    /// 设置当前space
    pub fn set_current_space(&mut self, space: String) {
        self.current_space = Some(space);
        self.space_chosen = true;
    }

    /// 获取当前space
    pub fn current_space(&self) -> Option<&String> {
        self.current_space.as_ref()
    }

    /// 检查是否已选择space
    pub fn is_space_chosen(&self) -> bool {
        self.space_chosen
    }

    /// 重置space选择状态
    pub fn reset_space_status(&mut self) {
        self.current_space = None;
        self.space_chosen = false;
    }
}

impl Default for ValidateContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_validate_context() {
        let mut vctx = ValidateContext::new();
        
        // 测试命名空间
        vctx.add_namespace("ns1".to_string(), "namespace1".to_string());
        assert_eq!(vctx.get_namespace("ns1"), Some(&"namespace1".to_string()));
        
        // 测试变量类型
        vctx.set_var_type("var1".to_string(), "INT".to_string());
        assert_eq!(vctx.get_var_type("var1"), Some(&"INT".to_string()));
        
        // 测试参数
        vctx.set_parameter("param1".to_string(), Value::Int(42));
        assert_eq!(vctx.get_parameter("param1"), Some(&Value::Int(42)));
        
        // 测试space
        assert!(!vctx.is_space_chosen());
        vctx.set_current_space("test_space".to_string());
        assert!(vctx.is_space_chosen());
        assert_eq!(vctx.current_space(), Some(&"test_space".to_string()));
    }
}