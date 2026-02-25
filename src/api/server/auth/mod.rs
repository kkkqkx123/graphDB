//! 认证模块
//!
//! 提供用户认证和授权功能

pub mod authenticator;

pub use authenticator::{Authenticator, PasswordAuthenticator, AuthenticatorFactory, UserVerifier};
