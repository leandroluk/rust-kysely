//! Driver implementation for PostgreSQL using tokio-postgres.

use async_trait::async_trait;
use compact_str::CompactString;
use tokio_postgres::types::{IsNull, ToSql, Type};

use crate::driver::{DatabaseConnection, Driver, DriverError};
use crate::operation_node::Value;
use crate::query_compiler::CompiledQuery;
use crate::query_executor::{QueryResult, Row};

#[cfg(feature = "postgres")]
impl ToSql for Value {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut tokio_postgres::types::private::BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            Value::Null => Ok(IsNull::Yes),
            Value::Bool(b) => b.to_sql(ty, out),
            Value::I64(i) => match *ty {
                Type::INT2 => (*i as i16).to_sql(ty, out),
                Type::INT4 => (*i as i32).to_sql(ty, out),
                _ => i.to_sql(ty, out),
            },
            Value::F64(f) => match *ty {
                Type::FLOAT4 => (*f as f32).to_sql(ty, out),
                _ => f.to_sql(ty, out),
            },
            Value::Text(s) => s.as_str().to_sql(ty, out),
            Value::Bytes(b) => b.as_slice().to_sql(ty, out),
        }
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut tokio_postgres::types::private::BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}

/// Active connection to a PostgreSQL database wrapping `tokio_postgres::Client`.
// @spec REQ-EXEC-002
pub struct PostgresConnection {
    client: tokio_postgres::Client,
}

impl PostgresConnection {
    /// Creates a new `PostgresConnection` wrapping an active client.
    pub fn new(client: tokio_postgres::Client) -> Self {
        Self { client }
    }

    /// Returns a reference to the inner `tokio_postgres::Client`.
    pub fn client(&self) -> &tokio_postgres::Client {
        &self.client
    }
}

#[async_trait]
impl DatabaseConnection for PostgresConnection {
    async fn execute_query(&mut self, query: &CompiledQuery) -> Result<QueryResult, DriverError> {
        let params: Vec<&(dyn ToSql + Sync)> = query
            .parameters
            .iter()
            .map(|p| p as &(dyn ToSql + Sync))
            .collect();

        let rows = self
            .client
            .query(&query.sql, &params)
            .await
            .map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;

        if rows.is_empty() {
            return Ok(QueryResult::default());
        }

        let columns: Vec<CompactString> = rows[0]
            .columns()
            .iter()
            .map(|col| CompactString::new(col.name()))
            .collect();

        let mut result_rows = Vec::with_capacity(rows.len());
        for pg_row in rows {
            let mut values = Vec::with_capacity(columns.len());
            for idx in 0..columns.len() {
                let val = extract_value_from_row(&pg_row, idx)?;
                values.push(val);
            }
            result_rows.push(Row::new(columns.clone(), values));
        }

        Ok(QueryResult::with_rows(result_rows))
    }
}

fn extract_value_from_row(row: &tokio_postgres::Row, idx: usize) -> Result<Value, DriverError> {
    let col = &row.columns()[idx];
    let ty = col.type_();

    if ty == &Type::BOOL {
        let val: Option<bool> = row.try_get(idx).map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
        Ok(val.map(Value::Bool).unwrap_or(Value::Null))
    } else if ty == &Type::INT2 {
        let val: Option<i16> = row.try_get(idx).map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
        Ok(val.map(|i| Value::I64(i as i64)).unwrap_or(Value::Null))
    } else if ty == &Type::INT4 {
        let val: Option<i32> = row.try_get(idx).map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
        Ok(val.map(|i| Value::I64(i as i64)).unwrap_or(Value::Null))
    } else if ty == &Type::INT8 {
        let val: Option<i64> = row.try_get(idx).map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
        Ok(val.map(Value::I64).unwrap_or(Value::Null))
    } else if ty == &Type::FLOAT4 {
        let val: Option<f32> = row.try_get(idx).map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
        Ok(val.map(|f| Value::F64(f as f64)).unwrap_or(Value::Null))
    } else if ty == &Type::FLOAT8 {
        let val: Option<f64> = row.try_get(idx).map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
        Ok(val.map(Value::F64).unwrap_or(Value::Null))
    } else if ty == &Type::VARCHAR || ty == &Type::TEXT || ty == &Type::BPCHAR || ty == &Type::NAME {
        let val: Option<String> = row.try_get(idx).map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
        Ok(val.map(|s| Value::Text(CompactString::new(s))).unwrap_or(Value::Null))
    } else if ty == &Type::BYTEA {
        let val: Option<Vec<u8>> = row.try_get(idx).map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
        Ok(val.map(Value::Bytes).unwrap_or(Value::Null))
    } else {
        // Fallback: try string or null
        let val: Result<Option<String>, _> = row.try_get(idx);
        match val {
            Ok(Some(s)) => Ok(Value::Text(CompactString::new(s))),
            Ok(None) => Ok(Value::Null),
            Err(_) => Ok(Value::Null),
        }
    }
}

