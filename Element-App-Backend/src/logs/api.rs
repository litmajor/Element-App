
use actix_web::{web, HttpResponse, Responder};
use chrono::{Duration, Utc};
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use validator::Validate;

use crate::{
    errors::ServiceError,
    models::{Role, User, UserProfile},
    services::{auth, roles, user_profiles},
    utils::{hash_password, verify_password},
};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    pub username: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    pub email: String,
    pub full_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub message: String,
}

pub async fn register_user(data: web::Json<RegisterRequest>) -> impl Responder {
    if let Err(errors) = data.validate() {
        return HttpResponse::BadRequest().json(errors);
    }

    let hashed_password = hash_password(&data.password)?;
    match auth::register(&data.username, &hashed_password, &data.email, &data.full_name).await {
        Ok(user) => {
            // Assign a default role to the newly registered user
            let default_role = roles::get_role_by_name("User").await?;
            roles::assign_role_to_user(&user.id, default_role.id).await?;

            HttpResponse::Ok().json(RegisterResponse {
                message: "User registered successfully".to_string(),
            })
        }
        Err(ServiceError::UsernameTaken) => HttpResponse::Conflict().body("Username is already taken"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to register user"),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login_user(data: web::Json<LoginRequest>) -> impl Responder {
    if let Err(errors) = data.validate() {
        return HttpResponse::BadRequest().json(errors);
    }

    match auth::login(&data.username, &data.password).await {
        Ok(user) => {
            let token = generate_token(user.id, user.role_id)?;
            HttpResponse::Ok().json(LoginResponse { token })
        }
        Err(ServiceError::Unauthorized) => HttpResponse::Unauthorized().body("Invalid credentials"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to log in"),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

pub async fn request_password_reset(data: web::Json<PasswordResetRequest>) -> impl Responder {
    if let Err(errors) = data.validate() {
        return HttpResponse::BadRequest().json(errors);
    }

    match auth::request_password_reset(&data.email).await {
        Ok(_) => HttpResponse::Ok().body("Password reset instructions sent"),
        Err(ServiceError::UserNotFound) => HttpResponse::NotFound().body("User not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to request password reset"),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordResetConfirmRequest {
    pub token: String,
    pub new_password: String,
}

pub async fn confirm_password_reset(data: web::Json<PasswordResetConfirmRequest>) -> impl Responder {
    if let Err(errors) = data.validate() {
        return HttpResponse::BadRequest().json(errors);
    }

    match auth::confirm_password_reset(&data.token, &data.new_password).await {
        Ok(_) => HttpResponse::Ok().body("Password reset successful"),
        Err(ServiceError::InvalidToken) => HttpResponse::Unauthorized().body("Invalid or expired token"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to reset password"),
    }
}

// Authorization Middleware
pub async fn authorization_middleware(
    req: web::HttpRequest,
    next: web::NextMut<web::Local<Context<'_>>>,
) -> impl Responder {
    let header = req.headers().get("Authorization");
    if header.is_none() {
        return HttpResponse::Unauthorized().body("Missing authorization header");
    }

    let token = header.unwrap().to_str().unwrap_or_default();
    let secret_key = env::var("JWT_SECRET").unwrap_or_default();

    match verify_token(token, &secret_key) {
        Ok(claims) => {
            // Attach user ID and role ID to request context for access in subsequent handlers
            req.extensions_mut().insert(claims.user_id);
            req.extensions_mut().insert(claims.role_id);
            next.call(req).await
        }
        Err(_) => HttpResponse::Unauthorized().body("Invalid or expired token"),
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/register")
            .route(web::post().to(register_user))
            .wrap(rate_limit_middleware(4,4,4)),
    )
    .service(
        web::resource("/login")
            .route(web::post().to(login_user))
            .wrap(rate_limit_middleware(4,4,4)),
    )
    .service(
        web::resource("/password/reset")
            .route(web::post().to(request_password_reset))
            .wrap(rate_limit_middleware(4,4,4)),
    )
    .service(
        web::resource("/password/reset/confirm")
            .route(web::post().to(confirm_password_reset))
            .wrap(rate_limit_middleware(4,3,3,)),
    )
    .service(
        web::resource("/profile")
            .route(web::get().to(get_user_profile))
            .route(web::put().to(update_user_profile))
            .route(web::delete().to(delete_user_profile))
            .wrap(authorization_middleware),
    )
    .service(
        web::resource("/roles")
            .route(web::post().to(create_role))
            .route(web::get().to(get_roles))
            .wrap(authorization_middleware),
    )
    .service(
        web::resource("/users/{user_id}/role")
            .route(web::put().to(assign_role_to_user))
            .wrap(authorization_middleware),
    );
}

async fn get_user_profile(req: web::HttpRequest) -> impl Responder {
    let user_id = *req.extensions().get::<i32>().unwrap_or(&0);
    match user_profiles::get_user_profile(user_id).await {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve user profile"),
    }
}

async fn update_user_profile(
    req: web::HttpRequest,
    data: web::Json<UserProfile>,
) -> impl Responder {
    let user_id = *req.extensions().get::<i32>().unwrap_or(&0);
    match user_profiles::update_user_profile(user_id, data.into_inner()).await {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(_) => HttpResponse::InternalServerError().body("Failed to update user profile"),
    }
}

async fn delete_user_profile(req: web::HttpRequest) -> impl Responder {
    let user_id = *req.extensions().get::<i32>().unwrap_or(&0);
    match user_profiles::delete_user_profile(user_id).await {
        Ok(_) => HttpResponse::Ok().body("User profile deleted"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to delete user profile"),
    }
}

async fn create_role(data: web::Json<Role>) -> impl Responder {
    match roles::create_role(&data.name, &data.description).await {
        Ok(role) => HttpResponse::Ok().json(role),
        Err(_) => HttpResponse::InternalServerError().body("Failed to create role"),
    }
}

async fn get_roles() -> impl Responder {
    match roles::get_all_roles().await {
        Ok(roles) => HttpResponse::Ok().json(roles),
        Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve roles"),
    }
}

async fn assign_role_to_user(
    req: web::HttpRequest,
    data: web::Json<AssignRoleRequest>,
) -> impl Responder {
    let requester_role_id = *req.extensions().get::<i32>().unwrap_or(&0);
    match roles::assign_role_to_user(data.user_id, data.role_id, requester_role_id).await {
        Ok(_) => HttpResponse::Ok().body("Role assigned"),
        Err(ServiceError::Forbidden) => HttpResponse::Forbidden().body("Insufficient permissions"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to assign role"),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AssignRoleRequest {
    user_id: i32,
    role_id: i32,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub user_id: i32,
    pub role_id: i32,
}

fn generate_token(user_id: i32, role_id: i32) -> Result<String, ServiceError> {
    let secret_key = env::var("JWT_SECRET").map_err(|_| ServiceError::InternalServerError)?;
    let now = Utc::now();
    let exp = now + Duration::hours(24); // Token expires in 24 hours

    let claims = Claims {
        user_id,
        role_id,
        exp: exp.timestamp() as usize,
    };

    encode(
        &jsonwebtoken::Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret_key.as_bytes()),
    )
    .map_err(|_| ServiceError::InternalServerError)
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: i32,
    role_id: i32,
    exp: usize,
}

fn verify_token(token: &str, secret_key: &str) -> Result<Claims, ServiceError> {
    let decoding_key = DecodingKey::from_secret(secret_key.as_bytes());
    let token_data = decode::<Claims>(
        token,
        &decoding_key,
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| ServiceError::InvalidToken)?;

    Ok(token_data.claims)
}

async fn rate_limit_middleware(
    req: web::HttpRequest,
    payload: web::Payload,
    next: web::NextMut<web::Local<RateLimiter>>,
) -> impl Responder {
    let key = req.peer_addr().map(|addr| addr.to_string());
    if let Some(limiter) = next.get_ref() {
        if limiter.check_limit(key.as_deref()) {
            next.call(req, payload).await
        } else {
            HttpResponse::TooManyRequests().body("Too many requests")
        }
    } else {
        next.call(req, payload).await
    }
}

struct RateLimiter {
    max_requests: usize,
    time_window: Duration,
    request_counts: std::collections::HashMap<String, usize>,
}

impl RateLimiter {
    fn new(max_requests: usize, time_window: Duration) -> Self {
        RateLimiter {
            max_requests,
            time_window,
            request_counts: std::collections::HashMap::new(),
        }
    }

    fn check_limit(&mut self, key: Option<&str>) -> bool {
        if let Some(key) = key {
            let now = Utc::now();
            self.request_counts
                .entry(key.to_string())
                .and_modify(|count| {
                    if now - Duration::seconds(*count as i64) > self.time_window {
                        *count = 1;
                    } else {
                        *count += 1;
                    }
                })
                .or_insert(1);

            self.request_counts[key] <= self.max_requests
        } else {
            true
        }
    }
}


// use actix_web::{web, HttpResponse, Responder};
// use chrono::{Duration, Utc};
// use dotenv::dotenv;
// use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Validation};
// use serde::{Deserialize, Serialize};
// use std::env;
// use validator::Validate;
// use crate::{
//     errors::ServiceError,
//     models::{Role, User, UserProfile},
//     services::{auth, roles, user_profiles},
//     utils::{hash_password, verify_password},
//     models::PasswordChangeRequest,
// };

// #[derive(Debug, Serialize, Deserialize, Validate)]
// pub struct RegisterRequest {
//     #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
//     pub username: String,
//     #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
//     pub password: String,
//     pub email: String,
//     pub full_name: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct RegisterResponse {
//     pub message: String,
// }

// pub async fn register_user(data: web::Json<RegisterRequest>) -> impl Responder {
//     if let Err(errors) = data.validate() {
//         return HttpResponse::BadRequest().json(errors);
//     }

//     let hashed_password = hash_password(&data.password)?;
//     match auth::register(&data.username, &hashed_password, &data.email, &data.full_name).await {
//         Ok(user) => {
//             // Assign a default role to the newly registered user
//             let default_role = roles::get_role_by_name("User").await?;
//             roles::assign_role_to_user(&user.id, default_role.id).await?;

//             HttpResponse::Ok().json(RegisterResponse {
//                 message: "User registered successfully".to_string(),
//             })
//         }
//         Err(ServiceError::UsernameTaken) => HttpResponse::Conflict().body("Username is already taken"),
//         Err(_) => HttpResponse::InternalServerError().body("Failed to register user"),
//     }
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct LoginRequest {
//     pub username: String,
//     pub password: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct LoginResponse {
//     pub token: String,
// }

// pub async fn login_user(data: web::Json<LoginRequest>) -> impl Responder {
//     if let Err(errors) = data.validate() {
//         return HttpResponse::BadRequest().json(errors);
//     }

//     match auth::login(&data.username, &data.password).await {
//         Ok(user) => {
//             let token = generate_token(user.id, user.role_id)?;
//             HttpResponse::Ok().json(LoginResponse { token })
//         }
//         Err(ServiceError::Unauthorized) => HttpResponse::Unauthorized().body("Invalid credentials"),
//         Err(_) => HttpResponse::InternalServerError().body("Failed to log in"),
//     }
// }

// #[derive(Debug, Deserialize)]
// pub struct PasswordChangeRequest {
//     pub current_password: String,
//     pub new_password: String,
// }

// pub async fn change_password(data: web::Json<PasswordChangeRequest>) -> impl Responder {
//     match auth::change_password(&data.current_password, &data.new_password).await {
//         Ok(_) => HttpResponse::Ok().body("Password changed successfully"),
//         Err(ServiceError::Unauthorized) => HttpResponse::Unauthorized().body("Invalid current password"),
//         Err(_) => HttpResponse::InternalServerError().body("Failed to change password"),
//     }
// }

// pub fn init_routes(cfg: &mut web::ServiceConfig) {
//     cfg.service(
//         web::resource("/change-password")
//             .route(web::post().to(change_password))
//     );
// }

// use crate::errors::ServiceError;
// use crate::utils::hash_password;

// pub async fn change_password(current_password: &str, new_password: &str) -> Result<(), ServiceError> {
//     // Check if the current password is correct
//     let user_id = get_user_id_from_session(); // Implement this function to get the user ID from the session
//     let user = find_user_by_id(user_id).await?;
//     let is_password_valid = verify_password(current_password, &user.hashed_password)?;
//     if !is_password_valid {
//         return Err(ServiceError::Unauthorized);
//     }

//     // Hash the new password
//     let hashed_new_password = hash_password(new_password)?;

//     // Update the user's password in the database
//     update_user_password(user_id, &hashed_new_password).await?;

//     Ok(())
// }



// #[derive(Debug, Serialize, Deserialize)]
// pub struct PasswordResetRequest {
//     pub email: String,
// }

// pub async fn request_password_reset(data: web::Json<PasswordResetRequest>) -> impl Responder {
//     if let Err(errors) = data.validate() {
//         return HttpResponse::BadRequest().json(errors);
//     }

//     match auth::request_password_reset(&data.email).await {
//         Ok(_) => HttpResponse::Ok().body("Password reset instructions sent"),
//         Err(ServiceError::UserNotFound) => HttpResponse::NotFound().body("User not found"),
//         Err(_) => HttpResponse::InternalServerError().body("Failed to request password reset"),
//     }
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct PasswordResetConfirmRequest {
//     pub token: String,
//     pub new_password: String,
// }

// pub async fn confirm_password_reset(data: web::Json<PasswordResetConfirmRequest>) -> impl Responder {
//     if let Err(errors) = data.validate() {
//         return HttpResponse::BadRequest().json(errors);
//     }

//     match auth::confirm_password_reset(&data.token, &data.new_password).await {
//         Ok(_) => HttpResponse::Ok().body("Password reset successful"),
//         Err(ServiceError::InvalidToken) => HttpResponse::Unauthorized().body("Invalid or expired token"),
//         Err(_) => HttpResponse::InternalServerError().body("Failed to reset password"),
//     }
// }

// // Authorization Middleware
// pub async fn authorization_middleware(
//     req: web::HttpRequest,
//     next: web::NextMut<web::Local<Context<'_>>>,
// ) -> impl Responder {
//     let header = req.headers().get("Authorization");
//     if header.is_none() {
//         return HttpResponse::Unauthorized().body("Missing authorization header");
//     }

//     let token = header.unwrap().to_str().unwrap_or_default();
//     let secret_key = env::var("JWT_SECRET").unwrap_or_default();

//     match verify_token(token, &secret_key) {
//         Ok(claims) => {
//             // Attach user ID and role ID to request context for access in subsequent handlers
//             req.extensions_mut().insert(claims.user_id);
//             req.extensions_mut().insert(claims.role_id);
//             next.call(req).await
//         }
//         Err(_) => HttpResponse::Unauthorized().body("Invalid or expired token"),
//     }
// }

// pub fn init_routes(cfg: &mut web::ServiceConfig) {
//     cfg.service(
//         web::resource("/register")
//             .route(web::post().to(register_user))
//             .wrap(rate_limit_middleware(4,4,4)),
//     )
//     .service(
//         web::resource("/login")
//             .route(web::post().to(login_user))
//             .wrap(rate_limit_middleware(4,4,4)),
//     )
//     .service(
//         web::resource("/password/reset")
//             .route(web::post().to(request_password_reset))
//             .wrap(rate_limit_middleware(4,4,4)),
//     )
//     .service(
//         web::resource("/password/reset/confirm")
//             .route(web::post().to(confirm_password_reset))
//             .wrap(rate_limit_middleware(4,3,3,)),
//     )
//     .service(
//         web::resource("/profile")
//             .route(web::get().to(get_user_profile))
//             .route(web::put().to(update_user_profile))
//             .route(web::delete().to(delete_user_profile))
//             .wrap(authorization_middleware),
//     )
//     .service(
//         web::resource("/roles")
//             .route(web::post().to(create_role))
//             .route(web::get().to(get_roles))
//             .wrap(authorization_middleware),
//     )
//     .service(
//         web::resource("/users/{user_id}/role")
//             .route(web::put().to(assign_role_to_user))
//             .wrap(authorization_middleware),
//     );
// }

// async fn get_user_profile(req: web::HttpRequest) -> impl Responder {
//     let user_id = *req.extensions().get::<i32>().unwrap_or(&0);
//     match user_profiles::get_user_profile(user_id).await {
//         Ok(profile) => HttpResponse::Ok().json(profile),
//         Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve user profile"),
//     }
// }

// async fn update_user_profile(
//     req: web::HttpRequest,
//     data: web::Json<UserProfile>,
// ) -> impl Responder {
//     let user_id = *req.extensions().get::<i32>().unwrap_or(&0);
//     match user_profiles::update_user_profile(user_id, data.into_inner()).await {
//         Ok(profile) => HttpResponse::Ok().json(profile),
//         Err(_) => HttpResponse::InternalServerError().body("Failed to update user profile"),
//     }
// }

// async fn delete_user_profile(req: web::HttpRequest) -> impl Responder {
//     let user_id = *req.extensions().get::<i32>().unwrap_or(&0);
//     match user_profiles::delete_user_profile(user_id).await {
//         Ok(_) => HttpResponse::Ok().body("User profile deleted"),
//         Err(_) => HttpResponse::InternalServerError().body("Failed to delete user profile"),
//     }
// }

// async fn create_role(data: web::Json<Role>) -> impl Responder {
//     match roles::create_role(&data.name, &data.description).await {
//         Ok(role) => HttpResponse::Ok().json(role),
//         Err(_) => HttpResponse::InternalServerError().body("Failed to create role"),
//     }
// }

// async fn get_roles() -> impl Responder {
//     match roles::get_all_roles().await {
//         Ok(roles) => HttpResponse::Ok().json(roles),
//         Err(_) => HttpResponse::InternalServerError().body("Failed to retrieve roles"),
//     }
// }

// async fn assign_role_to_user(
//     req: web::HttpRequest,
//     data: web::Json<AssignRoleRequest>,
// ) -> impl Responder {
//     let requester_role_id = *req.extensions().get::<i32>().unwrap_or(&0);
//     match roles::assign_role_to_user(data.user_id, data.role_id, requester_role_id).await {
//         Ok(_) => HttpResponse::Ok().body("Role assigned"),
//         Err(ServiceError::Forbidden) => HttpResponse::Forbidden().body("Insufficient permissions"),
//         Err(_) => HttpResponse::InternalServerError().body("Failed to assign role"),
//     }
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct AssignRoleRequest {
//     user_id: i32,
//     role_id: i32,
// }

// #[derive(Debug, Clone)]
// pub struct Context {
//     pub user_id: i32,
//     pub role_id: i32,
// }

// fn generate_token(user_id: i32, role_id: i32) -> Result<String, ServiceError> {
//     let secret_key = env::var("JWT_SECRET").map_err(|_| ServiceError::InternalServerError)?;
//     let now = Utc::now();
//     let exp = now + Duration::hours(24); // Token expires in 24 hours

//     let claims = Claims {
//         user_id,
//         role_id,
//         exp: exp.timestamp() as usize,
//     };

//     encode(
//         &jsonwebtoken::Header::new(Algorithm::HS256),
//         &claims,
//         &EncodingKey::from_secret(secret_key.as_bytes()),
//     )
//     .map_err(|_| ServiceError::InternalServerError)
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct Claims {
//     user_id: i32,
//     role_id: i32,
//     exp: usize,
// }

// fn verify_token(token: &str, secret_key: &str) -> Result<Claims, ServiceError> {
//     let decoding_key = DecodingKey::from_secret(secret_key.as_bytes());
//     let token_data = decode::<Claims>(
//         token,
//         &decoding_key,
//         &Validation::new(Algorithm::HS256),
//     )
//     .map_err(|_| ServiceError::InvalidToken)?;

//     Ok(token_data.claims)
// }

// async fn rate_limit_middleware(
//     req: web::HttpRequest,
//     payload: web::Payload,
//     next: web::NextMut<web::Local<RateLimiter>>,
// ) -> impl Responder {
//     let key = req.peer_addr().map(|addr| addr.to_string());
//     if let Some(limiter) = next.get_ref() {
//         if limiter.check_limit(key.as_deref()) {
//             next.call(req, payload).await
//         } else {
//             HttpResponse::TooManyRequests().body("Too many requests")
//         }
//     } else {
//         next.call(req, payload).await
//     }
// }

// struct RateLimiter {
//     max_requests: usize,
//     time_window: Duration,
//     request_counts: std::collections::HashMap<String, usize>,
// }

// impl RateLimiter {
//     fn new(max_requests: usize, time_window: Duration) -> Self {
//         RateLimiter {
//             max_requests,
//             time_window,
//             request_counts: std::collections::HashMap::new(),
//         }
//     }

//     fn check_limit(&mut self, key: Option<&str>) -> bool {
//         if let Some(key) = key {
//             let now = Utc::now();
//             self.request_counts
//                 .entry(key.to_string())
//                 .and_modify(|count| {
//                     if now - Duration::seconds(*count as i64) > self.time_window {
//                         *count = 1;
//                     } else {
//                         *count += 1;
//                     }
//                 })
//                 .or_insert(1);

//             self.request_counts[key] <= self.max_requests
//         } else {
//             true
//         }
//     }
// }