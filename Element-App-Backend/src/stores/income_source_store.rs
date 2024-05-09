use crate::models::{IncomeSource, NewIncomeSource};
use crate::schema::income_sources;
use diesel::prelude::*;
use std::error::Error;

pub struct DieselIncomeSourceStore<'a, C: Connection>(pub &'a C);

impl<'a, C> IncomeSourceStore for DieselIncomeSourceStore<'a, C>
where
    C: Connection<Backend = diesel::pg::Pg>,
{
    fn create(
        &self,
        user_id: i32,
        name: &str,
        amount: f64,
        details: Option<String>,
        goal_id: Option<i32>,
    ) -> Result<IncomeSource, String> {
        let new_source = NewIncomeSource {
            user_id,
            name,
            amount,
            date: Utc::now().date(),
            details: details.as_ref().map(String::as_str),
        };

        diesel::insert_into(income_sources::table)
            .values(&new_source)
            .get_result(self.0)
            .map_err(|err| err.to_string())
    }

    fn get_all(&self, user_id: i32) -> Result<Vec<IncomeSource>, String> {
        income_sources::table
            .filter(income_sources::user_id.eq(user_id))
            .load::<IncomeSource>(self.0)
            .map_err(|err| err.to_string())
    }

    fn get_by_id(&self, user_id: i32, id: i32) -> Result<IncomeSource, String> {
        income_sources::table
            .filter(income_sources::id.eq(id))
            .filter(income_sources::user_id.eq(user_id))
            .first::<IncomeSource>(self.0)
            .map_err(|err| err.to_string())
    }

    fn update(
        &self,
        id: i32,
        name: &str,
        amount: f64,
        details: Option<String>,
    ) -> Result<IncomeSource, String> {
        diesel::update(income_sources::table.filter(income_sources::id.eq(id)))
            .set((
                income_sources::name.eq(name),
                income_sources::amount.eq(amount),
                income_sources::details.eq(details.as_ref().map(String::as_str)),
            ))
            .get_result::<IncomeSource>(self.0)
            .map_err(|err| err.to_string())
    }

    fn delete(&self, user_id: i32, id: i32) -> Result<(), String> {
        diesel::delete(
            income_sources::table
                .filter(income_sources::id.eq(id))
                .filter(income_sources::user_id.eq(user_id)),
        )
        .execute(self.0)
        .map(|_| ())
        .map_err(|err| err.to_string())
    }
}