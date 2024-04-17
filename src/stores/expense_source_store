use crate::models::{ExpenseCategory, ExpenseTransaction};
use crate::schema::{expense_categories, expense_transactions};
use diesel::prelude::*;
use std::error::Error;

pub struct DieselExpenseStore<'a, C: Connection>(pub &'a C);

impl<'a, C> ExpenseCategoryStore for DieselExpenseStore<'a, C>
where
    C: Connection<Backend = diesel::pg::Pg>,
{
    fn create(&self, name: &str) -> Result<ExpenseCategory, String> {
        let new_category = ExpenseCategory { id: 0, name: name.to_string() };
        diesel::insert_into(expense_categories::table)
            .values(&new_category)
            .get_result(self.0)
            .map_err(|err| err.to_string())
    }

    fn get_all(&self) -> Result<Vec<ExpenseCategory>, String> {
        expense_categories::table
            .load::<ExpenseCategory>(self.0)
            .map_err(|err| err.to_string())
    }

    fn get_by_id(&self, id: i32) -> Result<ExpenseCategory, String> {
        expense_categories::table
            .filter(expense_categories::id.eq(id))
            .first::<ExpenseCategory>(self.0)
            .map_err(|err| err.to_string())
    }

    fn update(&self, id: i32, name: &str) -> Result<ExpenseCategory, String> {
        diesel::update(expense_categories::table.filter(expense_categories::id.eq(id)))
            .set(expense_categories::name.eq(name))
            .get_result::<ExpenseCategory>(self.0)
            .map_err(|err| err.to_string())
    }

    fn delete(&self, id: i32) -> Result<(), String> {
        diesel::delete(expense_categories::table.filter(expense_categories::id.eq(id)))
            .execute(self.0)
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
}

impl<'a, C> ExpenseTransactionStore for DieselExpenseStore<'a, C>
where
    C: Connection<Backend = diesel::pg::Pg>,
{
    fn create(
        &self,
        user_id: i32,
        category_id: Option<i32>,
        amount: f64,
        description: Option<String>,
    ) -> Result<ExpenseTransaction, String> {
        let new_transaction = ExpenseTransaction {
            id: 0,
            user_id,
            category_id,
            amount,
            date: Utc::now().date(),
            description,
        };
        diesel::insert_into(expense_transactions::table)
            .values(&new_transaction)
            .get_result(self.0)
            .map_err(|err| err.to_string())
    }

    fn get_all(&self, user_id: i32) -> Result<Vec<ExpenseTransaction>, String> {
        expense_transactions::table
            .filter(expense_transactions::user_id.eq(user_id))
            .load::<ExpenseTransaction>(self.0)
            .map_err(|err| err.to_string())
    }

    fn get_by_id(&self, user_id: i32, id: i32) -> Result<ExpenseTransaction, String> {
        expense_transactions::table
            .filter(expense_transactions::id.eq(id))
            .filter(expense_transactions::user_id.eq(user_id))
            .first::<ExpenseTransaction>(self.0)
            .map_err(|err| err.to_string())
    }

    fn update(
        &self,
        id: i32,
        category_id: Option<i32>,
        amount: f64,
        description: Option<String>,
    ) -> Result<ExpenseTransaction, String> {
        diesel::update(expense_transactions::table.filter(expense_transactions::id.eq(id)))
            .set((
                expense_transactions::category_id.eq(category_id),
                expense_transactions::amount.eq(amount),
                expense_transactions::description.eq(description),
            ))
            .get_result::<ExpenseTransaction>(self.0)
            .map_err(|err| err.to_string())
    }

    fn delete(&self, user_id: i32, id: i32) -> Result<(), String> {
        diesel::delete(
            expense_transactions::table
                .filter(expense_transactions::id.eq(id))
                .filter(expense_transactions::user_id.eq(user_id)),
        )
        .execute(self.0)
        .map(|_| ())
        .map_err(|err| err.to_string())
    }
}