//! Schema 操作 API - 核心层
//!
//! 提供与传输层无关的 Schema 管理功能

use crate::storage::StorageClient;
use crate::api::core::{CoreResult, CoreError, PropertyDef, IndexTarget, SpaceConfig};
use crate::core::types::{SpaceInfo, TagInfo, EdgeTypeInfo};
use crate::index::{Index, IndexField, IndexType, IndexStatus};
use std::sync::Arc;
use parking_lot::Mutex;

/// Schema 操作 API - 核心层
pub struct SchemaApi<S: StorageClient> {
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient> SchemaApi<S> {
    /// 创建新的 Schema API 实例
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self { storage }
    }

    /// 创建图空间
    ///
    /// # 参数
    /// - `name`: 空间名称
    /// - `config`: 空间配置
    pub fn create_space(&self, name: &str, config: SpaceConfig) -> CoreResult<()> {
        let space_info = SpaceInfo::new(name.to_string())
            .with_vid_type(config.vid_type)
            .with_comment(config.comment);
        
        let mut storage = self.storage.lock();
        storage.create_space(&space_info)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        log::info!("创建图空间成功: {}", name);
        Ok(())
    }

    /// 删除图空间
    ///
    /// # 参数
    /// - `name`: 空间名称
    pub fn drop_space(&self, name: &str) -> CoreResult<()> {
        let mut storage = self.storage.lock();
        let result = storage.drop_space(name)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        if result {
            log::info!("删除图空间成功: {}", name);
            Ok(())
        } else {
            Err(CoreError::NotFound(format!("图空间 '{}' 不存在", name)))
        }
    }

    /// 使用图空间
    ///
    /// # 参数
    /// - `name`: 空间名称
    ///
    /// # 返回
    /// 空间 ID
    pub fn use_space(&self, name: &str) -> CoreResult<u64> {
        let storage = self.storage.lock();
        let space_id = storage.get_space_id(name)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        log::info!("使用图空间: {} (ID: {})", name, space_id);
        Ok(space_id)
    }

    /// 创建标签
    ///
    /// # 参数
    /// - `space_id`: 空间 ID
    /// - `name`: 标签名称
    /// - `properties`: 属性定义列表
    pub fn create_tag(
        &self,
        space_id: u64,
        name: &str,
        properties: Vec<PropertyDef>,
    ) -> CoreResult<()> {
        // 获取空间名称
        let space_name = self.get_space_name_by_id(space_id)?;
        
        // 转换属性定义
        let core_properties: Vec<crate::core::types::PropertyDef> = properties
            .into_iter()
            .map(|p| p.into())
            .collect();
        
        let tag_info = TagInfo::new(name.to_string())
            .with_properties(core_properties);
        
        let mut storage = self.storage.lock();
        let result = storage.create_tag(&space_name, &tag_info)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        if result {
            log::info!("创建标签成功: {} in space {}", name, space_id);
            Ok(())
        } else {
            Err(CoreError::SchemaOperationFailed(format!("标签 '{}' 已存在", name)))
        }
    }

    /// 删除标签
    ///
    /// # 参数
    /// - `space_id`: 空间 ID
    /// - `name`: 标签名称
    pub fn drop_tag(&self, space_id: u64, name: &str) -> CoreResult<()> {
        let space_name = self.get_space_name_by_id(space_id)?;
        
        let mut storage = self.storage.lock();
        let result = storage.drop_tag(&space_name, name)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        if result {
            log::info!("删除标签成功: {} from space {}", name, space_id);
            Ok(())
        } else {
            Err(CoreError::NotFound(format!("标签 '{}' 不存在", name)))
        }
    }

