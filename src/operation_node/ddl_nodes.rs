//! AST operation nodes representing Data Definition Language (DDL) structures.

use std::sync::Arc;
use compact_str::CompactString;
use smallvec::SmallVec;

use crate::operation_node::OperationNode;

/// Action taken on foreign key modification (ON DELETE / ON UPDATE).
// @spec REQ-DDL-004
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnModifyForeignAction {
    /// NO ACTION
    NoAction,
    /// RESTRICT
    Restrict,
    /// CASCADE
    Cascade,
    /// SET NULL
    SetNull,
    /// SET DEFAULT
    SetDefault,
}

impl OnModifyForeignAction {
    /// Returns the SQL fragment for this action.
    pub fn as_sql(&self) -> &'static str {
        match self {
            OnModifyForeignAction::NoAction => "NO ACTION",
            OnModifyForeignAction::Restrict => "RESTRICT",
            OnModifyForeignAction::Cascade => "CASCADE",
            OnModifyForeignAction::SetNull => "SET NULL",
            OnModifyForeignAction::SetDefault => "SET DEFAULT",
        }
    }
}

/// AST node representing foreign key references specification on a column or constraint.
// @spec REQ-DDL-004
#[derive(Debug, Clone, PartialEq)]
pub struct ReferencesNode {
    /// Target foreign table name
    pub target_table: CompactString,
    /// Target column name
    pub target_column: CompactString,
    /// Action on delete
    pub on_delete: Option<OnModifyForeignAction>,
    /// Action on update
    pub on_update: Option<OnModifyForeignAction>,
}

impl ReferencesNode {
    /// Creates a new `ReferencesNode`.
    pub fn new(target_table: impl Into<CompactString>, target_column: impl Into<CompactString>) -> Self {
        Self {
            target_table: target_table.into(),
            target_column: target_column.into(),
            on_delete: None,
            on_update: None,
        }
    }
}

/// AST node representing a column definition in CREATE TABLE or ALTER TABLE.
// @spec REQ-DDL-002, REQ-DDL-004
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDefinitionNode {
    /// Column name
    pub name: CompactString,
    /// SQL data type (e.g. "varchar(255)", "serial", "bigint")
    pub data_type: CompactString,
    /// Is the column nullable (default true unless not_null called)
    pub is_nullable: bool,
    /// Does the column have a UNIQUE constraint
    pub is_unique: bool,
    /// Is the column a PRIMARY KEY
    pub is_primary_key: bool,
    /// Default expression (e.g. `now()`, `0`)
    pub default_to: Option<Arc<OperationNode>>,
    /// Check expression
    pub check: Option<Arc<OperationNode>>,
    /// Foreign key reference
    pub references: Option<ReferencesNode>,
}

impl ColumnDefinitionNode {
    /// Creates a new column definition node with defaults.
    pub fn new(name: impl Into<CompactString>, data_type: impl Into<CompactString>) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            is_nullable: true,
            is_unique: false,
            is_primary_key: false,
            default_to: None,
            check: None,
            references: None,
        }
    }
}

/// AST node representing a table-level constraint.
// @spec REQ-DDL-003
#[derive(Debug, Clone, PartialEq)]
pub enum TableConstraintNode {
    /// Primary key constraint
    PrimaryKey {
        /// Optional constraint name
        name: Option<CompactString>,
        /// Column list
        columns: SmallVec<[CompactString; 4]>,
    },
    /// Foreign key constraint
    ForeignKey {
        /// Optional constraint name
        name: Option<CompactString>,
        /// Local column names
        columns: SmallVec<[CompactString; 4]>,
        /// Target table name
        target_table: CompactString,
        /// Target column names
        target_columns: SmallVec<[CompactString; 4]>,
        /// On delete action
        on_delete: Option<OnModifyForeignAction>,
        /// On update action
        on_update: Option<OnModifyForeignAction>,
    },
    /// Unique constraint
    Unique {
        /// Optional constraint name
        name: Option<CompactString>,
        /// Column list
        columns: SmallVec<[CompactString; 4]>,
    },
    /// Check constraint
    Check {
        /// Optional constraint name
        name: Option<CompactString>,
        /// Boolean expression
        expr: Arc<OperationNode>,
    },
}

/// AST node representing a CREATE TABLE statement.
// @spec REQ-DDL-002, REQ-DDL-003
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableNode {
    /// Table name node
    pub name: Arc<OperationNode>,
    /// Column definitions
    pub columns: SmallVec<[ColumnDefinitionNode; 8]>,
    /// Table constraints
    pub constraints: SmallVec<[TableConstraintNode; 4]>,
    /// IF NOT EXISTS flag
    pub if_not_exists: bool,
    /// TEMPORARY flag
    pub temporary: bool,
}

impl CreateTableNode {
    /// Creates a new `CreateTableNode`.
    pub fn new(name: Arc<OperationNode>) -> Self {
        Self {
            name,
            columns: SmallVec::new(),
            constraints: SmallVec::new(),
            if_not_exists: false,
            temporary: false,
        }
    }
}

/// Action to perform in an ALTER TABLE statement.
// @spec REQ-DDL-005
#[derive(Debug, Clone, PartialEq)]
pub enum AlterTableAction {
    /// ADD COLUMN
    AddColumn(ColumnDefinitionNode),
    /// DROP COLUMN
    DropColumn {
        /// Column name
        name: CompactString,
        /// IF EXISTS
        if_exists: bool,
    },
    /// ALTER COLUMN TYPE
    AlterColumnType {
        /// Column name
        name: CompactString,
        /// New data type
        data_type: CompactString,
    },
    /// RENAME COLUMN
    RenameColumn {
        /// Old column name
        old_name: CompactString,
        /// New column name
        new_name: CompactString,
    },
    /// ADD CONSTRAINT
    AddConstraint(TableConstraintNode),
    /// DROP CONSTRAINT
    DropConstraint {
        /// Constraint name
        name: CompactString,
        /// IF EXISTS
        if_exists: bool,
    },
}

