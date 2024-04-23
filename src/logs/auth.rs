// auth.rs

use crate::{
    errors::ServiceError,
    models::{User as DbUser, UserType},
    logging::{log_user_registration, log_user_login, log_invalid_password, log_jwt_validation},
};
use diesel::prelude::*;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation};
use argon2::{hash_encoded, verify_encoded_ext};
use dotenv::dotenv;
use rand::{RngCore, thread_rng};
use std::{env, error::Error};

// Structs
#[derive(Serialize, Deserialize)]
pub struct RegisterUser {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32, // User ID
    pub exp: usize, // Expiration time (in seconds)
}

// Authentication Functions
pub fn register(conn: &PgConnection, user: RegisterUser) -> Result<(), ServiceError> {
    // Generate salt
    let salt = generate_salt();

    // Hash password using argon2 with salt
    let hashed_password = hash_password_with_salt(user.password.as_bytes(), &salt)?;

    // Insert user into database
    let new_user = crate::models::User {
        username: user.username,
        hashed_password,
        salt: salt.to_vec(),
    };
    diesel::insert_into(crate::models::users::table)
        .values(&new_user)
        .execute(&mut conn)?;

    log_user_registration(&user.username);
    Ok(())
}

pub fn login(conn: &PgConnection, user: LoginUser) -> Result<String, ServiceError> {
    // Query user by username
    let user_record = crate::models::users::table
        .filter(crate::models::users::username.eq(&user.username))
        .first::<crate::models::User>(conn)?;

    // Verify password
    if !verify_password_with_salt(&user_record.hashed_password, user.password.as_bytes(), &user_record.salt)? {
        log_invalid_password(&user.username);
        return Err(ServiceError::Unauthorized);
    }

    // Load JWT secret key from environment variable
    dotenv().ok(); // Load environment variables
    let secret_key = env::var("JWT_SECRET_KEY")?;

    // Generate JWT with user ID as claim
    let claims = Claims {
        sub: user_record.id,
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };
    let token = encode(&Header::new(Algorithm::HS256), &claims, secret_key.as_bytes())?;

    log_user_login(&user.username);
    Ok(token)
}

pub fn validate_jwt(token: &str) -> Result<Claims, ServiceError> {
    // Load JWT secret key from environment variable
    dotenv().ok(); // Load environment variables
    let secret_key = env::var("JWT_SECRET_KEY")?;

    // Define validation options
    let validation = Validation::new(Algorithm::HS256)
        .set_issuer(&["your_issuer".to_string()]) // Optional issuer validation
        .build();

    // Decode and validate JWT
    let decoded_token = decode::<Claims>(&token, secret_key.as_bytes(), &validation)?;
    let claims = decoded_token.claims;

    log_jwt_validation(claims.sub);
    Ok(claims)
}

// Function to generate a random salt
fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    thread_rng().fill_bytes(&mut salt);
    salt
}

fn hash_password_with_salt(password: &[u8], salt: &[u8]) -> Result<String, ServiceError> {
    let hashed_password = hash_encoded(password, salt, &argon2::Config::default())?;
    Ok(hashed_password)
}

fn verify_password_with_salt(hashed_password: &str, password: &[u8], salt: &[u8]) -> Result<bool, ServiceError> {
    Ok(verify_encoded_ext(hashed_password, password, salt, &argon2::Config::default())?)
}

use crate::errors::ServiceError;
use crate::utils::hash_password;

pub async fn change_password(current_password: &str, new_password: &str) -> Result<(), ServiceError> {
    // Check if the current password is correct
    let user_id = get_user_id_from_session(); // Implement this function to get the user ID from the session
    let user = find_user_by_id(user_id).await?;
    let is_password_valid = verify_password(current_password, &user.hashed_password)?;
    if !is_password_valid {
        return Err(ServiceError::Unauthorized);
    }

    // Hash the new password
    let hashed_new_password = hash_password(new_password)?;

    // Update the user's password in the database
    update_user_password(user_id, &hashed_new_password).await?;

    Ok(())
}
