//! DeduceTypeVisitor - 用于推导表达式类型的访问器
//! 对应 NebulaGraph DeduceTypeVisitor.h/.cpp 的功能

use std::collections::HashMap;
use crate::core::{Value, ValueTypeDef};
use crate::expressions::{Expression, ExpressionKind};
use crate::query::validator::ValidateContext;
use crate::storage::StorageEngine;
use crate::expressions::operations::{UnaryOp, BinaryOp};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeDeductionError {
    #[error("Semantic error: {0}")]
    SemanticError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
}

pub struct DeduceTypeVisitor<'a, S: StorageEngine> {
    /// 查询上下文
    storage: &'a S,
    /// 验证上下文
    validate_context: &'a ValidateContext,
    /// 输入列定义
    inputs: Vec<(String, ValueTypeDef)>,
    /// 图空间ID
    space: String,
    /// 当前状态
    status: Option<TypeDeductionError>,
    /// 推导出的类型
    type_: ValueTypeDef,
    /// VID类型
    vid_type: ValueTypeDef,
}

impl<'a, S: StorageEngine> DeduceTypeVisitor<'a, S> {
    pub fn new(
        storage: &'a S,
        validate_context: &'a ValidateContext,
        inputs: Vec<(String, ValueTypeDef)>,
        space: String,
    ) -> Self {
        let vid_type = ValueTypeDef::String; // 简化实现，实际应从空间配置获取

        Self {
            storage,
            validate_context,
            inputs,
            space,
            status: None,
            type_: ValueTypeDef::Empty,
            vid_type,
        }
    }

    pub fn ok(&self) -> bool {
        self.status.is_none()
    }

    pub fn status(&self) -> Option<&TypeDeductionError> {
        self.status.as_ref()
    }

    pub fn type_(&self) -> ValueTypeDef {
        self.type_.clone()
    }

    /// 推导表达式类型
    pub fn deduce_type(&mut self, expr: &Expression) -> Result<ValueTypeDef, TypeDeductionError> {
        self.visit(expr)?;
        Ok(self.type_.clone())
    }

    fn visit(&mut self, expr: &Expression) -> Result<(), TypeDeductionError> {
        match expr {
            Expression::Constant(value) => self.visit_constant(value),
            Expression::Unary { op, operand } => {
                self.visit(operand)?;
                self.visit_unary(op)
            }
            Expression::Binary { op, left, right } => {
                self.visit(left)?;
                let left_type = self.type_.clone();
                self.visit(right)?;
                let right_type = self.type_.clone();
                self.visit_binary(op, left_type, right_type)
            }
            Expression::Variable { name } => self.visit_variable(name),
            Expression::Property { entity, property } => {
                self.visit(entity)?;
                self.visit_property(property)  // property access result type depends on context
            }
            Expression::FunctionCall { name, args } => self.visit_function_call(name, args),
            Expression::List(items) => self.visit_list(items),
            Expression::Map(pairs) => self.visit_map_pairs(pairs),
            Expression::Set(items) => self.visit_set(items),
            Expression::Case { conditions, default } => self.visit_case(conditions, default),
            Expression::Vertex { name } => self.visit_vertex_name(name),
            Expression::Edge => self.visit_edge(),
            Expression::PathBuild { items } => self.visit_path_build(items),
            Expression::Aggregate { name, arg, distinct: _ } => {
                if let Some(arg_expr) = arg {
                    self.visit(arg_expr)?;
                }
                self.visit_aggregate(name)
            }
            Expression::ListComprehension { inner_var: _, collection, filter, mapping } => {
                self.visit(collection)?;
                if let Some(filter_expr) = filter {
                    self.visit(filter_expr)?;
                }
                if let Some(mapping_expr) = mapping {
                    self.visit(mapping_expr)?;
                }
                // List comprehension always returns a list
                self.type_ = ValueTypeDef::List;
                Ok(())
            }
        }
    }

    fn visit_constant(&mut self, value: &Value) -> Result<(), TypeDeductionError> {
        self.type_ = value.get_type();
        Ok(())
    }

