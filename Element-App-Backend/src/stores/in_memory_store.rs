use crate::models::{ExpenseTransaction, FinancialGoal, IncomeSource};
use std::collections::HashMap;
use chrono::{Date, Utc};

pub struct InMemoryStore {
    pub financial_goals: HashMap<i32, FinancialGoal>,
    pub income_sources: HashMap<i32, IncomeSource>,
    pub expense_transactions: HashMap<i32, ExpenseTransaction>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        InMemoryStore {
            financial_goals: HashMap::new(),
            income_sources: HashMap::new(),
            expense_transactions: HashMap::new(),
        }
    }
}