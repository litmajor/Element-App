use chrono::{Date, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseCategory {
    pub id: i32,
    pub name: String,
}
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct ExpenseCategory {
    pub id: i32,
    pub name: String,
    pub budget: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ExpenseTransaction {
    pub id: i32,
    pub user_id: i32,
    pub category_id: Option<i32>,
    pub amount: f64,
    pub goal_id: Option<i32>,
    pub date: chrono::Date<chrono::Utc>,
    pub description: Option<String>,
    pub budget: f64,
}

pub trait ExpenseCategoryStore {
    fn create(&self, name: &str) -> Result<ExpenseCategory, String>;
    fn get_all(&self) -> Result<Vec<ExpenseCategory>, String>;
    fn get_by_id(&self, id: i32) -> Result<ExpenseCategory, String>;
    fn update(&self, id: i32, name: &str) -> Result<ExpenseCategory, String>;
    fn delete(&self, id: i32) -> Result<(), String>;
}

pub trait ExpenseTransactionStore {
    fn create(
        &self,
        user_id: i32,
        category_id: Option<i32>,
        amount: f64,
        description: Option<String>,
        budget: f64,
    ) -> Result<ExpenseTransaction, String>;
    fn get_all(&self, user_id: i32) -> Result<Vec<ExpenseTransaction>, String>;
    fn get_by_id(&self, user_id: i32, id: i32) -> Result<ExpenseTransaction, String>;
    fn update(
        &self,
        id: i32,
        category_id: Option<i32>,
        amount: f64,
        description: Option<String>,
    ) -> Result<ExpenseTransaction, String>;
    fn delete(&self, user_id: i32, id: i32) -> Result<(), String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseTransaction {
    pub id: i32,
    pub user_id: i32,
    pub category_id: Option<i32>, // Can be None if not categorized
    pub amount: f64,
    pub goal_id: Option<i32>,
    pub date: Date<Utc>, // Date of expense (UTC)
    pub description: Option<String>,
    pub budget: f64,
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
        budget: f64,
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
    ) -> Result<ExpenseTransaction, String>;
    fn delete(&self, user_id: i32, id: i32) -> Result<(), String>;
}

// Example implementation using a dummy in-memory store
pub struct InMemoryExpenseStore {
    categories: Vec<ExpenseCategory>,
    transactions: Vec<ExpenseTransaction>,
}

impl InMemoryExpenseStore {
    pub fn new() -> Self {
        Self {
            categories: Vec::new(),
            transactions: Vec::new(),
        }
    }
}

impl ExpenseCategoryStore for InMemoryExpenseStore {
    fn create(&self, name: &str) -> Result<ExpenseCategory, String> {
        if name.is_empty() {
            return Err("Expense category name cannot be empty".to_string());
        }

        let id = (self.categories.len() + 1) as i32;
        let new_category = ExpenseCategory {
            id,
            name: name.to_string(),
        };
        self.categories.push(new_category.clone());
        Ok(new_category)
    }

    fn get_all(&self) -> Result<Vec<ExpenseCategory>, String> {
        Ok(self.categories.clone())
    }

    fn get_by_id(&self, id: i32) -> Result<ExpenseCategory, String> {
        for category in &self.categories {
            if category.id == id {
                return Ok(category.clone());
            }
        }
        Err(format!("Expense category with ID {} not found", id))
    }

    fn update(&self, id: i32, name: &str) -> Result<ExpenseCategory, String> {
        if name.is_empty() {
            return Err("Expense category name cannot be empty".to_string());
        }

        let mut updated_category = None;
        for i in 0..self.categories.len() {
            if self.categories[i].id == id {
                self.categories[i].name = name.to_string();
                updated_category = Some(self.categories[i].clone());
                break;
            }
        }

        match updated_category {
            Some(category) => Ok(category),
            None => Err(format!("Expense category with ID {} not found", id)),
        }
    }

    fn delete(&self, id: i32) -> Result<(), String> {
        let mut index = None;
        for i in 0..self.categories.len() {
            if self.categories[i].id == id {
                index = Some(i);
                break;
            }
        }

        match index {
            Some(i) => {
                self.categories.remove(i);
                Ok(())
            }
            None => Err(format!("Expense category with ID {} not found", id)),
        }
    }
}

impl ExpenseTransactionStore for InMemoryExpenseStore {
    fn create(
        &self,
        user_id: i32,
        category_id: Option<i32>,
        amount: f64,
        description: Option<String>,
        budget: f64,
    ) -> Result<ExpenseTransaction, String> {
        let id = (self.transactions.len() + 1) as i32;
        let date = Utc::now().date();
        let new_transaction = ExpenseTransaction {
            id,
            user_id,
            category_id,
            amount,
            goal_id,
            date,
            description,
            budget,
        };
        self.transactions.push(new_transaction.clone());
        Ok(new_transaction)
    }

    fn get_all(&self, user_id: i32) -> Result<Vec<ExpenseTransaction>, String> {
        let user_transactions: Vec<ExpenseTransaction> = self
            .transactions
            .iter()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect();
        Ok(user_transactions)
    }

    fn get_by_id(&self, user_id: i32, id: i32) -> Result<ExpenseTransaction, String> {
        for transaction in &self.transactions {
            if transaction.user_id == user_id && transaction.id == id {
                return Ok(transaction.clone());
            }
        }
        Err(format!(
            "Expense transaction with ID {} not found for user {}",
            id, user_id
        ))
    }

    fn update(
        &self,
        id: i32,
        category_id: Option<i32>,
        amount: f64,
        description: Option<String>,
    ) -> Result<ExpenseTransaction, String> {
        let mut updated_transaction = None;
        for i in 0..self.transactions.len() {
            if self.transactions[i].id == id {
                self.transactions[i].category_id = category_id;
                self.transactions[i].amount = amount;
                self.transactions[i].description = description;
                updated_transaction = Some(self.transactions[i].clone());
                break;
            }
        }
        match updated_transaction {
            Some(transaction) => Ok(transaction),
            None => Err(format!("Expense transaction with ID {} not found", id)),
        }
    }

    fn delete(&self, user_id: i32, id: i32) -> Result<(), String> {
        let mut index = None;
        for i in 0..self.transactions.len() {
            if self.transactions[i].user_id == user_id && self.transactions[i].id == id {
                index = Some(i);
                break;
            }
        }
        match index {
            Some(i) => {
                self.transactions.remove(i);
                Ok(())
            }
            None => Err(format!(
                "Expense transaction with ID {} not found for user {}",
                id, user_id
            )),
        }
    }
}
impl ExpenseCategory {
    pub fn new(id: i32, name: &str, budget: Option<f64>) -> Self {
        Self { id, name: name.to_string(),  
         }
    }
}
}