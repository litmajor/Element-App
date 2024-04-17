use log::{error, info};

// User-related logs
pub fn log_user_registration(username: &str) {
    info!("User registered successfully: {}", username);
}

pub fn log_user_login(username: &str) {
    info!("User logged in successfully: {}", username);
}

pub fn log_invalid_password(username: &str) {
    error!("Invalid password for user: {}", username);
}

// Database operation logs
pub fn log_database_operation_success(operation: &str) {
    info!("Database operation '{}' executed successfully", operation);
}

pub fn log_database_operation_error(operation: &str, error_message: &str) {
    error!("Error executing database operation '{}': {}", operation, error_message);
}

// Authentication and authorization logs
pub fn log_authentication_attempt(username: &str) {
    info!("Authentication attempt for user: {}", username);
}

pub fn log_authentication_failure(username: &str) {
    error!("Authentication failed for user: {}", username);
}

pub fn log_authorization_failure(username: &str, action: &str) {
    error!("Authorization failed for user '{}' while attempting '{}'", username, action);
}

// Validation logs
pub fn log_validation_failure(error_message: &str) {
    error!("Validation failed: {}", error_message);
}

// Error handling logs
pub fn log_error(error_message: &str) {
    error!("Error occurred: {}", error_message);
}

pub fn log_error_with_context(context: &str, error_message: &str) {
    error!("Error occurred in context '{}': {}", context, error_message);
}

// Performance logs
pub fn log_request_processing_time(milliseconds: u64) {
    info!("Request processing time: {} ms", milliseconds);
}

pub fn log_resource_usage(resource_name: &str, usage: &str) {
    info!("Resource '{}' usage: {}", resource_name, usage);
}

// Security logs
pub fn log_security_event(event: &str) {
    info!("Security event: {}", event);
}