    /// 创建边类型
    ///
    /// # 参数
    /// - `space_id`: 空间 ID
    /// - `name`: 边类型名称
    /// - `properties`: 属性定义列表
    pub fn create_edge_type(
        &self,
        space_id: u64,
        name: &str,
        properties: Vec<PropertyDef>,
    ) -> CoreResult<()> {
        let space_name = self.get_space_name_by_id(space_id)?;
        
        // 转换属性定义
        let core_properties: Vec<crate::core::types::PropertyDef> = properties
            .into_iter()
            .map(|p| p.into())
            .collect();
        
        let edge_type_info = EdgeTypeInfo::new(name.to_string())
            .with_properties(core_properties);
        
        let mut storage = self.storage.lock();
        let result = storage.create_edge_type(&space_name, &edge_type_info)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        if result {
            log::info!("创建边类型成功: {} in space {}", name, space_id);
            Ok(())
        } else {
            Err(CoreError::SchemaOperationFailed(format!("边类型 '{}' 已存在", name)))
        }
    }

    /// 删除边类型
    ///
    /// # 参数
    /// - `space_id`: 空间 ID
    /// - `name`: 边类型名称
    pub fn drop_edge_type(&self, space_id: u64, name: &str) -> CoreResult<()> {
        let space_name = self.get_space_name_by_id(space_id)?;
        
        let mut storage = self.storage.lock();
        let result = storage.drop_edge_type(&space_name, name)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        if result {
            log::info!("删除边类型成功: {} from space {}", name, space_id);
            Ok(())
        } else {
            Err(CoreError::NotFound(format!("边类型 '{}' 不存在", name)))
        }
    }

    /// 创建索引
    ///
    /// # 参数
    /// - `space_id`: 空间 ID
    /// - `name`: 索引名称
    /// - `target`: 索引目标（标签或边类型）
    pub fn create_index(
        &self,
        space_id: u64,
        name: &str,
        target: IndexTarget,
    ) -> CoreResult<()> {
        let space_name = self.get_space_name_by_id(space_id)?;
        
        // 根据目标类型构建索引
        let (schema_name, fields, index_type) = match target {
            IndexTarget::Tag { name: tag_name, fields } => {
                // 获取标签信息以确定字段类型
                let storage = self.storage.lock();
                let tag_info = storage.get_tag(&space_name, &tag_name)
                    .map_err(|e| CoreError::StorageError(e.to_string()))?;
                
                let tag_info = tag_info.ok_or_else(|| 
                    CoreError::NotFound(format!("标签 '{}' 不存在", tag_name))
                )?;
                
                // 构建索引字段
                let index_fields = self.build_index_fields(&fields, &tag_info.properties)?;
                (tag_name, index_fields, IndexType::TagIndex)
            }
            IndexTarget::Edge { name: edge_name, fields } => {
                // 获取边类型信息以确定字段类型
                let storage = self.storage.lock();
                let edge_info = storage.get_edge_type(&space_name, &edge_name)
                    .map_err(|e| CoreError::StorageError(e.to_string()))?;
                
                let edge_info = edge_info.ok_or_else(|| 
                    CoreError::NotFound(format!("边类型 '{}' 不存在", edge_name))
                )?;
                
                // 构建索引字段
                let index_fields = self.build_index_fields(&fields, &edge_info.properties)?;
                (edge_name, index_fields, IndexType::EdgeIndex)
            }
        };
        
        // 根据索引类型调用对应的创建方法
        let mut storage = self.storage.lock();
        let result = match index_type {
            IndexType::TagIndex => {
                let index = Index {
                    id: 0, // 由存储层分配
                    name: name.to_string(),
                    space_id,
                    schema_name,
                    fields,
                    properties: Vec::new(),
                    index_type: IndexType::TagIndex,
                    status: IndexStatus::Active,
                    is_unique: false,
                    comment: None,
                };
                storage.create_tag_index(&space_name, &index)
            }
            IndexType::EdgeIndex => {
                let index = Index {
                    id: 0, // 由存储层分配
                    name: name.to_string(),
                    space_id,
                    schema_name,
                    fields,
                    properties: Vec::new(),
                    index_type: IndexType::EdgeIndex,
                    status: IndexStatus::Active,
                    is_unique: false,
                    comment: None,
                };
                storage.create_edge_index(&space_name, &index)
            }
        }.map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        if result {
            log::info!("创建索引成功: {} in space {:?}", name, space_id);
            Ok(())
        } else {
            Err(CoreError::SchemaOperationFailed(format!("索引 '{}' 创建失败", name)))
        }
    }

