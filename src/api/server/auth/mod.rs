//! Authentication module
//!
//! Provide user authentication and authorization features.

pub mod authenticator;
pub mod error;
pub mod user_storage;

pub use authenticator::{Authenticator, AuthenticatorFactory, PasswordAuthenticator, UserVerifier};
pub use error::{AuthError, AuthResult};
pub use user_storage::UserStorage;
