use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use serde::Deserialize;
use argon2::{self, Config};
use std::env;
use anyhow::{Result, Context};
use std::error::Error;
use diesel::{PgConnection, RunQueryDsl};
use crate::models::BudgetCategory;
use crate::schema::budget_categories;
use crate::errors::ServiceError;
use crate::models::IncomeSource;
use crate::schema::income_sources::dsl::*;
use crate::schema::{expense_categories, expense_transactions};
use crate::models::{ExpenseCategory, ExpenseTransaction};

// Define a type alias for the connection pool
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

// Function to establish a database connection pool
pub fn establish_connection_pool() -> Result<Pool> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .with_context(|| "DATABASE_URL environment variable not set")?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .context("Failed to create database connection pool")
}

#[derive(Debug, Deserialize, Validate)]
pub struct User {
    pub id: i32,
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    pub hashed_password: String,
    pub role_id: Option<i32>, // Optional foreign key for role
}

impl User {
    pub fn from_row(row: &Row) -> Result<Self, diesel::result::Error> {
        Ok(User {
            id: row.get("id")?,
            username: row.get("username")?,
            email: row.get("email")?,
            hashed_password: row.get("hashed_password")?,
            role_id: row.get("role_id").optional()?,
        })
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct Role {
    pub id: i32,
    #[validate(length(min = 3, max = 50))]
    pub name: String,
}

impl Role {
    pub fn from_row(row: &Row) -> Result<Self, diesel::result::Error> {
        Ok(Role {
            id: row.get("id")?,
            name: row.get("name")?,
        })
    }
}

pub async fn create_user(conn: &Pool, user: &User) -> Result<usize, diesel::result::Error> {
    let mut connection = conn.get().await?;
    diesel::insert_into(diesel::table::users::table)
        .values(user)
        .execute(&mut connection)
}

pub async fn find_user_by_username(conn: &Pool, username: &str) -> Result<Option<User>, diesel::result::Error> {
    let mut connection = conn.get().await?;
    diesel::table::users::table
        .filter(diesel::sql::eq(diesel::column::users::username, username))
        .first(&mut connection)
        .optional()
}

pub async fn create_role(conn: &Pool, role: &Role) -> Result<usize, diesel::result::Error> {
    let mut connection = conn.get().await?;
    diesel::insert_into(diesel::table::roles::table)
        .values(role)
        .execute(&mut connection)
}

pub async fn find_role_by_name(conn: &Pool, name: &str) -> Result<Option<Role>, diesel::result::Error> {
    let mut connection = conn.get().await?;
    diesel::table::roles::table
        .filter(diesel::sql::eq(diesel::column::roles::name, name))
        .first(&mut connection)
        .optional()
}

pub async fn hash_password(password: &str) -> Result<String, argon2::Error> {
    let config = Config::default();
    argon2::hash_encoded(password.as_bytes(), b"salt", &config)
}
pub async fn register_user(
    conn: &Pool,
    username: &str,
    email: &str,
    password: &str,
    role_name: Option<&str>,
  ) -> Result<usize, Box<dyn Error>> {
    let mut connection = conn.begin().await?;
  
    let hashed_password = hash_password(password).await?;
  
    let role_id = if let Some(name) = role_name {
      let role = find_role_by_name(&mut connection, name).await?;
      match role {
        Some(role) => Some(role.id),
        None => return Err(Box::new(diesel::NotFound)),
      }
    } else {
      None
    };
  
    let user = User {
      id: 0,
      username: username.to_string(),
      email: email.to_string(),
      hashed_password,
      role_id,
    };
  
    user.validate()?;
    let result = create_user(&mut connection, &user).await?;
  
    match result {
      Ok(_) => connection.commit().await?,
      Err(err) => {
        connection.rollback().await?;
        return Err(Box::new(err));
      }
    }
  
    Ok(result)
  }
  
  fn insert_into_budget_categories(conn: &PgConnection, category: BudgetCategory) -> Result<(), ServiceError> {
    use crate::schema::budget_categories::dsl::*;

    // Insert the category into the database
    diesel::insert_into(budget_categories::table)
        .values(&category)
        .execute(conn)?;

    Ok(())
}



pub fn create_income_source(conn: &PgConnection, income_source: &IncomeSource) -> Result<(), ServiceError> {
    diesel::insert_into(income_sources)
        .values(income_source)
        .execute(conn)?;
    Ok(())
}

pub fn get_income_sources(conn: &PgConnection, user_id: i32) -> Result<Vec<IncomeSource>, ServiceError> {
    income_sources
        .filter(user_id.eq(user_id))
        .load::<IncomeSource>(conn)
        .map_err(|err| err.into())
}

// Implement other CRUD operations similarly...

pub fn create_expense_category(conn: &PgConnection, name: &str) -> Result<ExpenseCategory, diesel::result::Error> {
    let new_category = ExpenseCategory {
        id: None,
        name: name.to_string(),
    };

    diesel::insert_into(expense_categories::table)
        .values(&new_category)
        .get_result(conn)
}

pub fn create_expense_transaction(
    conn: &PgConnection,
    user_id: i32,
    category_id: Option<i32>,
    amount: f64,
    date: chrono::NaiveDate,
    description: Option<String>
) -> Result<ExpenseTransaction, diesel::result::Error> {
    let new_transaction = ExpenseTransaction {
        id: None,
        user_id,
        category_id,
        amount,
        date,
        description,
    };

    diesel::insert_into(expense_transactions::table)
        .values(&new_transaction)
        .get_result(conn)
}
// Function to get all expense categories
pub fn get_all_expense_categories(conn: &PgConnection) -> Result<Vec<ExpenseCategory>, diesel::result::Error> {
    expense_categories::table.load::<ExpenseCategory>(conn)
}

// Function to get an expense category by ID
pub fn get_expense_category_by_id(conn: &PgConnection, category_id: i32) -> Result<ExpenseCategory, diesel::result::Error> {
    expense_categories::table.find(category_id).first(conn)
}

// Function to update an expense category
pub fn update_expense_category(conn: &PgConnection, category_id: i32, new_name: &str) -> Result<ExpenseCategory, diesel::result::Error> {
    diesel::update(expense_categories::table.find(category_id))
        .set(expense_categories::name.eq(new_name))
        .get_result(conn)
}

// Function to delete an expense category by ID
pub fn delete_expense_category(conn: &PgConnection, category_id: i32) -> Result<(), diesel::result::Error> {
    diesel::delete(expense_categories::table.filter(expense_categories::id.eq(category_id)))
        .execute(conn)?;
    Ok(())
}

// Function to get all expense transactions for a user
pub fn get_all_expense_transactions_for_user(conn: &PgConnection, user_id: i32) -> Result<Vec<ExpenseTransaction>, diesel::result::Error> {
    expense_transactions::table.filter(expense_transactions::user_id.eq(user_id)).load::<ExpenseTransaction>(conn)
}

// Function to get an expense transaction by ID for a user
pub fn get_expense_transaction_by_id_for_user(conn: &PgConnection, user_id: i32, transaction_id: i32) -> Result<ExpenseTransaction, diesel::result::Error> {
    expense_transactions::table.filter(expense_transactions::id.eq(transaction_id).and(expense_transactions::user_id.eq(user_id))).first(conn)
}



// use diesel::prelude::*;
// use diesel::r2d2::{self, ConnectionManager};
// use dotenv::dotenv;
// use serde::Deserialize;
// use argon2::{self, Config};
// use std::env;
// use anyhow::{Result, Context};
// use std::error::Error;
// use diesel::{PgConnection, RunQueryDsl};
// use crate::models::BudgetCategory;
// use crate::schema::budget_categories;
// use crate::errors::ServiceError;
// use crate::models::IncomeSource;
// use crate::schema::income_sources::dsl::*;
// use crate::schema::{expense_categories, expense_transactions};
// use crate::models::{ExpenseCategory, ExpenseTransaction};

// // Define a type alias for the connection pool
// pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

// // Function to establish a database connection pool
// pub fn establish_connection_pool() -> Result<Pool> {
//     dotenv().ok();
//     let database_url = env::var("DATABASE_URL")
//         .with_context(|| "DATABASE_URL environment variable not set")?;
//     let manager = ConnectionManager::<PgConnection>::new(database_url);
//     r2d2::Pool::builder()
//         .build(manager)
//         .context("Failed to create database connection pool")
// }

// #[derive(Debug, Deserialize, Validate)]
// pub struct User {
//     pub id: i32,
//     #[validate(length(min = 3, max = 50))]
//     pub username: String,
//     #[validate(email)]
//     pub email: String,
//     pub hashed_password: String,
//     pub role_id: Option<i32>, // Optional foreign key for role
// }

// impl User {
//     pub fn from_row(row: &Row) -> Result<Self, diesel::result::Error> {
//         Ok(User {
//             id: row.get("id")?,
//             username: row.get("username")?,
//             email: row.get("email")?,
//             hashed_password: row.get("hashed_password")?,
//             role_id: row.get("role_id").optional()?,
//         })
//     }
// }

// #[derive(Debug, Deserialize, Validate)]
// pub struct Role {
//     pub id: i32,
//     #[validate(length(min = 3, max = 50))]
//     pub name: String,
// }

// impl Role {
//     pub fn from_row(row: &Row) -> Result<Self, diesel::result::Error> {
//         Ok(Role {
//             id: row.get("id")?,
//             name: row.get("name")?,
//         })
//     }
// }

// pub async fn create_user(conn: &Pool, user: &User) -> Result<usize, diesel::result::Error> {
//     let mut connection = conn.get().await?;
//     diesel::insert_into(diesel::table::users::table)
//         .values(user)
//         .execute(&mut connection)
// }

// pub async fn find_user_by_username(conn: &Pool, username: &str) -> Result<Option<User>, diesel::result::Error> {
//     let mut connection = conn.get().await?;
//     diesel::table::users::table
//         .filter(diesel::sql::eq(diesel::column::users::username, username))
//         .first(&mut connection)
//         .optional()
// }

// pub async fn create_role(conn: &Pool, role: &Role) -> Result<usize, diesel::result::Error> {
//     let mut connection = conn.get().await?;
//     diesel::insert_into(diesel::table::roles::table)
//         .values(role)
//         .execute(&mut connection)
// }

// pub async fn find_role_by_name(conn: &Pool, name: &str) -> Result<Option<Role>, diesel::result::Error> {
//     let mut connection = conn.get().await?;
//     diesel::table::roles::table
//         .filter(diesel::sql::eq(diesel::column::roles::name, name))
//         .first(&mut connection)
//         .optional()
// }

// pub async fn hash_password(password: &str) -> Result<String, argon2::Error> {
//     let config = Config::default();
//     argon2::hash_encoded(password.as_bytes(), b"salt", &config)
// }
// pub async fn register_user(
//     conn: &Pool,
//     username: &str,
//     email: &str,
//     password: &str,
//     role_name: Option<&str>,
//   ) -> Result<usize, Box<dyn Error>> {
//     let mut connection = conn.begin().await?;
  
//     let hashed_password = hash_password(password).await?;
  
//     let role_id = if let Some(name) = role_name {
//       let role = find_role_by_name(&mut connection, name).await?;
//       match role {
//         Some(role) => Some(role.id),
//         None => return Err(Box::new(diesel::NotFound)),
//       }
//     } else {
//       None
//     };
  
//     let user = User {
//       id: 0,
//       username: username.to_string(),
//       email: email.to_string(),
//       hashed_password,
//       role_id,
//     };
  
//     user.validate()?;
//     let result = create_user(&mut connection, &user).await?;
  
//     match result {
//       Ok(_) => connection.commit().await?,
//       Err(err) => {
//         connection.rollback().await?;
//         return Err(Box::new(err));
//       }
//     }
  
//     Ok(result)
//   }
  
//   fn insert_into_budget_categories(conn: &PgConnection, category: BudgetCategory) -> Result<(), ServiceError> {
//     use crate::schema::budget_categories::dsl::*;

//     // Insert the category into the database
//     diesel::insert_into(budget_categories::table)
//         .values(&category)
//         .execute(conn)?;

//     Ok(())
// }



// pub fn create_income_source(conn: &PgConnection, income_source: &IncomeSource) -> Result<(), ServiceError> {
//     diesel::insert_into(income_sources)
//         .values(income_source)
//         .execute(conn)?;
//     Ok(())
// }

// pub fn get_income_sources(conn: &PgConnection, user_id: i32) -> Result<Vec<IncomeSource>, ServiceError> {
//     income_sources
//         .filter(user_id.eq(user_id))
//         .load::<IncomeSource>(conn)
//         .map_err(|err| err.into())
// }

// // Implement other CRUD operations similarly...

// pub fn create_expense_category(conn: &PgConnection, name: &str) -> Result<ExpenseCategory, diesel::result::Error> {
//     let new_category = ExpenseCategory {
//         id: None,
//         name: name.to_string(),
//     };

//     diesel::insert_into(expense_categories::table)
//         .values(&new_category)
//         .get_result(conn)
// }

// pub fn create_expense_transaction(
//     conn: &PgConnection,
//     user_id: i32,
//     category_id: Option<i32>,
//     amount: f64,
//     date: chrono::NaiveDate,
//     description: Option<String>
// ) -> Result<ExpenseTransaction, diesel::result::Error> {
//     let new_transaction = ExpenseTransaction {
//         id: None,
//         user_id,
//         category_id,
//         amount,
//         date,
//         description,
//     };

//     diesel::insert_into(expense_transactions::table)
//         .values(&new_transaction)
//         .get_result(conn)
// }
// // Function to get all expense categories
// pub fn get_all_expense_categories(conn: &PgConnection) -> Result<Vec<ExpenseCategory>, diesel::result::Error> {
//     expense_categories::table.load::<ExpenseCategory>(conn)
// }

// // Function to get an expense category by ID
// pub fn get_expense_category_by_id(conn: &PgConnection, category_id: i32) -> Result<ExpenseCategory, diesel::result::Error> {
//     expense_categories::table.find(category_id).first(conn)
// }

// // Function to update an expense category
// pub fn update_expense_category(conn: &PgConnection, category_id: i32, new_name: &str) -> Result<ExpenseCategory, diesel::result::Error> {
//     diesel::update(expense_categories::table.find(category_id))
//         .set(expense_categories::name.eq(new_name))
//         .get_result(conn)
// }

// // Function to delete an expense category by ID
// pub fn delete_expense_category(conn: &PgConnection, category_id: i32) -> Result<(), diesel::result::Error> {
//     diesel::delete(expense_categories::table.filter(expense_categories::id.eq(category_id)))
//         .execute(conn)?;
//     Ok(())
// }

// // Function to get all expense transactions for a user
// pub fn get_all_expense_transactions_for_user(conn: &PgConnection, user_id: i32) -> Result<Vec<ExpenseTransaction>, diesel::result::Error> {
//     expense_transactions::table.filter(expense_transactions::user_id.eq(user_id)).load::<ExpenseTransaction>(conn)
// }

// // Function to get an expense transaction by ID for a user
// pub fn get_expense_transaction_by_id_for_user(conn: &PgConnection, user_id: i32, transaction_id: i32) -> Result<ExpenseTransaction, diesel::result::Error> {
//     expense_transactions::table.filter(expense_transactions::id.eq(transaction_id).and(expense_transactions::user_id.eq(user_id))).first(conn)
// }
