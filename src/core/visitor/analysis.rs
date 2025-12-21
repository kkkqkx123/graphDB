//! 分析类访问者
//!
//! 这个模块提供了用于分析 Value 类型的访问者实现，包括表达式类型推导

use crate::core::error::{DBError, DBResult};
use crate::core::value::{
    DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue, Value,
    ValueTypeDef,
};
use crate::core::vertex_edge_path::{Edge, Path, Vertex};
use crate::core::visitor::core::{
    DefaultVisitorState, ValueVisitor, VisitorConfig, VisitorContext, VisitorCore, VisitorResult,
    VisitorState,
};
use std::collections::HashMap;

/// Value 类型分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeCategory {
    Empty,
    Null,
    Bool,
    Numeric,
    String,
    Temporal,
    GraphElement,
    Collection,
    Geography,
    Dataset,
}

/// 类型检查访问者 - 用于确定 Value 的类型分类
#[derive(Debug)]
pub struct TypeCheckerVisitor {
    category: Option<TypeCategory>, // 使用Option存储单个类型分类
    context: VisitorContext,
    state: DefaultVisitorState,
}

impl TypeCheckerVisitor {
    pub fn new() -> Self {
        Self {
            category: None,
            context: VisitorContext::new(VisitorConfig::default()),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn with_config(config: VisitorConfig) -> Self {
        Self {
            category: None,
            context: VisitorContext::new(config),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn categories(&self) -> Vec<TypeCategory> {
        self.category.iter().copied().collect()
    }

    pub fn has_category(&self, category: TypeCategory) -> bool {
        self.category == Some(category)
    }

    pub fn get_primary_category(&self) -> Option<TypeCategory> {
        self.category
    }

    pub fn get_type_name(&self) -> &'static str {
        match self.get_primary_category() {
            Some(TypeCategory::Empty) => "Empty",
            Some(TypeCategory::Null) => "Null",
            Some(TypeCategory::Bool) => "Bool",
            Some(TypeCategory::Numeric) => "Numeric",
            Some(TypeCategory::String) => "String",
            Some(TypeCategory::Temporal) => "Temporal",
            Some(TypeCategory::GraphElement) => "GraphElement",
            Some(TypeCategory::Collection) => "Collection",
            Some(TypeCategory::Geography) => "Geography",
            Some(TypeCategory::Dataset) => "Dataset",
            None => "Unknown",
        }
    }

    pub fn reset(&mut self) {
        self.category = None;
        self.state.reset();
    }

    /// 批量类型检查（优化实现）
    pub fn check_batch(&mut self, values: &[crate::core::value::Value]) -> Vec<TypeCategory> {
        let mut categories = Vec::new();

        for value in values {
            let type_def = value.get_type();
            let category = self.convert_to_category(&type_def);
            if !categories.contains(&category) {
                categories.push(category);
            }
        }

        categories
    }

    /// 转换为类型分类（使用共享逻辑）
    fn convert_to_category(&self, type_def: &crate::core::value::ValueTypeDef) -> TypeCategory {
        match type_def {
            crate::core::value::ValueTypeDef::Bool => TypeCategory::Bool,
            crate::core::value::ValueTypeDef::Int | crate::core::value::ValueTypeDef::Float => {
                TypeCategory::Numeric
            }
            crate::core::value::ValueTypeDef::String => TypeCategory::String,
            crate::core::value::ValueTypeDef::Date
            | crate::core::value::ValueTypeDef::Time
            | crate::core::value::ValueTypeDef::DateTime => TypeCategory::Temporal,
            crate::core::value::ValueTypeDef::Vertex
            | crate::core::value::ValueTypeDef::Edge
            | crate::core::value::ValueTypeDef::Path => TypeCategory::GraphElement,
            crate::core::value::ValueTypeDef::List
            | crate::core::value::ValueTypeDef::Map
            | crate::core::value::ValueTypeDef::Set => TypeCategory::Collection,
            crate::core::value::ValueTypeDef::Geography => TypeCategory::Geography,
            crate::core::value::ValueTypeDef::DataSet => TypeCategory::Dataset,
            crate::core::value::ValueTypeDef::Null => TypeCategory::Null,
            crate::core::value::ValueTypeDef::Empty => TypeCategory::Empty,
            _ => TypeCategory::Empty,
        }
    }

    /// 添加类型分类
    fn add_category(&mut self, category: TypeCategory) {
        self.category = Some(category);
    }
}

impl ValueVisitor for TypeCheckerVisitor {
    type Result = ();

    fn visit_bool(&mut self, _value: bool) -> Self::Result {
        self.add_category(TypeCategory::Bool);
    }

    fn visit_int(&mut self, _value: i64) -> Self::Result {
        self.add_category(TypeCategory::Numeric);
    }

    fn visit_float(&mut self, _value: f64) -> Self::Result {
        self.add_category(TypeCategory::Numeric);
    }

    fn visit_string(&mut self, _value: &str) -> Self::Result {
        self.add_category(TypeCategory::String);
    }

    fn visit_date(&mut self, _value: &DateValue) -> Self::Result {
        self.add_category(TypeCategory::Temporal);
    }

    fn visit_time(&mut self, _value: &TimeValue) -> Self::Result {
        self.add_category(TypeCategory::Temporal);
    }

    fn visit_datetime(&mut self, _value: &DateTimeValue) -> Self::Result {
        self.add_category(TypeCategory::Temporal);
    }

    fn visit_vertex(&mut self, _value: &Vertex) -> Self::Result {
        self.add_category(TypeCategory::GraphElement);
    }

    fn visit_edge(&mut self, _value: &Edge) -> Self::Result {
        self.add_category(TypeCategory::GraphElement);
    }

    fn visit_path(&mut self, _value: &Path) -> Self::Result {
        self.add_category(TypeCategory::GraphElement);
    }

    fn visit_list(&mut self, _value: &[Value]) -> Self::Result {
        self.add_category(TypeCategory::Collection);
    }

    fn visit_map(&mut self, _value: &HashMap<String, Value>) -> Self::Result {
        self.add_category(TypeCategory::Collection);
    }

    fn visit_set(&mut self, _value: &std::collections::HashSet<Value>) -> Self::Result {
        self.add_category(TypeCategory::Collection);
    }

    fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
        self.add_category(TypeCategory::Geography);
    }

    fn visit_duration(&mut self, _value: &DurationValue) -> Self::Result {
        self.add_category(TypeCategory::Temporal);
    }

    fn visit_dataset(&mut self, _value: &DataSet) -> Self::Result {
        self.add_category(TypeCategory::Dataset);
    }

    fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
        self.add_category(TypeCategory::Null);
    }

    fn visit_empty(&mut self) -> Self::Result {
        self.add_category(TypeCategory::Empty);
    }
}

impl VisitorCore for TypeCheckerVisitor {
    type Result = ();

