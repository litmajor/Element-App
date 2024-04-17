
use crate::models::expense::{Expense, ExpenseCategory};
use crate::models::income::{Income, IncomeSource};
use crate::models::user::User;
use crate::repositories::expense_repository::ExpenseRepository;
use crate::repositories::income_repository::IncomeRepository;
use crate::repositories::user_repository::UserRepository;
use chrono::{Date, Utc};
use std::collections::HashMap;
use crate::models::user::User;
use crate::models::trend_analysis::TrendAnalysis;

pub struct TrendAnalysisService {
    expense_repository: ExpenseRepository,
    income_repository: IncomeRepository,
    user_repository: UserRepository,
}

impl TrendAnalysisService {
    pub fn new(
        expense_repository: ExpenseRepository,
        income_repository: IncomeRepository,
        user_repository: UserRepository,
    ) -> Self {
        Self {
            expense_repository,
            income_repository,
            user_repository,
        }
    }

    pub fn get_income_sources_in_range(
        &self,
        start_date: Date<Utc>,
        end_date: Date<Utc>,
    ) -> Result<Vec<IncomeSource>, String> {
        let income_sources = self.income_repository.get_all_in_range(start_date, end_date)?;
        Ok(income_sources)
    }
}

    pub fn get_expenses_in_range(
        &self,
        start_date: Date<Utc>,
        end_date: Date<Utc>,
    ) -> Result<Vec<Expense>, String> {
        let expenses = self.expense_repository.get_all_in_range(start_date, end_date)?;
        Ok(expenses)
    }

    pub fn get_user_by_id(&self, user_id: i32) -> Result<User, String> {
        let user = self.user_repository.get_by_id(user_id)?;
        Ok(user)
    }

    pub fn get_trend_analysis(
        &self,
        user_id: i32,
        start_date: Date<Utc>,
        end_date: Date<Utc>,
    ) -> Result<TrendAnalysis, String, String> {
        let (income_totals, expense_totals) = self.aggregate_income_and_expenses(start_date, end_date)?;
        let mut trend_analysis = TrendAnalysis::new(user)?;
        for (user_id, income_total) in income_totals.iter() {
            trend_analysis.add_income(*income_total);
        }
        for (user_id, expense_total) in expense_totals.iter() {
            trend_analysis.add_expense(*expense_total);
        }
        trend_analysis.calculate_averages();
        for (i, income_average) in trend_analysis.income_averages.iter().enumerate() {
            if i > 0 {
                trend_analysis.income_percentage_changes.push(calculate_percentage_change(*income_average, trend_analysis.income_averages[i - 1]));
            }
        }
        for (i, expense_average) in trend_analysis.expense_averages.iter().enumerate() {
            if i > 0 {
                trend_analysis.expense_percentage_changes.push(calculate_percentage_change(*expense_average, trend_analysis.expense_averages[i - 1]));
            }
        }
        Ok(trend_analysis)
    }
    pub struct TrendAnalysis {
        user: User,
        income_totals: Vec<f64>,
        expense_totals: Vec<f64>,
        income_averages: Vec<f64>,
        expense_averages: Vec<f64>,
        income_percentage_changes: Vec<Option<f64>>,
        expense_percentage_changes: Vec<Option<f64>>,
    }
    

fn calculate_percentage_change(current: f64, previous: f64) -> Option<f64> {
    if previous == 0.0 {
        return None; // Avoid division by zero
    }
    let change = (current - previous) / previous;
    Some(change * 100.0) // Express as percentage
}

fn calculate_percentage_change(current: f64, previous: f64) -> Option<f64> {
    if previous == 0.0 {
        return None; // Avoid division by zero
    }
    let change = (current - previous) / previous;
    Some(change * 100.0) // Express as percentage
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
    pub fn new(user: User) -> Self { Ok();{
        let user = User { id: 1, name: "John Doe" };
        let trend_analysis = TrendAnalysis::new(user);
        Ok(trend_analysis)
    }

    pub fn get_user(&self) -> &User {
        &self.user
    }
let (income_totals, expense_totals) = self.aggregate_income_and_expenses(start_date, end_date)?;
        

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


fn aggregate_income_and_expenses(
    &self,
    start_date: Date<Utc>,
    end_date: Date<Utc>,
) -> Result<(HashMap<i32, f64>, HashMap<i32, f64>), String> {
    let mut income_totals = HashMap::new();
    let mut expense_totals = HashMap::new();

    for income_source in self.get_income_sources_in_range(start_date, end_date)? {
        *income_totals.entry(income_source.user_id).or_insert(0.0) += income_source.amount;
    }

    for expense in self.get_expenses_in_range(start_date, end_date)? {
        *expense_totals.entry(expense.user_id).or_insert(0.0) += expense.amount;
    }

    Ok((income_totals, expense_totals))
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
}
    Ok(trend_analysis);
}