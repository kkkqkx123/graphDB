//! DeduceTypeVisitor - 用于推导表达式类型的访问器
//! 对应 NebulaGraph DeduceTypeVisitor.h/.cpp 的功能

use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState, GenericExpressionVisitor},
    Expression, TypeUtils, DataType, BinaryOperator, UnaryOperator, Value,
};
use crate::expression::Expr;
use crate::query::validator::ValidationContext;
use crate::storage::StorageEngine;
use thiserror::Error;

#[cfg(test)]
use crate::core::{Edge, Vertex};
#[cfg(test)]
use crate::core::EdgeDirection;
#[cfg(test)]
use crate::storage::StorageError;

#[derive(Error, Debug, Clone)]
pub enum TypeDeductionError {
    #[error("语义错误: {0}")]
    SemanticError(String),
    #[error("存储错误: {0}")]
    StorageError(String),
    #[error("类型不匹配: {0}")]
    TypeMismatch(String),
}

/// 类型推导访问器
/// 用于递归遍历表达式树，推导表达式的结果类型
pub struct DeduceTypeVisitor<'a, S: StorageEngine> {
    /// 存储引擎
    _storage: &'a S,
    /// 验证上下文
    validate_context: &'a ValidationContext,
    /// 输入列定义：列名 -> 列类型
    inputs: Vec<(String, DataType)>,
    /// 图空间ID
    _space: String,
    /// 当前推导状态
    status: Option<TypeDeductionError>,
    /// 推导出的类型
    type_: DataType,
    /// VID(顶点ID)类型
    vid_type: DataType,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl<'a, S: StorageEngine> DeduceTypeVisitor<'a, S> {
    pub fn new(
        storage: &'a S,
        validate_context: &'a ValidationContext,
        inputs: Vec<(String, DataType)>,
        space: String,
    ) -> Self {
        // VID类型通常从空间配置获取，这里简化为String
        let vid_type = DataType::String;

        Self {
            _storage: storage,
            validate_context,
            inputs,
            _space: space,
            status: None,
            type_: DataType::Empty,
            vid_type,
            state: ExpressionVisitorState::new(),
        }
}

    /// 创建用于测试的访问器（不需要存储和验证上下文）
    pub fn new_for_test(
        _inputs: Vec<(String, DataType)>,
        _space: String,
    ) -> (Self, ValidationContext) {
        let _vctx = ValidationContext::new();
        let _vid_type = DataType::String;

        // 返回值类型无法直接满足要求，这里需要特殊处理
        // 实现中应该使用默认存储引擎或Mock
        panic!("使用new_for_test需要实现Mock存储引擎");
    }

    /// 推导是否成功
    pub fn ok(&self) -> bool {
        self.status.is_none()
    }

    /// 获取当前状态
    pub fn status(&self) -> Option<&TypeDeductionError> {
        self.status.as_ref()
    }

    /// 获取推导出的类型
    pub fn type_(&self) -> DataType {
        self.type_.clone()
    }

    /// 设置VID类型
    pub fn set_vid_type(&mut self, vid_type: DataType) {
        self.vid_type = vid_type;
    }

    /// 主推导方法 - 推导表达式的类型
    pub fn deduce_type(&mut self, expr: &Expr) -> Result<DataType, TypeDeductionError> {
        self.visit_expression(expr)?;
        Ok(self.type_.clone())
    }

    /// 推导字面量表达式的类型
    fn deduce_literal_type(&mut self, value: &crate::core::Value) -> Result<(), TypeDeductionError> {
        self.type_ = match value {
            Value::Bool(_) => DataType::Bool,
            Value::Int(_) => DataType::Int,
            Value::Float(_) => DataType::Float,
            Value::String(_) => DataType::String,
            Value::Null(_) => DataType::Null,
            Value::Empty => DataType::Empty,
            Value::Date(_) => DataType::Date,
            Value::Time(_) => DataType::Time,
            Value::DateTime(_) => DataType::DateTime,
            Value::Vertex(_) => DataType::Vertex,
            Value::Edge(_) => DataType::Edge,
            Value::Path(_) => DataType::Path,
            Value::List(_) => DataType::List,
            Value::Map(_) => DataType::Map,
            Value::Set(_) => DataType::Set,
            Value::Geography(_) => DataType::Geography,
            Value::Duration(_) => DataType::Duration,
            Value::DataSet(_) => DataType::DataSet,
        };
        Ok(())
    }

    /// 推导二元操作符的类型
    fn deduce_binary_op_type(
        &mut self,
        op: &BinaryOperator,
        left_type: DataType,
        right_type: DataType,
    ) -> Result<(), TypeDeductionError> {
        match op {
            BinaryOperator::Add => {
                if left_type == DataType::String && right_type == DataType::String {
                    self.type_ = DataType::String;
                } else if left_type == DataType::Int && right_type == DataType::Int {
                    self.type_ = DataType::Int;
                } else if left_type == DataType::Float && right_type == DataType::Float {
                    self.type_ = DataType::Float;
                } else if (left_type == DataType::Int && right_type == DataType::Float)
                    || (left_type == DataType::Float && right_type == DataType::Int)
                {
                    self.type_ = DataType::Float;
                } else if self.is_superior_type(&left_type) || self.is_superior_type(&right_type) {
                    // NULL或EMPTY类型兼容任何类型
                    self.type_ = if self.is_superior_type(&left_type) {
                        right_type
                    } else {
                        left_type
                    };
                } else {
                    let msg = format!(
                        "无法对类型 {:?} 和 {:?} 执行加法操作",
                        left_type, right_type
                    );
                    self.status = Some(TypeDeductionError::SemanticError(msg.clone()));
                    return Err(TypeDeductionError::SemanticError(msg));
                }
            }
            BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo => {
                if left_type == DataType::Int && right_type == DataType::Int {
                    self.type_ = DataType::Int;
                } else if left_type == DataType::Float && right_type == DataType::Float {
                    self.type_ = DataType::Float;
                } else if (left_type == DataType::Int && right_type == DataType::Float)
                    || (left_type == DataType::Float && right_type == DataType::Int)
                {
                    self.type_ = DataType::Float;
                } else if self.is_superior_type(&left_type) || self.is_superior_type(&right_type) {
                    // NULL或EMPTY类型兼容任何类型
                    self.type_ = if self.is_superior_type(&left_type) {
                        right_type
                    } else {
                        left_type
                    };
                } else {
                    let op_name = match op {
                        BinaryOperator::Subtract => "减法",
                        BinaryOperator::Multiply => "乘法",
                        BinaryOperator::Divide => "除法",
                        BinaryOperator::Modulo => "模运算",
                        _ => "数学运算",
                    };
                    let msg = format!(
                        "无法对类型 {:?} 和 {:?} 执行{}操作",
                        left_type, right_type, op_name
                    );
                    self.status = Some(TypeDeductionError::SemanticError(msg.clone()));
                    return Err(TypeDeductionError::SemanticError(msg));
                }
            }
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual => {
                // 关系操作的结果类型是布尔值
                self.type_ = DataType::Bool;
            }
            BinaryOperator::And | BinaryOperator::Or => {
                // 逻辑操作的结果类型是布尔值
                self.type_ = DataType::Bool;
            }
            BinaryOperator::In => {
                // 集合操作的结果类型是布尔值
                self.type_ = DataType::Bool;
            }
            _ => {
                // 其他操作默认返回布尔值
                self.type_ = DataType::Bool;
            }
        }
        Ok(())
    }

    /// 推导一元操作符的类型
    fn deduce_unary_op_type(&mut self, op: &UnaryOperator) -> Result<(), TypeDeductionError> {
        match op {
            UnaryOperator::Plus | UnaryOperator::Minus => {
                // 正负号操作保持原类型
                // 类型已在visit_expression中推导
            }
            UnaryOperator::Not => {
                // 逻辑非操作的结果类型是布尔值
                self.type_ = DataType::Bool;
            }
            _ => {
                // 其他操作保持原类型
            }
        }
        Ok(())
    }

    /// 推导属性表达式的类型

    fn deduce_property_type(&mut self, _property: &str) -> Result<(), TypeDeductionError> {
        // 属性访问的结果类型需要根据上下文来确定
        // 简化实现，返回Empty类型
        self.type_ = DataType::Empty;
        Ok(())
    }

    /// 推导函数调用表达式的类型
    fn deduce_function_call_type(
        &mut self,
        name: &str,
        args: &[Expression],
    ) -> Result<(), TypeDeductionError> {
        // 推导参数类型
        let mut _arg_types = Vec::new();
        for arg in args {
            self.visit_expression(arg)?;
            _arg_types.push(self.type_.clone());
        }

        // 根据函数名确定返回类型
        let name_upper = name.to_uppercase();
        self.type_ = match name_upper.as_str() {
            // ID提取函数
            "ID" | "SRC" | "DST" | "NONE_DIRECT_SRC" | "NONE_DIRECT_DST" => self.vid_type.clone(),
            // 聚合函数
            "COUNT" => DataType::Int,
            "AVG" | "SUM" => DataType::Float,
            "MAX" | "MIN" => {
                if _arg_types.is_empty() {
                    DataType::Empty
                } else {
                    _arg_types[0].clone()
                }
            }
            "COLLECT" => DataType::List,
            "COLLECT_SET" => DataType::Set,
            // 字符串函数
            "LOWER" | "UPPER" | "TRIM" | "LTRIM" | "RTRIM" | "SUBSTR" | "REVERSE" => {
                DataType::String
            }
            // 数学函数
            "ABS" | "CEIL" | "FLOOR" | "SQRT" | "POW" | "EXP" | "LOG" | "LOG10" => {
                DataType::Float
            }
            // 其他函数默认返回Empty
            _ => DataType::Empty,
        };
        Ok(())
    }

    /// 推导聚合表达式的类型
    fn deduce_aggregate_func_type(
        &mut self,
        func: &crate::core::AggregateFunction,
    ) -> Result<(), TypeDeductionError> {
        use crate::core::AggregateFunction;
        self.type_ = match func {
            AggregateFunction::Count(_) => DataType::Int,
            AggregateFunction::Sum(_) => DataType::Float,
            AggregateFunction::Avg(_) => DataType::Float,
            AggregateFunction::Min(_) | AggregateFunction::Max(_) => {
                // 保持参数类型，已在visit中处理
                self.type_.clone()
            }
            AggregateFunction::Collect(_) => DataType::List,
            AggregateFunction::Distinct(_) => DataType::List,
            AggregateFunction::Percentile(_, _) => DataType::Float,
        };
        Ok(())
    }

    fn visit_property(&mut self, object: &Expr, property: &str) -> Result<(), TypeDeductionError> {
        // 推导属性访问表达式的类型
        // 先推导对象类型，再获取属性类型
        self.visit_expression(object)?;
        
        // 在实际实现中，这里会根据对象的schema来确定属性类型
        // 简化实现，返回Empty类型
        self.type_ = DataType::Empty;
        Ok(())
    }

    fn visit_variable(&mut self, _name: &str) -> Result<(), TypeDeductionError> {
        // 变量表达式的结果类型不确定，使用Empty
        self.type_ = DataType::Empty;
        Ok(())
    }

    /// 检查两种类型是否兼容
    fn are_types_compatible(&self, type1: &DataType, type2: &DataType) -> bool {
        TypeUtils::are_types_compatible(type1, type2)
    }

    /// 检查类型是否为"优越类型"
    /// 优越类型包括NULL和EMPTY，它们可以与任何类型兼容
    fn is_superior_type(&self, type_: &DataType) -> bool {
        TypeUtils::is_superior_type(type_)
    }

    /// 将字符串解析为DataType

    fn parse_type_def(&self, type_str: &str) -> DataType {
        match type_str.to_uppercase().as_str() {
            "INT" => DataType::Int,
            "FLOAT" | "DOUBLE" => DataType::Float,
            "STRING" => DataType::String,
            "BOOL" => DataType::Bool,
            "DATE" => DataType::Date,
            "TIME" => DataType::Time,
            "DATETIME" => DataType::DateTime,
            "VERTEX" => DataType::Vertex,
            "EDGE" => DataType::Edge,
            "PATH" => DataType::Path,
            "LIST" => DataType::List,
            "SET" => DataType::Set,
            "MAP" => DataType::Map,
            "NULL" => DataType::Null,
            _ => DataType::Empty,
        }
    }

    /// 将DataType解析为DataType
    fn parse_data_type(&self, data_type: &crate::core::DataType) -> DataType {
        use crate::core::DataType;
        match data_type {
            DataType::Empty => DataType::Empty,
            DataType::Null => DataType::Null,
            DataType::Bool => DataType::Bool,
            DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => DataType::Int,
            DataType::Float | DataType::Double => DataType::Float,
            DataType::String => DataType::String,
            DataType::Date => DataType::Date,
            DataType::Time => DataType::Time,
            DataType::DateTime => DataType::DateTime,
            DataType::Vertex => DataType::Vertex,
            DataType::Edge => DataType::Edge,
            DataType::Path => DataType::Path,
            DataType::List => DataType::List,
            DataType::Map => DataType::Map,
            DataType::Set => DataType::Set,
            DataType::Geography => DataType::Geography,
            DataType::Duration => DataType::Duration,
            DataType::DataSet => DataType::DataSet,
        }
    }
}

