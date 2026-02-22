//! 权限控制类语句验证器
//! 对应 NebulaGraph ACLValidator 的功能
//! 验证 CREATE USER, DROP USER, ALTER USER, GRANT, REVOKE 等权限类语句
//!
//! 设计原则：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. 所有权限类语句都是全局语句，不需要预先选择空间
//! 3. 验证用户存在性和角色合法性

use std::sync::Arc;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::context::QueryContext;
use crate::query::parser::ast::stmt::{
    CreateUserStmt, AlterUserStmt, DropUserStmt, ChangePasswordStmt,
    GrantStmt, RevokeStmt, DescribeUserStmt, ShowUsersStmt, ShowRolesStmt,
    RoleType,
};
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};

/// 验证后的用户信息
#[derive(Debug, Clone)]
pub struct ValidatedUser {
    pub username: String,
    pub role: Option<String>,
}

/// CREATE USER 语句验证器
#[derive(Debug)]
pub struct CreateUserValidator {
    username: String,
    password: String,
    role: Option<String>,
    if_not_exists: bool,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl CreateUserValidator {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            role: None,
            if_not_exists: false,
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "Result".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &CreateUserStmt) -> Result<(), ValidationError> {
        self.username = stmt.username.clone();
        self.password = stmt.password.clone();
        self.role = stmt.role.clone();
        self.if_not_exists = stmt.if_not_exists;

        // 验证用户名非空
        if self.username.is_empty() {
            return Err(ValidationError::new(
                "Username cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证密码非空
        if self.password.is_empty() {
            return Err(ValidationError::new(
                "Password cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证角色合法性
        if let Some(ref role) = self.role {
            Self::validate_role(role)?;
        }

        Ok(())
    }

    fn validate_role(role: &str) -> Result<(), ValidationError> {
        match role.to_uppercase().as_str() {
            "GOD" | "ADMIN" | "DBA" | "USER" | "GUEST" => Ok(()),
            _ => Err(ValidationError::new(
                format!("Invalid role: {}", role),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    pub fn validated_result(&self) -> ValidatedUser {
        ValidatedUser {
            username: self.username.clone(),
            role: self.role.clone(),
        }
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for CreateUserValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let create_user_stmt = match stmt {
            crate::query::parser::ast::Stmt::CreateUser(create_user_stmt) => create_user_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected CREATE USER statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(create_user_stmt)?;
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::ShowSpaces
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        true
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for CreateUserValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// DROP USER 语句验证器
#[derive(Debug)]
pub struct DropUserValidator {
    username: String,
    if_exists: bool,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl DropUserValidator {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            if_exists: false,
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "Result".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &DropUserStmt) -> Result<(), ValidationError> {
        self.username = stmt.username.clone();
        self.if_exists = stmt.if_exists;

        // 验证用户名非空
        if self.username.is_empty() {
            return Err(ValidationError::new(
                "Username cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    pub fn validated_result(&self) -> ValidatedUser {
        ValidatedUser {
            username: self.username.clone(),
            role: None,
        }
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for DropUserValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let drop_user_stmt = match stmt {
            crate::query::parser::ast::Stmt::DropUser(drop_user_stmt) => drop_user_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected DROP USER statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(drop_user_stmt)?;
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::ShowSpaces
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        true
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for DropUserValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// ALTER USER 语句验证器
#[derive(Debug)]
pub struct AlterUserValidator {
    username: String,
    password: Option<String>,
    new_role: Option<String>,
    is_locked: Option<bool>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl AlterUserValidator {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: None,
            new_role: None,
            is_locked: None,
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "Result".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &AlterUserStmt) -> Result<(), ValidationError> {
        self.username = stmt.username.clone();
        self.password = stmt.password.clone();
        self.new_role = stmt.new_role.clone();
        self.is_locked = stmt.is_locked;

        // 验证用户名非空
        if self.username.is_empty() {
            return Err(ValidationError::new(
                "Username cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证至少有一个修改项
        if self.password.is_none() && self.new_role.is_none() && self.is_locked.is_none() {
            return Err(ValidationError::new(
                "At least one modification is required".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证角色合法性
        if let Some(ref role) = self.new_role {
            CreateUserValidator::validate_role(role)?;
        }

        Ok(())
    }

    pub fn validated_result(&self) -> ValidatedUser {
        ValidatedUser {
            username: self.username.clone(),
            role: self.new_role.clone(),
        }
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for AlterUserValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let alter_user_stmt = match stmt {
            crate::query::parser::ast::Stmt::AlterUser(alter_user_stmt) => alter_user_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected ALTER USER statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(alter_user_stmt)?;
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::ShowSpaces
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        true
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for AlterUserValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// CHANGE PASSWORD 语句验证器
#[derive(Debug)]
pub struct ChangePasswordValidator {
    username: Option<String>,
    old_password: String,
    new_password: String,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl ChangePasswordValidator {
    pub fn new() -> Self {
        Self {
            username: None,
            old_password: String::new(),
            new_password: String::new(),
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "Result".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &ChangePasswordStmt) -> Result<(), ValidationError> {
        self.username = stmt.username.clone();
        self.old_password = stmt.old_password.clone();
        self.new_password = stmt.new_password.clone();

        // 验证旧密码非空
        if self.old_password.is_empty() {
            return Err(ValidationError::new(
                "Old password cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证新密码非空
        if self.new_password.is_empty() {
            return Err(ValidationError::new(
                "New password cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证新旧密码不同
        if self.old_password == self.new_password {
            return Err(ValidationError::new(
                "New password must be different from old password".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for ChangePasswordValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let change_password_stmt = match stmt {
            crate::query::parser::ast::Stmt::ChangePassword(change_password_stmt) => change_password_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected CHANGE PASSWORD statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(change_password_stmt)?;
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::ShowSpaces
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        true
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for ChangePasswordValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证后的权限信息
#[derive(Debug, Clone)]
pub struct ValidatedGrant {
    pub role: RoleType,
    pub space_name: String,
    pub username: String,
}

/// GRANT 语句验证器
#[derive(Debug)]
pub struct GrantValidator {
    role: RoleType,
    space_name: String,
    username: String,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl GrantValidator {
    pub fn new() -> Self {
        Self {
            role: RoleType::Guest,
            space_name: String::new(),
            username: String::new(),
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "Result".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &GrantStmt) -> Result<(), ValidationError> {
        self.role = stmt.role;
        self.space_name = stmt.space_name.clone();
        self.username = stmt.username.clone();

        // 验证空间名非空
        if self.space_name.is_empty() {
            return Err(ValidationError::new(
                "Space name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证用户名非空
        if self.username.is_empty() {
            return Err(ValidationError::new(
                "Username cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    pub fn validated_result(&self) -> ValidatedGrant {
        ValidatedGrant {
            role: self.role,
            space_name: self.space_name.clone(),
            username: self.username.clone(),
        }
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for GrantValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let grant_stmt = match stmt {
            crate::query::parser::ast::Stmt::Grant(grant_stmt) => grant_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected GRANT statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(grant_stmt)?;
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::ShowSpaces
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        true
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for GrantValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// REVOKE 语句验证器
#[derive(Debug)]
pub struct RevokeValidator {
    role: RoleType,
    space_name: String,
    username: String,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl RevokeValidator {
    pub fn new() -> Self {
        Self {
            role: RoleType::Guest,
            space_name: String::new(),
            username: String::new(),
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "Result".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &RevokeStmt) -> Result<(), ValidationError> {
        self.role = stmt.role;
        self.space_name = stmt.space_name.clone();
        self.username = stmt.username.clone();

        // 验证空间名非空
        if self.space_name.is_empty() {
            return Err(ValidationError::new(
                "Space name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证用户名非空
        if self.username.is_empty() {
            return Err(ValidationError::new(
                "Username cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    pub fn validated_result(&self) -> ValidatedGrant {
        ValidatedGrant {
            role: self.role,
            space_name: self.space_name.clone(),
            username: self.username.clone(),
        }
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for RevokeValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let revoke_stmt = match stmt {
            crate::query::parser::ast::Stmt::Revoke(revoke_stmt) => revoke_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected REVOKE statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(revoke_stmt)?;
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::ShowSpaces
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        true
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for RevokeValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// DESCRIBE USER 语句验证器
#[derive(Debug)]
pub struct DescribeUserValidator {
    username: String,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl DescribeUserValidator {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "User".to_string(), type_: ValueType::String },
                ColumnDef { name: "Roles".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &DescribeUserStmt) -> Result<(), ValidationError> {
        self.username = stmt.username.clone();

        // 验证用户名非空
        if self.username.is_empty() {
            return Err(ValidationError::new(
                "Username cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for DescribeUserValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let describe_user_stmt = match stmt {
            crate::query::parser::ast::Stmt::DescribeUser(describe_user_stmt) => describe_user_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected DESCRIBE USER statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(describe_user_stmt)?;
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::ShowSpaces
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        true
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for DescribeUserValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// SHOW USERS 语句验证器
#[derive(Debug)]
pub struct ShowUsersValidator {
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl ShowUsersValidator {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "Account".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, _stmt: &ShowUsersStmt) -> Result<(), ValidationError> {
        Ok(())
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for ShowUsersValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let show_users_stmt = match stmt {
            crate::query::parser::ast::Stmt::ShowUsers(show_users_stmt) => show_users_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected SHOW USERS statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(show_users_stmt)?;
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::ShowSpaces
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        true
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for ShowUsersValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// SHOW ROLES 语句验证器
#[derive(Debug)]
pub struct ShowRolesValidator {
    space_name: Option<String>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl ShowRolesValidator {
    pub fn new() -> Self {
        Self {
            space_name: None,
            inputs: Vec::new(),
            outputs: vec![
                ColumnDef { name: "Account".to_string(), type_: ValueType::String },
                ColumnDef { name: "Role".to_string(), type_: ValueType::String },
            ],
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &ShowRolesStmt) -> Result<(), ValidationError> {
        self.space_name = stmt.space_name.clone();
        Ok(())
    }
}

impl StatementValidator for ShowRolesValidator {
    fn validate(&mut self, stmt: &crate::query::parser::ast::Stmt, _qctx: Arc<QueryContext>) -> Result<ValidationResult, ValidationError> {
        let show_roles_stmt = match stmt {
            crate::query::parser::ast::Stmt::ShowRoles(s) => s,
            _ => {
                return Err(ValidationError::new(
                    "Expected SHOW ROLES statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(show_roles_stmt)?;
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::ShowSpaces
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        true
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for ShowRolesValidator {
    fn default() -> Self {
        Self::new()
    }
}
