//! 存储层运行时上下文
//!
//! RuntimeContext用于存储层执行节点，包含计划上下文引用和运行时可变信息
//! 对应C++版本中的RuntimeContext结构

use crate::common::base::id::{EdgeType, TagId};
use crate::core::error::ManagerResult;
use crate::core::Value;
use std::sync::Arc;

use crate::query::context::managers::SchemaManager;

/// 结果状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultStatus {
    /// 正常结果
    Normal = 0,
    /// 非法数据
    IllegalData = -1,
    /// 被过滤掉的结果
    FilterOut = -2,
    /// 标签被过滤掉
    TagFilterOut = -3,
}

/// 属性上下文（简化版本）
#[derive(Debug, Clone)]
pub struct PropContext {
    pub name: String,
    pub prop_type: String,
    pub nullable: bool,
}

/// 计划上下文（存储层）
/// 存储处理过程中不变的信息
#[derive(Debug, Clone)]
pub struct PlanContext {
    /// 存储环境引用
    pub storage_env: Arc<StorageEnv>,
    /// 空间ID
    pub space_id: i32,
    /// 会话ID
    pub session_id: i64,
    /// 计划ID
    pub plan_id: i64,
    /// 顶点ID长度
    pub v_id_len: usize,
    /// 是否为整数ID
    pub is_int_id: bool,
    /// 是否为边查询
    pub is_edge: bool,
    /// 默认边版本
    pub default_edge_ver: i64,
    /// 是否被终止
    pub is_killed: bool,
}

/// 存储环境（简化版本）
#[derive(Debug, Clone)]
pub struct StorageEnv {
    /// 存储引擎
    pub storage_engine: Arc<dyn StorageEngine>,
    /// Schema管理器
    pub schema_manager: Arc<dyn SchemaManager>,
    /// 索引管理器
    pub index_manager: Arc<dyn IndexManager>,
}

/// 存储引擎trait
pub trait StorageEngine: Send + Sync + std::fmt::Debug {
    // 基本存储操作
    fn insert_node(&mut self, vertex: Vertex) -> Result<Value, StorageError>;
    fn get_node(&self, id: &Value) -> Result<Option<Vertex>, StorageError>;
    fn update_node(&mut self, vertex: Vertex) -> Result<(), StorageError>;
    fn delete_node(&mut self, id: &Value) -> Result<(), StorageError>;

    /// 全表扫描所有顶点
    fn scan_all_vertices(&self) -> Result<Vec<Vertex>, StorageError>;
    /// 按标签扫描顶点
    fn scan_vertices_by_tag(&self, tag: &str) -> Result<Vec<Vertex>, StorageError>;

    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError>;
    fn get_edge(
        &self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError>;
    fn get_node_edges(
        &self,
        node_id: &Value,
        direction: Direction,
    ) -> Result<Vec<Edge>, StorageError>;
    fn delete_edge(
        &mut self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError>;
}

/// 存储Schema管理器trait
pub trait StorageSchemaManager: Send + Sync + std::fmt::Debug {
    fn get_schema(&self, name: &str) -> Option<Schema>;
    fn get_all_schemas(&self) -> Vec<Schema>;
    fn add_schema(&mut self, name: String, schema: Schema);
    fn remove_schema(&mut self, name: &str) -> bool;
}

/// 索引管理器trait
pub trait IndexManager: Send + Sync + std::fmt::Debug {
    fn create_index(&mut self, name: String, schema: Schema) -> ManagerResult<()>;
    fn drop_index(&mut self, name: &str) -> ManagerResult<()>;
    fn get_index(&self, name: &str) -> Option<Index>;
}

/// 运行时上下文
/// 存储处理过程中可能变化的信息
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// 计划上下文引用
    pub plan_context: Arc<PlanContext>,

    /// 标签ID
    pub tag_id: TagId,
    /// 标签名称
    pub tag_name: String,
    /// 标签Schema（可选）
    pub tag_schema: Option<Arc<dyn SchemaManager>>,

    /// 边类型
    pub edge_type: EdgeType,
    /// 边名称
    pub edge_name: String,
    /// 边Schema（可选）
    pub edge_schema: Option<Arc<dyn SchemaManager>>,

    /// 列索引（用于GetNeighbors）
    pub column_idx: usize,
    /// 属性上下文列表（可选）
    pub props: Option<Vec<PropContext>>,

    /// 是否为插入操作
    pub insert: bool,
    /// 是否过滤无效结果
    pub filter_invalid_result_out: bool,
    /// 结果状态
    pub result_stat: ResultStatus,
}

impl RuntimeContext {
    /// 创建新的运行时上下文
    pub fn new(plan_context: Arc<PlanContext>) -> Self {
        Self {
            plan_context,
            tag_id: TagId::new(0),
            tag_name: String::new(),
            tag_schema: None,
            edge_type: EdgeType::new(0),
            edge_name: String::new(),
            edge_schema: None,
            column_idx: 0,
            props: None,
            insert: false,
            filter_invalid_result_out: false,
            result_stat: ResultStatus::Normal,
        }
    }

