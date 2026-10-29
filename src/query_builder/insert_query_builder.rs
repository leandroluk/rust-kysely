//! Fluent, immutable InsertQueryBuilder implementation.

use std::sync::Arc;
use compact_str::CompactString;
use smallvec::SmallVec;

use crate::expression::Expression;
use crate::operation_node::{
    ColumnNode, InsertQueryNode, IntoValue, OperationNode, PrimitiveValueListNode, SelectionNode,
    Value, ValuesNode,
};
use crate::driver::DriverError;
use crate::parser::{IntoSelections, IntoTableSource};
use crate::query_builder::on_conflict_builder::OnConflictBuilder;
use crate::query_compiler::{CompiledQuery, QueryCompiler};
use crate::query_executor::{QueryExecutor, QueryResult, Row};

/// Fluent query builder for SQL INSERT statements.
// @spec REQ-MUT-001
#[derive(Clone, Default)]
pub struct InsertQueryBuilder {
    pub(crate) node: Arc<InsertQueryNode>,
    pub(crate) executor: Option<Arc<dyn QueryExecutor>>,
}

impl std::fmt::Debug for InsertQueryBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InsertQueryBuilder")
            .field("node", &self.node)
            .finish()
    }
}

impl PartialEq for InsertQueryBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl InsertQueryBuilder {
    /// Creates a new empty INSERT query builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a builder wrapping an existing AST node.
    pub fn from_node(node: Arc<InsertQueryNode>) -> Self {
        Self {
            node,
            executor: None,
        }
    }

