//! EitherOr and ErrorOr implementations
//!
//! This module provides Either/Left-Right types similar to NebulaGraph's EitherOr and ErrorOr,
//! though Rust's Result type covers many of these use cases.

use std::fmt::Debug;

/// Represents a value that can be either of two types: Left or Right
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    /// Create a Left value
    pub fn left(value: L) -> Self {
        Either::Left(value)
    }

    /// Create a Right value
    pub fn right(value: R) -> Self {
        Either::Right(value)
    }

    /// Check if this is a Left value
    pub fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }

    /// Check if this is a Right value
    pub fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }

    /// Get a reference to the Left value, if it exists
    pub fn left_ref(&self) -> Option<&L> {
        match self {
            Either::Left(l) => Some(l),
            Either::Right(_) => None,
        }
    }

    /// Get a reference to the Right value, if it exists
    pub fn right_ref(&self) -> Option<&R> {
        match self {
            Either::Right(r) => Some(r),
            Either::Left(_) => None,
        }
    }

    /// Get a mutable reference to the Left value, if it exists
    pub fn left_mut(&mut self) -> Option<&mut L> {
        match self {
            Either::Left(l) => Some(l),
            Either::Right(_) => None,
        }
    }

    /// Get a mutable reference to the Right value, if it exists
    pub fn right_mut(&mut self) -> Option<&mut R> {
        match self {
            Either::Right(r) => Some(r),
            Either::Left(_) => None,
        }
    }

    /// Convert to an option, taking the Left value if present
    pub fn left_into_option(self) -> Option<L> {
        match self {
            Either::Left(l) => Some(l),
            Either::Right(_) => None,
        }
    }

    /// Convert to an option, taking the Right value if present
    pub fn right_into_option(self) -> Option<R> {
        match self {
            Either::Right(r) => Some(r),
            Either::Left(_) => None,
        }
    }

    /// Map the Left value with a function, leaving Right unchanged
    pub fn map_left<M, F>(self, f: F) -> Either<M, R>
    where
        F: FnOnce(L) -> M,
    {
        match self {
            Either::Left(l) => Either::Left(f(l)),
            Either::Right(r) => Either::Right(r),
        }
    }

    /// Map the Right value with a function, leaving Left unchanged
    pub fn map_right<M, F>(self, f: F) -> Either<L, M>
    where
        F: FnOnce(R) -> M,
    {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(f(r)),
        }
    }

    /// Apply a function to the Right value if it exists, otherwise return the Left value
    pub fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce(L) -> U,
        F: FnOnce(R) -> U,
    {
        match self {
            Either::Left(l) => default(l),
            Either::Right(r) => f(r),
        }
    }
}

impl<L, R> From<Result<R, L>> for Either<L, R> {
    /// Convert a Result to an Either (Err -> Left, Ok -> Right)
    fn from(result: Result<R, L>) -> Self {
        match result {
            Ok(r) => Either::Right(r),
            Err(e) => Either::Left(e),
        }
    }
}

impl<L, R> Into<Result<R, L>> for Either<L, R> {
    /// Convert an Either to a Result (Left -> Err, Right -> Ok)
    fn into(self) -> Result<R, L> {
        match self {
            Either::Left(l) => Err(l),
            Either::Right(r) => Ok(r),
        }
    }
}

/// A type alias for ErrorOr (similar to Either where Left is an error code and Right is the result)
pub type ErrorOr<ErrorCode, ResultType> = Either<ErrorCode, ResultType>;

/// Helper functions for working with ErrorOr/Either types
pub mod error_or {
    use super::{Either, ErrorOr};
    use std::fmt::Debug;

    /// Check if the ErrorOr is ok (contains a result, not an error)
    pub fn is_ok<ErrorCode, ResultType>(error_or: &ErrorOr<ErrorCode, ResultType>) -> bool {
        error_or.is_right()
    }

    /// Check if the ErrorOr contains an error
    pub fn is_error<ErrorCode, ResultType>(error_or: &ErrorOr<ErrorCode, ResultType>) -> bool {
        error_or.is_left()
    }

    /// Get the error code if present
    pub fn error<ErrorCode, ResultType>(error_or: &ErrorOr<ErrorCode, ResultType>) -> Option<&ErrorCode> {
        error_or.left_ref()
    }

    /// Get the result if present
    pub fn result<ErrorCode, ResultType>(error_or: &ErrorOr<ErrorCode, ResultType>) -> Option<&ResultType> {
        error_or.right_ref()
    }

    /// Unwrap the result or panic with a default message
    pub fn unwrap<ErrorCode: Debug, ResultType>(error_or: Either<ErrorCode, ResultType>) -> ResultType {
        match error_or {
            Either::Right(r) => r,
            Either::Left(e) => panic!("Attempted to unwrap error: {:?}", e),
        }
    }

    /// Unwrap the result or return a default value
    pub fn unwrap_or<ErrorCode, ResultType>(error_or: Either<ErrorCode, ResultType>, default: ResultType) -> ResultType {
        match error_or {
            Either::Right(r) => r,
            Either::Left(_) => default,
        }
    }

    /// Unwrap the result or compute a default value
    pub fn unwrap_or_else<ErrorCode, ResultType, F>(error_or: Either<ErrorCode, ResultType>, f: F) -> ResultType
    where
        F: FnOnce() -> ResultType,
    {
        match error_or {
            Either::Right(r) => r,
            Either::Left(_) => f(),
        }
    }
}

pub use error_or::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_either_creation() {
        let left: Either<i32, &str> = Either::left(42);
        let right: Either<i32, &str> = Either::right("hello");

        assert!(left.is_left());
        assert!(right.is_right());
        assert_eq!(left.left_ref(), Some(&42));
        assert_eq!(right.right_ref(), Some(&"hello"));
    }

    #[test]
    fn test_either_mapping() {
        let left: Either<i32, &str> = Either::left(42);
        let mapped_left = left.map_left(|x| x * 2);
        assert!(mapped_left.is_left());
        assert_eq!(mapped_left.left_ref(), Some(&84));

        let right: Either<i32, &str> = Either::right("hello");
        let mapped_right = right.map_right(|s| s.len());
        assert!(mapped_right.is_right());
        assert_eq!(mapped_right.right_ref(), Some(&5));
    }

    #[test]
    fn test_result_conversion() {
        let result: Result<&str, i32> = Ok("success");
        let either: Either<i32, &str> = result.into();
        assert!(either.is_right());
        assert_eq!(either.right_ref(), Some(&"success"));

        let result: Result<&str, i32> = Err(404);
        let either: Either<i32, &str> = result.into();
        assert!(either.is_left());
        assert_eq!(either.left_ref(), Some(&404));
    }

    #[test]
    fn test_error_or() {
        type MyErrorOr<T> = ErrorOr<&'static str, T>;
        
        let success: MyErrorOr<i32> = Either::right(123);
        assert!(is_ok(&success));
        assert_eq!(result(&success), Some(&123));
        
        let error_value: MyErrorOr<i32> = Either::left("error occurred");
        assert!(is_error(&error_value));
        assert_eq!(error(&error_value), Some(&"error occurred"));
    }

    #[test]
    fn test_error_or_unwrap() {
        let success: ErrorOr<&'static str, i32> = Either::right(123);
        assert_eq!(unwrap_or(success, 0), 123);

        let error: ErrorOr<&'static str, i32> = Either::left("error");
        assert_eq!(unwrap_or(error, 999), 999);
    }
}