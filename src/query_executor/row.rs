//! Database row representation and typed value extraction.

use compact_str::CompactString;
use thiserror::Error;

use crate::operation_node::Value;

/// Error occurring during column lookup or type casting on a row.
// @spec REQ-EXEC-005
#[derive(Debug, Error, PartialEq)]
pub enum ColumnError {
    /// Column not found in row
    #[error("Column '{0}' not found in query result row")]
    NotFound(CompactString),

    /// Column index out of bounds
    #[error("Column index {0} out of bounds (row has {1} columns)")]
    IndexOutOfBounds(usize, usize),

    /// Value type mismatch
    #[error("Column '{0}' has value '{1}' which cannot be converted to target type")]
    TypeMismatch(CompactString, String),

    /// Deserialization error
    #[error("Failed to deserialize row: {0}")]
    Deserialization(String),
}

/// Trait for converting an AST `Value` reference into a target Rust type.
// @spec REQ-EXEC-005
pub trait FromRowValue: Sized {
    /// Converts a row value into the target type, with column name for error reporting.
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError>;
}

impl FromRowValue for Value {
    fn from_row_value(val: &Value, _col: &str) -> Result<Self, ColumnError> {
        Ok(val.clone())
    }
}

impl FromRowValue for bool {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::Bool(b) => Ok(*b),
            Value::I64(i) => match *i {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(ColumnError::TypeMismatch(
                    CompactString::new(col),
                    val.to_string(),
                )),
            },
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for i64 {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::I64(i) => Ok(*i),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for i32 {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::I64(i) => (*i).try_into().map_err(|_| {
                ColumnError::TypeMismatch(CompactString::new(col), val.to_string())
            }),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for i16 {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::I64(i) => (*i).try_into().map_err(|_| {
                ColumnError::TypeMismatch(CompactString::new(col), val.to_string())
            }),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for i8 {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::I64(i) => (*i).try_into().map_err(|_| {
                ColumnError::TypeMismatch(CompactString::new(col), val.to_string())
            }),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for isize {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::I64(i) => (*i).try_into().map_err(|_| {
                ColumnError::TypeMismatch(CompactString::new(col), val.to_string())
            }),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for u64 {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::I64(i) => (*i).try_into().map_err(|_| {
                ColumnError::TypeMismatch(CompactString::new(col), val.to_string())
            }),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for u32 {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::I64(i) => (*i).try_into().map_err(|_| {
                ColumnError::TypeMismatch(CompactString::new(col), val.to_string())
            }),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for u16 {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::I64(i) => (*i).try_into().map_err(|_| {
                ColumnError::TypeMismatch(CompactString::new(col), val.to_string())
            }),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for u8 {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::I64(i) => (*i).try_into().map_err(|_| {
                ColumnError::TypeMismatch(CompactString::new(col), val.to_string())
            }),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for usize {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::I64(i) => (*i).try_into().map_err(|_| {
                ColumnError::TypeMismatch(CompactString::new(col), val.to_string())
            }),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for f64 {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::F64(f) => Ok(*f),
            Value::I64(i) => Ok(*i as f64),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for f32 {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::F64(f) => Ok(*f as f32),
            Value::I64(i) => Ok(*i as f32),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for String {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::Text(t) => Ok(t.to_string()),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for CompactString {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::Text(t) => Ok(t.clone()),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl FromRowValue for Vec<u8> {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        match val {
            Value::Bytes(b) => Ok(b.clone()),
            _ => Err(ColumnError::TypeMismatch(
                CompactString::new(col),
                val.to_string(),
            )),
        }
    }
}

impl<T: FromRowValue> FromRowValue for Option<T> {
    fn from_row_value(val: &Value, col: &str) -> Result<Self, ColumnError> {
        if val.is_null() {
            Ok(None)
        } else {
            T::from_row_value(val, col).map(Some)
        }
    }
}

/// Map-like container representing a single database result row.
// @spec REQ-EXEC-005
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Row {
    pub(crate) columns: Vec<CompactString>,
    pub(crate) values: Vec<Value>,
}

impl Row {
    /// Creates a new row from columns and values.
    pub fn new(
        columns: impl IntoIterator<Item = impl Into<CompactString>>,
        values: impl IntoIterator<Item = impl Into<Value>>,
    ) -> Self {
        Self {
            columns: columns.into_iter().map(Into::into).collect(),
            values: values.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns the number of columns in this row.
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Returns true if this row has no columns.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Returns the column names in order.
    pub fn columns(&self) -> &[CompactString] {
        &self.columns
    }

    /// Returns the raw scalar values in order.
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    /// Finds the index of a column by name (exact match first, then case-insensitive).
    pub fn index_of(&self, col: &str) -> Option<usize> {
        self.columns
            .iter()
            .position(|c| c.as_str() == col)
            .or_else(|| self.columns.iter().position(|c| c.eq_ignore_ascii_case(col)))
    }

    /// Returns the raw `Value` at the given column index.
    pub fn get_value_by_index(&self, idx: usize) -> Result<&Value, ColumnError> {
        self.values
            .get(idx)
            .ok_or(ColumnError::IndexOutOfBounds(idx, self.values.len()))
    }

    /// Returns the raw `Value` for a given column name.
    pub fn get_value(&self, col: &str) -> Result<&Value, ColumnError> {
        let idx = self.index_of(col).ok_or_else(|| ColumnError::NotFound(CompactString::new(col)))?;
        Ok(&self.values[idx])
    }

    /// Extracts a strongly-typed value from a column by name.
    // @spec REQ-EXEC-005
    pub fn get<T: FromRowValue>(&self, col: &str) -> Result<T, ColumnError> {
        let val = self.get_value(col)?;
        T::from_row_value(val, col)
    }

    /// Extracts an optional strongly-typed value from a column by name.
    /// Returns `Ok(None)` if the value is NULL.
    /// Returns `Err(ColumnError::NotFound)` if the column does not exist.
    // @spec REQ-EXEC-005
    pub fn get_opt<T: FromRowValue>(&self, col: &str) -> Result<Option<T>, ColumnError> {
        let val = self.get_value(col)?;
        if val.is_null() {
            Ok(None)
        } else {
            T::from_row_value(val, col).map(Some)
        }
    }

    /// Extracts a strongly-typed value from a column by index.
    // @spec REQ-EXEC-005
    pub fn get_by_index<T: FromRowValue>(&self, idx: usize) -> Result<T, ColumnError> {
        let val = self.get_value_by_index(idx)?;
        let col = &self.columns[idx];
        T::from_row_value(val, col.as_str())
    }

    /// Extracts an optional strongly-typed value from a column by index.
    // @spec REQ-EXEC-005
    pub fn get_by_index_opt<T: FromRowValue>(&self, idx: usize) -> Result<Option<T>, ColumnError> {
        let val = self.get_value_by_index(idx)?;
        let col = &self.columns[idx];
        if val.is_null() {
            Ok(None)
        } else {
            T::from_row_value(val, col.as_str()).map(Some)
        }
    }

    /// Deserializes the row into a typed struct implementing `serde::de::DeserializeOwned`.
    // @spec REQ-EXEC-006
    #[cfg(feature = "json")]
    pub fn into_record<T: serde::de::DeserializeOwned>(&self) -> Result<T, ColumnError> {
        let mut map = serde_json::Map::with_capacity(self.columns.len());
        for (col, val) in self.columns.iter().zip(self.values.iter()) {
            let json_val = match val {
                Value::Null => serde_json::Value::Null,
                Value::Bool(b) => serde_json::Value::Bool(*b),
                Value::I64(i) => serde_json::Value::Number((*i).into()),
                Value::F64(f) => serde_json::Number::from_f64(*f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null),
                Value::Text(t) => serde_json::Value::String(t.to_string()),
                Value::Bytes(b) => serde_json::Value::Array(
                    b.iter().map(|&byte| serde_json::Value::Number(byte.into())).collect(),
                ),
            };
            map.insert(col.to_string(), json_val);
        }
        serde_json::from_value(serde_json::Value::Object(map))
            .map_err(|e| ColumnError::Deserialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_extraction() {
        let row = Row::new(
            vec!["id", "name", "is_active", "score", "extra_bytes", "nullable_val"],
            vec![
                Value::I64(42),
                Value::Text("Alice".into()),
                Value::Bool(true),
                Value::F64(98.5),
                Value::Bytes(vec![0xDE, 0xAD]),
                Value::Null,
            ],
        );

        // Typed gets
        assert_eq!(row.get::<i64>("id").unwrap(), 42);
        assert_eq!(row.get::<i32>("id").unwrap(), 42);
        assert_eq!(row.get::<String>("name").unwrap(), "Alice");
        assert_eq!(row.get::<CompactString>("name").unwrap(), "Alice");
        assert_eq!(row.get::<bool>("is_active").unwrap(), true);
        assert_eq!(row.get::<f64>("score").unwrap(), 98.5);
        assert_eq!(row.get::<Vec<u8>>("extra_bytes").unwrap(), vec![0xDE, 0xAD]);

        // Case-insensitive lookup
        assert_eq!(row.get::<i64>("ID").unwrap(), 42);

        // Nullable get_opt
        assert_eq!(row.get_opt::<i64>("id").unwrap(), Some(42));
        assert_eq!(row.get_opt::<i64>("nullable_val").unwrap(), None);
        assert_eq!(row.get::<Option<i64>>("nullable_val").unwrap(), None);

        // Index-based gets
        assert_eq!(row.get_by_index::<i64>(0).unwrap(), 42);
        assert_eq!(row.get_by_index::<String>(1).unwrap(), "Alice");
        assert_eq!(row.get_by_index_opt::<i64>(5).unwrap(), None);

        // Errors
        assert!(matches!(
            row.get::<i64>("nonexistent"),
            Err(ColumnError::NotFound(_))
        ));
        assert!(matches!(
            row.get::<i64>("name"),
            Err(ColumnError::TypeMismatch(_, _))
        ));
        assert!(matches!(
            row.get_by_index::<i64>(99),
            Err(ColumnError::IndexOutOfBounds(99, 6))
        ));

        // Serde deserialization
        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct UserRow {
            id: i64,
            name: String,
            is_active: bool,
            score: f64,
            nullable_val: Option<String>,
        }

        let user: UserRow = row.into_record().expect("Failed to deserialize user");
        assert_eq!(
            user,
            UserRow {
                id: 42,
                name: "Alice".into(),
                is_active: true,
                score: 98.5,
                nullable_val: None,
            }
        );
    }
}
