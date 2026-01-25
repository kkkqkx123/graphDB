//! 基本验证上下文模块
//! 提供查询验证阶段的基础上下文管理功能

use super::types::{ColsDef, SpaceInfo, Variable};
use std::collections::{HashMap, HashSet};

/// 基本验证上下文
///
/// 验证阶段的上下文，包含验证所需的基础信息
///
/// 主要功能：
/// 1. 追踪图空间的选择
/// 2. 管理查询中定义的变量（如MATCH中的别名）
/// 3. 管理创建的空间和索引
#[derive(Debug, Clone)]
pub struct BasicValidationContext {
    /// 图空间栈 - 追踪空间切换的历史
    spaces: Vec<SpaceInfo>,

    /// 已定义的变量映射 (变量名 -> 列定义)
    /// 例如：MATCH (n:Person) -> 变量 n 的列定义
    variables: HashMap<String, ColsDef>,

    /// 创建的空间集合
    create_spaces: HashSet<String>,

    /// 索引集合
    indexes: HashSet<String>,
}

impl BasicValidationContext {
    /// 创建新的验证上下文
    pub fn new() -> Self {
        Self {
            spaces: Vec::new(),
            variables: HashMap::new(),
            create_spaces: HashSet::new(),
            indexes: HashSet::new(),
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
}

impl Default for BasicValidationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::Column;
    use crate::core::types::DataType;

    #[test]
    fn test_basic_validate_context_new() {
        let ctx = BasicValidationContext::new();
        assert!(!ctx.space_chosen());
        assert!(ctx.get_all_variables().is_empty());
    }

    #[test]
    fn test_exists_var_basic() {
        let mut ctx = BasicValidationContext::new();

        assert!(!ctx.exists_var("n"));

        ctx.register_variable(
            "n".to_string(),
            vec![
                Column {
                    name: "id".to_string(),
                    type_: DataType::Int,
                    nullable: false,
                    default_value: None,
                    comment: None,
                },
                Column {
                    name: "name".to_string(),
                    type_: DataType::String,
                    nullable: false,
                    default_value: None,
                    comment: None,
                },
            ],
        );

        assert!(ctx.exists_var("n"));
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
}
