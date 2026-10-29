//! Strongly-typed SQL scalar literal representation.

use compact_str::CompactString;
use std::fmt;

/// Represents a scalar parameterized value in an SQL query AST.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// SQL NULL value
    Null,
    /// Boolean value
    Bool(bool),
    /// 64-bit signed integer
    I64(i64),
    /// 64-bit floating point number
    F64(f64),
    /// Text string (using CompactString for stack optimization <= 24 bytes)
    Text(CompactString),
    /// Raw byte sequence (BLOB / BYTEA)
    Bytes(Vec<u8>),
}

impl Value {
    /// Returns true if this value is SQL NULL.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::I64(i) => write!(f, "{}", i),
            Value::F64(n) => write!(f, "{}", n),
            Value::Text(t) => write!(f, "{}", t),
            Value::Bytes(b) => write!(f, "<{} bytes>", b.len()),
        }
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Value::I64(v as i64)
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Value::I64(v as i64)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::I64(v as i64)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::I64(v)
    }
}

impl From<isize> for Value {
    fn from(v: isize) -> Self {
        Value::I64(v as i64)
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::I64(v as i64)
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Value::I64(v as i64)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::I64(v as i64)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::F64(v as f64)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::F64(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Text(CompactString::new(v))
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Text(CompactString::new(v))
    }
}

impl From<CompactString> for Value {
    fn from(v: CompactString) -> Self {
        Value::Text(v)
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Bytes(v)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}

/// Trait for converting Rust scalar and reference values into an AST `Value` parameter without explicit `.into()`.
// @spec REQ-EXPR-002
pub trait IntoValue {
    /// Converts this value into an SQL AST `Value`.
    fn into_value(self) -> Value;
}

impl IntoValue for Value {
    fn into_value(self) -> Value {
        self
    }
}

impl IntoValue for bool {
    fn into_value(self) -> Value {
        Value::Bool(self)
    }
}

impl IntoValue for i8 {
    fn into_value(self) -> Value {
        Value::I64(self as i64)
    }
}

impl IntoValue for i16 {
    fn into_value(self) -> Value {
        Value::I64(self as i64)
    }
}

impl IntoValue for i32 {
    fn into_value(self) -> Value {
        Value::I64(self as i64)
    }
}

impl IntoValue for i64 {
    fn into_value(self) -> Value {
        Value::I64(self)
    }
}

impl IntoValue for isize {
    fn into_value(self) -> Value {
        Value::I64(self as i64)
    }
}

impl IntoValue for u8 {
    fn into_value(self) -> Value {
        Value::I64(self as i64)
    }
}

impl IntoValue for u16 {
    fn into_value(self) -> Value {
        Value::I64(self as i64)
    }
}

impl IntoValue for u32 {
    fn into_value(self) -> Value {
        Value::I64(self as i64)
    }
}

impl IntoValue for f32 {
    fn into_value(self) -> Value {
        Value::F64(self as f64)
    }
}

impl IntoValue for f64 {
    fn into_value(self) -> Value {
        Value::F64(self)
    }
}

impl IntoValue for &str {
    fn into_value(self) -> Value {
        Value::Text(CompactString::new(self))
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::Text(CompactString::new(self))
    }
}

impl IntoValue for CompactString {
    fn into_value(self) -> Value {
        Value::Text(self)
    }
}

impl IntoValue for Vec<u8> {
    fn into_value(self) -> Value {
        Value::Bytes(self)
    }
}

impl IntoValue for &[u8] {
    fn into_value(self) -> Value {
        Value::Bytes(self.to_vec())
    }
}

impl<T: IntoValue> IntoValue for Option<T> {
    fn into_value(self) -> Value {
        match self {
            Some(v) => v.into_value(),
            None => Value::Null,
        }
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Value> for Value {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::I64(i)
                } else if let Some(f) = n.as_f64() {
                    Value::F64(f)
                } else {
                    Value::Null
                }
            }
            serde_json::Value::String(s) => Value::Text(CompactString::from(s)),
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                Value::Text(CompactString::from(v.to_string()))
            }
        }
    }
}

#[cfg(feature = "json")]
impl IntoValue for serde_json::Value {
    fn into_value(self) -> Value {
        Value::from(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_conversions() {
        assert_eq!(Value::from(42i32), Value::I64(42));
        assert_eq!(
            Value::from("hello"),
            Value::Text(CompactString::new("hello"))
        );
        assert_eq!(Value::from(true), Value::Bool(true));
        assert_eq!(Value::from(Option::<i32>::None), Value::Null);
        assert_eq!(Value::from(Some(100i64)), Value::I64(100));
        assert_eq!(format!("{}", Value::Null), "NULL");
        assert_eq!(format!("{}", Value::I64(10)), "10");
        assert!(Value::Null.is_null());
        assert!(!Value::I64(10).is_null());

        // REQ-EXPR-002: Test into_value frictionless conversions
        assert_eq!(123i32.into_value(), Value::I64(123));
        assert_eq!("foo".into_value(), Value::Text(CompactString::new("foo")));
        assert_eq!(true.into_value(), Value::Bool(true));
        assert_eq!(None::<i32>.into_value(), Value::Null);
        assert_eq!(Some(456i64).into_value(), Value::I64(456));
    }
}