impl<'a, S: StorageEngine> std::fmt::Debug for DeduceTypeVisitor<'a, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeduceTypeVisitor")
            .field("status", &self.status)
            .field("type_", &self.type_)
            .field("vid_type", &self.vid_type)
            .finish()
    }
}

impl<'a, S: StorageEngine> ExpressionVisitor for DeduceTypeVisitor<'a, S> {
    type Result = Result<(), TypeDeductionError>;

    fn visit_literal(&mut self, value: &Value) -> Self::Result {
        self.type_ = match value {
            Value::Bool(_) => DataType::Bool,
            Value::Int(_) => DataType::Int,
            Value::Float(_) => DataType::Float,
            Value::String(_) => DataType::String,
            Value::Null(_) => DataType::Null,
            Value::Empty => DataType::Empty,
            Value::Date(_) => DataType::Date,
            Value::Time(_) => DataType::Time,
            Value::DateTime(_) => DataType::DateTime,
            Value::Vertex(_) => DataType::Vertex,
            Value::Edge(_) => DataType::Edge,
            Value::Path(_) => DataType::Path,
            Value::List(_) => DataType::List,
            Value::Map(_) => DataType::Map,
            Value::Set(_) => DataType::Set,
            Value::Geography(_) => DataType::Geography,
            Value::Duration(_) => DataType::Duration,
            Value::DataSet(_) => DataType::DataSet,
        };
        Ok(())
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        // 变量表达式的结果类型不确定，使用Empty
        self.type_ = DataType::Empty;
        Ok(())
    }

