//! DeduceTypeVisitor - 用于推导表达式类型的访问器

use crate::core::types::expression::Expression;
use crate::core::types::expression::visitor::{ExpressionVisitor, ExpressionVisitorState, GenericExpressionVisitor};
use crate::core::types::metadata::PropertyDef;
use crate::core::{
    TypeUtils, DataType, BinaryOperator, UnaryOperator, Value,
};
use crate::query::context::validate::{ColsDef, ValidationContext};
use crate::storage::StorageClient;
use thiserror::Error;

#[cfg(test)]
use crate::core::{Edge, Vertex};
#[cfg(test)]
use crate::core::EdgeDirection;
#[cfg(test)]
use crate::core::error::StorageError;


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
pub struct DeduceTypeVisitor<'a, S: StorageClient> {
    /// 存储引擎
    storage: &'a S,
    /// 验证上下文
    validate_context: &'a ValidationContext,
    /// 输入列定义：列名 -> 列类型
    inputs: ColsDef,
    /// 图空间ID
    space: String,
    /// 当前推导状态
    status: Option<TypeDeductionError>,
    /// 推导出的类型
    type_: DataType,
    /// VID(顶点ID)类型
    vid_type: DataType,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl<'a, S: StorageClient> DeduceTypeVisitor<'a, S> {
    pub fn new(
        storage: &'a S,
        validate_context: &'a ValidationContext,
        inputs: ColsDef,
        space: String,
    ) -> Self {
        let vid_type = if validate_context.space_chosen() {
            validate_context.which_space().vid_type.clone()
        } else {
            DataType::Empty
        };

        Self {
            storage,
            validate_context,
            inputs,
            space,
            status: None,
            type_: DataType::Empty,
            vid_type,
            state: ExpressionVisitorState::new(),
        }
    }

    fn find_tag_type_from_inputs(&self, _object: &Expression) -> Option<String> {
        None
    }

    fn find_edge_type_from_inputs(&self, _object: &Expression) -> Option<String> {
        None
    }

    fn get_property_type_from_tag(&self, tag_name: &str, property: &str) -> DataType {
        if let Ok(Some(tag_info)) = self.storage.get_tag(&self.space, tag_name) {
            if let Some(prop_type) = Self::find_property_type(&tag_info.properties, property) {
                return prop_type;
            }
        }
        DataType::Empty
    }

    fn get_property_type_from_edge(&self, edge_name: &str, property: &str) -> DataType {
        if let Ok(Some(edge_info)) = self.storage.get_edge_type(&self.space, edge_name) {
            if let Some(prop_type) = Self::find_property_type(&edge_info.properties, property) {
                return prop_type;
            }
        }
        DataType::Empty
    }

    fn find_property_type(properties: &[PropertyDef], prop_name: &str) -> Option<DataType> {
        for prop in properties {
            if prop.name == prop_name {
                return Some(prop.data_type.clone());
            }
        }
        None
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
    pub fn deduce_type(&mut self, expression: &Expression) -> Result<DataType, TypeDeductionError> {
        self.visit_expression(expression)?;
        Ok(self.type_.clone())
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
                let left_is_numeric = matches!(left_type, DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 | DataType::Float | DataType::Double);
                let right_is_numeric = matches!(right_type, DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 | DataType::Float | DataType::Double);
                
                if left_is_numeric && right_is_numeric {
                    let left_is_float = matches!(left_type, DataType::Float | DataType::Double);
                    let right_is_float = matches!(right_type, DataType::Float | DataType::Double);
                    self.type_ = if left_is_float || right_is_float {
                        DataType::Float
                    } else {
                        DataType::Int
                    };
                } else if self.is_superior_type(&left_type) || self.is_superior_type(&right_type) {
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
            | BinaryOperator::Modulo
            | BinaryOperator::Exponent => {
                let left_is_numeric = matches!(left_type, DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 | DataType::Float | DataType::Double);
                let right_is_numeric = matches!(right_type, DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 | DataType::Float | DataType::Double);
                
                if left_is_numeric && right_is_numeric {
                    let left_is_float = matches!(left_type, DataType::Float | DataType::Double);
                    let right_is_float = matches!(right_type, DataType::Float | DataType::Double);
                    self.type_ = if left_is_float || right_is_float {
                        DataType::Float
                    } else {
                        DataType::Int
                    };
                } else if self.is_superior_type(&left_type) || self.is_superior_type(&right_type) {
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
                        BinaryOperator::Exponent => "幂运算",
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
                self.type_ = DataType::Bool;
            }
            BinaryOperator::And | BinaryOperator::Or => {
                self.type_ = DataType::Bool;
            }
            BinaryOperator::In | BinaryOperator::NotIn => {
                self.type_ = DataType::Bool;
            }
            BinaryOperator::StringConcat => {
                self.type_ = DataType::String;
            }
            BinaryOperator::Like => {
                self.type_ = DataType::Bool;
            }
            BinaryOperator::Contains => {
                self.type_ = DataType::Bool;
            }
            BinaryOperator::StartsWith => {
                self.type_ = DataType::Bool;
            }
            BinaryOperator::EndsWith => {
                self.type_ = DataType::Bool;
            }
            _ => {
                self.type_ = DataType::Bool;
            }
        }
        Ok(())
    }

    /// 推导一元操作符的类型
    fn deduce_unary_op_type(&mut self, op: &UnaryOperator) -> Result<(), TypeDeductionError> {
        match op {
            UnaryOperator::Plus | UnaryOperator::Minus => {
                let is_numeric = matches!(self.type_, 
                    DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 
                    | DataType::Float | DataType::Double
                );
                if !is_numeric && !self.is_superior_type(&self.type_) {
                    let msg = format!(
                        "无法对类型 {:?} 执行一元{}操作",
                        self.type_,
                        if *op == UnaryOperator::Plus { "正号" } else { "负号" }
                    );
                    self.status = Some(TypeDeductionError::SemanticError(msg.clone()));
                    return Err(TypeDeductionError::SemanticError(msg));
                }
            }
            UnaryOperator::Not => {
                if self.type_ != DataType::Bool && !self.is_superior_type(&self.type_) {
                    let msg = format!(
                        "无法对类型 {:?} 执行逻辑非操作",
                        self.type_
                    );
                    self.status = Some(TypeDeductionError::SemanticError(msg.clone()));
                    return Err(TypeDeductionError::SemanticError(msg));
                }
                self.type_ = DataType::Bool;
            }
            UnaryOperator::IsNull
            | UnaryOperator::IsNotNull
            | UnaryOperator::IsEmpty
            | UnaryOperator::IsNotEmpty => {
                self.type_ = DataType::Bool;
            }
        }
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
            "ABS" | "CEIL" | "FLOOR" => DataType::Int,
            "SQRT" | "POW" | "EXP" | "LOG" | "LOG10" => {
                DataType::Float
            }
            // 列表操作函数
            "HEAD" | "LAST" | "TAIL" => {
                if _arg_types.is_empty() {
                    DataType::Empty
                } else if _arg_types[0] == DataType::List {
                    DataType::Empty
                } else {
                    DataType::Empty
                }
            }
            "KEYS" => DataType::List,
            "VALUES" => DataType::List,
            "PROPERTIES" => DataType::Map,
            "LABELS" => DataType::List,
            "TYPE" => DataType::String,
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
            AggregateFunction::CollectSet(_) => DataType::Set,
            AggregateFunction::Distinct(_) => DataType::Set,
            AggregateFunction::Percentile(_, _) => DataType::Float,
            AggregateFunction::Std(_) => DataType::Float,
            AggregateFunction::BitAnd(_) | AggregateFunction::BitOr(_) => DataType::Int,
            AggregateFunction::GroupConcat(_, _) => DataType::String,
        };
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
            DataType::FixedString(_) => DataType::String,
            DataType::Date => DataType::Date,
            DataType::Time => DataType::Time,
            DataType::Timestamp => DataType::Time,
            DataType::DateTime => DataType::DateTime,
            DataType::VID => DataType::String,
            DataType::Vertex => DataType::Vertex,
            DataType::Edge => DataType::Edge,
            DataType::Path => DataType::Path,
            DataType::List => DataType::List,
            DataType::Map => DataType::Map,
            DataType::Set => DataType::Set,
            DataType::Blob => DataType::String,
            DataType::Geography => DataType::Geography,
            DataType::Duration => DataType::Duration,
            DataType::DataSet => DataType::DataSet,
        }
    }
}

