use diesel::{Queryable, result::Error as QueryableError, Row};
use serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime, DateTime, Utc};
use uuid::Uuid;
use validator::Validate;
use diesel::{Queryable, Insertable};
use chrono::NaiveDate;

#[derive(Queryable, Serialize, Deserialize, Validate)]
pub struct User {
    pub id: i32,
    #[validate(email)]
    pub email: String,
    pub balance: f64,
    pub created_at: NaiveDateTime,
    pub user_type: UserType,
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
  pub username: String,
  pub email: String,
  pub hashed_password: String,
  pub profile_picture: Option<String>,
}

#[derive(Queryable, Serialize, Deserialize, Validate)]

use chrono::NaiveDate;
use diesel::{Queryable, Insertable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable)]
pub struct EscrowAccount {
    pub id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub release_conditions: String,
}

#[derive(Debug, Insertable)]
#[table_name = "escrow_accounts"]
pub struct NewEscrowAccount {
    pub sender_id: i32,
    pub receiver_id: i32,
    pub amount: f64,
    pub currency: String,
    pub release_conditions: String,
}

#[derive(Debug, Queryable)]
pub struct Budget {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Insertable)]
#[table_name = "budgets"]
pub struct NewBudget {
    pub user_id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Queryable)]
pub struct BudgetCategory {
    pub id: i32,
    pub budget_id: i32,
    pub name: String,
    pub planned_amount: f64,
}

#[derive(Debug, Insertable)]
#[table_name = "budget_categories"]
pub struct NewBudgetCategory {
    pub budget_id: i32,
    pub name: String,
    pub planned_amount: f64,
}

#[derive(Debug, Queryable)]
pub struct SavingsGoal {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub target_amount: f64,
    pub deadline: NaiveDate,
}

#[derive(Debug, Insertable)]
#[table_name = "savings_goals"]
pub struct NewSavingsGoal {
    pub user_id: i32,
    pub name: String,
    pub target_amount: f64,
    pub deadline: NaiveDate,
}

pub struct Transaction {
    pub id: Uuid,
    pub escrow_account_id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub amount: f64,
    #[validate(length(min = 3))]
    pub status: String,
    pub fee: f64,
    pub created_at: chrono::NaiveDateTime,
    pub custom_data: Option<serde_json::Value>, // Optional JSON field for custom data
    pub transaction_type: TransactionType,
    pub(crate) description: Option<String>,
    pub(crate) project_id: i32, // Deposit, Fee, Payout
}

impl User {
    pub fn from_row(row: &Row) -> Result<User, QueryableError> {
        Ok(User {
            id: row.get("id")?,
            email: row.get("email")?,
            balance: row.get("balance")?,
            created_at: row.get("created_at")?,
            user_type: row.get("user_type")?,
        })
    }
}

#[derive(diesel::Insertable, diesel::AsChangeset, Validate)]
#[diesel(table_name = projects)]
pub struct Project {
    pub id: i32,
    #[validate(length(min = 3, max = 100))]
    pub name: String,
    pub budget: Option<f64>,
    pub client_id: Option<i32>, // Foreign key to client table (optional)
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>, // Optional end date for the project
    #[validate(length(min = 3, max = 50))]
    pub category: Option<String>,  // Project category (e.g., web development, design)
    pub description: Option<String>, // More detailed project description
    pub files: Option<Vec<String>>,  // List of file paths related to the project
    pub client_name: Option<String>, // Client name (optional)
    pub priority: Option<i32>,       // Project priority level (e.g., 1 for high)
    pub communication_channels: Option<Vec<String>>, // Communication channels (e.g., email, Slack)
    pub milestones: Option<Vec<Milestone>>, // List of associated milestones (optional)
    pub enable_milestones: bool,    // Flag indicating if milestone-based payments are enabled
    pub status: ProjectStatus,       // Project status (Pending, InProgress, Completed, Cancelled)
    pub dependent_projects: Vec<i32>,// List of project IDs that this project depends on
}

