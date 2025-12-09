//! 验证上下文
//! 对应 NebulaGraph ValidateContext 的功能
//! 用于在验证过程中存储和管理验证相关的信息

use std::collections::{HashMap, HashSet};
use crate::core::{Value, ValueTypeDef};

#[derive(Debug, Clone)]
pub struct Space {
    pub id: i32,
    pub name: String,
    pub vid_type: ValueTypeDef,  // 顶点ID类型
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub type_: ValueTypeDef,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub columns: Vec<Column>,
}

pub struct ValidateContext {
    /// 当前选择的图空间
    space: Option<Space>,
    /// 已定义的变量集合
    variables: HashMap<String, Variable>,
    /// 参数集合
    parameters: HashMap<String, Value>,
    /// 当前作用域中的别名
    aliases: HashMap<String, ValueTypeDef>,  // 别名 -> 类型
    /// 是否已选择空间
    space_chosen: bool,
    /// 错误信息
    errors: Vec<String>,
}

impl Default for ValidateContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidateContext {
    pub fn new() -> Self {
        Self {
            space: None,
            variables: HashMap::new(),
            parameters: HashMap::new(),
            aliases: HashMap::new(),
            space_chosen: false,
            errors: Vec::new(),
        }
    }

    pub fn set_space(&mut self, space: Space) {
        self.space = Some(space);
        self.space_chosen = true;
    }

    pub fn get_space(&self) -> Option<&Space> {
        self.space.as_ref()
    }

    pub fn space_chosen(&self) -> bool {
        self.space_chosen
    }

    pub fn which_space(&self) -> &Space {
        self.space.as_ref().expect("Space not chosen")
    }

    pub fn add_variable(&mut self, var: Variable) {
        self.variables.insert(var.name.clone(), var);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Variable> {
        self.variables.get(name)
    }

    pub fn exists_var(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    pub fn get_var(&self, name: &str) -> Vec<Column> {
        if let Some(var) = self.variables.get(name) {
            var.columns.clone()
        } else {
            Vec::new()
        }
    }

    pub fn add_parameter(&mut self, name: String, value: Value) {
        self.parameters.insert(name, value);
    }

    pub fn get_parameter(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name)
    }

    pub fn exist_parameter(&self, name: &str) -> bool {
        self.parameters.contains_key(name)
    }

    pub fn add_alias(&mut self, alias: String, type_: ValueTypeDef) {
        self.aliases.insert(alias, type_);
    }

    pub fn get_alias_type(&self, alias: &str) -> Option<&ValueTypeDef> {
        self.aliases.get(alias)
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn get_errors(&self) -> &Vec<String> {
        &self.errors
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }
}