use chrono::{Date, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseCategory {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseTransaction {
    pub id: i32,
    pub user_id: i32,
    pub category_id: Option<i32>, // Can be None if not categorized
    pub amount: f64,
    pub date: Date<Utc>, // Date of expense (UTC)
    pub description: Option<String>,
    pub goal_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomeSource {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub amount: f64,
    pub date: Date<Utc>, // Date of income received (UTC)
    pub details: Option<String>, // Optional additional details
}

// Trait to define CRUD operations for expense categories
pub trait ExpenseCategoryStore {
    fn create(&self, name: &str) -> Result<ExpenseCategory, String>;
    fn get_all(&self) -> Result<Vec<ExpenseCategory>, String>;
    fn get_by_id(&self, id: i32) -> Result<ExpenseCategory, String>;
    fn update(&self, id: i32, name: &str) -> Result<ExpenseCategory, String>;
    fn delete(&self, id: i32) -> Result<(), String>;
}

// Trait to define CRUD operations for expense transactions
pub trait ExpenseTransactionStore {
    fn create(
        &self,
        user_id: i32,
        category_id: Option<i32>,
        amount: f64,
        description: Option<String>,
        goal_id: Option<i32>,
    ) -> Result<ExpenseTransaction, String>;
    fn get_all(&self, user_id: i32) -> Result<Vec<ExpenseTransaction>, String>;
    fn get_by_id(&self, user_id: i32, id: i32) -> Result<ExpenseTransaction, String>;
    fn update(
        &self,
        id: i32,
        category_id: Option<i32>,
        amount: f64,
        description: Option<String>,
        goal_id: Option<i32>,
    ) -> Result<ExpenseTransaction, String>;
    fn delete(&self, user_id: i32, id: i32) -> Result<(), String>;
}

// Trait to define CRUD operations for income sources
pub trait IncomeSourceStore {
    fn create(
        &self,
        user_id: i32,
        name: &str,
        amount: f64,
        details: Option<String>,
        goal_id: Option<i32>,
    ) -> Result<IncomeSource, String>;
    fn get_all(&self, user_id: i32) -> Result<Vec<IncomeSource>, String>;
    fn get_by_id(&self, user_id: i32, id: i32) -> Result<IncomeSource, String>;
    fn update(
        &self,
        id: i32,
        name: &str,
        amount: f64,
        details: Option<String>,
        goal_id: Option<i32>,
    ) -> Result<IncomeSource, String>;
    fn delete(&self, user_id: i32, id: i32) -> Result<(), String>;
}

// Example implementation using a dummy in-memory store
pub struct InMemoryExpenseStore {
    categories: Vec<ExpenseCategory>,
    transactions: Vec<ExpenseTransaction>,
    income_sources: Vec<IncomeSource>,
}

impl InMemoryExpenseStore {
    pub fn new() -> Self {
        Self {
            categories: Vec::new(),
            transactions: Vec::new(),
            income_sources: Vec::new(),
        }
    }
}

impl ExpenseCategoryStore for InMemoryExpenseStore {
    // Implemented previously
}

impl ExpenseTransactionStore for InMemoryExpenseStore {
    // Implemented previously
}

impl IncomeSourceStore for InMemoryExpenseStore {
    fn create(
        &self,
        user_id: i32,
        name: &str,
        amount: f64,
        details: Option<String>,
    ) -> Result<IncomeSource, String> {
        if name.is_empty() {
            return Err("Income source name cannot be empty".to_string());
        }

        if amount <= 0.0 {
            return Err("Income amount must be a positive number".to_string());
        }

        let id = (self.income_sources.len() + 1) as i32;
        let new_source = IncomeSource {
            id,
            user_id,
            name: name.to_string(),
            amount,
            date: Utc::now().date(),
            details,
        };
        self.income_sources.push(new_source.clone());
        Ok(new_source)
    }

    fn get_all(&self, user_id: i32) -> Result<Vec<IncomeSource>, String> {
        let mut user_sources = Vec::new();
        for source in &self.income_sources {
            if source.user_id == user_id {
                user_sources.push(source.clone());
            }
        }
        Ok(user_sources)
    }

    fn get_by_id(&self, user_id: i32, id: i32) -> Result<IncomeSource, String> {
        for source in &self.income_sources {
            if source.user_id == user_id && source.id == id {
                return Ok(source.clone());
            }
        }
        Err(format!("Income source with ID {} not found for user {}", id, user_id))
    }

    fn update(
        &self,
        id: i32,
        name: &str,
        amount: f64,
        details: Option<String>,
    ) -> Result<IncomeSource, String> {
        let mut updated_source = None;
        for i in 0..self.income_sources.len() {
            if self.income_sources[i].id == id {
                if name.is_empty() {
                    return Err("Income source name cannot be empty".to_string());
                }
                if amount <= 0.0 {
                    return Err("Income amount must be a positive number".to_string());
                }
                self.income_sources[i].name = name.to_string();
                self.income_sources[i].amount = amount;
                self.income_sources[i].details = details.clone();
                updated_source = Some(self.income_sources[i].clone());
                break;
            }
        }
        match updated_source {
            Some(source) => Ok(source),
            None => Err(format!("Income source with ID {} not found", id)),
        }
    }

    fn delete(&self, user_id: i32, id: i32) -> Result<(), String> {
        let mut deleted = false;
        self.income_sources.retain(|source| {
            if source.user_id == user_id && source.id == id {
                deleted = true;
                false
            } else {
                true
            }
        });
        if deleted {
            Ok(())
        } else {
            Err(format!("Income source with ID {} not found for user {}", id, user_id))
        }
    }
}