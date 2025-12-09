//! DeduceTypeVisitor - 用于推导表达式类型的访问器
//! 对应 NebulaGraph DeduceTypeVisitor.h/.cpp 的功能

use crate::core::{Value, ValueTypeDef, Vertex, Edge, Direction};
use crate::graph::expression::{BinaryOperator, UnaryOperator};
use crate::graph::expression::Expression;
use crate::query::validator::ValidateContext;
use crate::storage::{StorageEngine, StorageError};
use thiserror::Error;

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
    validate_context: &'a ValidateContext,
    /// 输入列定义：列名 -> 列类型
    inputs: Vec<(String, ValueTypeDef)>,
    /// 图空间ID
    space: String,
    /// 当前推导状态
    status: Option<TypeDeductionError>,
    /// 推导出的类型
    type_: ValueTypeDef,
    /// VID(顶点ID)类型
    vid_type: ValueTypeDef,
}

impl<'a, S: StorageEngine> DeduceTypeVisitor<'a, S> {
    pub fn new(
        storage: &'a S,
        validate_context: &'a ValidateContext,
        inputs: Vec<(String, ValueTypeDef)>,
        space: String,
    ) -> Self {
        // VID类型通常从空间配置获取，这里简化为String
        let vid_type = ValueTypeDef::String;

        Self {
            _storage: storage,
            validate_context,
            inputs,
            space,
            status: None,
            type_: ValueTypeDef::Empty,
            vid_type,
        }
    }

