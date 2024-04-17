
use chrono::{DateTime, Utc};
use validator::{Validate, ValidationErrors};
use sanitizer::{Cleaner, Sanitization, Error as SanitizeError};
use rand::{distributions::Alphanumeric, thread_rng};
use rand::Rng;
use argon2::{self, Config};
use regex::Regex; // For email validation
use num_traits::Num;

#[derive(Debug)]
pub enum SanitizationError {
    InvalidHtml(String),
}

impl From<SanitizeError> for SanitizationError {
    fn from(err: SanitizeError) -> Self {
        SanitizationError::InvalidHtml(err.to_string())
    }
}

#[derive(Debug)]
pub enum RandomStringError {
    RngError(rand::Error),
}

impl From<rand::Error> for RandomStringError {
    fn from(err: rand::Error) -> Self {
        RandomStringError::RngError(err)
    }
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidEmail(String),
    EmptyField(String),
    InvalidLength(String),
    NonPositiveNumber(String),
    InvalidAmountFormat(String),
    Custom(String),
}

impl From<ValidationErrors> for ValidationError {
    fn from(errors: ValidationErrors) -> Self {
        let mut error_message = String::new();
        for err in errors.errors() {
            match err.code {
                validator::ValidationErrorsKind::Email => error_message.push_str(&format!("Invalid email: {}\n", err.message)),
                validator::ValidationErrorsKind::Empty => error_message.push_str(&format!("Empty field: {}\n", err.message)),
                validator::ValidationErrorsKind::Length => error_message.push_str(&format!("Invalid length: {}\n", err.message)),
                _ => error_message.push_str(&format!("Validation error: {}\n", err.message)),
            }
        }
        ValidationError::Custom(error_message)
    }
}

pub fn get_current_timestamp() -> DateTime<Utc> {
    Utc::now()
}

pub fn format_date(date: &DateTime<Utc>, format: &str) -> Result<String, String> {
    date.format(format).to_string().map_err(|err| format!("Date formatting error: {}", err))
}

pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    // Using regex for more comprehensive email validation
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if re.is_match(email) {
        Ok(())
    } else {
        Err(ValidationError::InvalidEmail("Invalid email format".to_string()))
    }
}

pub fn is_positive_number<T>(value: T) -> Result<(), ValidationError>
where
    T: Num + PartialOrd + Copy,
{
    if value <= T::zero() {
        Err(ValidationError::NonPositiveNumber("Value must be positive".to_string()))
    } else {
        Ok(())
    }
}

pub fn validate_project_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        Err(ValidationError::EmptyField("Project name cannot be empty".to_string()))
    } else if name.len() > 255 {
        Err(ValidationError::InvalidLength("Project name exceeds 255 characters".to_string()))
    } else {
        Ok(())
    }
}

pub fn sanitize_description(description: &str, allowed_tags: &[&str]) -> Result<String, SanitizationError> {
    let cleaner = Cleaner::from_preset(Sanitization::HtmlBasic)?.allow_tags(allowed_tags);
    cleaner.clean(description).map_err(|err| SanitizationError::from(err))
}

pub fn hash_password(password: &str) -> Result<String, argon2::Error> {
    let config = Config::default();
    argon2::hash_encoded(password.as_bytes(), rand::random::<[u8; 16]>(), &config)
}

pub fn generate_random_string(length: usize) -> Result<String, RandomStringError> {
    let mut rng = thread_rng();
    let random_string: String = rng.sample_iter(&Alphanumeric).take(length).collect();
    Ok(random_string)
}

pub fn validate_amount<T>(amount: T) -> Result<(), ValidationError>
where
    T: Num + PartialOrd + Copy,
{
    if amount <= T::zero() {
        Err(ValidationError::NonPositiveNumber("Amount must be positive".to_string()))
    } else {
        Ok(())
    }
}

// Function to validate a struct using the `validator` crate (example with Project struct)
pub fn validate_struct<T>(data: &T) -> Result<(), ValidationError>
where
    T: Validate,
{
    match data.validate() {
        Ok(_) => Ok(()),
        Err(errors) => Err(errors.into()),
    }
}
