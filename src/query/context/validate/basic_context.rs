//! 基本验证上下文模块
//! 提供查询验证阶段的基础上下文管理功能

use super::types::{ColsDef, SpaceInfo, Variable};
use crate::core::Value;
use std::collections::{HashMap, HashSet};

/// 基本验证上下文
///
/// 验证阶段的上下文，包含验证所需的基础信息
///
/// 主要功能：
/// 1. 追踪图空间的选择
/// 2. 管理查询中定义的变量（如MATCH中的别名）
/// 3. 存储参数
/// 4. 追踪别名到类型的映射
/// 5. 管理创建的空间和索引
/// 6. 收集验证错误信息
#[derive(Debug, Clone)]
pub struct BasicValidationContext {
    // 图空间栈 - 追踪空间切换的历史
    spaces: Vec<SpaceInfo>,

    // 已定义的变量映射 (变量名 -> 列定义)
    // 例如：MATCH (n:Person) -> 变量 n 的列定义
    variables: HashMap<String, ColsDef>,

    // 参数映射
    parameters: HashMap<String, Value>,

    // 别名到类型的映射
    aliases: HashMap<String, String>,

    // 创建的空间集合
    create_spaces: HashSet<String>,

    // 索引集合
    indexes: HashSet<String>,

    // 收集的错误信息
    errors: Vec<String>,
}

impl BasicValidationContext {
    /// 创建新的验证上下文
    pub fn new() -> Self {
        Self {
            spaces: Vec::new(),
            variables: HashMap::new(),
            parameters: HashMap::new(),
            aliases: HashMap::new(),
            create_spaces: HashSet::new(),
            indexes: HashSet::new(),
            errors: Vec::new(),
        }
    }

    // ==================== 空间管理 ====================

    /// 切换到指定的图空间
    /// 此操作将空间压入栈中，允许跟踪空间切换历史
    pub fn switch_to_space(&mut self, space: SpaceInfo) {
        self.spaces.push(space);
    }

    /// 检查是否已选择空间
    pub fn space_chosen(&self) -> bool {
        !self.spaces.is_empty()
    }

    /// 获取当前选择的空间
    ///
    /// # Panics
    /// 如果尚未选择空间，则panic
    pub fn which_space(&self) -> &SpaceInfo {
        self.spaces.last().expect("空间未被选择")
    }

    /// 获取当前空间（可选）
    pub fn current_space(&self) -> Option<&SpaceInfo> {
        self.spaces.last()
    }

    // ==================== 变量管理 ====================

    /// 注册一个变量（例如MATCH中的别名）
    ///
    /// # 参数
    /// * `var` - 变量名称
    /// * `cols` - 变量包含的列定义
    pub fn register_variable(&mut self, var: String, cols: ColsDef) {
        self.variables.insert(var, cols);
    }

    /// 获取变量的列定义
    pub fn get_var(&self, var: &str) -> ColsDef {
        self.variables
            .get(var)
            .map(|cols| cols.clone())
            .unwrap_or_default()
    }

    /// 检查变量是否存在
    ///
    /// 这是最常用的方法，用于验证一个变量是否已在查询中定义
    ///
    /// # 参数
    /// * `var` - 变量名称
    ///
    /// # 返回值
    /// 如果变量已注册，返回 `true`；否则返回 `false`
    ///
    /// # 示例
    /// ```ignore
    /// let mut ctx = BasicValidationContext::new();
    /// assert!(!ctx.exists_var("n"));
    ///
    /// ctx.register_variable("n".to_string(), vec![
    ///     Column { name: "id".to_string(), type_: "INT".to_string() },
    ///     Column { name: "name".to_string(), type_: "STRING".to_string() },
    /// ]);
    ///
    /// assert!(ctx.exists_var("n"));
    /// ```
    pub fn exists_var(&self, var: &str) -> bool {
        self.variables.contains_key(var)
    }

    /// 添加变量对象
    pub fn add_variable(&mut self, var: Variable) {
        self.variables.insert(var.name, var.columns);
    }

    /// 获取变量对象
    pub fn get_variable(&self, name: &str) -> Option<Variable> {
        self.variables.get(name).map(|cols| Variable {
            name: name.to_string(),
            columns: cols.clone(),
        })
    }

    /// 获取所有变量名
    pub fn get_all_variables(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    /// 检查变量是否有某个列
    pub fn var_has_column(&self, var: &str, col: &str) -> bool {
        self.variables
            .get(var)
            .map_or(false, |cols| cols.iter().any(|c| c.name == col))
    }

    // ==================== 参数管理 ====================

    /// 设置参数
    pub fn set_parameter(&mut self, name: String, value: Value) {
        self.parameters.insert(name, value);
    }

    /// 获取参数
    pub fn get_parameter(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name)
    }