    fn visit_property(&mut self, object: &Expr, _property: &str) -> Self::Result {
        self.visit_expression(object)?;
        self.type_ = DataType::Empty;
        Ok(())
    }

    fn visit_binary(
        &mut self,
        left: &Expr,
        op: &BinaryOperator,
        right: &Expr,
    ) -> Self::Result {
        self.visit_expression(left)?;
        let left_type = self.type_.clone();
        self.visit_expression(right)?;
        let right_type = self.type_.clone();
        self.deduce_binary_op_type(op, left_type, right_type)
    }

    fn visit_unary(&mut self, op: &UnaryOperator, operand: &Expr) -> Self::Result {
        self.visit_expression(operand)?;
        self.deduce_unary_op_type(op)
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        self.deduce_function_call_type(name, args)
    }

    fn visit_aggregate(
        &mut self,
        func: &crate::core::AggregateFunction,
        arg: &Expr,
        _distinct: bool,
    ) -> Self::Result {
        self.visit_expression(arg)?;
        self.deduce_aggregate_func_type(func)
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        self.visit_list(items)
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_key, value) in pairs {
            self.visit_expression(value)?;
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expr, Expr)],
        default: &Option<Box<Expr>>,
    ) -> Self::Result {
        let mut result_type: Option<DataType> = None;

        for (condition_expr, then_expr) in conditions {
            self.visit_expression(condition_expr)?;
            self.visit_expression(then_expr)?;
            let then_type = self.type_.clone();

            if let Some(ref existing_type) = result_type {
                if !self.are_types_compatible(existing_type, &then_type) {
                    let msg = format!(
                        "CASE表达式分支类型不一致: {:?} vs {:?}",
                        existing_type, then_type
                    );
                    self.status = Some(TypeDeductionError::TypeMismatch(msg.clone()));
                    return Err(TypeDeductionError::TypeMismatch(msg));
                }
            } else {
                result_type = Some(then_type);
            }
        }

        if let Some(default_expr) = default {
            self.visit_expression(default_expr)?;
            let default_type = self.type_.clone();
            if let Some(ref existing_type) = result_type {
                if !self.are_types_compatible(existing_type, &default_type) {
                    let msg = format!(
                        "CASE表达式DEFAULT分支类型不一致: {:?} vs {:?}",
                        existing_type, default_type
                    );
                    self.status = Some(TypeDeductionError::TypeMismatch(msg.clone()));
                    return Err(TypeDeductionError::TypeMismatch(msg));
                }
            } else {
                result_type = Some(default_type);
            }
        }

        if let Some(result_type) = result_type {
            self.type_ = result_type;
        }
        Ok(())
    }

    fn visit_type_cast(&mut self, expr: &Expr, target_type: &DataType) -> Self::Result {
        self.visit_expression(expr)?;
        self.type_ = self.parse_data_type(target_type);
        Ok(())
    }

    fn visit_subscript(&mut self, collection: &Expr, index: &Expr) -> Self::Result {
        self.visit_expression(collection)?;
        let container_type = self.type_.clone();
        self.visit_expression(index)?;
        self.type_ = match container_type {
            DataType::List => DataType::Empty,
            DataType::Map => DataType::Empty,
            _ => DataType::Empty,
        };
        Ok(())
    }

    fn visit_range(
        &mut self,
        collection: &Expr,
        start: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
    ) -> Self::Result {
        self.visit_expression(collection)?;
        if let Some(start_expr) = start {
            self.visit_expression(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit_expression(end_expr)?;
        }
        self.type_ = DataType::List;
        Ok(())
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        self.type_ = DataType::Path;
        Ok(())
    }

    fn visit_label(&mut self, _name: &str) -> Self::Result {
        self.type_ = DataType::String;
        Ok(())
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock 存储引擎用于测试
    #[derive(Debug)]
    struct MockStorageEngine;

    impl StorageEngine for MockStorageEngine {
        fn insert_node(&mut self, _vertex: Vertex) -> Result<Value, StorageError> {
            Ok(Value::Int(0))
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
        
        fn scan_all_edges(&self) -> Result<Vec<Edge>, StorageError> {
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
            _direction: EdgeDirection,
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

        fn begin_transaction(&mut self) -> Result<u64, StorageError> {
            Ok(1)
        }

        fn commit_transaction(&mut self, _tx_id: u64) -> Result<(), StorageError> {
            Ok(())
        }

        fn rollback_transaction(&mut self, _tx_id: u64) -> Result<(), StorageError> {
            Ok(())
        }

        fn scan_edges_by_type(&self, _edge_type: &str) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_vertices_by_prop(&self, _tag: &str, _prop: &str, _value: &Value) -> Result<Vec<Vertex>, StorageError> {
            Ok(Vec::new())
        }

        fn get_node_edges_filtered(
            &self,
            _node_id: &Value,
            _direction: EdgeDirection,
            _filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
        ) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn batch_insert_nodes(&mut self, _vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
            Ok(Vec::new())
        }

        fn batch_insert_edges(&mut self, _edges: Vec<Edge>) -> Result<(), StorageError> {
            Ok(())
        }
    }

    #[test]
    fn test_is_superior_type() {
        let validate_context = ValidationContext::new();
        let visitor = DeduceTypeVisitor::new(
            &MockStorageEngine,
            &validate_context,
            vec![],
            "test_space".to_string(),
        );

        assert!(visitor.is_superior_type(&DataType::Null));
        assert!(visitor.is_superior_type(&DataType::Empty));
        assert!(!visitor.is_superior_type(&DataType::Int));
        assert!(!visitor.is_superior_type(&DataType::String));
    }

    #[test]
    fn test_are_types_compatible() {
        let validate_context = ValidationContext::new();
        let visitor = DeduceTypeVisitor::new(
            &MockStorageEngine,
            &validate_context,
            vec![],
            "test_space".to_string(),
        );

        // 相同类型兼容
        assert!(visitor.are_types_compatible(&DataType::Int, &DataType::Int));

        // 优越类型与任何类型兼容
        assert!(visitor.are_types_compatible(&DataType::Null, &DataType::Int));
        assert!(visitor.are_types_compatible(&DataType::Empty, &DataType::String));

        // Int和Float兼容
        assert!(visitor.are_types_compatible(&DataType::Int, &DataType::Float));
        assert!(visitor.are_types_compatible(&DataType::Float, &DataType::Int));

        // 不同类型不兼容
        assert!(!visitor.are_types_compatible(&DataType::Int, &DataType::String));
    }

    #[test]
    fn test_type_utils() {
        // 测试统一的类型工具
        assert!(TypeUtils::are_types_compatible(
            &DataType::Int,
            &DataType::Int
        ));
        assert!(TypeUtils::are_types_compatible(
            &DataType::Null,
            &DataType::String
        ));
        assert!(TypeUtils::is_superior_type(&DataType::Null));

        // 测试类型优先级
        assert_eq!(TypeUtils::get_type_priority(&DataType::Int), 2);
        assert_eq!(TypeUtils::get_type_priority(&DataType::Float), 3);
        assert_eq!(TypeUtils::get_type_priority(&DataType::String), 4);

        // 测试公共类型
        assert_eq!(
            TypeUtils::get_common_type(&DataType::Int, &DataType::Float),
            DataType::Float
        );
        assert_eq!(
            TypeUtils::get_common_type(&DataType::Null, &DataType::String),
            DataType::String
        );
    }
}

/// DeduceTypeVisitor构建器
///
/// 提供链式API构建DeduceTypeVisitor实例
pub struct DeduceTypeVisitorBuilder<'a, S: StorageEngine> {
    storage: Option<&'a S>,
    validate_context: Option<&'a ValidationContext>,
    inputs: Vec<(String, DataType)>,
    space: Option<String>,
    vid_type: DataType,
}

impl<'a, S: StorageEngine> DeduceTypeVisitorBuilder<'a, S> {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            storage: None,
            validate_context: None,
            inputs: Vec::new(),
            space: None,
            vid_type: DataType::String,
        }
    }

    /// 设置存储引擎
    pub fn with_storage(mut self, storage: &'a S) -> Self {
        self.storage = Some(storage);
        self
    }

    /// 设置验证上下文
    pub fn with_validate_context(mut self, validate_context: &'a ValidationContext) -> Self {
        self.validate_context = Some(validate_context);
        self
    }

    /// 设置输入列定义
    pub fn with_inputs(mut self, inputs: Vec<(String, DataType)>) -> Self {
        self.inputs = inputs;
        self
    }

    /// 添加输入列定义
    pub fn add_input(mut self, name: String, type_: DataType) -> Self {
        self.inputs.push((name, type_));
        self
    }

    /// 设置图空间
    pub fn with_space(mut self, space: String) -> Self {
        self.space = Some(space);
        self
    }

    /// 设置VID类型
    pub fn with_vid_type(mut self, vid_type: DataType) -> Self {
        self.vid_type = vid_type;
        self
    }

    /// 构建DeduceTypeVisitor实例
    pub fn build(self) -> DeduceTypeVisitor<'a, S> {
        let storage = self.storage.expect("存储引擎必须设置");
        let validate_context = self.validate_context.expect("验证上下文必须设置");
        let space = self.space.unwrap_or_else(|| "default".to_string());

        let mut visitor = DeduceTypeVisitor::new(storage, validate_context, self.inputs, space);
        visitor.set_vid_type(self.vid_type);
        visitor
    }
}

impl<'a, S: StorageEngine> Default for DeduceTypeVisitorBuilder<'a, S> {
    fn default() -> Self {
        Self::new()
    }
}

/// 为DeduceTypeVisitor实现GenericExpressionVisitor<Expression>
/// 提供统一的泛型访问接口
impl<'a, S: StorageEngine> GenericExpressionVisitor<Expression> for DeduceTypeVisitor<'a, S> {
    type Result = Result<(), TypeDeductionError>;

    fn visit(&mut self, expr: &Expr) -> Self::Result {
        self.visit_expression(expr)
    }
}