    /// 获取存储环境
    pub fn env(&self) -> &Arc<StorageEnv> {
        &self.plan_context.storage_env
    }

    /// 获取空间ID
    pub fn space_id(&self) -> i32 {
        self.plan_context.space_id
    }

    /// 获取顶点ID长度
    pub fn v_id_len(&self) -> usize {
        self.plan_context.v_id_len
    }

    /// 检查是否为整数ID
    pub fn is_int_id(&self) -> bool {
        self.plan_context.is_int_id
    }

    /// 检查是否为边查询
    pub fn is_edge(&self) -> bool {
        self.plan_context.is_edge
    }

    /// 检查计划是否被终止
    pub fn is_plan_killed(&self) -> bool {
        self.plan_context.is_killed
    }

    /// 设置标签信息
    pub fn set_tag_info(
        &mut self,
        tag_id: TagId,
        tag_name: String,
        tag_schema: Option<Arc<dyn SchemaManager>>,
    ) {
        self.tag_id = tag_id;
        self.tag_name = tag_name;
        self.tag_schema = tag_schema;
    }

    /// 设置边信息
    pub fn set_edge_info(
        &mut self,
        edge_type: EdgeType,
        edge_name: String,
        edge_schema: Option<Arc<dyn SchemaManager>>,
    ) {
        self.edge_type = edge_type;
        self.edge_name = edge_name;
        self.edge_schema = edge_schema;
    }

    /// 设置属性上下文
    pub fn set_props(&mut self, props: Vec<PropContext>) {
        self.props = Some(props);
    }

    /// 设置属性上下文（引用版本）
    pub fn set_props_ref(&mut self, props: &[PropContext]) {
        self.props = Some(props.to_vec());
    }

    /// 设置插入标志
    pub fn set_insert(&mut self, insert: bool) {
        self.insert = insert;
    }

    /// 设置过滤标志
    pub fn set_filter_invalid_result_out(&mut self, filter: bool) {
        self.filter_invalid_result_out = filter;
    }

    /// 设置结果状态
    pub fn set_result_stat(&mut self, stat: ResultStatus) {
        self.result_stat = stat;
    }

    /// 重置运行时状态（保留计划上下文）
    pub fn reset(&mut self) {
        self.tag_id = TagId::new(0);
        self.tag_name.clear();
        self.tag_schema = None;
        self.edge_type = EdgeType::new(0);
        self.edge_name.clear();
        self.edge_schema = None;
        self.column_idx = 0;
        self.props = None;
        self.insert = false;
        self.filter_invalid_result_out = false;
        self.result_stat = ResultStatus::Normal;
    }
}

// 类型别名和简化定义
pub type StorageError = String;
pub type IndexError = String;
pub type Schema = crate::core::schema::Schema;
pub type Index = crate::core::schema::Schema;
pub type Vertex = crate::core::vertex_edge_path::Vertex;
pub type Edge = crate::core::vertex_edge_path::Edge;
pub type Direction = crate::core::vertex_edge_path::Direction;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct MockStorageEngine;

    impl StorageEngine for MockStorageEngine {
        fn insert_node(&mut self, _vertex: Vertex) -> Result<Value, StorageError> {
            Ok(Value::Int(1))
        }