    fn context(&self) -> &VisitorContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut VisitorContext {
        &mut self.context
    }

    fn state(&self) -> &dyn VisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut dyn VisitorState {
        &mut self.state
    }

    fn pre_visit(&mut self) -> VisitorResult<()> {
        self.state.inc_visit_count();
        if self.state.depth() > self.context.config().max_depth {
            return Err(crate::core::visitor::core::VisitorError::Validation(
                format!("访问深度超过限制: {}", self.context.config().max_depth),
            ));
        }
        Ok(())
    }

    fn post_visit(&mut self) -> VisitorResult<()> {
        Ok(())
    }
}

/// 复杂度分析访问者 - 分析 Value 的复杂度
#[derive(Debug)]
pub struct ComplexityAnalyzerVisitor {
    depth: usize,
    max_depth: usize,
    total_nodes: usize,
    container_nodes: usize,
    primitive_nodes: usize,
    context: VisitorContext,
    state: DefaultVisitorState,
}

impl ComplexityAnalyzerVisitor {
    pub fn new() -> Self {
        Self {
            depth: 0,
            max_depth: 0,
            total_nodes: 0,
            container_nodes: 0,
            primitive_nodes: 0,
            context: VisitorContext::new(VisitorConfig::default()),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn with_config(config: VisitorConfig) -> Self {
        Self {
            depth: 0,
            max_depth: 0,
            total_nodes: 0,
            container_nodes: 0,
            primitive_nodes: 0,
            context: VisitorContext::new(config),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn analyze(&self) -> ComplexityMetrics {
        ComplexityMetrics {
            depth: self.depth,
            total_nodes: self.total_nodes,
            container_nodes: self.container_nodes,
            primitive_nodes: self.primitive_nodes,
            complexity_ratio: if self.total_nodes > 0 {
                self.container_nodes as f64 / self.total_nodes as f64
            } else {
                0.0
            },
        }
    }

    pub fn reset(&mut self) {
        self.depth = 0;
        self.max_depth = 0;
        self.total_nodes = 0;
        self.container_nodes = 0;
        self.primitive_nodes = 0;
        self.state.reset();
    }

    fn update_depth(&mut self, new_depth: usize) {
        if new_depth > self.depth {
            self.depth = new_depth;
        }
    }
}

impl ValueVisitor for ComplexityAnalyzerVisitor {
    type Result = ();

    fn visit_bool(&mut self, _value: bool) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_int(&mut self, _value: i64) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_float(&mut self, _value: f64) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_string(&mut self, value: &str) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
        // 字符串的复杂度基于长度
        if value.len() > 100 {
            self.max_depth += 1;
        }
    }

    fn visit_date(&mut self, _value: &DateValue) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_time(&mut self, _value: &TimeValue) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_datetime(&mut self, _value: &DateTimeValue) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
        // 顶点的复杂度基于标签和属性数量
        let complexity = value.tags().len() + value.vertex_properties().len();
        if complexity > 5 {
            self.max_depth += 1;
        }
    }

    fn visit_edge(&mut self, value: &Edge) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
        // 边的复杂度基于属性数量
        if value.property_count() > 3 {
            self.max_depth += 1;
        }
    }

    fn visit_path(&mut self, value: &Path) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
        // 路径的复杂度基于长度
        if value.len() > 10 {
            self.max_depth += 1;
        }
    }

