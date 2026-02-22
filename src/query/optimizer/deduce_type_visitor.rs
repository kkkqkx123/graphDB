//! DeduceTypeVisitor - 用于推导表达式类型的访问器

use crate::core::types::expression::Expression;
use crate::core::types::expression::visitor::{ExpressionVisitor, ExpressionVisitorState, GenericExpressionVisitor};
use crate::core::types::metadata::PropertyDef;
use crate::core::{
    TypeUtils, DataType, BinaryOperator, UnaryOperator, Value,
};
use crate::query::validator::structs::ValidationContextImpl;
use crate::query::validator::validator_trait::ColumnDef;
use crate::storage::StorageClient;
use thiserror::Error;

#[cfg(test)]
use crate::core::{Edge, Vertex};
#[cfg(test)]
use crate::core::EdgeDirection;
#[cfg(test)]
use crate::core::error::StorageError;

/// 从 ValueType 转换为 DataType
fn value_type_to_value_type_def(type_: &crate::query::validator::validator_trait::ValueType) -> DataType {
    match type_ {
        crate::query::validator::validator_trait::ValueType::Unknown => DataType::Empty,
        crate::query::validator::validator_trait::ValueType::Bool => DataType::Bool,
        crate::query::validator::validator_trait::ValueType::Int => DataType::Int,
        crate::query::validator::validator_trait::ValueType::Float => DataType::Float,
        crate::query::validator::validator_trait::ValueType::String => DataType::String,
        crate::query::validator::validator_trait::ValueType::Date => DataType::Date,
        crate::query::validator::validator_trait::ValueType::Time => DataType::Time,
        crate::query::validator::validator_trait::ValueType::DateTime => DataType::DateTime,
        crate::query::validator::validator_trait::ValueType::Vertex => DataType::Vertex,
        crate::query::validator::validator_trait::ValueType::Edge => DataType::Edge,
        crate::query::validator::validator_trait::ValueType::Path => DataType::Path,
        crate::query::validator::validator_trait::ValueType::List => DataType::List,
        crate::query::validator::validator_trait::ValueType::Map => DataType::Map,
        crate::query::validator::validator_trait::ValueType::Set => DataType::Set,
        crate::query::validator::validator_trait::ValueType::Null => DataType::Null,
    }
}


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
    validate_context: &'a ValidationContextImpl,
    /// 输入列定义：列名 -> 列类型
    inputs: Vec<ColumnDef>,
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
        validate_context: &'a ValidationContextImpl,
        inputs: Vec<ColumnDef>,
        space: String,
    ) -> Self {
        // VID 类型从存储引擎获取，如果无法获取则使用默认值
        let vid_type = DataType::String;

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
    ) -> (Self, ValidationContextImpl) {
        let _vctx = ValidationContextImpl::new();
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
                let value_type = col.type_.clone();
                self.type_ = value_type_to_value_type_def(&value_type);
                return Ok(());
            }
        }

        if self.validate_context.exists_var(name) {
            let var_cols = self.validate_context.get_var(name);
            if !var_cols.is_empty() {
                let value_type = var_cols[0].type_.clone();
                self.type_ = value_type_to_value_type_def(&value_type);
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
        func: &crate::core::types::operators::AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        // 先访问参数以获取其类型
        self.visit_expression(arg)?;
        // 然后根据聚合函数类型确定返回类型
        self.deduce_aggregate_func_type(func)
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        self.type_ = DataType::List;
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_, value) in pairs {
            self.visit_expression(value)?;
        }
        self.type_ = DataType::Map;
        Ok(())
    }

    fn visit_case(
        &mut self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) -> Self::Result {
        // 如果有test_expr，先访问它
        if let Some(expr) = test_expr {
            self.visit_expression(expr)?;
        }

        // 访问所有条件
        for (when, then) in conditions {
            self.visit_expression(when)?;
            self.visit_expression(then)?;
        }

        // 访问默认值
        if let Some(expr) = default {
            self.visit_expression(expr)?;
        }

        // CASE表达式的类型默认为Empty，实际应该根据then分支推导
        self.type_ = DataType::Empty;
        Ok(())
    }

    fn visit_type_cast(&mut self, expression: &Expression, target_type: &DataType) -> Self::Result {
        self.visit_expression(expression)?;
        self.type_ = target_type.clone();
        Ok(())
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.visit_expression(collection)?;
        self.visit_expression(index)?;
        // 下标访问的类型默认为Empty，实际应该根据集合元素类型推导
        self.type_ = DataType::Empty;
        Ok(())
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) -> Self::Result {
        self.visit_expression(collection)?;
        if let Some(expr) = start {
            self.visit_expression(expr)?;
        }
        if let Some(expr) = end {
            self.visit_expression(expr)?;
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

    fn visit_list_comprehension(
        &mut self,
        _variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    ) -> Self::Result {
        self.visit_expression(source)?;
        if let Some(expr) = filter {
            self.visit_expression(expr)?;
        }
        if let Some(expr) = map {
            self.visit_expression(expr)?;
        }
        self.type_ = DataType::List;
        Ok(())
    }

    fn visit_label_tag_property(&mut self, _tag: &Expression, _property: &str) -> Self::Result {
        self.type_ = DataType::Empty;
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

    fn visit_predicate(
        &mut self,
        _func: &str,
        args: &[Expression],
    ) -> Self::Result {
        for arg in args {
            self.visit_expression(arg)?;
        }
        self.type_ = DataType::Bool;
        Ok(())
    }

    fn visit_reduce(
        &mut self,
        _accumulator: &str,
        initial: &Expression,
        _variable: &str,
        source: &Expression,
        mapping: &Expression,
    ) -> Self::Result {
        self.visit_expression(initial)?;
        self.visit_expression(source)?;
        self.visit_expression(mapping)?;
        self.type_ = DataType::Empty;
        Ok(())
    }

    fn visit_path_build(&mut self, exprs: &[Expression]) -> Self::Result {
        for expr in exprs {
            self.visit_expression(expr)?;
        }
        self.type_ = DataType::Path;
        Ok(())
    }

    fn visit_parameter(&mut self, _name: &str) -> Self::Result {
        self.type_ = DataType::Empty;
        Ok(())
    }
}

// 为DeduceTypeVisitor实现GenericExpressionVisitor
impl<'a, S: StorageClient> GenericExpressionVisitor<Expression> for DeduceTypeVisitor<'a, S> {
    type Result = Result<(), TypeDeductionError>;

    fn visit(&mut self, expression: &Expression) -> Self::Result {
        self.visit_expression(expression)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 基础测试：验证DeduceTypeVisitor可以被创建
    // 注意：完整测试需要Mock存储引擎
    #[test]
    fn test_deduce_type_visitor_creation() {
        // 由于需要存储引擎，这里只做编译时检查
        // 实际测试应该在集成测试中进行
    }
}
