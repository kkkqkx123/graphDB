//! 配置操作相关的计划节点
//! 包括显示、设置和获取配置等操作

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use std::sync::Arc;

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
    pub config_type: ConfigType,
    pub module_name: Option<String>, // 可选的模块名称
}

impl ShowConfigs {
    pub fn new(config_type: ConfigType, module_name: Option<String>) -> Self {
        Self {
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

impl From<ShowConfigs> for PlanNodeEnum {
    fn from(configs: ShowConfigs) -> Self {
        PlanNodeEnum::ShowConfigs(Arc::new(configs))
    }
}

/// 设置配置计划节点
#[derive(Debug, Clone)]
pub struct SetConfig {
    pub module_name: String,
    pub config_name: String,
    pub config_value: String,
}

impl SetConfig {
    pub fn new(module_name: &str, config_name: &str, config_value: &str) -> Self {
        Self {
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

impl From<SetConfig> for PlanNodeEnum {
    fn from(config: SetConfig) -> Self {
        PlanNodeEnum::SetConfig(Arc::new(config))
    }
}

/// 获取配置计划节点
#[derive(Debug, Clone)]
pub struct GetConfig {
    pub module_name: String,
    pub config_name: String,
}

impl GetConfig {
    pub fn new(module_name: &str, config_name: &str) -> Self {
        Self {
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

impl From<GetConfig> for PlanNodeEnum {
    fn from(config: GetConfig) -> Self {
        PlanNodeEnum::GetConfig(Arc::new(config))
    }
}