    fn visit_list(&mut self, value: &[Value]) -> Self::Result {
        self.container_nodes += 1;
        self.total_nodes += 1;
        self.update_depth(value.len());
    }

    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
        self.container_nodes += 1;
        self.total_nodes += 1;
        self.update_depth(value.len());
    }

    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
        self.container_nodes += 1;
        self.total_nodes += 1;
        self.update_depth(value.len());
    }

    fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_duration(&mut self, _value: &DurationValue) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
        self.container_nodes += 1;
        self.total_nodes += 1;
        self.update_depth(value.rows.len());
    }

    fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_empty(&mut self) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }
}

impl VisitorCore for ComplexityAnalyzerVisitor {
    type Result = ();

    fn context(&self) -> &VisitorContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut VisitorContext {
        &mut self.context
    }

    fn state(&self) -> &dyn VisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut dyn VisitorState {
        &mut self.state
    }

    fn pre_visit(&mut self) -> VisitorResult<()> {
        self.state.inc_visit_count();
        self.state.inc_depth();
        if self.state.depth() > self.context.config().max_depth {
            return Err(crate::core::visitor::core::VisitorError::Validation(
                format!("访问深度超过限制: {}", self.context.config().max_depth),
            ));
        }
        Ok(())
    }

    fn post_visit(&mut self) -> VisitorResult<()> {
        self.state.dec_depth();
        Ok(())
    }
}

/// 复杂度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
}

/// 复杂度指标
#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    pub depth: usize,
    pub total_nodes: usize,
    pub container_nodes: usize,
    pub primitive_nodes: usize,
    pub complexity_ratio: f64,
}

impl ComplexityMetrics {
    pub fn is_simple(&self) -> bool {
        self.complexity_ratio < 0.3 && self.depth < 5
    }

    pub fn is_complex(&self) -> bool {
        self.complexity_ratio > 0.7 || self.depth > 10
    }

    pub fn is_moderate(&self) -> bool {
        !self.is_simple() && !self.is_complex()
    }

    pub fn get_level(&self) -> ComplexityLevel {
        if self.is_simple() {
            ComplexityLevel::Simple
        } else if self.is_complex() {
            ComplexityLevel::Complex
        } else {
            ComplexityLevel::Moderate
        }
    }
}

