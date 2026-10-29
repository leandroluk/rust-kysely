//! Abstract Syntax Tree (AST) operation nodes for rust-kysely.

pub mod clause_nodes;
pub mod expression_nodes;
pub mod terminal_nodes;
pub mod value;

pub use clause_nodes::{
    CommonTableExpressionNode, FromNode, GroupByNode, HavingNode, JoinNode, JoinType, LimitNode,
    OffsetNode, OrderByItemNode, OrderByNode, OrderDirection, OrderNulls, SelectionNode, WhereNode,
    WithNode,
};
pub use expression_nodes::{
    AliasNode, AndNode, BinaryOperationNode, CaseNode, CastNode, OrNode, ParensNode,
    UnaryOperationNode, WhenNode,
};
pub use terminal_nodes::{
    ColumnNode, IdentifierNode, PrimitiveValueListNode, RawNode, ReferenceNode, TableNode,
    ValueNode,
};
pub use value::Value;

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
}
