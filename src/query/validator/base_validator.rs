//! 基础验证器
//! 对应 NebulaGraph Validator.h/.cpp 的功能
//! 所有验证器的基类
//!
//! 验证生命周期：
//! 1. space_chosen() - 检查是否选择了图空间
//! 2. validate_impl() - 子类实现具体验证逻辑
//! 3. check_permission() - 权限检查
//! 4. to_plan() - 转换为执行计划

use std::sync::Arc;

use crate::core::error::{DBError, DBResult, QueryError, ValidationError as CoreValidationError, ValidationErrorType};
use crate::core::{Expression, Value};
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::context::validate::ValidationContext;
use crate::query::context::validate::schema::SchemaProvider;
use crate::query::parser::ast::Stmt;

pub struct Validator {
    context: Option<ValidationContext>,
    schema_manager: Option<Arc<dyn SchemaProvider>>,
    input_var_name: String,
    no_space_required: bool,
    outputs: Vec<ColumnDef>,
    inputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub type_: ValueType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Unknown,
    Bool,
    Int,
    Float,
    String,
    Date,
    Time,
    DateTime,
    Vertex,
    Edge,
    Path,
    List,
    Map,
    Set,
    Null,
}

#[derive(Debug, Clone, Default)]
pub struct ExpressionProps {
    pub input_props: Vec<InputProperty>,
    pub var_props: Vec<VarProperty>,
    pub tag_props: Vec<TagProperty>,
    pub edge_props: Vec<EdgeProperty>,
}

#[derive(Debug, Clone)]
pub struct InputProperty {
    pub prop_name: String,
    pub type_: ValueType,
}

#[derive(Debug, Clone)]
pub struct VarProperty {
    pub var_name: String,
    pub prop_name: String,
    pub type_: ValueType,
}

#[derive(Debug, Clone)]
pub struct TagProperty {
    pub tag_name: String,
    pub prop_name: String,
    pub type_: ValueType,
}

#[derive(Debug, Clone)]
pub struct EdgeProperty {
    pub edge_type: i32,
    pub prop_name: String,
    pub type_: ValueType,
}

