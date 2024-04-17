use crate::models::{Project, Transaction, TransactionType};
use crate::schema::transactions;
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text};
use uuid::Uuid;
use diesel::result::{Error as DieselError, ErrorCode};
use log::{error, info};
use chrono::{NaiveDate, Duration};

#[derive(Queryable, Insertable, AsChangeset)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub id: Uuid,
    pub project_id: i32,
    pub sender_id: i32,
    pub receiver_id: i32,
    pub amount: f64,
    pub status: String,
    pub fee: f64,
    pub created_at: chrono::NaiveDateTime,
    pub custom_data: Option<serde_json::Value>,
    pub transaction_type: TransactionType,
    pub description: Option<String>,
}

impl Transaction {
    pub fn create(conn: &PgConnection, project_id: i32, sender_id: i32, receiver_id: i32, amount: f64, description: Option<String>, transaction_type: TransactionType) -> Result<Transaction, Box<dyn std::error::Error>> {
        let new_transaction = Transaction {
            id: Uuid::new_v4(),
            project_id,
            sender_id,
            receiver_id,
            amount,
            status: "pending".to_string(),
            fee: 0.0, // Initialize fee to 0.0
            created_at: chrono::Utc::now(),
            custom_data: None, // Initialize custom_data to None
            transaction_type,
            description,
        };

        diesel::insert_into(transactions::table)
            .values(&new_transaction)
            .execute(conn)?;

        Ok(new_transaction)
    }

    pub fn get_by_id(conn: &PgConnection, id: Uuid) -> Result<Option<Transaction>, Box<dyn std::error::Error>> {
        Ok(transactions::table.filter(transactions::id.eq(id)).first(conn).optional()?)
    }

    pub fn get_by_project_id(conn: &PgConnection, project_id: i32) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        Ok(transactions::table.filter(transactions::project_id.eq(project_id)).load::<Transaction>(conn)?)
    }

    pub fn get_by_sender_id(conn: &PgConnection, sender_id: i32) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        Ok(transactions::table.filter(transactions::sender_id.eq(sender_id)).load::<Transaction>(conn)?)
    }

    pub fn get_by_receiver_id(conn: &PgConnection, receiver_id: i32) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        Ok(transactions::table.filter(transactions::receiver_id.eq(receiver_id)).load::<Transaction>(conn)?)
    }

    pub fn get_all(conn: &PgConnection) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        Ok(transactions::table.load::<Transaction>(conn)?)
    }

    pub fn delete(conn: &PgConnection, id: Uuid) -> Result<usize, Box<dyn std::error::Error>> {
        diesel::delete(transactions::table.filter(transactions::id.eq(id))).execute(conn)
    }

    pub fn create_transaction_in_tx<T>(conn: &T, project_id: i32, sender_id: i32, receiver_id: i32, amount: f64, description: Option<String>, transaction_type: TransactionType) -> Result<Transaction, Box<dyn std::error::Error>>
    where
        T: TransactionManager,
    {
        conn.transaction(|| {
            let transaction = Transaction::create(conn, project_id, sender_id, receiver_id, amount, description, transaction_type)?;
            Ok(transaction)
        })
    }

    pub fn update(self, conn: &PgConnection, amount: f64, description: Option<String>) -> Result<Transaction, Box<dyn std::error::Error>> {
        diesel::update(transactions::table.filter(transactions::id.eq(self.id)))
            .set((
                transactions::amount.eq(amount),
                transactions::description.eq(description),
            ))
            .execute(conn)?;

        Ok(self)
    }
}

#[derive(Debug)]
pub enum TransactionError {
    InvalidInput(String),
    InsufficientFunds,
    PaymentProcessingError(String),
    ForeignKeyViolation(String),
    DatabaseError(diesel::result::Error),
    UnknownError(String),
}

impl From<diesel::result::Error> for TransactionError {
    fn from(err: DieselError) -> Self {
        match err {
            DieselError(DieselError::Kind::InvalidValue(field, value)) => {
                TransactionError::InvalidInput(format!("Invalid value for field '{}': {:?}", field, value))
            }
            DieselError(DieselError::Kind::ForeignKeyViolation(_)) => {
                TransactionError::ForeignKeyViolation(String::from("Foreign key constraint violation"))
            }
            err => TransactionError::DatabaseError(err),
        }
    }
}

pub fn process_and_record_transaction(conn: &PgConnection, transaction: &Transaction) -> Result<Transaction, TransactionError> {
    // 1. Validate transaction data
    if transaction.amount <= 0.0 {
        return Err(TransactionError::InvalidInput(String::from("Transaction amount must be positive")));
    }

    // 2. Process transaction based on type
    match transaction.transaction_type {
        TransactionType::Deposit => {
            // Code to handle deposits (e.g., update project budget)
            // ... (implementation based on your project management logic)
        },
        TransactionType::Fee => {
            // Code to handle platform fees (deduct from deposit)
            // ... (implementation for calculating and deducting fees)
        },
        TransactionType::Payout => {
            // Code to release payment to freelancer (potentially using an external service)
            match release_payment(transaction.amount) {
                Ok(_) => info!("Payment released successfully for transaction {}", transaction.id),
                Err(err) => {
                    error!("Payment processing error: {}", err);
                    return Err(TransactionError::PaymentProcessingError(err.to_string()));
                }
            }
        },
    }

    // 3. Record transaction in the database with transaction isolation
    conn.transaction(|| {
        let new_transaction = Transaction::create_transaction_in_tx(conn, transaction.project_id, transaction.sender_id, transaction.receiver_id, transaction.amount, transaction.description.clone(), transaction.transaction_type)?;
        Ok(new_transaction)
    })
    .map_err(|err| match err {
        TransactionError::DatabaseError(err) => err, // Already a specific error type
        err => TransactionError::UnknownError(format!("Unexpected error: {:?}", err)),
    })
}

