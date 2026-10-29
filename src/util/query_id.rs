//! Query ID generator for lifecycle tracking and debugging.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

static QUERY_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// A unique 64-bit identifier assigned to each query for tracking and lifecycle debugging.
// @spec REQ-AST-005
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QueryId(pub u64);

impl QueryId {
    /// Generates a new sequentially increasing QueryId.
    pub fn generate() -> Self {
        Self(QUERY_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Creates a QueryId from a raw u64 value.
    pub const fn from_raw(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw u64 representation.
    pub const fn raw(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for QueryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_id_generation() {
        let id1 = QueryId::generate();
        let id2 = QueryId::generate();
        assert_ne!(id1, id2);
        assert!(id2.raw() > id1.raw());
        assert_eq!(format!("{}", id1), id1.raw().to_string());
    }
}