#[derive(diesel::Insertable, diesel::AsChangeset, Validate)]
#[diesel(table_name = milestones)]
pub struct Milestone {
    pub id: i32,
    pub project_id: i32,
    #[validate(length(min = 3, max = 100))]
    pub description: String,
    pub due_date: DateTime<Utc>,
    pub completion_status: bool,
    pub associated_payment: f64,
    pub deliverables: Option<Vec<String>>,
    pub acceptance_criteria: Option<String>,
    pub estimated_effort: Option<i32>,
    pub start_date: Option<DateTime<Utc>>,
    pub payment_released: bool,
    pub deadline: DateTime<Utc>,
    pub budget: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TransactionType {
    Deposit,
    Fee,
    Payout,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UserType {
    Client,
    Freelancer,
}

use diesel::{Queryable, Insertable};
use serde::{Deserialize, Serialize};


// Define structs for budget categories, income sources, and planned expenses

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct IncomeSource {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub amount: f64,
    pub date: NaiveDate,
    pub details: Option<String>,
}

#[derive(Insertable)]
#[table_name = "income_sources"]
pub struct NewIncomeSource<'a> {
    pub user_id: i32,
    pub name: &'a str,
    pub amount: f64,
    pub date: Date<Utc>,
    pub details: Option<&'a str>,
}

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct PlannedExpense {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub amount: f64,
    pub due_date: NaiveDate,
}

#[derive(Debug, Insertable)]
#[table_name = "planned_expenses"]
pub struct NewPlannedExpense {
    pub user_id: i32,
    pub name: String,
    pub amount: f64,
    pub due_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseCategory {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "expense_transactions"]
pub struct ExpenseTransaction {
    pub id: i32,
    pub user_id: i32,
    pub category_id: Option<i32>, // Can be None if not categorized
    pub amount: f64,
    pub date: Date<Utc>, // Date of expense (UTC)
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct FinancialGoal {
    pub id: i32,
    pub name: String,
    pub target_amount: f64,
    pub target_date: Date<Utc>,
    pub current_amount: f64,
}

pub mod trend_analysis {
    use crate::models::expense::{Expense, ExpenseCategory};
    use crate::models::income::{Income, IncomeSource};
    use crate::models::user::User;
    use crate::repositories::expense_repository::ExpenseRepository;
    use crate::repositories::income_repository::IncomeRepository;
    use crate::repositories::user_repository::UserRepository;
    use chrono::{Date, Utc};
    use std::collections::HashMap;

    pub struct TrendAnalysis {
        user: User,
        income_totals: Vec<f64>,
        expense_totals: Vec<f64>,
        income_averages: Vec<f64>,
        expense_averages: Vec<f64>,
        income_percentage_changes: Vec<Option<f64>>,
        expense_percentage_changes: Vec<Option<f64>>,
    }

    impl TrendAnalysis {
        pub fn new(user: User) -> Self {
            Self {
                user,
                income_totals: Vec::new(),
                expense_totals: Vec::new(),
                income_averages: Vec::new(),
                expense_averages: Vec::new(),
                income_percentage_changes: Vec::new(),
                expense_percentage_changes: Vec::new(),
            }
        }

        pub fn add_income(&mut self, amount: f64) {
            self.income_totals.push(amount);
        }

        pub fn add_expense(&mut self, amount: f64) {
            self.expense_totals.push(amount);
        }

        pub fn calculate_averages(&mut self) {
            self.income_averages = calculate_moving_average(&self.income_totals, 7);
            self.expense_averages = calculate_moving_average(&self.expense_totals, 7);
        }
    }

    fn calculate_moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
        let mut averages = Vec::with_capacity(values.len());
        let mut sum = 0.0;

        for i in 0..values.len() {
            sum += values[i];

            if i >= window_size - 1 {
                averages.push(sum / window_size as f64);
                sum -= values[i - window_size + 1];
            }
        }

        averages
    }

    fn calculate_percentage_change(current: f64, previous: f64) -> Option<f64> {
        if previous == 0.0 {
            return None; // Avoid division by zero
        }
        let change = (current - previous) / previous;
        Some(change * 100.0) // Express as percentage
    }
}