// Placeholder function for payment processing (replace with your actual implementation)
fn release_payment(amount: f64) -> Result<(), String> {
    // Code to interact with your payment processing service
    // ... (implementation for secure payment release)
    Err(String::from("Simulated payment processing error"))
}

pub fn filter(conn: &PgConnection, filters: &[(&str, &dyn ToSql + Sync)]) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
    let mut query = transactions::table.into_boxed();

    for (field, value) in filters {
        match field {
            &"transaction_type" => query = query.filter(transactions::transaction_type.eq(value)),
            &"created_at" => {
                // Handle date range filtering (assuming value is a string in YYYY-MM-DD - YYYY-MM-DD format)
                let parts: Vec<&str> = value.split('-').collect();
                if parts.len() == 2 {
                    let start_date = NaiveDate::parse_from_str(parts[0].trim(), "%Y-%m-%d")?;
                    let end_date = NaiveDate::parse_from_str(parts[1].trim(), "%Y-%m-%d")? + Duration::days(1);
                    query = query.filter(transactions::created_at.between(start_date.and(end_date)));
                }
            },
            &"sender_id" => query = query.filter(transactions::sender_id.eq(value)),
            &"receiver_id" => query = query.filter(transactions::receiver_id.eq(value)),
            &"status" => query = query.filter(transactions::status.eq(value)),
        }
    }

    Ok(query.load::<Transaction>(conn)?)
}

pub fn paginate(conn: &PgConnection, page: i64, per_page: i64) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
    let offset = (page - 1) * per_page;
    Ok(transactions::table.offset(offset).limit(per_page).load::<Transaction>(conn)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Project, TransactionType};
    use diesel::r2d2::{ConnectionManager, PooledConnection};
    use diesel::PgConnection;
    use mockall::mock;
    use std::sync::Arc;

    mock! {
        PgConnection {}
        impl Connection for PgConnection {
            fn execute(&mut self, query: &str) -> QueryResult<usize>;
            fn query_by_index<T, U>(&mut self, source: T) -> QueryResult<Vec<U>>
            where
                T: LoadQuery<Self, U>;
        }
    }

    type MockConnection = Arc<MockPgConnection>;

    #[test]
    fn test_create_transaction() {
        let mut mock_conn = MockConnection::new();
        mock_conn
            .expect_execute()
            .returning(|_| Ok(1));

        let result = Transaction::create(
            &mock_conn,
            1,
            1,
            2,
            100.0,
            Some("Test transaction".to_string()),
            TransactionType::Deposit,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_get_transaction_by_id() {
        let mut mock_conn = MockConnection::new();
        mock_conn
            .expect_first()
            .returning(|_| Ok(Some(Transaction {
                id: Uuid::new_v4(),
                project_id: 1,
                sender_id: 1,
                receiver_id: 2,
                amount: 100.0,
                status: "pending".to_string(),
                fee: 0.0,
                created_at: chrono::Utc::now(),
                custom_data: None,
                transaction_type: TransactionType::Deposit,
                description: Some("Test transaction".to_string()),
            })));

        let result = Transaction::get_by_id(&mock_conn, Uuid::new_v4());
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_update_transaction() {
        let mut mock_conn = MockConnection::new();
        mock_conn
            .expect_execute()
            .returning(|_| Ok(1));

        let transaction = Transaction {
            id: Uuid::new_v4(),
            project_id: 1,
            sender_id: 1,
            receiver_id: 2,
            amount: 100.0,
            status: "pending".to_string(),
            fee: 0.0,
            created_at: chrono::Utc::now(),
            custom_data: None,
            transaction_type: TransactionType::Deposit,
            description: Some("Test transaction".to_string()),
        };

        let result = transaction.update(&mock_conn, 150.0, Some("Updated transaction".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_and_record_transaction() {
        let mut mock_conn = MockConnection::new();
        mock_conn
            .expect_transaction()
            .returning(|f| f());
        mock_conn
            .expect_execute()
            .returning(|_| Ok(1));

        let transaction = Transaction {
            id: Uuid::new_v4(),
            project_id: 1,
            sender_id: 1,
            receiver_id: 2,
            amount: 100.0,
            status: "pending".to_string(),
            fee: 0.0,
            created_at: chrono::Utc::now(),
            custom_data: None,
            transaction_type: TransactionType::Deposit,
            description: Some("Test transaction".to_string()),
        };

        let result = process_and_record_transaction(&mock_conn, &transaction);
        assert!(result.is_ok());
    }
}