    /// 创建用于测试的访问器（不需要存储和验证上下文）
    pub fn new_for_test(
        inputs: Vec<(String, ValueTypeDef)>,
        space: String,
    ) -> (Self, ValidateContext) {
        let _vctx = ValidateContext::new();
        let _vid_type = ValueTypeDef::String;

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
    pub fn type_(&self) -> ValueTypeDef {
        self.type_.clone()
    }

    /// 设置VID类型
    pub fn set_vid_type(&mut self, vid_type: ValueTypeDef) {
        self.vid_type = vid_type;
    }

    /// 主推导方法 - 推导表达式的类型
    pub fn deduce_type(&mut self, expr: &Expression) -> Result<ValueTypeDef, TypeDeductionError> {
        self.visit(expr)?;
        Ok(self.type_.clone())
    }

    /// 递归访问表达式树
    fn visit(&mut self, expr: &Expression) -> Result<(), TypeDeductionError> {
        match expr {
            Expression::Constant(value) => self.visit_constant(value),
            Expression::Property(name) => self.visit_property(name),
            Expression::Function(name, args) => self.visit_function_call(name, args),
            Expression::BinaryOp(left, op, right) => {
                self.visit(left)?;
                let left_type = self.type_.clone();
                self.visit(right)?;
                let right_type = self.type_.clone();
                self.visit_binary(op, left_type, right_type)
            }
            Expression::UnaryOp(op, operand) => {
                self.visit(operand)?;
                self.visit_unary(op)
            }
            Expression::List(items) => self.visit_list(items),
            Expression::Map(pairs) => self.visit_map_items(pairs),
            Expression::Set(items) => self.visit_set(items),
            Expression::TagProperty { tag, prop } => self.visit_tag_property(tag, prop),
            Expression::EdgeProperty { edge, prop } => self.visit_edge_property(edge, prop),
            Expression::InputProperty(prop) => self.visit_input_property(prop),
            Expression::VariableProperty { var, prop } => self.visit_variable_property(var, prop),
            Expression::SourceProperty { tag, prop } => self.visit_source_property(tag, prop),
            Expression::DestinationProperty { tag, prop } => self.visit_dest_property(tag, prop),
            Expression::UnaryPlus(operand) => {
                self.visit(operand)?;
                Ok(())
            }
            Expression::UnaryNegate(operand) => {
                self.visit(operand)?;
                // 检查是否可以取反
                match &self.type_ {
                    ValueTypeDef::Int
                    | ValueTypeDef::Float
                    | ValueTypeDef::Empty
                    | ValueTypeDef::Null => Ok(()),
                    _ => {
                        let msg = format!("无法对类型 {:?} 执行取反操作", self.type_);
                        self.status = Some(TypeDeductionError::SemanticError(msg.clone()));
                        Err(TypeDeductionError::SemanticError(msg))
                    }
                }
            }
            Expression::UnaryNot(operand) => {
                self.visit(operand)?;
                self.type_ = ValueTypeDef::Bool;
                Ok(())
            }
            Expression::UnaryIncr(operand) => {
                self.visit(operand)?;
                // 自增后类型保持不变（如果是数字类型）
                match &self.type_ {
                    ValueTypeDef::Int | ValueTypeDef::Float => Ok(()),
                    _ => {
                        let msg = format!("无法对类型 {:?} 执行自增操作", self.type_);
                        self.status = Some(TypeDeductionError::SemanticError(msg.clone()));
                        Err(TypeDeductionError::SemanticError(msg))
                    }
                }
            }
            Expression::UnaryDecr(operand) => {
                self.visit(operand)?;
                // 自减后类型保持不变（如果是数字类型）
                match &self.type_ {
                    ValueTypeDef::Int | ValueTypeDef::Float => Ok(()),
                    _ => {
                        let msg = format!("无法对类型 {:?} 执行自减操作", self.type_);
                        self.status = Some(TypeDeductionError::SemanticError(msg.clone()));
                        Err(TypeDeductionError::SemanticError(msg))
                    }
                }
            }
            Expression::IsNull(operand) => {
                self.visit(operand)?;
                self.type_ = ValueTypeDef::Bool;
                Ok(())
            }
            Expression::IsNotNull(operand) => {
                self.visit(operand)?;
                self.type_ = ValueTypeDef::Bool;
                Ok(())
            }
            Expression::IsEmpty(operand) => {
                self.visit(operand)?;
                self.type_ = ValueTypeDef::Bool;
                Ok(())
            }
            Expression::IsNotEmpty(operand) => {
                self.visit(operand)?;
                self.type_ = ValueTypeDef::Bool;
                Ok(())
            }
            Expression::TypeCasting { expr, target_type } => {
                self.visit(expr.as_ref())?;
                self.type_ = self.parse_type_def(target_type);
                Ok(())
            }
            Expression::Case {
                conditions,
                default,
            } => {
                // 检查所有条件和默认分支的类型是否一致
                let mut result_type: Option<ValueTypeDef> = None;

                for (condition_expr, then_expr) in conditions {
                    self.visit(condition_expr)?;
                    self.visit(then_expr)?;
                    let then_type = self.type_.clone();

                    if let Some(ref existing_type) = result_type {
                        // 检查类型一致性
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
                    self.visit(default_expr)?;
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
            Expression::Aggregate {
                func,
                arg,
                distinct: _,
            } => {
                self.visit(arg.as_ref())?;
                self.visit_aggregate(func)
            }
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                self.visit(generator.as_ref())?;
                if let Some(condition_expr) = condition.as_ref() {
                    self.visit(condition_expr.as_ref())?;
                }
                // 列表推导始终返回列表类型
                self.type_ = ValueTypeDef::List;
                Ok(())
            }
            Expression::Predicate { list, condition } => {
                self.visit(list.as_ref())?;
                self.visit(condition.as_ref())?;
                // 谓词表达式返回布尔值
                self.type_ = ValueTypeDef::Bool;
                Ok(())
            }
            Expression::Reduce {
                list,
                var: _,
                initial,
                expr,
            } => {
                self.visit(initial)?;
                let accumulator_type = self.type_.clone();
                self.visit(list)?;
                self.visit(expr)?;
                // 归约结果类型为累加器类型
                self.type_ = accumulator_type;
                Ok(())
            }
            Expression::PathBuild(items) => self.visit_path_build(items),
            Expression::ESQuery(_) => {
                // 文本搜索结果为字符串
                self.type_ = ValueTypeDef::String;
                Ok(())
            }
            Expression::UUID => {
                // UUID为字符串
                self.type_ = ValueTypeDef::String;
                Ok(())
            }
            Expression::Variable(name) => self.visit_variable(name),
            Expression::Subscript { collection, index } => {
                self.visit(collection)?;
                let container_type = self.type_.clone();
                self.visit(index)?;
                // 下标访问的结果类型取决于容器类型
                self.type_ = match container_type {
                    ValueTypeDef::List => ValueTypeDef::Empty, // 列表元素类型未知
                    ValueTypeDef::Map => ValueTypeDef::Empty,  // Map值类型未知
                    _ => ValueTypeDef::Empty,
                };
                Ok(())
            }
            Expression::SubscriptRange {
                collection,
                start,
                end,
            } => {
                self.visit(collection)?;
                if let Some(start_idx) = start.as_ref() {
                    self.visit(start_idx)?;
                }
                if let Some(end_idx) = end.as_ref() {
                    self.visit(end_idx)?;
                }
                // 范围下标始终返回列表
                self.type_ = ValueTypeDef::List;
                Ok(())
            }
            Expression::Label(name) => {
                // 标签通常是字符串
                self.type_ = ValueTypeDef::String;
                Ok(())
            }
            Expression::MatchPathPattern {
                path_alias: _,
                patterns,
            } => {
                for pattern in patterns {
                    self.visit(pattern)?;
                }
                // 路径匹配返回路径类型
                self.type_ = ValueTypeDef::Path;
                Ok(())
            }
        }
    }

    /// 推导常量表达式的类型
    fn visit_constant(&mut self, value: &Value) -> Result<(), TypeDeductionError> {
        self.type_ = value.get_type();
        Ok(())
    }

    /// 推导一元操作符的类型
    fn visit_unary(&mut self, _op: &UnaryOperator) -> Result<(), TypeDeductionError> {
        // 具体操作已在visit方法中处理
        Ok(())
    }

    /// 推导二元操作符的类型
    fn visit_binary(
        &mut self,
        op: &BinaryOperator,
        left_type: ValueTypeDef,
        right_type: ValueTypeDef,
    ) -> Result<(), TypeDeductionError> {
        match op {
            BinaryOperator::Add => {
                if left_type == ValueTypeDef::String && right_type == ValueTypeDef::String {
                    self.type_ = ValueTypeDef::String;
                } else if left_type == ValueTypeDef::Int && right_type == ValueTypeDef::Int {
                    self.type_ = ValueTypeDef::Int;
                } else if left_type == ValueTypeDef::Float && right_type == ValueTypeDef::Float {
                    self.type_ = ValueTypeDef::Float;
                } else if (left_type == ValueTypeDef::Int && right_type == ValueTypeDef::Float)
                    || (left_type == ValueTypeDef::Float && right_type == ValueTypeDef::Int)
                {
                    self.type_ = ValueTypeDef::Float;
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
            BinaryOperator::Sub
            | BinaryOperator::Mul
            | BinaryOperator::Div
            | BinaryOperator::Mod => {
                if left_type == ValueTypeDef::Int && right_type == ValueTypeDef::Int {
                    self.type_ = ValueTypeDef::Int;
                } else if left_type == ValueTypeDef::Float && right_type == ValueTypeDef::Float {
                    self.type_ = ValueTypeDef::Float;
                } else if (left_type == ValueTypeDef::Int && right_type == ValueTypeDef::Float)
                    || (left_type == ValueTypeDef::Float && right_type == ValueTypeDef::Int)
                {
                    self.type_ = ValueTypeDef::Float;
                } else if self.is_superior_type(&left_type) || self.is_superior_type(&right_type) {
                    // NULL或EMPTY类型兼容任何类型
                    self.type_ = if self.is_superior_type(&left_type) {
                        right_type
                    } else {
                        left_type
                    };
                } else {
                    let op_name = match op {
                        BinaryOperator::Sub => "减法",
                        BinaryOperator::Mul => "乘法",
                        BinaryOperator::Div => "除法",
                        BinaryOperator::Mod => "模运算",
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
            BinaryOperator::Eq
            | BinaryOperator::Ne
            | BinaryOperator::Lt
            | BinaryOperator::Le
            | BinaryOperator::Gt
            | BinaryOperator::Ge => {
                // 关系操作的结果类型是布尔值
                self.type_ = ValueTypeDef::Bool;
            }
            BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor => {
                // 逻辑操作的结果类型是布尔值
                self.type_ = ValueTypeDef::Bool;
            }
            BinaryOperator::In | BinaryOperator::NotIn => {
                // 集合操作的结果类型是布尔值
                self.type_ = ValueTypeDef::Bool;
            }
            _ => {
                // 其他操作默认返回布尔值
                self.type_ = ValueTypeDef::Bool;
            }
        }
        Ok(())
    }

    /// 推导属性表达式的类型
    fn visit_property(&mut self, _property: &str) -> Result<(), TypeDeductionError> {
        // 属性访问的结果类型需要根据上下文来确定
        // 简化实现，返回Empty类型
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    /// 推导函数调用表达式的类型
    fn visit_function_call(
        &mut self,
        name: &str,
        args: &[Expression],
    ) -> Result<(), TypeDeductionError> {
        // 推导参数类型
        let mut _arg_types = Vec::new();
        for arg in args {
            self.visit(arg)?;
            _arg_types.push(self.type_.clone());
        }

        // 根据函数名确定返回类型
        let name_upper = name.to_uppercase();
        self.type_ = match name_upper.as_str() {
            // ID提取函数
            "ID" | "SRC" | "DST" | "NONE_DIRECT_SRC" | "NONE_DIRECT_DST" => self.vid_type.clone(),
            // 聚合函数
            "COUNT" => ValueTypeDef::Int,
            "AVG" | "SUM" => ValueTypeDef::Float,
            "MAX" | "MIN" => {
                if _arg_types.is_empty() {
                    ValueTypeDef::Empty
                } else {
                    _arg_types[0].clone()
                }
            }
            "COLLECT" => ValueTypeDef::List,
            "COLLECT_SET" => ValueTypeDef::Set,
            // 字符串函数
            "LOWER" | "UPPER" | "TRIM" | "LTRIM" | "RTRIM" | "SUBSTR" | "REVERSE" => {
                ValueTypeDef::String
            }
            // 数学函数
            "ABS" | "CEIL" | "FLOOR" | "SQRT" | "POW" | "EXP" | "LOG" | "LOG10" => {
                ValueTypeDef::Float
            }
            // 其他函数默认返回Empty
            _ => ValueTypeDef::Empty,
        };
        Ok(())
    }

    /// 推导聚合表达式的类型
    fn visit_aggregate(&mut self, name: &str) -> Result<(), TypeDeductionError> {
        let name_upper = name.to_uppercase();
        self.type_ = match name_upper.as_str() {
            "COUNT" => ValueTypeDef::Int,
            "COLLECT" => ValueTypeDef::List,
            "COLLECT_SET" => ValueTypeDef::Set,
            "AVG" | "SUM" => ValueTypeDef::Float,
            "MAX" | "MIN" => {
                // 保持参数类型，已在visit中处理
                self.type_.clone()
            }
            _ => ValueTypeDef::Empty,
        };
        Ok(())
    }

    /// 推导标签属性表达式的类型
    fn visit_tag_property(&mut self, _tag: &str, _prop: &str) -> Result<(), TypeDeductionError> {
        // 在实际实现中，这里会查询标签的schema来确定属性类型
        // 简化实现，返回Empty类型
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    /// 推导边属性表达式的类型
    fn visit_edge_property(&mut self, _edge: &str, _prop: &str) -> Result<(), TypeDeductionError> {
        // 在实际实现中，这里会查询边的schema来确定属性类型
        // 简化实现，返回Empty类型
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    /// 推导输入属性表达式的类型
    fn visit_input_property(&mut self, name: &str) -> Result<(), TypeDeductionError> {
        // 查找输入列
        for (col_name, col_type) in &self.inputs {
            if col_name == name {
                self.type_ = col_type.clone();
                return Ok(());
            }
        }

        let msg = format!("输入属性 {} 不存在", name);
        self.status = Some(TypeDeductionError::SemanticError(msg.clone()));
        Err(TypeDeductionError::SemanticError(msg))
    }

    /// 推导变量属性表达式的类型
    fn visit_variable_property(
        &mut self,
        var: &str,
        _prop: &str,
    ) -> Result<(), TypeDeductionError> {
        // 检查变量是否存在
        if !self.validate_context.exists_var(var) {
            let msg = format!("变量 {} 不存在", var);
            let err = TypeDeductionError::SemanticError(msg.clone());
            self.status = Some(err.clone());
            return Err(err);
        }

        // 在实际实现中，这里会查询变量的schema来确定属性类型
        // 简化实现，返回Empty类型
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    /// 推导源顶点属性表达式的类型
    fn visit_source_property(&mut self, _tag: &str, _prop: &str) -> Result<(), TypeDeductionError> {
        // 源顶点属性，简化实现返回Empty
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    /// 推导目标顶点属性表达式的类型
    fn visit_dest_property(&mut self, _tag: &str, _prop: &str) -> Result<(), TypeDeductionError> {
        // 目标顶点属性，简化实现返回Empty
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    /// 推导变量表达式的类型
    fn visit_variable(&mut self, _name: &str) -> Result<(), TypeDeductionError> {
        // 变量表达式的结果类型不确定，使用Empty
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    /// 推导列表表达式的类型
    fn visit_list(&mut self, _items: &[Expression]) -> Result<(), TypeDeductionError> {
        self.type_ = ValueTypeDef::List;
        Ok(())
    }

    /// 推导集合表达式的类型
    fn visit_set(&mut self, _items: &[Expression]) -> Result<(), TypeDeductionError> {
        self.type_ = ValueTypeDef::Set;
        Ok(())
    }

    /// 推导映射表达式的类型
    fn visit_map_items(
        &mut self,
        _pairs: &[(String, Expression)],
    ) -> Result<(), TypeDeductionError> {
        self.type_ = ValueTypeDef::Map;
        Ok(())
    }

    /// 推导路径构建表达式的类型
    fn visit_path_build(&mut self, _items: &[Expression]) -> Result<(), TypeDeductionError> {
        self.type_ = ValueTypeDef::Path;
        Ok(())
    }

    /// 检查两种类型是否兼容
    fn are_types_compatible(&self, type1: &ValueTypeDef, type2: &ValueTypeDef) -> bool {
        if type1 == type2 {
            return true;
        }
        // NULL和EMPTY类型与任何类型兼容
        if self.is_superior_type(type1) || self.is_superior_type(type2) {
            return true;
        }
        // Int和Float可以相互兼容
        if (type1 == &ValueTypeDef::Int && type2 == &ValueTypeDef::Float)
            || (type1 == &ValueTypeDef::Float && type2 == &ValueTypeDef::Int)
        {
            return true;
        }
        false
    }

    /// 检查类型是否为"优越类型"
    /// 优越类型包括NULL和EMPTY，它们可以与任何类型兼容
    fn is_superior_type(&self, type_: &ValueTypeDef) -> bool {
        matches!(type_, ValueTypeDef::Null | ValueTypeDef::Empty)
    }

    /// 将字符串解析为ValueTypeDef
    fn parse_type_def(&self, type_str: &str) -> ValueTypeDef {
        match type_str.to_uppercase().as_str() {
            "INT" => ValueTypeDef::Int,
            "FLOAT" | "DOUBLE" => ValueTypeDef::Float,
            "STRING" => ValueTypeDef::String,
            "BOOL" => ValueTypeDef::Bool,
            "DATE" => ValueTypeDef::Date,
            "TIME" => ValueTypeDef::Time,
            "DATETIME" => ValueTypeDef::DateTime,
            "VERTEX" => ValueTypeDef::Vertex,
            "EDGE" => ValueTypeDef::Edge,
            "PATH" => ValueTypeDef::Path,
            "LIST" => ValueTypeDef::List,
            "SET" => ValueTypeDef::Set,
            "MAP" => ValueTypeDef::Map,
            "NULL" => ValueTypeDef::Null,
            _ => ValueTypeDef::Empty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_superior_type() {
        let validate_context = ValidateContext::new();
        let visitor = DeduceTypeVisitor::new(
            &MockStorageEngine,
            &validate_context,
            vec![],
            "test_space".to_string(),
        );

        assert!(visitor.is_superior_type(&ValueTypeDef::Null));
        assert!(visitor.is_superior_type(&ValueTypeDef::Empty));
        assert!(!visitor.is_superior_type(&ValueTypeDef::Int));
        assert!(!visitor.is_superior_type(&ValueTypeDef::String));
    }

    #[test]
    fn test_are_types_compatible() {
        let validate_context = ValidateContext::new();
        let visitor = DeduceTypeVisitor::new(
            &MockStorageEngine,
            &validate_context,
            vec![],
            "test_space".to_string(),
        );

        // 相同类型兼容
        assert!(visitor.are_types_compatible(&ValueTypeDef::Int, &ValueTypeDef::Int));

        // 优越类型与任何类型兼容
        assert!(visitor.are_types_compatible(&ValueTypeDef::Null, &ValueTypeDef::Int));
        assert!(visitor.are_types_compatible(&ValueTypeDef::Empty, &ValueTypeDef::String));

        // Int和Float兼容
        assert!(visitor.are_types_compatible(&ValueTypeDef::Int, &ValueTypeDef::Float));
        assert!(visitor.are_types_compatible(&ValueTypeDef::Float, &ValueTypeDef::Int));

        // 不同类型不兼容
        assert!(!visitor.are_types_compatible(&ValueTypeDef::Int, &ValueTypeDef::String));
    }

    #[test]
    fn test_parse_type_def() {
        let validate_context = ValidateContext::new();
        let visitor = DeduceTypeVisitor::new(
            &MockStorageEngine,
            &validate_context,
            vec![],
            "test_space".to_string(),
        );

        assert_eq!(visitor.parse_type_def("int"), ValueTypeDef::Int);
        assert_eq!(visitor.parse_type_def("INT"), ValueTypeDef::Int);
        assert_eq!(visitor.parse_type_def("string"), ValueTypeDef::String);
        assert_eq!(visitor.parse_type_def("BOOL"), ValueTypeDef::Bool);
        assert_eq!(visitor.parse_type_def("unknown"), ValueTypeDef::Empty);
    }

    #[test]
    fn test_visit_constant() {
        let validate_context = ValidateContext::new();
        let mut visitor = DeduceTypeVisitor::new(
            &MockStorageEngine,
            &validate_context,
            vec![],
            "test_space".to_string(),
        );

        let value = Value::Int(42);
        let result = visitor.visit_constant(&value);

        assert!(result.is_ok());
        assert_eq!(visitor.type_(), ValueTypeDef::Int);
    }

    #[test]
    fn test_visit_list() {
        let validate_context = ValidateContext::new();
        let mut visitor = DeduceTypeVisitor::new(
            &MockStorageEngine,
            &validate_context,
            vec![],
            "test_space".to_string(),
        );

        let result = visitor.visit_list(&[]);

        assert!(result.is_ok());
        assert_eq!(visitor.type_(), ValueTypeDef::List);
    }
}

// Mock 存储引擎用于测试
#[cfg(test)]
struct MockStorageEngine;

#[cfg(test)]
impl StorageEngine for MockStorageEngine {
    fn insert_node(&mut self, vertex: Vertex) -> Result<Value, StorageError> {
        Ok(vertex.vid.as_ref().clone())
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

    fn begin_transaction(&mut self) -> Result<u64, StorageError> {
        Ok(1)
    }

    fn commit_transaction(&mut self, _tx_id: u64) -> Result<(), StorageError> {
        Ok(())
    }

    fn rollback_transaction(&mut self, _tx_id: u64) -> Result<(), StorageError> {
        Ok(())
    }
}
