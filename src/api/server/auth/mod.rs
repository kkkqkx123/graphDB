//! Authentication module
//!
//! Provide user authentication and authorization features.

pub mod authenticator;

pub use authenticator::{Authenticator, AuthenticatorFactory, PasswordAuthenticator, UserVerifier};