/// AST node representing an ALTER TABLE statement.
// @spec REQ-DDL-005
#[derive(Debug, Clone, PartialEq)]
pub struct AlterTableNode {
    /// Target table
    pub table: Arc<OperationNode>,
    /// Alter actions list
    pub actions: SmallVec<[AlterTableAction; 4]>,
}

impl AlterTableNode {
    /// Creates a new `AlterTableNode`.
    pub fn new(table: Arc<OperationNode>) -> Self {
        Self {
            table,
            actions: SmallVec::new(),
        }
    }
}

/// AST node representing a DROP TABLE statement.
// @spec REQ-DDL-006
#[derive(Debug, Clone, PartialEq)]
pub struct DropTableNode {
    /// Target table name node
    pub name: Arc<OperationNode>,
    /// IF EXISTS flag
    pub if_exists: bool,
    /// CASCADE flag
    pub cascade: bool,
}

impl DropTableNode {
    /// Creates a new `DropTableNode`.
    pub fn new(name: Arc<OperationNode>) -> Self {
        Self {
            name,
            if_exists: false,
            cascade: false,
        }
    }
}

/// AST node representing a CREATE INDEX statement.
// @spec REQ-DDL-007
#[derive(Debug, Clone, PartialEq)]
pub struct CreateIndexNode {
    /// Index name
    pub name: CompactString,
    /// Target table name node
    pub table: Arc<OperationNode>,
    /// Columns in index
    pub columns: SmallVec<[CompactString; 4]>,
    /// UNIQUE index flag
    pub unique: bool,
    /// USING method (e.g. btree, gin, gist)
    pub using_method: Option<CompactString>,
    /// Partial index WHERE clause
    pub where_clause: Option<Arc<OperationNode>>,
    /// IF NOT EXISTS flag
    pub if_not_exists: bool,
}

impl CreateIndexNode {
    /// Creates a new `CreateIndexNode`.
    pub fn new(name: impl Into<CompactString>, table: Arc<OperationNode>) -> Self {
        Self {
            name: name.into(),
            table,
            columns: SmallVec::new(),
            unique: false,
            using_method: None,
            where_clause: None,
            if_not_exists: false,
        }
    }
}

/// AST node representing a DROP INDEX statement.
// @spec REQ-DDL-007
#[derive(Debug, Clone, PartialEq)]
pub struct DropIndexNode {
    /// Index name
    pub name: CompactString,
    /// IF EXISTS flag
    pub if_exists: bool,
    /// CASCADE flag
    pub cascade: bool,
}

impl DropIndexNode {
    /// Creates a new `DropIndexNode`.
    pub fn new(name: impl Into<CompactString>) -> Self {
        Self {
            name: name.into(),
            if_exists: false,
            cascade: false,
        }
    }
}

/// AST node representing a CREATE VIEW statement.
// @spec REQ-DDL-009
#[derive(Debug, Clone, PartialEq)]
pub struct CreateViewNode {
    /// View name
    pub name: CompactString,
    /// View SELECT query
    pub query: Arc<OperationNode>,
    /// OR REPLACE flag
    pub or_replace: bool,
    /// MATERIALIZED VIEW flag
    pub materialized: bool,
    /// IF NOT EXISTS flag
    pub if_not_exists: bool,
}

impl CreateViewNode {
    /// Creates a new `CreateViewNode`.
    pub fn new(name: impl Into<CompactString>, query: Arc<OperationNode>) -> Self {
        Self {
            name: name.into(),
            query,
            or_replace: false,
            materialized: false,
            if_not_exists: false,
        }
    }
}

/// AST node representing a DROP VIEW statement.
// @spec REQ-DDL-009
#[derive(Debug, Clone, PartialEq)]
pub struct DropViewNode {
    /// View name
    pub name: CompactString,
    /// MATERIALIZED VIEW flag
    pub materialized: bool,
    /// IF EXISTS flag
    pub if_exists: bool,
    /// CASCADE flag
    pub cascade: bool,
}

impl DropViewNode {
    /// Creates a new `DropViewNode`.
    pub fn new(name: impl Into<CompactString>) -> Self {
        Self {
            name: name.into(),
            materialized: false,
            if_exists: false,
            cascade: false,
        }
    }
}

/// AST node representing a CREATE TYPE AS ENUM statement.
// @spec REQ-DDL-008
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTypeNode {
    /// Custom type name
    pub name: CompactString,
    /// Enum variant values
    pub enum_values: SmallVec<[CompactString; 8]>,
}

impl CreateTypeNode {
    /// Creates a new `CreateTypeNode`.
    pub fn new(name: impl Into<CompactString>) -> Self {
        Self {
            name: name.into(),
            enum_values: SmallVec::new(),
        }
    }
}

/// AST node representing a DROP TYPE statement.
// @spec REQ-DDL-008
#[derive(Debug, Clone, PartialEq)]
pub struct DropTypeNode {
    /// Custom type name
    pub name: CompactString,
    /// IF EXISTS flag
    pub if_exists: bool,
    /// CASCADE flag
    pub cascade: bool,
}

impl DropTypeNode {
    /// Creates a new `DropTypeNode`.
    pub fn new(name: impl Into<CompactString>) -> Self {
        Self {
            name: name.into(),
            if_exists: false,
            cascade: false,
        }
    }
}