    /// 删除索引
    ///
    /// # 参数
    /// - `space_id`: 空间 ID
    /// - `name`: 索引名称
    pub fn drop_index(&self, space_id: u64, name: &str) -> CoreResult<()> {
        let space_name = self.get_space_name_by_id(space_id)?;
        
        let mut storage = self.storage.lock();
        
        // 尝试删除标签索引
        if let Ok(Some(_)) = storage.get_tag_index(&space_name, name) {
            let result = storage.drop_tag_index(&space_name, name)
                .map_err(|e| CoreError::StorageError(e.to_string()))?;
            if result {
                log::info!("删除标签索引成功: {} from space {}", name, space_id);
                return Ok(());
            }
        }
        
        // 尝试删除边索引
        if let Ok(Some(_)) = storage.get_edge_index(&space_name, name) {
            let result = storage.drop_edge_index(&space_name, name)
                .map_err(|e| CoreError::StorageError(e.to_string()))?;
            if result {
                log::info!("删除边索引成功: {} from space {}", name, space_id);
                return Ok(());
            }
        }
        
        Err(CoreError::NotFound(format!("索引 '{}' 不存在", name)))
    }

    /// 查看 Schema
    ///
    /// # 参数
    /// - `space_id`: 空间 ID
    ///
    /// # 返回
    /// Schema 描述字符串
    pub fn describe_schema(&self, space_id: u64) -> CoreResult<String> {
        let storage = self.storage.lock();
        
        // 获取空间信息
        let space_info = storage.get_space_by_id(space_id)
            .map_err(|e| CoreError::StorageError(e.to_string()))?
            .ok_or_else(|| CoreError::NotFound(format!("空间 ID {} 不存在", space_id)))?;
        
        let space_name = &space_info.space_name;
        
        // 获取所有标签
        let tags = storage.list_tags(space_name)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        // 获取所有边类型
        let edge_types = storage.list_edge_types(space_name)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        // 获取所有索引
        let tag_indexes = storage.list_tag_indexes(space_name)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        let edge_indexes = storage.list_edge_indexes(space_name)
            .map_err(|e| CoreError::StorageError(e.to_string()))?;
        
        // 构建描述字符串
        let mut description = format!("图空间: {} (ID: {})\n", space_name, space_id);
        description.push_str(&format!("VID 类型: {:?}\n", space_info.vid_type));
        if let Some(ref comment) = space_info.comment {
            description.push_str(&format!("注释: {}\n", comment));
        }
        description.push('\n');
        
        // 标签信息
        description.push_str("标签:\n");
        if tags.is_empty() {
            description.push_str("  (无)\n");
        } else {
            for tag in &tags {
                description.push_str(&format!("  - {}\n", tag.tag_name));
                for prop in &tag.properties {
                    description.push_str(&format!("      {}: {:?}{}\n", 
                        prop.name, 
                        prop.data_type,
                        if prop.nullable { " (nullable)" } else { "" }
                    ));
                }
            }
        }
        description.push('\n');
        
        // 边类型信息
        description.push_str("边类型:\n");
        if edge_types.is_empty() {
            description.push_str("  (无)\n");
        } else {
            for edge in &edge_types {
                description.push_str(&format!("  - {}\n", edge.edge_type_name));
                for prop in &edge.properties {
                    description.push_str(&format!("      {}: {:?}{}\n", 
                        prop.name, 
                        prop.data_type,
                        if prop.nullable { " (nullable)" } else { "" }
                    ));
                }
            }
        }
        description.push('\n');
        
        // 索引信息
        description.push_str("索引:\n");
        if tag_indexes.is_empty() && edge_indexes.is_empty() {
            description.push_str("  (无)\n");
        } else {
            for idx in &tag_indexes {
                description.push_str(&format!("  - {} (标签: {})\n", idx.name, idx.schema_name));
            }
            for idx in &edge_indexes {
                description.push_str(&format!("  - {} (边: {})\n", idx.name, idx.schema_name));
            }
        }
        
        log::info!("查看 Schema: space {}", space_id);
        Ok(description)
    }
}

