//! 配置操作相关的计划节点
//! 包括显示、设置和获取配置等操作

use crate::query::planner::plan::core::nodes::management_node_enum::ManagementNodeEnum;
use crate::query::planner::plan::core::nodes::management_node_traits::ManagementNode;

/// 配置参数类型
#[derive(Debug, Clone)]
pub enum ConfigType {
    Mutable,
    Immutable,
    All,
}

/// 配置参数
#[derive(Debug, Clone)]
pub struct ConfigItem {
    pub name: String,
    pub value: String,
    pub default_value: String,
    pub mutable: bool,
    pub description: String,
}

/// 显示配置计划节点
#[derive(Debug, Clone)]
pub struct ShowConfigs {
    pub id: i64,
    pub cost: f64,
    pub config_type: ConfigType,
    pub module_name: Option<String>, // 可选的模块名称
}

impl ShowConfigs {
    pub fn new(id: i64, cost: f64, config_type: ConfigType, module_name: Option<String>) -> Self {
        Self {
            id,
            cost,
            config_type,
            module_name,
        }
    }

    pub fn config_type(&self) -> &ConfigType {
        &self.config_type
    }

    pub fn module_name(&self) -> Option<&str> {
        self.module_name.as_deref()
    }
}

impl ManagementNode for ShowConfigs {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShowConfigs"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::ShowConfigs(self)
    }
}

/// 设置配置计划节点
#[derive(Debug, Clone)]
pub struct SetConfig {
    pub id: i64,
    pub cost: f64,
    pub module_name: String,
    pub config_name: String,
    pub config_value: String,
}

impl SetConfig {
    pub fn new(
        id: i64,
        cost: f64,
        module_name: &str,
        config_name: &str,
        config_value: &str,
    ) -> Self {
        Self {
            id,
            cost,
            module_name: module_name.to_string(),
            config_name: config_name.to_string(),
            config_value: config_value.to_string(),
        }
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn config_name(&self) -> &str {
        &self.config_name
    }

    pub fn config_value(&self) -> &str {
        &self.config_value
    }
}

impl ManagementNode for SetConfig {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "SetConfig"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::SetConfig(self)
    }
}

/// 获取配置计划节点
#[derive(Debug, Clone)]
pub struct GetConfig {
    pub id: i64,
    pub cost: f64,
    pub module_name: String,
    pub config_name: String,
}

impl GetConfig {
    pub fn new(id: i64, cost: f64, module_name: &str, config_name: &str) -> Self {
        Self {
            id,
            cost,
            module_name: module_name.to_string(),
            config_name: config_name.to_string(),
        }
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn config_name(&self) -> &str {
        &self.config_name
    }
}

impl ManagementNode for GetConfig {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "GetConfig"
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn into_enum(self) -> ManagementNodeEnum {
        ManagementNodeEnum::GetConfig(self)
    }
}
