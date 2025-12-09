//! 验证上下文模块 - 管理查询验证阶段的上下文信息
//! 对应原C++中的ValidateContext.h

use std::collections::{HashMap, HashSet};
use crate::core::Value;

/// 图空间信息
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub id: i32,
    pub name: String,
    pub vid_type: String,  // 顶点ID类型
}

/// 列定义
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub type_: String,
}

/// 列定义集合 - 一个变量包含多个列
pub type ColsDef = Vec<Column>;

/// 变量定义 - 在查询中定义的变量（如MATCH中的别名）
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub columns: ColsDef,
}

/// 验证上下文
/// 
/// 验证阶段的上下文，包含验证所需的信息
/// 对应原C++中的ValidateContext类
/// 
/// 主要功能：
/// 1. 追踪图空间的选择
/// 2. 管理查询中定义的变量（如MATCH中的别名）
/// 3. 存储参数
/// 4. 追踪别名到类型的映射
#[derive(Debug, Clone)]
pub struct ValidateContext {
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

impl ValidateContext {
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
        self.spaces
            .last()
            .expect("空间未被选择")
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
    /// let mut ctx = ValidateContext::new();
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
    fn test_validate_context_new() {
        let ctx = ValidateContext::new();
        assert!(!ctx.space_chosen());
        assert!(ctx.get_all_variables().is_empty());
        assert!(!ctx.has_errors());
    }

    #[test]
    fn test_exists_var_basic() {
        let mut ctx = ValidateContext::new();
        
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
    fn test_exists_var_multiple() {
        let mut ctx = ValidateContext::new();
        
        ctx.register_variable("a".to_string(), vec![]);
        ctx.register_variable("b".to_string(), vec![]);
        ctx.register_variable("c".to_string(), vec![]);
        
        assert!(ctx.exists_var("a"));
        assert!(ctx.exists_var("b"));
        assert!(ctx.exists_var("c"));
        assert!(!ctx.exists_var("d"));
    }

    #[test]
    fn test_register_and_get_variable() {
        let mut ctx = ValidateContext::new();
        
        let cols = vec![
            Column {
                name: "id".to_string(),
                type_: "INT".to_string(),
            },
            Column {
                name: "name".to_string(),
                type_: "STRING".to_string(),
            },
        ];
        
        ctx.register_variable("person".to_string(), cols.clone());
        
        let retrieved = ctx.get_var("person");
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].name, "id");
        assert_eq!(retrieved[1].name, "name");
    }

    #[test]
    fn test_var_has_column() {
        let mut ctx = ValidateContext::new();
        
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
        
        assert!(ctx.var_has_column("n", "id"));
        assert!(ctx.var_has_column("n", "name"));
        assert!(!ctx.var_has_column("n", "age"));
        assert!(!ctx.var_has_column("m", "id"));
    }

    #[test]
    fn test_space_management() {
        let mut ctx = ValidateContext::new();
        
        assert!(!ctx.space_chosen());
        
        let space = SpaceInfo {
            id: 1,
            name: "test_space".to_string(),
            vid_type: "INT".to_string(),
        };
        
        ctx.switch_to_space(space.clone());
        
        assert!(ctx.space_chosen());
        assert_eq!(ctx.which_space().name, "test_space");
        assert_eq!(ctx.current_space().map(|s| s.id), Some(1));
    }

    #[test]
    fn test_parameter_management() {
        let mut ctx = ValidateContext::new();
        
        ctx.set_parameter("param1".to_string(), Value::Integer(42));
        
        assert!(ctx.exist_parameter("param1"));
        assert!(!ctx.exist_parameter("param2"));
        assert_eq!(ctx.get_parameter("param1"), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_alias_management() {
        let mut ctx = ValidateContext::new();
        
        ctx.add_alias("my_alias".to_string(), "STRING".to_string());
        
        assert!(ctx.exist_alias("my_alias"));
        assert!(!ctx.exist_alias("other_alias"));
        assert_eq!(ctx.get_alias_type("my_alias"), Some(&"STRING".to_string()));
    }

    #[test]
    fn test_space_creation() {
        let mut ctx = ValidateContext::new();
        
        ctx.add_space("new_space".to_string());
        
        assert!(ctx.has_space("new_space"));
        assert!(!ctx.has_space("other_space"));
        
        let spaces = ctx.get_create_spaces();
        assert_eq!(spaces.len(), 1);
        assert!(spaces.contains(&"new_space".to_string()));
    }

    #[test]
    fn test_index_management() {
        let mut ctx = ValidateContext::new();
        
        ctx.add_index("idx_name".to_string());
        ctx.add_index("idx_age".to_string());
        
        assert!(ctx.has_index("idx_name"));
        assert!(ctx.has_index("idx_age"));
        assert!(!ctx.has_index("idx_unknown"));
        
        let indexes = ctx.get_indexes();
        assert_eq!(indexes.len(), 2);
    }

    #[test]
    fn test_error_management() {
        let mut ctx = ValidateContext::new();
        
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

    #[test]
    fn test_get_all_variables() {
        let mut ctx = ValidateContext::new();
        
        ctx.register_variable("n".to_string(), vec![]);
        ctx.register_variable("m".to_string(), vec![]);
        ctx.register_variable("k".to_string(), vec![]);
        
        let vars = ctx.get_all_variables();
        assert_eq!(vars.len(), 3);
        assert!(vars.contains(&"n".to_string()));
        assert!(vars.contains(&"m".to_string()));
        assert!(vars.contains(&"k".to_string()));
    }

    #[test]
    fn test_get_variable_object() {
        let mut ctx = ValidateContext::new();
        
        let cols = vec![
            Column {
                name: "id".to_string(),
                type_: "INT".to_string(),
            },
        ];
        
        ctx.register_variable("n".to_string(), cols);
        
        let var = ctx.get_variable("n");
        assert!(var.is_some());
        assert_eq!(var.unwrap().name, "n");
        
        assert!(ctx.get_variable("m").is_none());
    }

    #[test]
    fn test_validate_context_comprehensive() {
        let mut ctx = ValidateContext::new();
        
        // 添加空间
        let space = SpaceInfo {
            id: 1,
            name: "my_space".to_string(),
            vid_type: "INT".to_string(),
        };
        ctx.switch_to_space(space);
        
        // 添加变量
        ctx.register_variable(
            "person".to_string(),
            vec![
                Column {
                    name: "id".to_string(),
                    type_: "INT".to_string(),
                },
            ],
        );
        
        // 添加参数
        ctx.set_parameter("limit".to_string(), Value::Integer(10));
        
        // 添加别名
        ctx.add_alias("person_alias".to_string(), "VERTEX".to_string());
        
        // 验证所有功能
        assert!(ctx.space_chosen());
        assert!(ctx.exists_var("person"));
        assert!(ctx.exist_parameter("limit"));
        assert!(ctx.exist_alias("person_alias"));
        assert!(!ctx.has_errors());
    }
}