// 内部辅助方法
impl<S: StorageClient> SchemaApi<S> {
    /// 根据空间 ID 获取空间名称
    fn get_space_name_by_id(&self, space_id: u64) -> CoreResult<String> {
        let storage = self.storage.lock();
        let space_info = storage.get_space_by_id(space_id)
            .map_err(|e| CoreError::StorageError(e.to_string()))?
            .ok_or_else(|| CoreError::NotFound(format!("空间 ID {} 不存在", space_id)))?;
        Ok(space_info.space_name)
    }
    
    /// 构建索引字段列表
    fn build_index_fields(
        &self,
        field_names: &[String],
        properties: &[crate::core::types::PropertyDef],
    ) -> CoreResult<Vec<IndexField>> {
        let mut fields = Vec::new();
        
        for field_name in field_names {
            let prop = properties.iter()
                .find(|p| &p.name == field_name)
                .ok_or_else(|| CoreError::InvalidParameter(
                    format!("字段 '{}' 不存在", field_name)
                ))?;
            
            // 创建对应的 Value 类型用于 IndexField
            let value_type = Self::datatype_to_value(&prop.data_type);
            
            fields.push(IndexField::new(
                field_name.clone(),
                value_type,
                prop.nullable,
            ));
        }
        
        Ok(fields)
    }
    
    /// 将 DataType 转换为 Value（用于索引字段类型）
    fn datatype_to_value(data_type: &crate::core::DataType) -> crate::core::Value {
        use crate::core::DataType;
        use crate::core::Value;
        use crate::core::value::NullType;
        use crate::core::value::date_time::{DateTimeValue, TimeValue, DateValue};
        
        match data_type {
            DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => {
                Value::Int(0)
            }
            DataType::Float | DataType::Double => {
                Value::Float(0.0)
            }
            DataType::String | DataType::FixedString(_) => {
                Value::String(String::new())
            }
            DataType::Bool => {
                Value::Bool(false)
            }
            DataType::Date => {
                Value::Date(DateValue { year: 1970, month: 1, day: 1 })
            }
            DataType::DateTime | DataType::Timestamp => {
                Value::DateTime(DateTimeValue { 
                    year: 1970, month: 1, day: 1, 
                    hour: 0, minute: 0, sec: 0, microsec: 0 
                })
            }
            DataType::Time => {
                Value::Time(TimeValue { hour: 0, minute: 0, sec: 0, microsec: 0 })
            }
            _ => Value::Null(NullType::Null),
        }
    }
}

impl<S: StorageClient> Clone for SchemaApi<S> {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
        }
    }
}

// 类型转换实现
impl From<PropertyDef> for crate::core::types::PropertyDef {
    fn from(prop: PropertyDef) -> Self {
        Self {
            name: prop.name,
            data_type: prop.data_type,
            nullable: prop.nullable,
            default: prop.default_value,
            comment: prop.comment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;

    fn create_mock_storage() -> Arc<Mutex<MockStorage>> {
        Arc::new(Mutex::new(MockStorage::new().expect("创建 MockStorage 失败")))
    }

    #[test]
    fn test_schema_api_new() {
        let storage = create_mock_storage();
        let _schema_api = SchemaApi::new(storage);
        assert!(true); // 创建成功
    }

    #[test]
    fn test_schema_api_clone() {
        let storage = create_mock_storage();
        let schema_api = SchemaApi::new(storage);
        let _cloned = schema_api.clone();
        assert!(true); // 克隆成功
    }

    #[test]
    fn test_property_def_conversion() {
        let api_prop = PropertyDef {
            name: "test".to_string(),
            data_type: crate::core::DataType::String,
            nullable: true,
            default_value: None,
            comment: Some("test comment".to_string()),
        };

        let core_prop: crate::core::types::PropertyDef = api_prop.into();
        assert_eq!(core_prop.name, "test");
        assert_eq!(core_prop.data_type, crate::core::DataType::String);
        assert!(core_prop.nullable);
        assert_eq!(core_prop.comment, Some("test comment".to_string()));
    }
}
