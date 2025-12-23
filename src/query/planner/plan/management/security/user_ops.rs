//! 用户操作相关的计划节点
//! 包括创建/删除用户等操作

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt,
        PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use std::sync::Arc;

/// 创建用户计划节点
#[derive(Debug)]
pub struct CreateUser {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
    pub password: String,
    pub if_not_exists: bool,
}

impl CreateUser {
    pub fn new(id: i64, username: &str, password: &str, if_not_exists: bool) -> Self {
        Self {
            id,
            kind: PlanNodeKind::CreateUser,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            username: username.to_string(),
            password: password.to_string(),
            if_not_exists,
        }
    }
}

impl Clone for CreateUser {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            username: self.username.clone(),
            password: self.password.clone(),
            if_not_exists: self.if_not_exists,
        }
    }
}

impl PlanNodeIdentifiable for CreateUser {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for CreateUser {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for CreateUser {
    fn dependencies(&self) -> Vec<PlanNodeEnum> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for CreateUser {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[PlanNodeEnum]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for CreateUser {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for CreateUser {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for CreateUser {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for CreateUser {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 删除用户计划节点
#[derive(Debug)]
pub struct DropUser {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub if_exist: bool,
    pub username: String,
}

impl DropUser {
    pub fn new(id: i64, if_exist: bool, username: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DropUser,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            if_exist,
            username: username.to_string(),
        }
    }
}

impl Clone for DropUser {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            if_exist: self.if_exist,
            username: self.username.clone(),
        }
    }
}

impl PlanNodeIdentifiable for DropUser {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for DropUser {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for DropUser {
    fn dependencies(&self) -> Vec<PlanNodeEnum> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for DropUser {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[PlanNodeEnum]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for DropUser {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DropUser {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for DropUser {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DropUser {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 修改用户密码计划节点
#[derive(Debug)]
pub struct UpdateUser {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
    pub new_password: String,
}

impl UpdateUser {
    pub fn new(id: i64, username: &str, new_password: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::UpdateUser,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            username: username.to_string(),
            new_password: new_password.to_string(),
        }
    }
}

impl Clone for UpdateUser {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            username: self.username.clone(),
            new_password: self.new_password.clone(),
        }
    }
}

impl PlanNodeIdentifiable for UpdateUser {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for UpdateUser {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for UpdateUser {
    fn dependencies(&self) -> Vec<PlanNodeEnum> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for UpdateUser {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[PlanNodeEnum]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for UpdateUser {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for UpdateUser {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for UpdateUser {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for UpdateUser {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 修改密码计划节点
#[derive(Debug)]
pub struct ChangePassword {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
    pub password: String,
    pub new_password: String,
}

impl ChangePassword {
    pub fn new(id: i64, username: &str, password: &str, new_password: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ChangePassword,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            username: username.to_string(),
            password: password.to_string(),
            new_password: new_password.to_string(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn new_password(&self) -> &str {
        &self.new_password
    }
}

impl Clone for ChangePassword {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            username: self.username.clone(),
            password: self.password.clone(),
            new_password: self.new_password.clone(),
        }
    }
}

impl PlanNodeIdentifiable for ChangePassword {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ChangePassword {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for ChangePassword {
    fn dependencies(&self) -> Vec<PlanNodeEnum> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for ChangePassword {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[PlanNodeEnum]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for ChangePassword {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ChangePassword {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ChangePassword {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ChangePassword {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 列出用户计划节点
#[derive(Debug)]
pub struct ListUsers {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl ListUsers {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ListUsers,
            deps: Vec::new(),
            output_var: None,
            col_names: vec!["Account".to_string()],
            cost: 0.0,
        }
    }
}

impl Clone for ListUsers {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl PlanNodeIdentifiable for ListUsers {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ListUsers {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for ListUsers {
    fn dependencies(&self) -> Vec<PlanNodeEnum> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for ListUsers {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[PlanNodeEnum]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for ListUsers {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ListUsers {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ListUsers {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ListUsers {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 列出用户角色计划节点
#[derive(Debug)]
pub struct ListUserRoles {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
}

impl ListUserRoles {
    pub fn new(id: i64, username: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ListUserRoles,
            deps: Vec::new(),
            output_var: None,
            col_names: vec![
                "Account".to_string(),
                "Space".to_string(),
                "Role".to_string(),
            ],
            cost: 0.0,
            username: username.to_string(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl Clone for ListUserRoles {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            username: self.username.clone(),
        }
    }
}

impl PlanNodeIdentifiable for ListUserRoles {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ListUserRoles {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for ListUserRoles {
    fn dependencies(&self) -> Vec<PlanNodeEnum> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for ListUserRoles {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[PlanNodeEnum]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for ListUserRoles {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ListUserRoles {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ListUserRoles {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ListUserRoles {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 描述用户计划节点
#[derive(Debug)]
pub struct DescribeUser {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub username: String,
}

impl DescribeUser {
    pub fn new(id: i64, username: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::DescribeUser,
            deps: Vec::new(),
            output_var: None,
            col_names: vec![
                "Account".to_string(),
                "Role".to_string(),
                "Time Zone".to_string(),
                "Locked".to_string(),
            ],
            cost: 0.0,
            username: username.to_string(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }
}

impl Clone for DescribeUser {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            username: self.username.clone(),
        }
    }
}

impl PlanNodeIdentifiable for DescribeUser {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for DescribeUser {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for DescribeUser {
    fn dependencies(&self) -> Vec<PlanNodeEnum> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for DescribeUser {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[PlanNodeEnum]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for DescribeUser {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for DescribeUser {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for DescribeUser {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for DescribeUser {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
