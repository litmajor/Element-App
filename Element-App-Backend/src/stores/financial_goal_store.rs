use crate::models::FinancialGoal;
use std::collections::HashMap;
use chrono::{Date, Utc};

pub trait FinancialGoalStore {
    fn create(
        &mut self,
        name: &str,
        target_amount: f64,
        target_date: Date<Utc>,
    ) -> Result<FinancialGoal, String>;
    fn get_all(&self) -> Result<Vec<FinancialGoal>, String>;
    fn get_by_id(&self, id: i32) -> Result<FinancialGoal, String>;
    fn update(
        &mut self,
        id: i32,
        name: Option<&str>,
        target_amount: Option<f64>,
        target_date: Option<Date<Utc>>,
    ) -> Result<FinancialGoal, String>;
    fn delete(&mut self, id: i32) -> Result<(), String>;
}

pub struct InMemoryStore {
    pub financial_goals: HashMap<i32, FinancialGoal>,
    // Other store data (e.g., income sources, expense transactions)
}

impl FinancialGoalStore for InMemoryStore {
    fn create(
        &mut self,
        name: &str,
        target_amount: f64,
        target_date: Date<Utc>,
    ) -> Result<FinancialGoal, String> {
        if target_amount <= 0.0 {
            return Err("Target amount must be positive".to_string());
        }

        let id = self
            .financial_goals
            .keys()
            .max()
            .map(|max_id| max_id + 1)
            .unwrap_or(1);

        let new_goal = FinancialGoal {
            id,
            name: name.to_string(),
            target_amount,
            target_date,
            current_amount: 0.0,
        };

        self.financial_goals.insert(id, new_goal.clone());
        Ok(new_goal)
    }

    fn get_all(&self) -> Result<Vec<FinancialGoal>, String> {
        Ok(self.financial_goals.values().cloned().collect())
    }

    fn get_by_id(&self, id: i32) -> Result<FinancialGoal, String> {
        self.financial_goals
            .get(&id)
            .cloned()
            .ok_or_else(|| format!("Financial goal with ID {} not found", id))
    }

    fn update(
        &mut self,
        id: i32,
        name: Option<&str>,
        target_amount: Option<f64>,
        target_date: Option<Date<Utc>>,
    ) -> Result<FinancialGoal, String> {
        if let Some(goal) = self.financial_goals.get_mut(&id) {
            if let Some(name) = name {
                goal.name = name.to_string();
            }
            if let Some(target_amount) = target_amount {
                if target_amount <= 0.0 {
                    return Err("Target amount must be positive".to_string());
                }
                goal.target_amount = target_amount;
            }
            if let Some(target_date) = target_date {
                goal.target_date = target_date;
            }
            Ok(goal.clone())
        } else {
            Err(format!("Financial goal with ID {} not found", id))
        }
    }

    fn delete(&mut self, id: i32) -> Result<(), String> {
        if self.financial_goals.remove(&id).is_some() {
            Ok(())
        } else {
            Err(format!("Financial goal with ID {} not found", id))
        }
    }
}