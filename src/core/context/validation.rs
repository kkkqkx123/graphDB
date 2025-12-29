//! 验证上下文模块
//!
//! 提供查询验证过程中的上下文管理，整合自query/context/validate/

use std::collections::HashMap;

use super::base::ContextType;
use super::traits::BaseContext;
use crate::core::Value;

/// 验证上下文
///
/// 管理查询验证过程中的上下文信息
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// 上下文ID
    pub id: String,

    /// 验证阶段
    pub phase: ValidationPhase,

    /// 验证选项
    pub options: ValidationOptions,

    /// 已验证的符号表
    pub validated_symbols: HashMap<String, ValidatedSymbol>,

    pub attributes: HashMap<String, Value>,

    /// 创建时间
    pub created_at: std::time::SystemTime,

    /// 最后更新时间
    pub updated_at: std::time::SystemTime,

    /// 是否有效
    pub valid: bool,
}

/// 验证阶段
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationPhase {
    /// 语法验证
    Syntax,
    /// 语义验证
    Semantic,
    /// 类型验证
    Type,
    /// 权限验证
    Permission,
    /// 优化验证
    Optimization,
    /// 完成
    Completed,
}

/// 验证选项
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationOptions {
    /// 是否启用严格模式
    pub strict_mode: bool,
    /// 是否跳过权限验证
    pub skip_permission_check: bool,
    /// 是否启用类型推断
    pub enable_type_inference: bool,
    /// 最大递归深度
    pub max_recursion_depth: usize,
    /// 超时时间（毫秒）
    pub timeout_ms: Option<u64>,
}

/// 已验证的符号
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedSymbol {
    /// 符号名称
    pub name: String,
    /// 符号类型
    pub symbol_type: ValidationSymbolType,
    /// 数据类型
    pub data_type: DataType,
    /// 定义位置
    pub definition_position: Option<Position>,
    /// 使用位置列表
    pub usage_positions: Vec<Position>,
    /// 是否为常量
    pub is_constant: bool,
    /// 是否为null
    pub is_nullable: bool,
}

/// 验证符号类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationSymbolType {
    /// 变量
    Variable,
    /// 函数
    Function,
    /// 标签
    Tag,
    /// 属性
    Property,
    /// 边类型
    EdgeType,
    /// 路径
    Path,
    /// 参数
    Parameter,
}

/// 数据类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataType {
    /// 未知类型
    Unknown,
    /// 布尔类型
    Boolean,
    /// 整数类型
    Integer,
    /// 浮点类型
    Float,
    /// 字符串类型
    String,
    /// 列表类型
    List(Box<DataType>),
    /// 映射类型
    Map(Box<DataType>, Box<DataType>),
    /// 顶点类型
    Vertex,
    /// 边类型
    Edge,
    /// 路径类型
    Path,
    /// 任意类型
    Any,
}

/// 位置信息
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// 行号
    pub line: usize,
    /// 列号
    pub column: usize,
    /// 偏移量
    pub offset: usize,
    /// 长度
    pub length: usize,
}

/// 验证警告类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationWarningType {
    /// 未使用的变量
    UnusedVariable,
    /// 隐式类型转换
    ImplicitTypeConversion,
    /// 性能警告
    PerformanceWarning,
    /// 废弃的语法
    DeprecatedSyntax,
    /// 潜在的空值
    PotentialNullValue,
}

impl ValidationContext {
    /// 创建新的验证上下文
    pub fn new(id: String) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            id,
            phase: ValidationPhase::Syntax,
            options: ValidationOptions::default(),
            validated_symbols: HashMap::new(),
            attributes: HashMap::new(),
            created_at: now,
            updated_at: now,
            valid: true,
        }
    }

    /// 进入下一个验证阶段
    pub fn next_phase(&mut self, phase: ValidationPhase) {
        self.phase = phase;
        self.updated_at = std::time::SystemTime::now();
    }

    /// 添加已验证的符号
    pub fn add_validated_symbol(&mut self, symbol: ValidatedSymbol) {
        self.validated_symbols.insert(symbol.name.clone(), symbol);
        self.updated_at = std::time::SystemTime::now();
    }

    /// 获取已验证的符号
    pub fn get_validated_symbol(&self, name: &str) -> Option<&ValidatedSymbol> {
        self.validated_symbols.get(name)
    }

    /// 检查符号是否已验证
    pub fn is_symbol_validated(&self, name: &str) -> bool {
        self.validated_symbols.contains_key(name)
    }
}

impl BaseContext for ValidationContext {
    fn id(&self) -> &str {
        &self.id
    }

    fn context_type(&self) -> ContextType {
        ContextType::Validation
    }

    fn created_at(&self) -> std::time::SystemTime {
        self.created_at
    }

    fn updated_at(&self) -> std::time::SystemTime {
        self.updated_at
    }

    fn is_valid(&self) -> bool {
        self.valid
    }

    fn touch(&mut self) {
        self.updated_at = std::time::SystemTime::now();
    }

    fn invalidate(&mut self) {
        self.valid = false;
        self.updated_at = std::time::SystemTime::now();
    }

    fn revalidate(&mut self) -> bool {
        self.valid = true;
        self.updated_at = std::time::SystemTime::now();
        self.valid
    }

    fn parent_id(&self) -> Option<&str> {
        None
    }

    fn depth(&self) -> usize {
        2
    }

    fn get_attribute(&self, key: &str) -> Option<Value> {
        self.attributes.get(key).cloned()
    }

    fn set_attribute(&mut self, key: String, value: Value) {
        self.attributes.insert(key, value);
        self.updated_at = std::time::SystemTime::now();
    }

    fn attribute_keys(&self) -> Vec<String> {
        self.attributes.keys().cloned().collect()
    }

    fn remove_attribute(&mut self, key: &str) -> Option<Value> {
        let removed = self.attributes.remove(key);
        self.updated_at = std::time::SystemTime::now();
        removed
    }

    fn clear_attributes(&mut self) {
        self.attributes.clear();
        self.updated_at = std::time::SystemTime::now();
    }
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            strict_mode: false,
            skip_permission_check: false,
            enable_type_inference: true,
            max_recursion_depth: 1000,
            timeout_ms: Some(30000), // 30秒
        }
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new("default_validation".to_string())
    }
}