    fn visit_unary(&mut self, op: &UnaryOp) -> Result<(), TypeDeductionError> {
        match op {
            UnaryOp::Plus => Ok(()),
            UnaryOp::Minus => {
                // 检查类型是否支持取负操作
                match &self.type_ {
                    ValueTypeDef::Int | ValueTypeDef::Float => Ok(()),
                    _ => {
                        let msg = format!("Cannot apply unary minus to type {:?}", self.type_);
                        self.status = Some(TypeDeductionError::SemanticError(msg));
                        Err(TypeDeductionError::SemanticError(msg))
                    }
                }
            }
            UnaryOp::Not => {
                // 检查类型是否支持取反操作
                if self.type_ == ValueTypeDef::Bool || self.type_ == ValueTypeDef::Empty || self.type_ == ValueTypeDef::Null {
                    Ok(())
                } else {
                    let msg = format!("Cannot apply unary not to type {:?}", self.type_);
                    self.status = Some(TypeDeductionError::SemanticError(msg));
                    Err(TypeDeductionError::SemanticError(msg))
                }
            }
            UnaryOp::IsNull => {
                self.type_ = ValueTypeDef::Bool;
                Ok(())
            }
            UnaryOp::IsNotNull => {
                self.type_ = ValueTypeDef::Bool;
                Ok(())
            }
            UnaryOp::IsEmpty => {
                self.type_ = ValueTypeDef::Bool;
                Ok(())
            }
            UnaryOp::IsNotEmpty => {
                self.type_ = ValueTypeDef::Bool;
                Ok(())
            }
        }
    }