impl<'a, S: StorageClient> std::fmt::Debug for DeduceTypeVisitor<'a, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeduceTypeVisitor")
            .field("status", &self.status)
            .field("type_", &self.type_)
            .field("vid_type", &self.vid_type)
            .finish()
    }
}

impl<'a, S: StorageClient> ExpressionVisitor for DeduceTypeVisitor<'a, S> {
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

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        if self.inputs.iter().any(|col| col.name == name) {
            if let Some(col) = self.inputs.iter().find(|col| col.name == name) {
                self.type_ = col.type_.clone();
                return Ok(());
            }
        }

        if self.validate_context.exists_var(name) {
            let var_cols = self.validate_context.get_var(name);
            if !var_cols.is_empty() {
                self.type_ = var_cols[0].type_.clone();
                return Ok(());
            }
        }

        self.type_ = DataType::Empty;
        Ok(())
    }

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result {
        self.visit_expression(object)?;

        match &self.type_ {
            DataType::Vertex => {
                if let Some(tag_type) = self.find_tag_type_from_inputs(object) {
                    self.type_ = self.get_property_type_from_tag(&tag_type, property);
                } else {
                    self.type_ = DataType::Empty;
                }
            }
            DataType::Edge => {
                if let Some(edge_type) = self.find_edge_type_from_inputs(object) {
                    self.type_ = self.get_property_type_from_edge(&edge_type, property);
                } else {
                    self.type_ = DataType::Empty;
                }
            }
            DataType::Map => {
                self.type_ = DataType::Empty;
            }
            _ => {
                self.type_ = DataType::Empty;
            }
        }
        Ok(())
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        self.visit_expression(left)?;
        let left_type = self.type_.clone();
        self.visit_expression(right)?;
        let right_type = self.type_.clone();
        self.deduce_binary_op_type(op, left_type, right_type)
    }

    fn visit_unary(&mut self, op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit_expression(operand)?;
        self.deduce_unary_op_type(op)
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        self.deduce_function_call_type(name, args)
    }

    fn visit_aggregate(
        &mut self,
        func: &crate::core::AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        self.visit_expression(arg)?;
        self.deduce_aggregate_func_type(func)
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        if items.is_empty() {
            self.type_ = DataType::List;
            return Ok(());
        }

        let mut item_types: Vec<DataType> = Vec::new();
        for item in items {
            self.visit_expression(item)?;
            item_types.push(self.type_.clone());
        }

        self.type_ = DataType::List;
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_key, value) in pairs {
            self.visit_expression(value)?;
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) -> Self::Result {
        let mut result_type: Option<DataType> = None;

        if let Some(expr) = test_expr {
            self.visit_expression(expr)?;
        }

        for (condition_expression, then_expression) in conditions {
            self.visit_expression(condition_expression)?;
            self.visit_expression(then_expression)?;
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

        if let Some(default_expression) = default {
            self.visit_expression(default_expression)?;
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

    fn visit_type_cast(&mut self, expression: &Expression, target_type: &DataType) -> Self::Result {
        self.visit_expression(expression)?;
        self.type_ = self.parse_data_type(target_type);
        Ok(())
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
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
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) -> Self::Result {
        self.visit_expression(collection)?;
        if let Some(start_expression) = start {
            self.visit_expression(start_expression)?;
        }
        if let Some(end_expression) = end {
            self.visit_expression(end_expression)?;
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

    fn visit_list_comprehension(
        &mut self,
        _variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    ) -> Self::Result {
        self.visit_expression(source)?;
        if let Some(f) = filter {
            self.visit_expression(f)?;
        }
        if let Some(m) = map {
            self.visit_expression(m)?;
        }
        self.type_ = DataType::List;
        Ok(())
    }

    fn visit_tag_property(&mut self, tag_name: &str, property: &str) -> Self::Result {
        self.type_ = self.get_property_type_from_tag(tag_name, property);
        Ok(())
    }

    fn visit_edge_property(&mut self, edge_name: &str, property: &str) -> Self::Result {
        self.type_ = self.get_property_type_from_edge(edge_name, property);
        Ok(())
    }

    fn visit_label_tag_property(&mut self, tag: &Expression, _property: &str) -> Self::Result {
        self.visit_expression(tag)?;
        self.type_ = DataType::Empty;
        Ok(())
    }

    fn visit_parameter(&mut self, _name: &str) -> Self::Result {
        self.type_ = DataType::Empty;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{
        EdgeTypeSchema, InsertEdgeInfo, InsertVertexInfo, PasswordInfo,
        PropertyDef, SpaceInfo, TagInfo, UpdateInfo,
    };
    use crate::index::Index;
    use crate::storage::Schema;

    /// Mock 存储引擎用于测试
    #[derive(Debug)]
    struct MockStorageEngine;

    impl StorageClient for MockStorageEngine {
        fn get_vertex(&self, _space: &str, _id: &Value) -> Result<Option<Vertex>, StorageError> {
            Ok(None)
        }

        fn scan_vertices(&self, _space: &str) -> Result<Vec<Vertex>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_vertices_by_tag(&self, _space: &str, _tag: &str) -> Result<Vec<Vertex>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_vertices_by_prop(
            &self,
            _space: &str,
            _tag: &str,
            _prop: &str,
            _value: &Value,
        ) -> Result<Vec<Vertex>, StorageError> {
            Ok(Vec::new())
        }

        fn get_edge(
            &self,
            _space: &str,
            _src: &Value,
            _dst: &Value,
            _edge_type: &str,
        ) -> Result<Option<Edge>, StorageError> {
            Ok(None)
        }

        fn get_node_edges(
            &self,
            _space: &str,
            _node_id: &Value,
            _direction: EdgeDirection,
        ) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn get_node_edges_filtered(
            &self,
            _space: &str,
            _node_id: &Value,
            _direction: EdgeDirection,
            _filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
        ) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_edges_by_type(&self, _space: &str, _edge_type: &str) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_all_edges(&self, _space: &str) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn insert_vertex(&mut self, _space: &str, _vertex: Vertex) -> Result<Value, StorageError> {
            Ok(Value::Int(0))
        }

        fn update_vertex(&mut self, _space: &str, _vertex: Vertex) -> Result<(), StorageError> {
            Ok(())
        }

        fn delete_vertex(&mut self, _space: &str, _id: &Value) -> Result<(), StorageError> {
            Ok(())
        }

        fn batch_insert_vertices(
            &mut self,
            _space: &str,
            _vertices: Vec<Vertex>,
        ) -> Result<Vec<Value>, StorageError> {
            Ok(Vec::new())
        }

        fn insert_edge(&mut self, _space: &str, _edge: Edge) -> Result<(), StorageError> {
            Ok(())
        }

        fn delete_edge(
            &mut self,
            _space: &str,
            _src: &Value,
            _dst: &Value,
            _edge_type: &str,
        ) -> Result<(), StorageError> {
            Ok(())
        }

        fn batch_insert_edges(&mut self, _space: &str, _edges: Vec<Edge>) -> Result<(), StorageError> {
            Ok(())
        }

        fn create_space(&mut self, _space: &SpaceInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn drop_space(&mut self, _space: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_space(&self, _space: &str) -> Result<Option<SpaceInfo>, StorageError> {
            Ok(None)
        }

        fn get_space_by_id(&self, _space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
            Ok(None)
        }

        fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
            Ok(Vec::new())
        }

        fn create_tag(&mut self, _space: &str, _info: &TagInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn alter_tag(
            &mut self,
            _space: &str,
            _tag: &str,
            _additions: Vec<PropertyDef>,
            _deletions: Vec<String>,
        ) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_tag(&self, _space: &str, _tag: &str) -> Result<Option<TagInfo>, StorageError> {
            Ok(None)
        }

        fn drop_tag(&mut self, _space: &str, _tag: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn list_tags(&self, _space: &str) -> Result<Vec<TagInfo>, StorageError> {
            Ok(Vec::new())
        }

        fn create_edge_type(
            &mut self,
            _space: &str,
            _info: &EdgeTypeSchema,
        ) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn alter_edge_type(
            &mut self,
            _space: &str,
            _edge_type: &str,
            _additions: Vec<PropertyDef>,
            _deletions: Vec<String>,
        ) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_edge_type(
            &self,
            _space: &str,
            _edge_type: &str,
        ) -> Result<Option<EdgeTypeSchema>, StorageError> {
            Ok(None)
        }

        fn drop_edge_type(&mut self, _space: &str, _edge_type: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn list_edge_types(&self, _space: &str) -> Result<Vec<EdgeTypeSchema>, StorageError> {
            Ok(Vec::new())
        }

        fn create_tag_index(&mut self, _space: &str, _info: &Index) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn drop_tag_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_tag_index(
            &self,
            _space: &str,
            _index: &str,
        ) -> Result<Option<Index>, StorageError> {
            Ok(None)
        }

        fn list_tag_indexes(&self, _space: &str) -> Result<Vec<Index>, StorageError> {
            Ok(Vec::new())
        }

        fn rebuild_tag_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn create_edge_index(&mut self, _space: &str, _info: &Index) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn drop_edge_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_edge_index(
            &self,
            _space: &str,
            _index: &str,
        ) -> Result<Option<Index>, StorageError> {
            Ok(None)
        }

        fn list_edge_indexes(&self, _space: &str) -> Result<Vec<Index>, StorageError> {
            Ok(Vec::new())
        }

        fn rebuild_edge_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn lookup_index(
            &self,
            _space: &str,
            _index: &str,
            _value: &Value,
        ) -> Result<Vec<Value>, StorageError> {
            Ok(Vec::new())
        }

        fn lookup_index_with_score(
            &self,
            _space: &str,
            _index: &str,
            _value: &Value,
        ) -> Result<Vec<(Value, f32)>, StorageError> {
            Ok(Vec::new())
        }

        fn insert_vertex_data(
            &mut self,
            _space: &str,
            _info: &InsertVertexInfo,
        ) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn insert_edge_data(&mut self, _space: &str, _info: &InsertEdgeInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn delete_vertex_data(&mut self, _space: &str, _vertex_id: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn delete_edge_data(
            &mut self,
            _space: &str,
            _src: &str,
            _dst: &str,
            _rank: i64,
        ) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn update_data(&mut self, _space: &str, _info: &UpdateInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn change_password(&mut self, _info: &PasswordInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn create_user(&mut self, _info: &crate::core::types::metadata::UserInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn alter_user(&mut self, _info: &crate::core::types::metadata::UserAlterInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn drop_user(&mut self, _username: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_space_id(&self, _space: &str) -> Result<u64, StorageError> {
            Ok(1)
        }

        fn space_exists(&self, _space: &str) -> bool {
            false
        }

        fn clear_space(&mut self, _space: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn alter_space_partition_num(&mut self, _space_id: u64, _partition_num: usize) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn alter_space_replica_factor(&mut self, _space_id: u64, _replica_factor: usize) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn alter_space_comment(&mut self, _space_id: u64, _comment: String) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn grant_role(&mut self, _username: &str, _space_id: u64, _role: crate::api::service::permission_manager::RoleType) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn revoke_role(&mut self, _username: &str, _space_id: u64) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_vertex_with_schema(
            &self,
            _space: &str,
            _tag: &str,
            _id: &Value,
        ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
            Ok(None)
        }

        fn get_edge_with_schema(
            &self,
            _space: &str,
            _edge_type: &str,
            _src: &Value,
            _dst: &Value,
        ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
            Ok(None)
        }

        fn scan_vertices_with_schema(
            &self,
            _space: &str,
            _tag: &str,
        ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_edges_with_schema(
            &self,
            _space: &str,
            _edge_type: &str,
        ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
            Ok(Vec::new())
        }

        fn load_from_disk(&mut self) -> Result<(), StorageError> {
            Ok(())
        }

        fn save_to_disk(&self) -> Result<(), StorageError> {
            Ok(())
        }

        fn get_storage_stats(&self) -> crate::storage::storage_client::StorageStats {
            crate::storage::storage_client::StorageStats {
                total_vertices: 0,
                total_edges: 0,
                total_spaces: 0,
                total_tags: 0,
                total_edge_types: 0,
            }
        }

        fn delete_vertex_with_edges(&mut self, _space: &str, _id: &Value) -> Result<(), StorageError> {
            Ok(())
        }

        fn delete_tags(
            &mut self,
            _space: &str,
            _vertex_id: &Value,
            _tag_names: &[String],
        ) -> Result<usize, StorageError> {
            Ok(0)
        }

        fn find_dangling_edges(&self, _space: &str) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn repair_dangling_edges(&mut self, _space: &str) -> Result<usize, StorageError> {
            Ok(0)
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
        assert_eq!(TypeUtils::get_type_priority(&DataType::Int), 20);
        assert_eq!(TypeUtils::get_type_priority(&DataType::Float), 30);
        assert_eq!(TypeUtils::get_type_priority(&DataType::String), 40);

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

/// 为DeduceTypeVisitor实现GenericExpressionVisitor<Expression>
/// 提供统一的泛型访问接口
impl<'a, S: StorageClient> GenericExpressionVisitor<Expression> for DeduceTypeVisitor<'a, S> {
    type Result = Result<(), TypeDeductionError>;

    fn visit(&mut self, expression: &Expression) -> <Self as GenericExpressionVisitor<Expression>>::Result {
        self.visit_expression(expression)
    }
}