impl Validator {
    pub fn new() -> Self {
        Self {
            context: Some(ValidationContext::new()),
            schema_manager: None,
            input_var_name: String::new(),
            no_space_required: false,
            outputs: Vec::new(),
            inputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    /// 创建带有 SchemaManager 的验证器
    pub fn with_schema_manager(schema_manager: Arc<dyn SchemaProvider>) -> Self {
        let mut context = ValidationContext::new();
        context.set_schema_manager(schema_manager.clone());
        Self {
            context: Some(context),
            schema_manager: Some(schema_manager),
            input_var_name: String::new(),
            no_space_required: false,
            outputs: Vec::new(),
            inputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    /// 设置 SchemaManager
    pub fn set_schema_manager(&mut self, schema_manager: Arc<dyn SchemaProvider>) {
        self.schema_manager = Some(schema_manager.clone());
        if let Some(ref mut ctx) = self.context {
            ctx.set_schema_manager(schema_manager);
        }
    }

    /// 获取 SchemaManager
    pub fn schema_manager(&self) -> Option<&Arc<dyn SchemaProvider>> {
        self.schema_manager.as_ref()
    }

    pub fn with_context(context: ValidationContext) -> Self {
        Self {
            context: Some(context),
            schema_manager: None,
            input_var_name: String::new(),
            no_space_required: false,
            outputs: Vec::new(),
            inputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    pub fn validate_with_ast_context(
        &mut self,
        query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> DBResult<()> {
        self.outputs.clear();
        self.inputs.clear();
        self.expr_props = ExpressionProps::default();
        self.user_defined_vars.clear();

        self.validate_lifecycle_with_ast(query_context, ast)?;

        for output in &self.outputs {
            ast.add_output(output.name.clone(), output.type_.clone());
        }

        for input in &self.inputs {
            ast.add_input(input.name.clone(), input.type_.clone());
        }

        let validation_errors = self.get_validation_errors();
        for error in validation_errors {
            ast.add_validation_error(error.clone());
        }

        if ast.has_validation_errors() {
            let errors = ast.validation_errors();
            let first_error = errors.first();
            if let Some(error) = first_error {
                return Err(DBError::Query(QueryError::InvalidQuery(format!(
                    "验证失败: {}",
                    error.message
                ))));
            }
        }

        Ok(())
    }

    fn validate_lifecycle_with_ast(
        &mut self,
        query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Result<(), CoreValidationError> {
        let no_space_required = self.no_space_required || Self::is_global_statement(ast);
        if !no_space_required && !self.space_chosen_in_ast(ast) {
            return Err(CoreValidationError::new(
                "No space selected. Use `USE <space>` to select a graph space first.".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        self.validate_impl_with_ast(query_context, ast)?;

        let errors = self.get_validation_errors();
        if let Some(first_error) = errors.first() {
            return Err(first_error.clone());
        }

        self.check_permission()?;

        self.to_plan_with_ast(ast)?;

        Ok(())
    }

    fn space_chosen_in_ast(&self, ast: &AstContext) -> bool {
        ast.space().space_id.is_some()
    }

    fn is_global_statement(ast: &AstContext) -> bool {
        let stmt_type = ast.statement_type();
        if matches!(
            stmt_type,
            "CREATE_USER" | "ALTER_USER" | "DROP_USER" | "CHANGE_PASSWORD"
                | "SHOW_SPACES" | "DESC_SPACE"
                | "SHOW_USERS" | "DESC_USER"
                | "USE"  // USE 语句是全局语句，不需要预先选择空间
        ) {
            return true;
        }
        
        // 检查 CREATE 语句是否是 CREATE SPACE
        if stmt_type == "CREATE" {
            if let Some(ref stmt) = ast.sentence() {
                if let crate::query::parser::ast::Stmt::Create(create_stmt) = stmt {
                    if let crate::query::parser::ast::stmt::CreateTarget::Space { .. } = create_stmt.target {
                        return true;
                    }
                }
            }
        }
        
        // 检查 DROP 语句是否是 DROP SPACE
        if stmt_type == "DROP" {
            if let Some(ref stmt) = ast.sentence() {
                if let crate::query::parser::ast::Stmt::Drop(drop_stmt) = stmt {
                    if let crate::query::parser::ast::stmt::DropTarget::Space(_) = drop_stmt.target {
                        return true;
                    }
                }
            }
        }
        
        false
    }

    fn validate_impl_with_ast(
        &mut self,
        _query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Result<(), CoreValidationError> {
        if let Some(ref stmt) = ast.sentence() {
            self.validate_statement_with_ast(stmt, ast)?;
        }
        Ok(())
    }

    fn validate_statement_with_ast(
        &mut self,
        stmt: &Stmt,
        ast: &AstContext,
    ) -> Result<(), CoreValidationError> {
        use crate::query::parser::ast::Stmt::*;
        match stmt {
            Match(match_stmt) => {
                self.validate_match_stmt(match_stmt)?;
            }
            Go(go_stmt) => {
                self.validate_go_stmt(go_stmt)?;
            }
            Fetch(fetch_stmt) => {
                self.validate_fetch_stmt(fetch_stmt)?;
            }
            Lookup(lookup_stmt) => {
                self.validate_lookup_stmt(lookup_stmt)?;
            }
            FindPath(find_path_stmt) => {
                self.validate_find_path_stmt(find_path_stmt)?;
            }
            Subgraph(subgraph_stmt) => {
                self.validate_subgraph_stmt(subgraph_stmt)?;
            }
            Insert(insert_stmt) => {
                self.validate_insert_stmt(insert_stmt)?;
            }
            Delete(delete_stmt) => {
                self.validate_delete_stmt(delete_stmt)?;
            }
            Update(update_stmt) => {
                self.validate_update_stmt(update_stmt)?;
            }
            Create(create_stmt) => {
                self.validate_create_stmt(create_stmt)?;
            }
            Drop(drop_stmt) => {
                self.validate_drop_stmt(drop_stmt)?;
            }
            Alter(alter_stmt) => {
                self.validate_alter_stmt(alter_stmt)?;
            }
            Use(use_stmt) => {
                self.validate_use_stmt(use_stmt)?;
            }
            Pipe(pipe_stmt) => {
                self.validate_pipe_stmt(pipe_stmt, ast)?;
            }
            Yield(yield_stmt) => {
                self.validate_yield_stmt(yield_stmt)?;
            }
            Unwind(unwind_stmt) => {
                self.validate_unwind_stmt(unwind_stmt)?;
            }
            Set(set_stmt) => {
                self.validate_set_stmt(set_stmt)?;
            }
            _ => {
                // 其他语句类型暂未实现详细验证
            }
        }
        Ok(())
    }

    fn validate_match_stmt(
        &mut self,
        match_stmt: &crate::query::parser::ast::stmt::MatchStmt,
    ) -> Result<(), CoreValidationError> {
        // 使用 MatchValidator 进行详细验证
        use super::match_validator::MatchValidator;
        
        let validation_context = self.context.take().unwrap_or_default();
        let mut match_validator = MatchValidator::new(validation_context);
        
        match match_validator.validate_match_statement(match_stmt) {
            Ok(()) => {
                // 验证成功，更新上下文
                self.context = Some(match_validator.context().clone());
                Ok(())
            }
            Err(e) => {
                // 验证失败，恢复上下文并返回错误
                self.context = Some(match_validator.context().clone());
                Err(CoreValidationError::new(
                    e.message,
                    ValidationErrorType::SemanticError,
                ))
            }
        }
    }

    fn validate_go_stmt(
        &mut self,
        go_stmt: &crate::query::parser::ast::stmt::GoStmt,
    ) -> Result<(), CoreValidationError> {
        // 使用 GoValidator 进行详细验证
        use super::go_validator::GoValidator;
        
        let validation_context = self.context.take().unwrap_or_default();
        let mut go_validator = GoValidator::new(validation_context);
        
        match go_validator.validate_from_stmt(go_stmt) {
            Ok(()) => {
                // 验证成功，更新上下文
                self.context = Some(go_validator.base_context().clone());
                Ok(())
            }
            Err(e) => {
                // 验证失败，恢复上下文并返回错误
                self.context = Some(go_validator.base_context().clone());
                Err(CoreValidationError::new(
                    e.message,
                    ValidationErrorType::SemanticError,
                ))
            }
        }
    }

    fn validate_fetch_stmt(
        &mut self,
        fetch_stmt: &crate::query::parser::ast::stmt::FetchStmt,
    ) -> Result<(), CoreValidationError> {
        // 验证 FETCH 语句的基本结构
        match &fetch_stmt.target {
            crate::query::parser::ast::stmt::FetchTarget::Vertices { ids, .. } => {
                if ids.is_empty() {
                    return Err(CoreValidationError::new(
                        "FETCH VERTICES 必须指定至少一个顶点 ID".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            crate::query::parser::ast::stmt::FetchTarget::Edges { .. } => {
                // TODO: 验证边的源顶点、目标顶点、边类型
            }
        }
        Ok(())
    }

    fn validate_lookup_stmt(
        &mut self,
        lookup_stmt: &crate::query::parser::ast::stmt::LookupStmt,
    ) -> Result<(), CoreValidationError> {
        // 验证 LOOKUP 语句的基本结构
        match &lookup_stmt.target {
            crate::query::parser::ast::stmt::LookupTarget::Tag(tag_name) => {
                if tag_name.is_empty() {
                    return Err(CoreValidationError::new(
                        "LOOKUP ON TAG 必须指定标签名".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            crate::query::parser::ast::stmt::LookupTarget::Edge(edge_name) => {
                if edge_name.is_empty() {
                    return Err(CoreValidationError::new(
                        "LOOKUP ON EDGE 必须指定边类型名".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        // 验证 WHERE 子句（如果存在）
        if let Some(ref where_clause) = lookup_stmt.where_clause {
            self.validate_where_expression(where_clause)?;
        }

        Ok(())
    }

    fn validate_find_path_stmt(
        &mut self,
        find_path_stmt: &crate::query::parser::ast::stmt::FindPathStmt,
    ) -> Result<(), CoreValidationError> {
        // 验证 FIND PATH 语句的基本结构
        // 1. 验证 FROM 子句不为空
        if find_path_stmt.from.vertices.is_empty() {
            return Err(CoreValidationError::new(
                "FIND PATH 必须指定起始顶点".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 验证目标顶点
        match &find_path_stmt.to {
            Expression::Variable(_) | Expression::List(_) => {}
            _ => {
                // 其他表达式类型也允许，但可能不是预期用法
            }
        }

        // 3. 验证最大步数（如果指定）
        if let Some(max_steps) = find_path_stmt.max_steps {
            if max_steps == 0 {
                return Err(CoreValidationError::new(
                    "FIND PATH 的最大步数必须大于 0".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            if max_steps > 100 {
                // 警告：步数过大可能影响性能
            }
        }

        // 4. 验证 LIMIT（如果指定）
        if let Some(limit) = find_path_stmt.limit {
            if limit == 0 {
                return Err(CoreValidationError::new(
                    "FIND PATH 的 LIMIT 必须大于 0".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        // 5. 验证 OFFSET（如果指定）
        if let Some(offset) = find_path_stmt.offset {
            if let Some(limit) = find_path_stmt.limit {
                if offset >= limit {
                    return Err(CoreValidationError::new(
                        format!("FIND PATH 的 OFFSET ({}) 必须小于 LIMIT ({})", offset, limit),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        // 6. 验证 WHERE 子句（如果存在）
        if let Some(ref where_clause) = find_path_stmt.where_clause {
            self.validate_where_expression(where_clause)?;
        }

        Ok(())
    }

    fn validate_subgraph_stmt(
        &mut self,
        subgraph_stmt: &crate::query::parser::ast::stmt::SubgraphStmt,
    ) -> Result<(), CoreValidationError> {
        // 验证 GET SUBGRAPH 语句的基本结构
        // 1. 验证 FROM 子句不为空
        if subgraph_stmt.from.vertices.is_empty() {
            return Err(CoreValidationError::new(
                "GET SUBGRAPH 必须指定起始顶点".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 验证步数
        match &subgraph_stmt.steps {
            crate::query::parser::ast::stmt::Steps::Fixed(steps) => {
                if *steps == 0 {
                    return Err(CoreValidationError::new(
                        "GET SUBGRAPH 的步数必须大于 0".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                if *steps > 10 {
                    // 警告：步数过大可能导致子图过大
                }
            }
            crate::query::parser::ast::stmt::Steps::Range { min, max } => {
                if *min > *max {
                    return Err(CoreValidationError::new(
                        format!("GET SUBGRAPH 的最小步数 ({}) 不能大于最大步数 ({})", min, max),
                        ValidationErrorType::SemanticError,
                    ));
                }
                if *max > 10 {
                    // 警告：步数过大可能导致子图过大
                }
            }
            crate::query::parser::ast::stmt::Steps::Variable(_) => {
                // 变量步数，运行时验证
            }
        }

        // 3. 验证 OVER 子句（如果指定）
        if let Some(ref over) = subgraph_stmt.over {
            if over.edge_types.is_empty() {
                return Err(CoreValidationError::new(
                    "GET SUBGRAPH 的 OVER 子句必须指定至少一个边类型".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        // 4. 验证 WHERE 子句（如果存在）
        if let Some(ref where_clause) = subgraph_stmt.where_clause {
            self.validate_where_expression(where_clause)?;
        }

        Ok(())
    }

    fn validate_insert_stmt(
        &mut self,
        insert_stmt: &crate::query::parser::ast::stmt::InsertStmt,
    ) -> Result<(), CoreValidationError> {
        // 验证 INSERT 语句的基本结构
        match &insert_stmt.target {
            crate::query::parser::ast::stmt::InsertTarget::Vertices { tags, values } => {
                if tags.is_empty() {
                    return Err(CoreValidationError::new(
                        "INSERT VERTICES 必须指定至少一个 Tag".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                if values.is_empty() {
                    return Err(CoreValidationError::new(
                        "INSERT VERTICES 必须包含至少一个值行".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            crate::query::parser::ast::stmt::InsertTarget::Edge { edge_name, edges, .. } => {
                if edge_name.is_empty() {
                    return Err(CoreValidationError::new(
                        "INSERT EDGE 必须指定边类型".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                if edges.is_empty() {
                    return Err(CoreValidationError::new(
                        "INSERT EDGE 必须包含至少一条边".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }
        // TODO: 验证插入的数据类型与 Schema 匹配
        Ok(())
    }

    fn validate_delete_stmt(
        &mut self,
        delete_stmt: &crate::query::parser::ast::stmt::DeleteStmt,
    ) -> Result<(), CoreValidationError> {
        // 验证 DELETE 语句的基本结构
        match &delete_stmt.target {
            crate::query::parser::ast::stmt::DeleteTarget::Vertices(vertex_ids) => {
                if vertex_ids.is_empty() {
                    return Err(CoreValidationError::new(
                        "DELETE VERTICES 必须指定至少一个顶点 ID".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            crate::query::parser::ast::stmt::DeleteTarget::Edges { edge_type, edges } => {
                if edges.is_empty() {
                    return Err(CoreValidationError::new(
                        "DELETE EDGES 必须指定至少一条边".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                // 如果指定了边类型，验证其不为空
                if let Some(ref et) = edge_type {
                    if et.is_empty() {
                        return Err(CoreValidationError::new(
                            "DELETE EDGES 的边类型不能为空".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
            crate::query::parser::ast::stmt::DeleteTarget::Tags { tag_names, vertex_ids, .. } => {
                if tag_names.is_empty() {
                    return Err(CoreValidationError::new(
                        "DELETE TAGS 必须指定至少一个标签名".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                if vertex_ids.is_empty() {
                    return Err(CoreValidationError::new(
                        "DELETE TAGS 必须指定至少一个顶点 ID".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            crate::query::parser::ast::stmt::DeleteTarget::Index(index_name) => {
                if index_name.is_empty() {
                    return Err(CoreValidationError::new(
                        "DELETE INDEX 必须指定索引名".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        // 验证 WHERE 子句（如果存在）
        if let Some(ref where_clause) = delete_stmt.where_clause {
            self.validate_where_expression(where_clause)?;
        }

        Ok(())
    }

    fn validate_update_stmt(
        &mut self,
        update_stmt: &crate::query::parser::ast::stmt::UpdateStmt,
    ) -> Result<(), CoreValidationError> {
        // 验证 UPDATE 语句的基本结构
        match &update_stmt.target {
            crate::query::parser::ast::stmt::UpdateTarget::Vertex(_) => {
                // 单顶点更新，验证通过
            }
            crate::query::parser::ast::stmt::UpdateTarget::Edge { src: _, dst: _, edge_type, rank: _ } => {
                // 边更新，验证边类型
                if let Some(ref et) = edge_type {
                    if et.is_empty() {
                        return Err(CoreValidationError::new(
                            "UPDATE EDGE 的边类型不能为空".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
            crate::query::parser::ast::stmt::UpdateTarget::Tag(tag_name) => {
                if tag_name.is_empty() {
                    return Err(CoreValidationError::new(
                        "UPDATE TAG 必须指定标签名".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            crate::query::parser::ast::stmt::UpdateTarget::TagOnVertex { vid: _, tag_name } => {
                if tag_name.is_empty() {
                    return Err(CoreValidationError::new(
                        "UPDATE VERTEX ON TAG 必须指定标签名".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        // 验证 SET 子句不为空
        if update_stmt.set_clause.assignments.is_empty() {
            return Err(CoreValidationError::new(
                "UPDATE 语句必须包含 SET 子句".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证 WHERE 子句（如果存在）
        if let Some(ref where_clause) = update_stmt.where_clause {
            self.validate_where_expression(where_clause)?;
        }

        Ok(())
    }

    /// 验证 WHERE 表达式
    fn validate_where_expression(
        &self,
        expr: &Expression,
    ) -> Result<(), CoreValidationError> {
        // 基本验证：确保表达式不是常量 true/false（可能是错误）
        match expr {
            Expression::Literal(Value::Bool(true)) => {
                // 警告：WHERE true 会匹配所有记录
            }
            Expression::Literal(Value::Bool(false)) => {
                return Err(CoreValidationError::new(
                    "WHERE false 不会匹配任何记录".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_create_stmt(
        &mut self,
        create_stmt: &crate::query::parser::ast::stmt::CreateStmt,
    ) -> Result<(), CoreValidationError> {
        // 验证 CREATE 语句的基本结构
        match &create_stmt.target {
            crate::query::parser::ast::stmt::CreateTarget::Node { labels, .. } => {
                if labels.is_empty() {
                    return Err(CoreValidationError::new(
                        "CREATE 节点必须指定至少一个标签".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            crate::query::parser::ast::stmt::CreateTarget::Edge { edge_type, .. } => {
                if edge_type.is_empty() {
                    return Err(CoreValidationError::new(
                        "CREATE 边必须指定边类型".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_drop_stmt(
        &mut self,
        _drop_stmt: &crate::query::parser::ast::stmt::DropStmt,
    ) -> Result<(), CoreValidationError> {
        // TODO: 实现 DROP 语句验证
        Ok(())
    }

    fn validate_alter_stmt(
        &mut self,
        _alter_stmt: &crate::query::parser::ast::stmt::AlterStmt,
    ) -> Result<(), CoreValidationError> {
        // TODO: 实现 ALTER 语句验证
        Ok(())
    }

    fn validate_use_stmt(
        &mut self,
        use_stmt: &crate::query::parser::ast::stmt::UseStmt,
    ) -> Result<(), CoreValidationError> {
        // 验证 USE 语句的基本结构
        if use_stmt.space.is_empty() {
            return Err(CoreValidationError::new(
                "USE 语句必须指定图空间名称".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_pipe_stmt(
        &mut self,
        pipe_stmt: &crate::query::parser::ast::stmt::PipeStmt,
        ast: &AstContext,
    ) -> Result<(), CoreValidationError> {
        // 验证 PIPE 语句的基本结构
        // PIPE 语句连接左右两个语句
        // 验证左语句
        self.validate_statement_with_ast(&pipe_stmt.left, ast)?;
        // 验证右语句
        self.validate_statement_with_ast(&pipe_stmt.right, ast)?;
        Ok(())
    }

    fn validate_yield_stmt(
        &mut self,
        _yield_stmt: &crate::query::parser::ast::stmt::YieldStmt,
    ) -> Result<(), CoreValidationError> {
        // TODO: 实现 YIELD 语句验证
        Ok(())
    }

    fn validate_unwind_stmt(
        &mut self,
        _unwind_stmt: &crate::query::parser::ast::stmt::UnwindStmt,
    ) -> Result<(), CoreValidationError> {
        // TODO: 实现 UNWIND 语句验证
        Ok(())
    }

    fn validate_set_stmt(
        &mut self,
        _set_stmt: &crate::query::parser::ast::stmt::SetStmt,
    ) -> Result<(), CoreValidationError> {
        // TODO: 实现 SET 语句验证
        Ok(())
    }

    fn check_permission(&self) -> Result<(), CoreValidationError> {
        Ok(())
    }

    fn to_plan_with_ast(&mut self, _ast: &mut AstContext) -> Result<(), CoreValidationError> {
        Ok(())
    }

    pub fn get_validation_errors(&self) -> Vec<CoreValidationError> {
        if let Some(ref ctx) = self.context {
            ctx.get_validation_errors().to_vec()
        } else {
            Vec::new()
        }
    }

    fn validate_impl(&mut self) -> Result<(), CoreValidationError> {
        Ok(())
    }

    pub fn validate_unified(&mut self) -> Result<(), DBError> {
        let ctx = match self.context {
            Some(ref mut ctx) => ctx,
            None => {
                return Err(DBError::Query(QueryError::InvalidQuery(
                    "验证上下文未初始化".to_string(),
                )));
            }
        };

        let has_errors = ctx.has_validation_errors();
        ctx.clear_validation_errors();

        if !self.no_space_required && !ctx.space_chosen() {
            return Err(DBError::Query(QueryError::InvalidQuery(
                "No space selected. Use `USE <space>` to select a graph space first.".to_string(),
            )));
        }

        if let Err(e) = self.validate_impl() {
            return Err(DBError::Query(QueryError::InvalidQuery(format!(
                "验证失败: {}",
                e.message
            ))));
        }

        let ctx = self.context.as_mut().expect("ValidationContext 未初始化");
        if ctx.has_validation_errors() {
            let errors = ctx.get_validation_errors();
            if let Some(first_error) = errors.first() {
                return Err(DBError::Query(QueryError::InvalidQuery(format!(
                    "验证失败: {}",
                    first_error.message
                ))));
            }
        }

        if let Err(e) = self.check_permission() {
            return Err(DBError::Query(QueryError::InvalidQuery(format!(
                "权限检查失败: {}",
                e.message
            ))));
        }

        if let Err(e) = self.to_plan() {
            return Err(DBError::Query(QueryError::InvalidQuery(format!(
                "计划生成失败: {}",
                e.message
            ))));
        }

        let ctx = self.context.as_mut().expect("ValidationContext 未初始化");
        if has_errors || ctx.has_validation_errors() {
            let errors = ctx.get_validation_errors();
            if let Some(first_error) = errors.first() {
                return Err(DBError::Query(QueryError::InvalidQuery(format!(
                    "验证失败: {}",
                    first_error.message
                ))));
            }
        }

        Ok(())
    }

    fn to_plan(&mut self) -> Result<(), CoreValidationError> {
        Ok(())
    }

    pub fn context_mut(&mut self) -> &mut ValidationContext {
        self.context.as_mut().expect("ValidationContext 未初始化")
    }

    pub fn context(&self) -> &ValidationContext {
        self.context.as_ref().expect("ValidationContext 未初始化")
    }

    pub fn set_input_var_name(&mut self, name: String) {
        self.input_var_name = name;
    }

    pub fn input_var_name(&self) -> &str {
        &self.input_var_name
    }

    pub fn set_no_space_required(&mut self, required: bool) {
        self.no_space_required = required;
    }

    pub fn no_space_required(&self) -> bool {
        self.no_space_required
    }

    pub fn add_output(&mut self, name: String, type_: ValueType) {
        self.outputs.push(ColumnDef { name, type_ });
    }

    pub fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    pub fn outputs_mut(&mut self) -> &mut Vec<ColumnDef> {
        &mut self.outputs
    }

    pub fn add_input(&mut self, name: String, type_: ValueType) {
        self.inputs.push(ColumnDef { name, type_ });
    }

    pub fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    pub fn add_input_property(&mut self, prop_name: String, type_: ValueType) {
        self.expr_props.input_props.push(InputProperty { prop_name, type_ });
    }

    pub fn add_var_property(&mut self, var_name: String, prop_name: String, type_: ValueType) {
        self.expr_props.var_props.push(VarProperty { var_name, prop_name, type_ });
    }

    pub fn add_tag_property(&mut self, tag_name: String, prop_name: String, type_: ValueType) {
        self.expr_props.tag_props.push(TagProperty { tag_name, prop_name, type_ });
    }

    pub fn add_edge_property(&mut self, edge_type: i32, prop_name: String, type_: ValueType) {
        self.expr_props.edge_props.push(EdgeProperty { edge_type, prop_name, type_ });
    }

    pub fn expr_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    pub fn expr_props_mut(&mut self) -> &mut ExpressionProps {
        &mut self.expr_props
    }

    pub fn add_user_defined_var(&mut self, var_name: String) {
        self.user_defined_vars.push(var_name);
    }

    pub fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }

    pub fn add_error(&mut self, error: CoreValidationError) {
        if let Some(ref mut ctx) = self.context {
            ctx.add_validation_error(error);
        }
    }

    pub fn add_semantic_error(&mut self, message: String) {
        self.add_error(CoreValidationError::new(
            message,
            ValidationErrorType::SemanticError,
        ));
    }

    pub fn add_type_error(&mut self, message: String) {
        self.add_error(CoreValidationError::new(
            message,
            ValidationErrorType::TypeError,
        ));
    }

    pub fn add_syntax_error(&mut self, message: String) {
        self.add_error(CoreValidationError::new(
            message,
            ValidationErrorType::SyntaxError,
        ));
    }

    pub fn deduce_expr_type(&self, expression: &Expression) -> ValueType {
        match expression {
            Expression::Literal(value) => {
                match value {
                    Value::Bool(_) => ValueType::Bool,
                    Value::Int(_) => ValueType::Int,
                    Value::Float(_) => ValueType::Float,
                    Value::String(_) => ValueType::String,
                    Value::Null(_) => ValueType::Null,
                    Value::Date(_) => ValueType::Date,
                    Value::Time(_) => ValueType::Time,
                    Value::DateTime(_) => ValueType::DateTime,
                    Value::Vertex(_) => ValueType::Vertex,
                    Value::Edge(_) => ValueType::Edge,
                    Value::Path(_) => ValueType::Path,
                    Value::List(_) => ValueType::List,
                    Value::Map(_) => ValueType::Map,
                    Value::Set(_) => ValueType::Set,
                    _ => ValueType::Unknown,
                }
            }
            Expression::Variable(_) => ValueType::Unknown,
            Expression::Property { .. } => ValueType::Unknown,
            Expression::Binary { op, .. } => {
                match op {
                    crate::core::BinaryOperator::Equal
                    | crate::core::BinaryOperator::NotEqual
                    | crate::core::BinaryOperator::LessThan
                    | crate::core::BinaryOperator::LessThanOrEqual
                    | crate::core::BinaryOperator::GreaterThan
                    | crate::core::BinaryOperator::GreaterThanOrEqual => ValueType::Bool,
                    crate::core::BinaryOperator::And | crate::core::BinaryOperator::Or => ValueType::Bool,
                    _ => ValueType::Unknown,
                }
            }
            Expression::Unary { .. } => ValueType::Unknown,
            Expression::Function { name, .. } => {
                match name.to_lowercase().as_str() {
                    "id" => ValueType::String,
                    "count" | "sum" | "avg" | "min" | "max" => ValueType::Float,
                    "length" | "size" => ValueType::Int,
                    "to_string" | "string" => ValueType::String,
                    "abs" => ValueType::Float,
                    "floor" | "ceil" | "round" => ValueType::Int,
                    _ => ValueType::Unknown,
                }
            }
            Expression::Aggregate { func, .. } => {
                match func {
                    crate::core::AggregateFunction::Count(_) => ValueType::Int,
                    crate::core::AggregateFunction::Sum(_) => ValueType::Float,
                    crate::core::AggregateFunction::Avg(_) => ValueType::Float,
                    crate::core::AggregateFunction::Collect(_) => ValueType::List,
                    _ => ValueType::Unknown,
                }
            }
            Expression::List(_) => ValueType::List,
            Expression::Map(_) => ValueType::Map,
            _ => ValueType::Unknown,
        }
     }
}