    fn visit_binary(
        &mut self,
        op: &BinaryOp,
        left_type: ValueTypeDef,
        right_type: ValueTypeDef,
    ) -> Result<(), TypeDeductionError> {
        match op {
            BinaryOp::Add => {
                if left_type == ValueTypeDef::String && right_type == ValueTypeDef::String {
                    self.type_ = ValueTypeDef::String;
                } else if left_type == ValueTypeDef::Int && right_type == ValueTypeDef::Int {
                    self.type_ = ValueTypeDef::Int;
                } else if left_type == ValueTypeDef::Float && right_type == ValueTypeDef::Float {
                    self.type_ = ValueTypeDef::Float;
                } else if (left_type == ValueTypeDef::Int && right_type == ValueTypeDef::Float) ||
                          (left_type == ValueTypeDef::Float && right_type == ValueTypeDef::Int) {
                    self.type_ = ValueTypeDef::Float;
                } else {
                    let msg = format!("Cannot apply + operator to types {:?} and {:?}", left_type, right_type);
                    self.status = Some(TypeDeductionError::SemanticError(msg));
                    return Err(TypeDeductionError::SemanticError(msg));
                }
            }
            BinaryOp::Minus | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if left_type == ValueTypeDef::Int && right_type == ValueTypeDef::Int {
                    self.type_ = ValueTypeDef::Int;
                } else if left_type == ValueTypeDef::Float && right_type == ValueTypeDef::Float {
                    self.type_ = ValueTypeDef::Float;
                } else if (left_type == ValueTypeDef::Int && right_type == ValueTypeDef::Float) ||
                          (left_type == ValueTypeDef::Float && right_type == ValueTypeDef::Int) {
                    self.type_ = ValueTypeDef::Float;
                } else {
                    let op_str = match op {
                        BinaryOp::Minus => "Minus",
                        BinaryOp::Mul => "Multiply",
                        BinaryOp::Div => "Division",
                        BinaryOp::Mod => "Mod",
                        _ => "BinaryOp",
                    };
                    let msg = format!("Cannot apply {} operator to types {:?} and {:?}", op_str, left_type, right_type);
                    self.status = Some(TypeDeductionError::SemanticError(msg));
                    return Err(TypeDeductionError::SemanticError(msg));
                }
            }
            BinaryOp::EQ | BinaryOp::NE | BinaryOp::LT | BinaryOp::LE | BinaryOp::GT | BinaryOp::GE => {
                // 关系操作的结果类型是布尔值
                self.type_ = ValueTypeDef::Bool;
            }
            BinaryOp::And | BinaryOp::Or => {
                // 逻辑操作的结果类型是布尔值
                self.type_ = ValueTypeDef::Bool;
            }
            BinaryOp::In | BinaryOp::NotIn => {
                // 集合操作的结果类型是布尔值
                self.type_ = ValueTypeDef::Bool;
            }
        }
        Ok(())
    }

    fn visit_property(&mut self, _property: &String) -> Result<(), TypeDeductionError> {
        // 属性访问的结果类型需要根据上下文来确定
        // 这里简化实现，返回Empty类型
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    fn visit_function_call(&mut self, name: &String, args: &Vec<Expression>) -> Result<(), TypeDeductionError> {
        // 推导参数类型
        let mut arg_types = Vec::new();
        for arg in args {
            self.visit(arg)?;
            arg_types.push(self.type_.clone());
        }

        // 根据函数名确定返回类型
        match name.as_str() {
            "id" | "src" | "dst" | "none_direct_src" | "none_direct_dst" => {
                self.type_ = self.vid_type.clone();
            }
            "count" | "COUNT" => {
                self.type_ = ValueTypeDef::Int;
            }
            "avg" | "AVG" | "sum" | "SUM" => {
                self.type_ = ValueTypeDef::Float;
            }
            "max" | "MAX" | "min" | "MIN" => {
                // 返回参数类型
                if !arg_types.is_empty() {
                    self.type_ = arg_types[0].clone();
                }
            }
            "collect" | "COLLECT" => {
                self.type_ = ValueTypeDef::List;
            }
            "collect_set" | "COLLECT_SET" => {
                self.type_ = ValueTypeDef::Set;
            }
            _ => {
                // 对于其他函数，需要更复杂的类型推导
                self.type_ = ValueTypeDef::Empty;
            }
        }
        Ok(())
    }

    fn visit_aggregate(&mut self, name: &String) -> Result<(), TypeDeductionError> {
        match name.as_str().to_uppercase().as_str() {
            "COUNT" => {
                self.type_ = ValueTypeDef::Int;
            }
            "COLLECT" => {
                self.type_ = ValueTypeDef::List;
            }
            "COLLECT_SET" => {
                self.type_ = ValueTypeDef::Set;
            }
            "AVG" | "SUM" => {
                self.type_ = ValueTypeDef::Float;
            }
            "MAX" | "MIN" => {
                // 保持参数类型
            }
            _ => {
                // 其他聚合函数
                self.type_ = ValueTypeDef::Empty;
            }
        }
        Ok(())
    }

    fn visit_uuid(&mut self) -> Result<(), TypeDeductionError> {
        self.type_ = ValueTypeDef::String;
        Ok(())
    }

    fn visit_variable(&mut self, _name: &String) -> Result<(), TypeDeductionError> {
        // 变量表达式的结果类型不确定，使用Empty
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    fn visit_versioned_variable(&mut self) -> Result<(), TypeDeductionError> {
        // 版本化变量表达式的结果类型不确定，使用Empty
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    fn visit_list(&mut self, _items: &Vec<Expression>) -> Result<(), TypeDeductionError> {
        self.type_ = ValueTypeDef::List;
        Ok(())
    }

    fn visit_set(&mut self, _items: &Vec<Expression>) -> Result<(), TypeDeductionError> {
        self.type_ = ValueTypeDef::Set;
        Ok(())
    }

    fn visit_map(&mut self, _kvs: &HashMap<Expression, Expression>) -> Result<(), TypeDeductionError> {
        self.type_ = ValueTypeDef::Map;
        Ok(())
    }

    fn visit_label_tag_property(&mut self) -> Result<(), TypeDeductionError> {
        // 简化实现，返回Empty
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    fn visit_tag_property(&mut self, tag: &String, prop: &String) -> Result<(), TypeDeductionError> {
        // 在实际实现中，这里会查询标签的 schema 来确定属性类型
        // 简化实现，返回Empty
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    fn visit_edge_property(&mut self, edge: &String, prop: &String) -> Result<(), TypeDeductionError> {
        // 在实际实现中，这里会查询边的 schema 来确定属性类型
        // 简化实现，返回Empty
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    fn visit_input_property(&mut self, name: &String) -> Result<(), TypeDeductionError> {
        // 查找输入列
        for (col_name, col_type) in &self.inputs {
            if col_name == name {
                self.type_ = col_type.clone();
                return Ok(());
            }
        }
        
        let msg = format!("Property {} does not exist", name);
        let err = TypeDeductionError::SemanticError(msg.clone());
        self.status = Some(err.clone());
        Err(err)
    }

    fn visit_variable_property(&mut self, var: &String, prop: &String) -> Result<(), TypeDeductionError> {
        // 检查变量是否存在
        if !self.validate_context.exists_var(var) {
            let msg = format!("Variable {} does not exist", var);
            let err = TypeDeductionError::SemanticError(msg.clone());
            self.status = Some(err.clone());
            return Err(err);
        }
        
        // 在实际实现中，这里会查询变量的 schema 来确定属性类型
        // 简化实现，返回Empty
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    fn visit_dest_property(&mut self) -> Result<(), TypeDeductionError> {
        // 目标顶点属性，简化实现返回Empty
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    fn visit_source_property(&mut self) -> Result<(), TypeDeductionError> {
        // 源顶点属性，简化实现返回Empty
        self.type_ = ValueTypeDef::Empty;
        Ok(())
    }

    fn visit_vertex_name(&mut self, _name: &String) -> Result<(), TypeDeductionError> {
        self.type_ = ValueTypeDef::Vertex;
        Ok(())
    }

    fn visit_edge(&mut self) -> Result<(), TypeDeductionError> {
        self.type_ = ValueTypeDef::Edge;
        Ok(())
    }

    fn visit_path_build(&mut self, _items: &Vec<Expression>) -> Result<(), TypeDeductionError> {
        self.type_ = ValueTypeDef::Path;
        Ok(())
    }

}