/// 表达式类型推导访问者（基于现有TypeCheckerVisitor）
/// 整合了原有的deduce_type_visitor.rs功能
#[derive(Debug)]
pub struct ExpressionTypeDeductionVisitor {
    type_checker: TypeCheckerVisitor,
    current_type: ValueTypeDef,
    variable_scope: HashMap<String, ValueTypeDef>,
    context: VisitorContext,
    state: DefaultVisitorState,
}

impl ExpressionTypeDeductionVisitor {
    pub fn new() -> Self {
        Self {
            type_checker: TypeCheckerVisitor::new(),
            current_type: ValueTypeDef::Empty,
            variable_scope: HashMap::new(),
            context: VisitorContext::new(VisitorConfig::default()),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn with_config(config: VisitorConfig) -> Self {
        Self {
            type_checker: TypeCheckerVisitor::with_config(config.clone()),
            current_type: ValueTypeDef::Empty,
            variable_scope: HashMap::new(),
            context: VisitorContext::new(config),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn with_variables(variables: HashMap<String, ValueTypeDef>) -> Self {
        Self {
            type_checker: TypeCheckerVisitor::new(),
            current_type: ValueTypeDef::Empty,
            variable_scope: variables,
            context: VisitorContext::new(VisitorConfig::default()),
            state: DefaultVisitorState::new(),
        }
    }

    /// 推导表达式类型（主要接口）
    pub fn deduce_type(&mut self, expr: &crate::expression::Expression) -> DBResult<ValueTypeDef> {
        self.pre_visit()?;

        // 表达式特定的类型推导逻辑
        let result = self.visit_expression(expr)?;

        self.post_visit()?;
        Ok(result)
    }

    /// 访问表达式（核心逻辑）
    fn visit_expression(&mut self, expr: &crate::expression::Expression) -> DBResult<ValueTypeDef> {
        use crate::expression::Expression;

        match expr {
            Expression::Literal(value) => self.visit_literal(value),
            Expression::Variable(name) => self.visit_variable(name),
            Expression::Property { object, property } => {
                self.visit_expression(object)?;
                // 属性访问返回Empty类型（实际类型应该查询Schema）
                self.current_type = ValueTypeDef::Empty;
                Ok(ValueTypeDef::Empty)
            }
            Expression::Function { name, args } => self.visit_function_call(name, args),
            Expression::Binary { left, op, right } => {
                let left_type = self.visit_expression(left)?;
                let right_type = self.visit_expression(right)?;
                self.visit_binary(op, left_type, right_type)
            }
            Expression::Unary { op, operand } => {
                self.visit_expression(operand)?;
                self.visit_unary(op)
            }
            Expression::List(items) => {
                for item in items {
                    self.visit_expression(item)?;
                }
                self.current_type = ValueTypeDef::List;
                Ok(ValueTypeDef::List)
            }
            Expression::Map(pairs) => {
                for (_, expr) in pairs {
                    self.visit_expression(expr)?;
                }
                self.current_type = ValueTypeDef::Map;
                Ok(ValueTypeDef::Map)
            }
            // 其他表达式类型的处理...
            _ => {
                self.current_type = ValueTypeDef::Empty;
                Ok(ValueTypeDef::Empty)
            }
        }
    }

    fn visit_literal(&mut self, value: &crate::expression::LiteralValue) -> DBResult<ValueTypeDef> {
        use crate::expression::LiteralValue;

        self.current_type = match value {
            LiteralValue::Bool(_) => ValueTypeDef::Bool,
            LiteralValue::Int(_) => ValueTypeDef::Int,
            LiteralValue::Float(_) => ValueTypeDef::Float,
            LiteralValue::String(_) => ValueTypeDef::String,
            LiteralValue::Null => ValueTypeDef::Null,
        };
        Ok(self.current_type.clone())
    }

    fn visit_variable(&mut self, name: &str) -> DBResult<ValueTypeDef> {
        if let Some(var_type) = self.variable_scope.get(name) {
            self.current_type = var_type.clone();
            Ok(var_type.clone())
        } else {
            Err(DBError::TypeDeduction(format!("变量 {} 不存在", name)))
        }
    }

    fn visit_function_call(
        &mut self,
        name: &str,
        args: &[crate::expression::Expression],
    ) -> DBResult<ValueTypeDef> {
        // 推导参数类型
        let mut arg_types = Vec::new();
        for arg in args {
            let arg_type = self.visit_expression(arg)?;
            arg_types.push(arg_type);
        }

        // 根据函数名确定返回类型
        let name_upper = name.to_uppercase();
        self.current_type = match name_upper.as_str() {
            // ID提取函数
            "ID" | "SRC" | "DST" => ValueTypeDef::String,
            // 聚合函数
            "COUNT" => ValueTypeDef::Int,
            "AVG" | "SUM" => ValueTypeDef::Float,
            "MAX" | "MIN" => {
                if arg_types.is_empty() {
                    ValueTypeDef::Empty
                } else {
                    arg_types[0].clone()
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
        Ok(self.current_type.clone())
    }

    fn visit_binary(
        &mut self,
        op: &crate::expression::BinaryOperator,
        left_type: ValueTypeDef,
        right_type: ValueTypeDef,
    ) -> DBResult<ValueTypeDef> {
        use crate::expression::BinaryOperator;

        self.current_type = match op {
            BinaryOperator::Add => {
                if left_type == ValueTypeDef::String && right_type == ValueTypeDef::String {
                    ValueTypeDef::String
                } else if left_type == ValueTypeDef::Int && right_type == ValueTypeDef::Int {
                    ValueTypeDef::Int
                } else if left_type == ValueTypeDef::Float && right_type == ValueTypeDef::Float {
                    ValueTypeDef::Float
                } else if (left_type == ValueTypeDef::Int && right_type == ValueTypeDef::Float)
                    || (left_type == ValueTypeDef::Float && right_type == ValueTypeDef::Int)
                {
                    ValueTypeDef::Float
                } else if is_superior_type(&left_type) || is_superior_type(&right_type) {
                    // NULL或EMPTY类型兼容任何类型
                    if is_superior_type(&left_type) {
                        right_type
                    } else {
                        left_type
                    }
                } else {
                    return Err(DBError::TypeDeduction(format!(
                        "无法对类型 {:?} 和 {:?} 执行加法操作",
                        left_type, right_type
                    )));
                }
            }
            BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo => {
                if left_type == ValueTypeDef::Int && right_type == ValueTypeDef::Int {
                    ValueTypeDef::Int
                } else if left_type == ValueTypeDef::Float && right_type == ValueTypeDef::Float {
                    ValueTypeDef::Float
                } else if (left_type == ValueTypeDef::Int && right_type == ValueTypeDef::Float)
                    || (left_type == ValueTypeDef::Float && right_type == ValueTypeDef::Int)
                {
                    ValueTypeDef::Float
                } else if is_superior_type(&left_type) || is_superior_type(&right_type) {
                    // NULL或EMPTY类型兼容任何类型
                    if is_superior_type(&left_type) {
                        right_type
                    } else {
                        left_type
                    }
                } else {
                    let op_name = match op {
                        BinaryOperator::Subtract => "减法",
                        BinaryOperator::Multiply => "乘法",
                        BinaryOperator::Divide => "除法",
                        BinaryOperator::Modulo => "模运算",
                        _ => "数学运算",
                    };
                    return Err(DBError::TypeDeduction(format!(
                        "无法对类型 {:?} 和 {:?} 执行{}操作",
                        left_type, right_type, op_name
                    )));
                }
            }
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual => {
                // 关系操作的结果类型是布尔值
                ValueTypeDef::Bool
            }
            BinaryOperator::And | BinaryOperator::Or => {
                // 逻辑操作的结果类型是布尔值
                ValueTypeDef::Bool
            }
            BinaryOperator::In => {
                // 集合操作的结果类型是布尔值
                ValueTypeDef::Bool
            }
            _ => {
                // 其他操作默认返回布尔值
                ValueTypeDef::Bool
            }
        };
        Ok(self.current_type.clone())
    }

    fn visit_unary(&mut self, op: &crate::expression::UnaryOperator) -> DBResult<ValueTypeDef> {
        use crate::expression::UnaryOperator;

        match op {
            UnaryOperator::Not => {
                self.current_type = ValueTypeDef::Bool;
                Ok(ValueTypeDef::Bool)
            }
            UnaryOperator::Plus | UnaryOperator::Minus => {
                // 一元加减法保持类型不变
                Ok(self.current_type.clone())
            }
            // 其他一元操作符的处理...
            _ => Ok(self.current_type.clone()),
        }
    }

    /// 获取当前推导的类型
    pub fn current_type(&self) -> &ValueTypeDef {
        &self.current_type
    }

    /// 设置变量作用域
    pub fn set_variable_scope(&mut self, variables: HashMap<String, ValueTypeDef>) {
        self.variable_scope = variables;
    }

    /// 添加变量到作用域
    pub fn add_variable(&mut self, name: String, type_def: ValueTypeDef) {
        self.variable_scope.insert(name, type_def);
    }

    /// 重置访问者状态
    pub fn reset(&mut self) {
        self.type_checker.reset();
        self.current_type = ValueTypeDef::Empty;
        self.variable_scope.clear();
        self.state.reset();
    }
}

impl VisitorCore for ExpressionTypeDeductionVisitor {
    type Result = ();

    fn context(&self) -> &VisitorContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut VisitorContext {
        &mut self.context
    }

    fn state(&self) -> &dyn VisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut dyn VisitorState {
        &mut self.state
    }

    fn pre_visit(&mut self) -> VisitorResult<()> {
        self.state.inc_visit_count();
        if self.state.depth() > self.context.config().max_depth {
            return Err(DBError::Validation(format!(
                "访问深度超过限制: {}",
                self.context.config().max_depth
            )));
        }
        Ok(())
    }

    fn post_visit(&mut self) -> VisitorResult<()> {
        Ok(())
    }
}

/// 检查类型是否为"优越类型"
/// 优越类型包括NULL和EMPTY，它们可以与任何类型兼容
fn is_superior_type(type_: &ValueTypeDef) -> bool {
    matches!(type_, ValueTypeDef::Null | ValueTypeDef::Empty)
}

/// 统一的类型兼容性检查（复用现有逻辑）
pub fn are_types_compatible(type1: &ValueTypeDef, type2: &ValueTypeDef) -> bool {
    // 使用TypeCheckerVisitor中的类型分类逻辑
    let category1 = convert_to_category(type1);
    let category2 = convert_to_category(type2);

    // 简化的兼容性规则
    match (category1, category2) {
        (TypeCategory::Numeric, TypeCategory::Numeric) => true,
        (TypeCategory::String, TypeCategory::String) => true,
        (TypeCategory::Null, _) | (_, TypeCategory::Null) => true,
        (TypeCategory::Empty, _) | (_, TypeCategory::Empty) => true,
        _ => category1 == category2,
    }
}

/// 转换为类型分类（从TypeCheckerVisitor提取的共享逻辑）
fn convert_to_category(type_def: &ValueTypeDef) -> TypeCategory {
    match type_def {
        ValueTypeDef::Bool => TypeCategory::Bool,
        ValueTypeDef::Int
        | ValueTypeDef::Float
        | ValueTypeDef::IntRange
        | ValueTypeDef::FloatRange => TypeCategory::Numeric,
        ValueTypeDef::String | ValueTypeDef::StringRange => TypeCategory::String,
        ValueTypeDef::Date
        | ValueTypeDef::Time
        | ValueTypeDef::DateTime
        | ValueTypeDef::Duration => TypeCategory::Temporal,
        ValueTypeDef::Vertex | ValueTypeDef::Edge | ValueTypeDef::Path => {
            TypeCategory::GraphElement
        }
        ValueTypeDef::List | ValueTypeDef::Map | ValueTypeDef::Set => TypeCategory::Collection,
        ValueTypeDef::Geography => TypeCategory::Geography,
        ValueTypeDef::DataSet => TypeCategory::Dataset,
        ValueTypeDef::Null => TypeCategory::Null,
        ValueTypeDef::Empty => TypeCategory::Empty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_type_checker_visitor() {
        let mut visitor = TypeCheckerVisitor::new();

        let int_value = Value::Int(42);
        int_value.accept(&mut visitor);
        assert!(visitor.has_category(TypeCategory::Numeric));
        assert_eq!(visitor.get_type_name(), "Numeric");

        visitor.reset();
        let string_value = Value::String("test".to_string());
        string_value.accept(&mut visitor);
        assert!(visitor.has_category(TypeCategory::String));
        assert_eq!(visitor.get_type_name(), "String");
    }

    #[test]
    fn test_type_checker_visitor_batch() {
        let mut visitor = TypeCheckerVisitor::new();

        let values = vec![
            Value::Int(42),
            Value::Float(3.14),
            Value::String("test".to_string()),
            Value::Bool(true),
        ];

        let categories = visitor.check_batch(&values);
        assert!(categories.contains(&TypeCategory::Numeric));
        assert!(categories.contains(&TypeCategory::String));
        assert!(categories.contains(&TypeCategory::Bool));
        assert_eq!(categories.len(), 3); // Numeric, String, Bool
    }

    #[test]
    fn test_complexity_analyzer() {
        let mut visitor = ComplexityAnalyzerVisitor::new();

        let simple_value = Value::Int(42);
        simple_value.accept(&mut visitor);
        let metrics = visitor.analyze();
        assert!(metrics.is_simple());

        visitor.reset();
        let complex_value = Value::List(vec![
            Value::Int(1),
            Value::String("test".to_string()),
            Value::Map(std::collections::HashMap::from([(
                "key".to_string(),
                Value::Bool(true),
            )])),
        ]);
        complex_value.accept(&mut visitor);
        let metrics = visitor.analyze();
        assert!(metrics.is_moderate());
    }

    #[test]
    fn test_visitor_core_integration() {
        let config = VisitorConfig::new().with_max_depth(5);
        let mut visitor = TypeCheckerVisitor::with_config(config);

        // 测试VisitorCore方法
        assert!(visitor.should_continue());
        assert_eq!(visitor.state().depth(), 0);

        visitor.state_mut().inc_depth();
        assert_eq!(visitor.state().depth(), 1);

        visitor.reset();
        assert_eq!(visitor.state().depth(), 0);

        // 测试原始ValueVisitor功能
        let value = Value::Int(42);
        value.accept(&mut visitor);
        assert!(visitor.has_category(TypeCategory::Numeric));
    }

    #[test]
    fn test_complexity_analyzer_with_config() {
        let config = VisitorConfig::new().with_max_depth(3);
        let mut visitor = ComplexityAnalyzerVisitor::with_config(config);

        assert_eq!(visitor.context().config().max_depth, 3);

        let simple_value = Value::Int(42);
        simple_value.accept(&mut visitor);
        let metrics = visitor.analyze();
        assert!(metrics.is_simple());
        assert_eq!(visitor.state().visit_count(), 1);
    }

    #[cfg(test)]
    mod expression_tests {
        use super::*;
        use crate::expression::{BinaryOperator, Expression, LiteralValue};

        #[test]
        fn test_expression_type_deduction() {
            let mut visitor = ExpressionTypeDeductionVisitor::new();

            // 测试字面量
            let int_expr = Expression::Literal(LiteralValue::Int(42));
            let result = visitor.deduce_type(&int_expr).unwrap();
            assert_eq!(result, ValueTypeDef::Int);

            // 测试二元运算
            let add_expr = Expression::Binary {
                left: Box::new(Expression::Literal(LiteralValue::Int(10))),
                op: BinaryOperator::Add,
                right: Box::new(Expression::Literal(LiteralValue::Int(20))),
            };
            let result = visitor.deduce_type(&add_expr).unwrap();
            assert_eq!(result, ValueTypeDef::Int);
        }

        #[test]
        fn test_type_compatibility() {
            assert!(are_types_compatible(&ValueTypeDef::Int, &ValueTypeDef::Int));
            assert!(are_types_compatible(
                &ValueTypeDef::Null,
                &ValueTypeDef::String
            ));
            assert!(are_types_compatible(
                &ValueTypeDef::Empty,
                &ValueTypeDef::Int
            ));
            assert!(!are_types_compatible(
                &ValueTypeDef::Int,
                &ValueTypeDef::String
            ));
        }

        #[test]
        fn test_variable_scope() {
            let mut variables = HashMap::new();
            variables.insert("x".to_string(), ValueTypeDef::Int);
            variables.insert("y".to_string(), ValueTypeDef::String);

            let mut visitor = ExpressionTypeDeductionVisitor::with_variables(variables);

            let var_expr = Expression::Variable("x".to_string());
            let result = visitor.deduce_type(&var_expr).unwrap();
            assert_eq!(result, ValueTypeDef::Int);
        }
    }
}