    /// Attaches an executor to the query builder.
    pub fn with_executor(mut self, executor: Arc<dyn QueryExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Returns a reference to the executor if attached.
    pub fn executor(&self) -> Option<&Arc<dyn QueryExecutor>> {
        self.executor.as_ref()
    }

    /// Executes the query and returns the `QueryResult`.
    // @spec REQ-EXEC-004
    pub async fn execute(&self) -> Result<QueryResult, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on QueryBuilder".into())
        })?;
        executor.execute_node(&OperationNode::InsertQuery((*self.node).clone())).await
    }

    /// Executes the query and returns the first row, if any.
    // @spec REQ-EXEC-007
    pub async fn execute_take_first(&self) -> Result<Option<Row>, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on QueryBuilder".into())
        })?;
        executor.execute_node_take_first(&OperationNode::InsertQuery((*self.node).clone())).await
    }

    /// Executes the query and returns the first row, or returns an error if empty.
    // @spec REQ-EXEC-007
    pub async fn execute_take_first_or_throw(&self) -> Result<Row, DriverError> {
        let executor = self.executor.as_ref().ok_or_else(|| {
            DriverError::ExecutionFailed("No QueryExecutor configured on QueryBuilder".into())
        })?;
        executor.execute_node_take_first_or_throw(&OperationNode::InsertQuery((*self.node).clone())).await
    }

    /// Executes the query and deserializes rows into a `Vec<T>`.
    // @spec REQ-EXEC-006
    #[cfg(feature = "json")]
    pub async fn execute_into<T: serde::de::DeserializeOwned>(&self) -> Result<Vec<T>, DriverError> {
        let res = self.execute().await?;
        res.rows
            .into_iter()
            .map(|r| r.into_record::<T>().map_err(|e| DriverError::ExecutionFailed(e.to_string())))
            .collect()
    }

    /// Executes the query and deserializes the first row into `Option<T>`.
    // @spec REQ-EXEC-006
    #[cfg(feature = "json")]
    pub async fn execute_take_first_into<T: serde::de::DeserializeOwned>(&self) -> Result<Option<T>, DriverError> {
        if let Some(row) = self.execute_take_first().await? {
            let record = row.into_record::<T>().map_err(|e| DriverError::ExecutionFailed(e.to_string()))?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    /// Assigns a column value for single-row insert (`.set("name", "Alice")`).
    ///
    /// Accepts any type implementing `IntoValue` without requiring `.into()`.
    // @spec REQ-MUT-002
    pub fn set(mut self, col: impl Into<CompactString>, val: impl IntoValue) -> Self {
        let col_name: CompactString = col.into();
        let val = val.into_value();
        let col_node = Arc::new(OperationNode::Column(ColumnNode::new(col_name)));

        let q = InsertQueryNode::clone_mut(&mut self.node);

        // 1. Update or append columns
        let columns = q.columns.get_or_insert_with(SmallVec::new);
        columns.push(col_node);

        // 2. Update or create single row in values
        if let Some(ref mut values_op) = q.values {
            if let OperationNode::Values(ref mut values_node) = Arc::make_mut(values_op) {
                if let Some(first_row) = values_node.rows.first_mut() {
                    if let OperationNode::PrimitiveValueList(ref mut list) = Arc::make_mut(first_row) {
                        list.values.push(val);
                        return self;
                    }
                }
            }
        }

        let list_node = Arc::new(OperationNode::PrimitiveValueList(PrimitiveValueListNode::new([val])));
        q.values = Some(Arc::new(OperationNode::Values(ValuesNode::new([list_node]))));

        self
    }

    /// Sets row values using a closure builder (`|v| v.value(...).value(...)`) or row items.
    // @spec REQ-MUT-003
    pub fn values<R: IntoRowValues>(self, row: R) -> Self {
        self.values_batch([row])
    }

    /// Inserts multiple rows in a single batch query.
    // @spec REQ-MUT-003
    pub fn values_batch<I, R>(mut self, rows: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoRowValues,
    {
        let row_list: Vec<RowValues> = rows.into_iter().map(|r| r.into_row_values()).collect();
        if row_list.is_empty() {
            return self;
        }

        // Collect all unique column names in order of appearance
        let mut column_names: SmallVec<[CompactString; 8]> = SmallVec::new();
        for r in &row_list {
            for (col, _) in &r.entries {
                if !column_names.contains(col) {
                    column_names.push(col.clone());
                }
            }
        }

        // Build ValuesNode rows matching column_names order
        let mut value_rows: SmallVec<[Arc<OperationNode>; 4]> = SmallVec::new();
        for r in &row_list {
            let mut row_vals: SmallVec<[Value; 4]> = SmallVec::new();
            for col in &column_names {
                let val = r
                    .entries
                    .iter()
                    .find(|(c, _)| c == col)
                    .map(|(_, v)| v.clone())
                    .unwrap_or(Value::Null);
                row_vals.push(val);
            }
            value_rows.push(Arc::new(OperationNode::PrimitiveValueList(
                PrimitiveValueListNode::new(row_vals),
            )));
        }

        let q = InsertQueryNode::clone_mut(&mut self.node);
        q.columns = Some(
            column_names
                .into_iter()
                .map(|name| Arc::new(OperationNode::Column(ColumnNode::new(name))))
                .collect(),
        );
        q.values = Some(Arc::new(OperationNode::Values(ValuesNode::new(value_rows))));

        self
    }

    /// Populates insert columns and values from any serde-serializable struct without custom macros.
    // @spec REQ-MUT-004
    #[cfg(feature = "json")]
    pub fn values_record<T: serde::Serialize>(self, record: &T) -> Result<Self, serde_json::Error> {
        let json_val = serde_json::to_value(record)?;
        if let serde_json::Value::Object(map) = json_val {
            let mut row = RowValues::new();
            for (k, v) in map {
                row = row.value(k, Value::from(v));
            }
            Ok(self.values(row))
        } else {
            Ok(self)
        }
    }

    /// Configures PostgreSQL `ON CONFLICT` handling.
    // @spec REQ-MUT-005
    pub fn on_conflict(mut self, cb: impl FnOnce(OnConflictBuilder) -> OnConflictBuilder) -> Self {
        let builder = cb(OnConflictBuilder::new());
        let q = InsertQueryNode::clone_mut(&mut self.node);
        q.on_conflict = Some(Arc::new(OperationNode::OnConflict(builder.build())));
        self
    }

    /// Appends selections to the `RETURNING` clause.
    // @spec REQ-MUT-010
    pub fn returning(mut self, selections: impl IntoSelections) -> Self {
        let sel_nodes = selections.into_selections();
        let q = InsertQueryNode::clone_mut(&mut self.node);
        let ret = q.returning.get_or_insert_with(SmallVec::new);
        ret.extend(sel_nodes);
        self
    }

    /// Selects all columns in the `RETURNING` clause (`RETURNING *`).
    // @spec REQ-MUT-010
    pub fn returning_all(mut self) -> Self {
        let q = InsertQueryNode::clone_mut(&mut self.node);
        let ret = q.returning.get_or_insert_with(SmallVec::new);
        ret.push(Arc::new(OperationNode::Selection(SelectionNode::new(
            Arc::new(OperationNode::Column(ColumnNode::new("*"))),
        ))));
        self
    }

    /// Compiles the query using the specified query compiler.
    pub fn compile(&self, compiler: &impl QueryCompiler) -> CompiledQuery {
        compiler.compile(&OperationNode::InsertQuery((*self.node).clone()))
    }
}

impl Expression for InsertQueryBuilder {
    fn to_operation_node(&self) -> Arc<OperationNode> {
        Arc::new(OperationNode::InsertQuery((*self.node).clone()))
    }
}

/// Entry point creating an `InsertQueryBuilder` targeting the specified table.
// @spec REQ-MUT-001
pub fn insert_into(table: impl IntoTableSource) -> InsertQueryBuilder {
    let mut builder = InsertQueryBuilder::new();
    let q = InsertQueryNode::clone_mut(&mut builder.node);
    q.into = Some(table.into_table_source());
    builder
}

/// Helper context for defining column-value pairs within a row.
// @spec REQ-MUT-003
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RowValues {
    pub(crate) entries: SmallVec<[(CompactString, Value); 8]>,
}

impl RowValues {
    /// Creates a new empty RowValues container.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a column and value on this row.
    pub fn value(mut self, col: impl Into<CompactString>, val: impl IntoValue) -> Self {
        self.entries.push((col.into(), val.into_value()));
        self
    }
}

/// Trait for items that can be converted into a row of column-value pairs.
// @spec REQ-MUT-003
pub trait IntoRowValues {
    /// Converts this into a `RowValues` object.
    fn into_row_values(self) -> RowValues;
}

impl IntoRowValues for RowValues {
    fn into_row_values(self) -> RowValues {
        self
    }
}

impl<F> IntoRowValues for F
where
    F: FnOnce(RowValues) -> RowValues,
{
    fn into_row_values(self) -> RowValues {
        self(RowValues::new())
    }
}

impl<C, V, const N: usize> IntoRowValues for [(C, V); N]
where
    C: Into<CompactString>,
    V: IntoValue,
{
    fn into_row_values(self) -> RowValues {
        let mut row = RowValues::new();
        for (col, val) in self {
            row = row.value(col, val);
        }
        row
    }
}

impl<C, V> IntoRowValues for Vec<(C, V)>
where
    C: Into<CompactString>,
    V: IntoValue,
{
    fn into_row_values(self) -> RowValues {
        let mut row = RowValues::new();
        for (col, val) in self {
            row = row.value(col, val);
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::PostgresQueryCompiler;
    use crate::operation_node::Value;

    #[test]
    fn test_insert_single_row() {
        let compiler = PostgresQueryCompiler::new();

        let query = insert_into("users")
            .set("first_name", "Alice")
            .set("last_name", "Smith")
            .set("age", 30)
            .set("active", true)
            .compile(&compiler);

        assert_eq!(
            query.sql,
            "INSERT INTO \"users\" (\"first_name\", \"last_name\", \"age\", \"active\") VALUES ($1, $2, $3, $4)"
        );
        assert_eq!(
            query.parameters,
            vec![
                Value::from("Alice"),
                Value::from("Smith"),
                Value::from(30),
                Value::from(true),
            ]
        );
    }

    #[test]
    fn test_insert_batch_and_record() {
        let compiler = PostgresQueryCompiler::new();

        // 1. values_batch with closure builders
        let q1 = insert_into("users")
            .values_batch([
                |v: RowValues| v.value("name", "Alice").value("age", 25),
                |v: RowValues| v.value("name", "Bob").value("age", 30),
            ])
            .compile(&compiler);

        assert_eq!(
            q1.sql,
            "INSERT INTO \"users\" (\"name\", \"age\") VALUES ($1, $2), ($3, $4)"
        );
        assert_eq!(
            q1.parameters,
            vec![
                Value::from("Alice"),
                Value::from(25),
                Value::from("Bob"),
                Value::from(30),
            ]
        );

        // 2. values_record with serde struct
        #[derive(serde::Serialize)]
        struct NewUser {
            email: String,
            role: String,
        }

        let u = NewUser {
            email: "charlie@example.com".into(),
            role: "admin".into(),
        };

        let q2 = insert_into("users")
            .values_record(&u)
            .unwrap()
            .compile(&compiler);

        assert_eq!(
            q2.sql,
            "INSERT INTO \"users\" (\"email\", \"role\") VALUES ($1, $2)"
        );
        assert_eq!(
            q2.parameters,
            vec![
                Value::from("charlie@example.com"),
                Value::from("admin"),
            ]
        );
    }

    #[test]
    fn test_insert_on_conflict() {
        let compiler = PostgresQueryCompiler::new();

        // 1. DO NOTHING
        let q1 = insert_into("users")
            .set("id", 1)
            .set("name", "Alice")
            .on_conflict(|oc| oc.column("id").do_nothing())
            .compile(&compiler);

        assert_eq!(
            q1.sql,
            "INSERT INTO \"users\" (\"id\", \"name\") VALUES ($1, $2) ON CONFLICT (\"id\") DO NOTHING"
        );
        assert_eq!(q1.parameters, vec![Value::from(1), Value::from("Alice")]);

        // 2. DO UPDATE SET with excluded references and where filter
        let q2 = insert_into("users")
            .set("id", 1)
            .set("name", "Alice")
            .set("email", "alice@example.com")
            .on_conflict(|oc| {
                oc.column("id")
                  .do_update_set(|ub| {
                      ub.set_ref("name", "excluded.name")
                        .set_ref("email", "excluded.email")
                        .where_("users.active", "=", true)
                  })
            })
            .compile(&compiler);

        assert_eq!(
            q2.sql,
            "INSERT INTO \"users\" (\"id\", \"name\", \"email\") VALUES ($1, $2, $3) ON CONFLICT (\"id\") DO UPDATE SET \"name\" = \"excluded\".\"name\", \"email\" = \"excluded\".\"email\" WHERE \"users\".\"active\" = $4"
        );
        assert_eq!(
            q2.parameters,
            vec![
                Value::from(1),
                Value::from("Alice"),
                Value::from("alice@example.com"),
                Value::from(true),
            ]
        );
    }

    #[test]
    fn test_mutation_returning() {
        let compiler = PostgresQueryCompiler::new();

        // 1. INSERT RETURNING
        let q_ins = insert_into("users")
            .set("name", "Diana")
            .returning(["id", "created_at"])
            .compile(&compiler);

        assert_eq!(
            q_ins.sql,
            "INSERT INTO \"users\" (\"name\") VALUES ($1) RETURNING \"id\", \"created_at\""
        );
        assert_eq!(q_ins.parameters, vec![Value::from("Diana")]);

        let q_ins_all = insert_into("users")
            .set("name", "Diana")
            .returning_all()
            .compile(&compiler);

        assert_eq!(
            q_ins_all.sql,
            "INSERT INTO \"users\" (\"name\") VALUES ($1) RETURNING *"
        );

        // 2. UPDATE RETURNING
        let q_upd = crate::query_builder::update_table("users")
            .set("active", false)
            .where_("id", "=", 10)
            .returning(["id", "active"])
            .compile(&compiler);

        assert_eq!(
            q_upd.sql,
            "UPDATE \"users\" SET \"active\" = $1 WHERE \"id\" = $2 RETURNING \"id\", \"active\""
        );

        // 3. DELETE RETURNING
        let q_del = crate::query_builder::delete_from("users")
            .where_("id", "=", 10)
            .returning_all()
            .compile(&compiler);

        assert_eq!(
            q_del.sql,
            "DELETE FROM \"users\" WHERE \"id\" = $1 RETURNING *"
        );
    }
}