        fn get_node(&self, _id: &Value) -> Result<Option<Vertex>, StorageError> {
            Ok(None)
        }

        fn update_node(&mut self, _vertex: Vertex) -> Result<(), StorageError> {
            Ok(())
        }

        fn delete_node(&mut self, _id: &Value) -> Result<(), StorageError> {
            Ok(())
        }

        fn scan_all_vertices(&self) -> Result<Vec<Vertex>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_vertices_by_tag(&self, _tag: &str) -> Result<Vec<Vertex>, StorageError> {
            Ok(Vec::new())
        }

        fn insert_edge(&mut self, _edge: Edge) -> Result<(), StorageError> {
            Ok(())
        }

        fn get_edge(
            &self,
            _src: &Value,
            _dst: &Value,
            _edge_type: &str,
        ) -> Result<Option<Edge>, StorageError> {
            Ok(None)
        }

        fn get_node_edges(
            &self,
            _node_id: &Value,
            _direction: Direction,
        ) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn delete_edge(
            &mut self,
            _src: &Value,
            _dst: &Value,
            _edge_type: &str,
        ) -> Result<(), StorageError> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct MockSchemaManager {
        schemas: HashMap<String, crate::query::context::managers::Schema>,
    }

    impl MockSchemaManager {
        fn new() -> Self {
            Self {
                schemas: HashMap::new(),
            }
        }
    }

    impl SchemaManager for MockSchemaManager {
        fn get_schema(&self, name: &str) -> Option<crate::query::context::managers::Schema> {
            self.schemas.get(name).cloned()
        }

        fn list_schemas(&self) -> Vec<String> {
            self.schemas.keys().cloned().collect()
        }

        fn has_schema(&self, name: &str) -> bool {
            self.schemas.contains_key(name)
        }

        fn create_tag(
            &self,
            _space_id: i32,
            _tag_name: &str,
            _fields: Vec<crate::query::context::managers::FieldDef>,
        ) -> ManagerResult<i32> {
            Ok(1)
        }

        fn drop_tag(&self, _space_id: i32, _tag_id: i32) -> ManagerResult<()> {
            Ok(())
        }

        fn get_tag(
            &self,
            _space_id: i32,
            _tag_id: i32,
        ) -> Option<crate::query::context::managers::TagDef> {
            None
        }

        fn list_tags(
            &self,
            _space_id: i32,
        ) -> ManagerResult<Vec<crate::query::context::managers::TagDef>> {
            Ok(Vec::new())
        }

        fn has_tag(&self, _space_id: i32, _tag_id: i32) -> bool {
            false
        }

        fn create_edge_type(
            &self,
            _space_id: i32,
            _edge_type_name: &str,
            _fields: Vec<crate::query::context::managers::FieldDef>,
        ) -> ManagerResult<i32> {
            Ok(1)
        }

        fn drop_edge_type(&self, _space_id: i32, _edge_type_id: i32) -> ManagerResult<()> {
            Ok(())
        }

        fn get_edge_type(
            &self,
            _space_id: i32,
            _edge_type_id: i32,
        ) -> Option<crate::query::context::managers::EdgeTypeDef> {
            None
        }

        fn list_edge_types(
            &self,
            _space_id: i32,
        ) -> ManagerResult<Vec<crate::query::context::managers::EdgeTypeDef>> {
            Ok(Vec::new())
        }

        fn has_edge_type(&self, _space_id: i32, _edge_type_id: i32) -> bool {
            false
        }

        fn load_from_disk(&self) -> ManagerResult<()> {
            Ok(())
        }

        fn save_to_disk(&self) -> ManagerResult<()> {
            Ok(())
        }

        fn create_schema_version(
            &self,
            _space_id: i32,
            _comment: Option<String>,
        ) -> ManagerResult<i32> {
            Ok(1)
        }

        fn get_schema_version(
            &self,
            _space_id: i32,
            _version: i32,
        ) -> Option<crate::query::context::managers::SchemaVersion> {
            None
        }

        fn get_latest_schema_version(&self, _space_id: i32) -> Option<i32> {
            Some(1)
        }