/// Driver implementation connecting to PostgreSQL instances using `tokio-postgres`.
// @spec REQ-EXEC-002
pub struct PostgresDriver {
    config: tokio_postgres::Config,
}

impl PostgresDriver {
    /// Creates a new `PostgresDriver` from a PostgreSQL connection string.
    pub fn new(connection_string: &str) -> Result<Self, DriverError> {
        let config = connection_string
            .parse::<tokio_postgres::Config>()
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;
        Ok(Self { config })
    }

    /// Creates a new `PostgresDriver` from an explicit `tokio_postgres::Config`.
    pub fn from_config(config: tokio_postgres::Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Driver for PostgresDriver {
    async fn acquire_connection(&self) -> Result<Box<dyn DatabaseConnection>, DriverError> {
        let (client, connection) = self
            .config
            .connect(tokio_postgres::NoTls)
            .await
            .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;

        tokio::spawn(async move {
            let _ = connection.await;
        });

        Ok(Box::new(PostgresConnection::new(client)))
    }

    async fn release_connection(&self, _connection: Box<dyn DatabaseConnection>) -> Result<(), DriverError> {
        Ok(())
    }

    async fn destroy(&self) -> Result<(), DriverError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_driver_parameter_conversion() {
        let mut buf = tokio_postgres::types::private::BytesMut::new();

        // 1. Integer conversion
        let val_i64 = Value::I64(42);
        let res = val_i64.to_sql_checked(&Type::INT8, &mut buf);
        assert!(matches!(res, Ok(IsNull::No)));

        let res = val_i64.to_sql_checked(&Type::INT4, &mut buf);
        assert!(matches!(res, Ok(IsNull::No)));

        // 2. Text conversion
        let val_text = Value::Text("Hello Postgres".into());
        let res = val_text.to_sql_checked(&Type::TEXT, &mut buf);
        assert!(matches!(res, Ok(IsNull::No)));

        // 3. Boolean conversion
        let val_bool = Value::Bool(true);
        let res = val_bool.to_sql_checked(&Type::BOOL, &mut buf);
        assert!(matches!(res, Ok(IsNull::No)));

        // 4. Float conversion
        let val_f64 = Value::F64(3.14159);
        let res = val_f64.to_sql_checked(&Type::FLOAT8, &mut buf);
        assert!(matches!(res, Ok(IsNull::No)));

        // 5. Bytes conversion
        let val_bytes = Value::Bytes(vec![0xCA, 0xFE, 0xBA, 0xBE]);
        let res = val_bytes.to_sql_checked(&Type::BYTEA, &mut buf);
        assert!(matches!(res, Ok(IsNull::No)));

        // 6. Null conversion
        let val_null = Value::Null;
        let res = val_null.to_sql_checked(&Type::TEXT, &mut buf);
        assert!(matches!(res, Ok(IsNull::Yes)));
    }
}
