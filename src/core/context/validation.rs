//! 验证上下文模块
//!
//! 提供查询验证过程中的上下文管理，整合自query/context/validate/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::base::{ContextBase, ContextType, MutableContext};
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

    /// 验证错误列表
    pub errors: Vec<ValidationError>,

    /// 验证警告列表
    pub warnings: Vec<ValidationWarning>,

    /// 自定义属性
    pub attributes: HashMap<String, Value>,

    /// 创建时间
    pub created_at: std::time::SystemTime,

    /// 最后更新时间
    pub updated_at: std::time::SystemTime,

    /// 是否有效
    pub valid: bool,
}

/// 验证阶段
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatedSymbol {
    /// 符号名称
    pub name: String,
    /// 符号类型
    pub symbol_type: SymbolType,
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

/// 符号类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolType {
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// 验证错误
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 错误位置
    pub position: Option<Position>,
    /// 错误严重程度
    pub severity: ErrorSeverity,
    /// 错误类型
    pub error_type: ValidationErrorType,
}

/// 验证警告
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// 警告代码
    pub code: String,
    /// 警告消息
    pub message: String,
    /// 警告位置
    pub position: Option<Position>,
    /// 警告类型
    pub warning_type: ValidationWarningType,
}

/// 错误严重程度
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// 错误
    Error,
    /// 警告
    Warning,
    /// 信息
    Info,
}

/// 验证错误类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationErrorType {
    /// 语法错误
    SyntaxError,
    /// 语义错误
    SemanticError,
    /// 类型错误
    TypeError,
    /// 未定义符号
    UndefinedSymbol,
    /// 重复定义
    DuplicateDefinition,
    /// 权限错误
    PermissionError,
    /// 递归深度超限
    RecursionDepthExceeded,
    /// 超时
    Timeout,
}

/// 验证警告类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
            errors: Vec::new(),
            warnings: Vec::new(),
            attributes: HashMap::new(),
            created_at: now,
            updated_at: now,
            valid: true,
        }
    }

    /// 进入下一个验证阶段
    pub fn next_phase(&mut self, phase: ValidationPhase) {
        self.phase = phase;
        self.touch();
    }

    /// 添加已验证的符号
    pub fn add_validated_symbol(&mut self, symbol: ValidatedSymbol) {
        self.validated_symbols.insert(symbol.name.clone(), symbol);
        self.touch();
    }

    /// 获取已验证的符号
    pub fn get_validated_symbol(&self, name: &str) -> Option<&ValidatedSymbol> {
        self.validated_symbols.get(name)
    }

    /// 检查符号是否已验证
    pub fn is_symbol_validated(&self, name: &str) -> bool {
        self.validated_symbols.contains_key(name)
    }

    /// 添加验证错误
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.touch();
    }

    /// 添加验证警告
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
        self.touch();
    }

    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// 获取警告数量
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// 检查是否有警告
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// 检查验证是否通过
    pub fn is_validated(&self) -> bool {
        self.phase == ValidationPhase::Completed && !self.has_errors()
    }

    /// 清空错误和警告
    pub fn clear_diagnostics(&mut self) {
        self.errors.clear();
        self.warnings.clear();
        self.touch();
    }

    /// 获取指定类型的错误
    pub fn get_errors_by_type(&self, error_type: ValidationErrorType) -> Vec<&ValidationError> {
        self.errors
            .iter()
            .filter(|e| e.error_type == error_type)
            .collect()
    }

    /// 获取指定类型的警告
    pub fn get_warnings_by_type(
        &self,
        warning_type: ValidationWarningType,
    ) -> Vec<&ValidationWarning> {
        self.warnings
            .iter()
            .filter(|w| w.warning_type == warning_type)
            .collect()
    }
}

impl ContextBase for ValidationContext {
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
}

impl MutableContext for ValidationContext {
    fn touch(&mut self) {
        self.updated_at = std::time::SystemTime::now();
    }

    fn invalidate(&mut self) {
        self.valid = false;
        self.touch();
    }

    fn revalidate(&mut self) -> bool {
        // 重新验证逻辑
        self.valid = !self.has_errors();
        self.touch();
        self.valid
    }
}

impl super::base::AttributeSupport for ValidationContext {
    fn get_attribute(&self, key: &str) -> Option<Value> {
        self.attributes.get(key).cloned()
    }

    fn set_attribute(&mut self, key: String, value: Value) {
        self.attributes.insert(key, value);
        self.touch();
    }

    fn attribute_keys(&self) -> Vec<String> {
        self.attributes.keys().cloned().collect()
    }

    fn remove_attribute(&mut self, key: &str) -> Option<Value> {
        let removed = self.attributes.remove(key);
        self.touch();
        removed
    }

    fn clear_attributes(&mut self) {
        self.attributes.clear();
        self.touch();
    }
}

impl super::base::HierarchicalContext for ValidationContext {
    fn parent_id(&self) -> Option<&str> {
        None // 验证上下文通常是独立的
    }

    fn depth(&self) -> usize {
        2 // 验证上下文深度为2
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
