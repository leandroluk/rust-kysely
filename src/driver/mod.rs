//! Database driver abstractions, connection pooling, and errors.

pub mod error;
pub mod traits;

pub use error::DriverError;
pub use traits::{DatabaseConnection, Driver};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation_node::Value;
    use crate::query_compiler::CompiledQuery;
    use crate::query_executor::{QueryResult, Row};
    use async_trait::async_trait;

    struct MockConnection {
        executed: Vec<String>,
    }

    #[async_trait]
    impl DatabaseConnection for MockConnection {
        async fn execute_query(&mut self, query: &CompiledQuery) -> Result<QueryResult, DriverError> {
            self.executed.push(query.sql.clone());
            let row = Row::new(["id", "name"], [Value::from(1), Value::from("Test")]);
            Ok(QueryResult::with_rows(vec![row]))
        }
    }

    struct MockDriver;

    #[async_trait]
    impl Driver for MockDriver {
        async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
            Ok(Box::new(MockConnection {
                executed: Vec::new(),
            }))
        }

        async fn release_connection(
            &self,
            _connection: Box<dyn DatabaseConnection>,
        ) -> Result<(), DriverError> {
            Ok(())
        }

        async fn destroy(&self) -> Result<(), DriverError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_driver_traits() {
        let driver = MockDriver;
        let mut conn = driver.acquire_connection().await.unwrap();

        let compiled = CompiledQuery {
            sql: "SELECT * FROM users".to_string(),
            parameters: Vec::new(),
            query_id: crate::util::QueryId::generate(),
        };

        let result = conn.execute_query(&compiled).await.unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].get_value("id").unwrap(), &Value::from(1));

        driver.release_connection(conn).await.unwrap();
        driver.destroy().await.unwrap();
    }
}
