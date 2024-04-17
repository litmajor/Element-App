mod auth;
mod transactions;
mod projects;
mod models;
mod errors;
mod logging;
mod api;
mod db;
mod usermanagement;
mod utils;
mod tests;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;

use crate::expense_store::DieselExpenseStore;
use crate::income_source_store::DieselIncomeSourceStore;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initcialize logging
    // (Assuming you're using env_logger)
    dotenv().ok();
    env_logger::init();

    // Parse environment variables
    // Load from .env file or system environment
    // (dotenv is used to load environment variables from a .env file)
    dotenv::dotenv().ok();

    // Establish database connection
    // Initialize and establish a connection to your database
    // (Assuming you have a function `establish_connection` in the `database` module)
    let db_conn = database::establish_connection().await.expect("Failed to establish database connection");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // Configure routes and middleware
            // (Assuming you have an `init_routes` function in your `auth` module)
            .configure(auth::init_routes)
            // Add other middleware or configurations as needed
            // .wrap(middleware::Logger::default())
    })
    .bind("127.0.0.1:8080")? // Bind to localhost port 8080
    .run()
    .await
}

fn main() {
    // Set up the Diesel connection pool
    let database_url = "postgres://username:password@localhost/your_database";
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).expect("Failed to create pool");

    // Use the DieselExpenseStore and DieselIncomeSourceStore in your application
    let conn = pool.get().expect("Failed to get connection from pool");
    let expense_store = DieselExpenseStore(&conn);
    let income_source_store = DieselIncomeSourceStore(&conn);

    // Call the methods on the expense and income source stores
    let new_expense_category = expense_store.create("Rent");
    let new_income_source = income_source_store.create(1, "Salary", 5000.0, None);
    // ...
}