        fn get_schema_history(
            &self,
            _space_id: i32,
        ) -> ManagerResult<crate::query::context::managers::SchemaHistory> {
            Ok(crate::query::context::managers::SchemaHistory {
                space_id: _space_id,
                versions: Vec::new(),
                current_version: 1,
            })
        }

        fn rollback_schema(&self, _space_id: i32, _version: i32) -> ManagerResult<()> {
            Ok(())
        }

        fn get_current_version(&self, _space_id: i32) -> Option<i32> {
            Some(1)
        }
    }

    #[derive(Debug)]
    struct MockIndexManager;

    impl IndexManager for MockIndexManager {
        fn create_index(&mut self, _name: String, _schema: Schema) -> ManagerResult<()> {
            Ok(())
        }

        fn drop_index(&mut self, _name: &str) -> ManagerResult<()> {
            Ok(())
        }

        fn get_index(&self, _name: &str) -> Option<Index> {
            None
        }
    }

    #[test]
    fn test_runtime_context_creation() {
        let storage_env = Arc::new(StorageEnv {
            storage_engine: Arc::new(MockStorageEngine),
            schema_manager: Arc::new(MockSchemaManager::new()),
            index_manager: Arc::new(MockIndexManager),
        });

        let plan_context = Arc::new(PlanContext {
            storage_env,
            space_id: 1,
            session_id: 100,
            plan_id: 200,
            v_id_len: 16,
            is_int_id: false,
            is_edge: false,
            default_edge_ver: 0,
            is_killed: false,
        });

        let runtime_ctx = RuntimeContext::new(plan_context);

        assert_eq!(runtime_ctx.space_id(), 1);
        assert_eq!(runtime_ctx.v_id_len(), 16);
        assert!(!runtime_ctx.is_int_id());
        assert!(!runtime_ctx.is_edge());
        assert!(!runtime_ctx.is_plan_killed());
        assert_eq!(runtime_ctx.result_stat, ResultStatus::Normal);
    }

    #[test]
    fn test_runtime_context_setters() {
        let storage_env = Arc::new(StorageEnv {
            storage_engine: Arc::new(MockStorageEngine),
            schema_manager: Arc::new(MockSchemaManager::new()),
            index_manager: Arc::new(MockIndexManager),
        });

        let plan_context = Arc::new(PlanContext {
            storage_env,
            space_id: 1,
            session_id: 100,
            plan_id: 200,
            v_id_len: 16,
            is_int_id: false,
            is_edge: false,
            default_edge_ver: 0,
            is_killed: false,
        });

        let mut runtime_ctx = RuntimeContext::new(plan_context);

        // 设置标签信息
        runtime_ctx.set_tag_info(TagId::new(1), "player".to_string(), None);
        assert_eq!(runtime_ctx.tag_id.as_i32(), 1);
        assert_eq!(runtime_ctx.tag_name, "player");

        // 设置边信息
        runtime_ctx.set_edge_info(EdgeType::new(2), "follow".to_string(), None);
        assert_eq!(runtime_ctx.edge_type.as_i32(), 2);
        assert_eq!(runtime_ctx.edge_name, "follow");

        // 设置属性
        let props = vec![PropContext {
            name: "name".to_string(),
            prop_type: "string".to_string(),
            nullable: false,
        }];
        runtime_ctx.set_props_ref(&props);
        assert_eq!(
            runtime_ctx
                .props
                .as_ref()
                .expect("Props should exist")
                .len(),
            1
        );

        // 设置标志
        runtime_ctx.set_insert(true);
        assert!(runtime_ctx.insert);

        runtime_ctx.set_filter_invalid_result_out(true);
        assert!(runtime_ctx.filter_invalid_result_out);

        runtime_ctx.set_result_stat(ResultStatus::FilterOut);
        assert_eq!(runtime_ctx.result_stat, ResultStatus::FilterOut);

        // 重置
        runtime_ctx.reset();
        assert_eq!(runtime_ctx.tag_id.as_i32(), 0);
        assert!(runtime_ctx.tag_name.is_empty());
        assert_eq!(runtime_ctx.result_stat, ResultStatus::Normal);
    }
}
