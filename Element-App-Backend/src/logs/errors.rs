use diesel::result::Error as DieselError;
use jsonwebtoken::errors::Error as JwtError;
use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub enum ServiceError {
  DatabaseError(DieselError, String), // Include additional context information
  JwtError(JwtError),
  InternalServerError(String), // Include more context for InternalServerError
  Unauthorized,
  // Additional Custom Error Variants
  AuthenticationError(String), // Include specific details about authentication errors
  ValidationError(String), // Include specific details about validation errors
  NotFoundError(String), // Include specific details about resource not found errors
  ConflictError(String), // Include specific details about conflicts
  IOError(String), // Include specific details about IO errors
}

impl fmt::Display for ServiceError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      ServiceError::DatabaseError(ref err, ref ctx) => write!(f, "Database Error: {} ({})", err, ctx),
      ServiceError::JwtError(ref err) => write!(f, "JWT Error: {}", err),
      ServiceError::InternalServerError(ref ctx) => write!(f, "Internal Server Error: {}", ctx),
      ServiceError::Unauthorized => write!(f, "Unauthorized"),
      ServiceError::AuthenticationError(ref details) => write!(f, "Authentication Error: {}", details),
      ServiceError::ValidationError(ref details) => write!(f, "Validation Error: {}", details),
      ServiceError::NotFoundError(ref details) => write!(f, "Resource Not Found: {}", details),
      ServiceError::ConflictError(ref details) => write!(f, "Conflict: {}", details),
      ServiceError::IOError(ref details) => write!(f, "IO Error: {}", details),
    }
  }
}

impl StdError for ServiceError {}

impl From<DieselError> for ServiceError {
  fn from(err: DieselError) -> ServiceError {
    ServiceError::DatabaseError(err, String::from("Unknown operation")) // Provide default context
  }
}

impl From<JwtError> for ServiceError {
  fn from(err: JwtError) -> ServiceError {
    ServiceError::JwtError(err)
  }
}