    /// 检查参数是否存在
    pub fn exist_parameter(&self, name: &str) -> bool {
        self.parameters.contains_key(name)
    }

    /// 获取所有参数
    pub fn get_parameters(&self) -> &HashMap<String, Value> {
        &self.parameters
    }

    // ==================== 别名管理 ====================

    /// 添加别名及其类型
    pub fn add_alias(&mut self, alias: String, type_: String) {
        self.aliases.insert(alias, type_);
    }

    /// 获取别名的类型
    pub fn get_alias_type(&self, alias: &str) -> Option<&String> {
        self.aliases.get(alias)
    }

    /// 检查别名是否存在
    pub fn exist_alias(&self, alias: &str) -> bool {
        self.aliases.contains_key(alias)
    }

    // ==================== 空间创建管理 ====================

    /// 添加待创建的空间
    pub fn add_space(&mut self, space_name: String) {
        self.create_spaces.insert(space_name);
    }

    /// 检查空间是否待创建
    pub fn has_space(&self, space_name: &str) -> bool {
        self.create_spaces.contains(space_name)
    }

    /// 获取所有待创建的空间
    pub fn get_create_spaces(&self) -> Vec<String> {
        self.create_spaces.iter().cloned().collect()
    }

    // ==================== 索引管理 ====================

    /// 添加索引
    pub fn add_index(&mut self, index_name: String) {
        self.indexes.insert(index_name);
    }

    /// 检查索引是否存在
    pub fn has_index(&self, index_name: &str) -> bool {
        self.indexes.contains(index_name)
    }

    /// 获取所有索引
    pub fn get_indexes(&self) -> Vec<String> {
        self.indexes.iter().cloned().collect()
    }

    // ==================== 错误管理 ====================

    /// 添加错误信息
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// 获取所有错误
    pub fn get_errors(&self) -> &[String] {
        &self.errors
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// 清除所有错误
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

impl Default for BasicValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::Value, query::context::Column};

    #[test]
    fn test_basic_validate_context_new() {
        let ctx = BasicValidationContext::new();
        assert!(!ctx.space_chosen());
        assert!(ctx.get_all_variables().is_empty());
        assert!(!ctx.has_errors());
    }

    #[test]
    fn test_exists_var_basic() {
        let mut ctx = BasicValidationContext::new();

        // 变量不存在
        assert!(!ctx.exists_var("n"));

        // 添加变量
        ctx.register_variable(
            "n".to_string(),
            vec![
                Column {
                    name: "id".to_string(),
                    type_: "INT".to_string(),
                },
                Column {
                    name: "name".to_string(),
                    type_: "STRING".to_string(),
                },
            ],
        );

        // 变量存在
        assert!(ctx.exists_var("n"));

        // 其他变量不存在
        assert!(!ctx.exists_var("m"));
    }

    #[test]
    fn test_space_management() {
        let mut ctx = BasicValidationContext::new();

        assert!(!ctx.space_chosen());

        let space = SpaceInfo {
            space_id: Some(1),
            space_name: "test_space".to_string(),
            is_default: false,
        };

        ctx.switch_to_space(space.clone());

        assert!(ctx.space_chosen());
        assert_eq!(ctx.which_space().space_name, "test_space");
        assert_eq!(ctx.current_space().map(|s| s.space_id), Some(Some(1)));
    }

    #[test]
    fn test_parameter_management() {
        let mut ctx = BasicValidationContext::new();

        ctx.set_parameter("param1".to_string(), Value::Int(42));

        assert!(ctx.exist_parameter("param1"));
        assert!(!ctx.exist_parameter("param2"));
        assert_eq!(ctx.get_parameter("param1"), Some(&Value::Int(42)));
    }

    #[test]
    fn test_alias_management() {
        let mut ctx = BasicValidationContext::new();

        ctx.add_alias("my_alias".to_string(), "STRING".to_string());

        assert!(ctx.exist_alias("my_alias"));
        assert!(!ctx.exist_alias("other_alias"));
        assert_eq!(ctx.get_alias_type("my_alias"), Some(&"STRING".to_string()));
    }

    #[test]
    fn test_error_management() {
        let mut ctx = BasicValidationContext::new();

        assert!(!ctx.has_errors());
        assert_eq!(ctx.error_count(), 0);

        ctx.add_error("Error 1".to_string());
        assert!(ctx.has_errors());
        assert_eq!(ctx.error_count(), 1);

        ctx.add_error("Error 2".to_string());
        assert_eq!(ctx.error_count(), 2);

        let errors = ctx.get_errors();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0], "Error 1");

        ctx.clear_errors();
        assert!(!ctx.has_errors());
    }
}
