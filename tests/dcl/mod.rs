//! Data Control Language (DCL) Integration Tests
//!
//! Test coverage:
//! - CREATE USER - Create a user
//! - ALTER USER - Modifies a user account
//! - DROP USER - Deletes a user
//! - CHANGE PASSWORD - Change your password
//! - GRANT - Grant privileges to users
//! - REVOKE - Revoke privileges from users
//! - SHOW USERS - List all users
//! - SHOW ROLES - List all roles
//! - DESCRIBE USER - Describe user details

mod common;
mod user_management;
mod permission;
mod role;
