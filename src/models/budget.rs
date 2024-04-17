// Define a struct to represent a budget category
#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetCategory {
    pub id: i32,
    pub user_id: i32, // Foreign key to the user table
    pub name: String,
    pub limit_amount: f64,
    pub time_period: String, // e.g., monthly, weekly, yearly
}

// Define functions to interact with budget categories
impl BudgetCategory {
    // Function to create a new budget category
    pub fn create(user_id: i32, name: String, limit_amount: f64, time_period: String) -> Result<BudgetCategory, String> {
        // Validate inputs
        if name.is_empty() {
            return Err("Category name cannot be empty".to_string());
        }
        if limit_amount <= 0.0 {
            return Err("Budget limit must be a positive number".to_string());
        }
        
        // Insert the new budget category into the database
        // (Assuming you have a function to interact with the database)
        let new_category = BudgetCategory {
            id: 0, // Auto-generated ID
            user_id,
            name,
            limit_amount,
            time_period,
        };
        // Insert new_category into the database
        // (Assuming you have a function to insert data into the database)
        let inserted_category = insert_into_budget_categories(new_category)?;
        
        Ok(inserted_category)
    }
}

// Function to insert a new budget category into the database
fn insert_into_budget_categories(conn: &PgConnection, category: BudgetCategory) -> Result<(), ServiceError> {
    use crate::schema::budget_categories::dsl::*;

    // Insert the category into the database
    diesel::insert_into(budget_categories::table)
        .values(&category)
        .execute(conn)?;

    Ok(())
}

// Example usage:
fn main() {
    // Assume user inputs
    let user_id = 1;
    let category_name = "Groceries";
    let budget_limit = 500.0;
    let time_period = "monthly";
    
    // Create a new budget category
    match BudgetCategory::create(user_id, category_name.to_string(), budget_limit, time_period.to_string()) {
        Ok(category) => println!("Budget category created: {:?}", category),
        Err(err) => eprintln!("Failed to create budget category: {}", err),
    }
}

use chrono::{Date, Duration, Utc};

impl InMemoryStore {
    // Function to calculate budget utilization for a category within a time period
    pub fn calculate_budget_utilization(&self, category_id: i32, start_date: Date<Utc>, end_date: Date<Utc>) -> Option<f64> {
        let mut total_expense = 0.0;
        for transaction in &self.transactions {
            if transaction.category_id == Some(category_id) &&
                transaction.date >= start_date && transaction.date <= end_date {
                total_expense += transaction.amount;
            }
        }

        let category = self.categories.iter().find(|c| c.id == category_id)?;
        category.budget.map(|budget| total_expense / budget)
    }

    // Function to aggregate income and expenses by category within a time period
    pub fn aggregate_income_and_expenses(&self, start_date: Date<Utc>, end_date: Date<Utc>) -> Result<Vec<(String, f64, f64)>, String> {
        let mut results: Vec<(String, f64, f64)> = Vec::new();
        for category in &self.categories {
            let total_income = self.income_sources.iter()
                .filter(|s| s.date >= start_date && s.date <= end_date)
                .filter(|s| s.name == category.name) // Assuming income source name matches category name
                .fold(0.0, |acc, s| acc + s.amount);

            let total_expense = self.calculate_budget_utilization(category.id, start_date, end_date).unwrap_or(0.0);
            results.push((category.name.clone(), total_income, total_expense));
        }
        Ok(results)
    }
}
