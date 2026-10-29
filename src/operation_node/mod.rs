//! Abstract Syntax Tree (AST) operation nodes for rust-kysely.

pub mod clause_nodes;
pub mod ddl_nodes;
pub mod expression_nodes;
pub mod query_nodes;
pub mod terminal_nodes;
pub mod transformer;
pub mod value;
pub mod visitor;

pub use clause_nodes::{
    CommonTableExpressionNode, FromNode, GroupByNode, HavingNode, JoinNode, JoinType, LimitNode,
    OffsetNode, OrderByItemNode, OrderByNode, OrderDirection, OrderNulls, SelectionNode,
    SetOperationNode, SetOperator, WhereNode, WithNode,
};
pub use ddl_nodes::*;
pub use expression_nodes::{
    AliasNode, AndNode, BinaryOperationNode, CaseNode, CastNode, FunctionNode, OrNode, ParensNode,
    UnaryOperationNode, WhenNode,
};
pub use query_nodes::{
    ColumnUpdateNode, DeleteQueryNode, InsertQueryNode, OnConflictNode, SelectQueryNode,
    UpdateQueryNode, ValuesNode,
};
pub use terminal_nodes::{
    ColumnNode, IdentifierNode, PrimitiveValueListNode, RawNode, ReferenceNode, TableNode,
    ValueNode,
};
pub use transformer::OperationNodeTransformer;
pub use value::{IntoValue, Value};
pub use visitor::{OperationNodeVisitor, StackTrackingVisitor};

/// Top-level AST node enum representing any node in the query tree.
// @spec REQ-AST-001
#[derive(Debug, Clone, PartialEq)]
pub enum OperationNode {
    // Terminals & Literals
    /// Identifier node
    Identifier(IdentifierNode),
    /// Table node
    Table(TableNode),
    /// Column node
    Column(ColumnNode),
    /// Reference node
    Reference(ReferenceNode),
    /// Parameter value node
    Value(ValueNode),
    /// Primitive value list node (e.g. IN expressions)
    PrimitiveValueList(PrimitiveValueListNode),
    /// Raw SQL node
    Raw(RawNode),

    // Clauses
    /// FROM clause node
    From(FromNode),
    /// JOIN clause node
    Join(JoinNode),
    /// Selection item node
    Selection(SelectionNode),
    /// WHERE clause node
    Where(WhereNode),
    /// HAVING clause node
    Having(HavingNode),
    /// GROUP BY clause node
    GroupBy(GroupByNode),
    /// ORDER BY clause node
    OrderBy(OrderByNode),
    /// ORDER BY item node
    OrderByItem(OrderByItemNode),
    /// LIMIT clause node
    Limit(LimitNode),
    /// OFFSET clause node
    Offset(OffsetNode),
    /// WITH clause node
    With(WithNode),
    /// Common Table Expression node
    CommonTableExpression(CommonTableExpressionNode),
    /// Set operation node (UNION, INTERSECT, EXCEPT)
    SetOperation(SetOperationNode),

    // Expressions
    /// Binary operation node
    Binary(BinaryOperationNode),
    /// Unary operation node
    Unary(UnaryOperationNode),
    /// CASE expression node
    Case(CaseNode),
    /// WHEN branch node
    When(WhenNode),
    /// CAST expression node
    Cast(CastNode),
    /// ALIAS node
    Alias(AliasNode),
    /// Parentheses expression node
    Parens(ParensNode),
    /// AND boolean node
    And(AndNode),
    /// OR boolean node
    Or(OrNode),
    /// Function call node
    Function(FunctionNode),

    // Queries
    /// SELECT query node
    SelectQuery(SelectQueryNode),
    /// INSERT query node
    InsertQuery(InsertQueryNode),
    /// UPDATE query node
    UpdateQuery(UpdateQueryNode),
    /// DELETE query node
    DeleteQuery(DeleteQueryNode),
    /// Column update node (SET col = val)
    ColumnUpdate(ColumnUpdateNode),
    /// VALUES clause node
    Values(ValuesNode),
    /// ON CONFLICT clause node
    OnConflict(OnConflictNode),

    // DDL
    /// CREATE TABLE node
    CreateTable(CreateTableNode),
    /// ALTER TABLE node
    AlterTable(AlterTableNode),
    /// DROP TABLE node
    DropTable(DropTableNode),
    /// CREATE INDEX node
    CreateIndex(CreateIndexNode),
    /// DROP INDEX node
    DropIndex(DropIndexNode),
    /// CREATE VIEW node
    CreateView(CreateViewNode),
    /// DROP VIEW node
    DropView(DropViewNode),
    /// CREATE TYPE node
    CreateType(CreateTypeNode),
    /// DROP TYPE node
    DropType(DropTypeNode),
}
