//! AST traversal visitor trait and parent node stack tracking.

use crate::operation_node::*;

/// Trait providing recursive depth-first traversal of the `OperationNode` AST.
// @spec REQ-AST-006
pub trait OperationNodeVisitor {
    /// Dispatches visitor to the appropriate specialized method based on node variant.
    fn visit_node(&mut self, node: &OperationNode) {
        match node {
            OperationNode::Identifier(n) => self.visit_identifier(n),
            OperationNode::Table(n) => self.visit_table(n),
            OperationNode::Column(n) => self.visit_column(n),
            OperationNode::Reference(n) => self.visit_reference(n),
            OperationNode::Value(n) => self.visit_value(n),
            OperationNode::PrimitiveValueList(n) => self.visit_primitive_value_list(n),
            OperationNode::Raw(n) => self.visit_raw(n),
            OperationNode::From(n) => self.visit_from(n),
            OperationNode::Join(n) => self.visit_join(n),
            OperationNode::Selection(n) => self.visit_selection(n),
            OperationNode::Where(n) => self.visit_where(n),
            OperationNode::Having(n) => self.visit_having(n),
            OperationNode::GroupBy(n) => self.visit_group_by(n),
            OperationNode::OrderBy(n) => self.visit_order_by(n),
            OperationNode::OrderByItem(n) => self.visit_order_by_item(n),
            OperationNode::Limit(n) => self.visit_limit(n),
            OperationNode::Offset(n) => self.visit_offset(n),
            OperationNode::With(n) => self.visit_with(n),
            OperationNode::CommonTableExpression(n) => self.visit_common_table_expression(n),
            OperationNode::SetOperation(n) => self.visit_set_operation(n),
            OperationNode::Binary(n) => self.visit_binary(n),
            OperationNode::Unary(n) => self.visit_unary(n),
            OperationNode::Case(n) => self.visit_case(n),
            OperationNode::When(n) => self.visit_when(n),
            OperationNode::Cast(n) => self.visit_cast(n),
            OperationNode::Alias(n) => self.visit_alias(n),
            OperationNode::Parens(n) => self.visit_parens(n),
            OperationNode::And(n) => self.visit_and(n),
            OperationNode::Or(n) => self.visit_or(n),
            OperationNode::Function(n) => self.visit_function(n),
            OperationNode::SelectQuery(n) => self.visit_select_query(n),
            OperationNode::InsertQuery(n) => self.visit_insert_query(n),
            OperationNode::UpdateQuery(n) => self.visit_update_query(n),
            OperationNode::DeleteQuery(n) => self.visit_delete_query(n),
            OperationNode::ColumnUpdate(n) => self.visit_column_update(n),
            OperationNode::Values(n) => self.visit_values(n),
            OperationNode::OnConflict(n) => self.visit_on_conflict(n),

            // DDL
            OperationNode::CreateTable(n) => self.visit_create_table(n),
            OperationNode::AlterTable(n) => self.visit_alter_table(n),
            OperationNode::DropTable(n) => self.visit_drop_table(n),
            OperationNode::CreateIndex(n) => self.visit_create_index(n),
            OperationNode::DropIndex(n) => self.visit_drop_index(n),
            OperationNode::CreateView(n) => self.visit_create_view(n),
            OperationNode::DropView(n) => self.visit_drop_view(n),
            OperationNode::CreateType(n) => self.visit_create_type(n),
            OperationNode::DropType(n) => self.visit_drop_type(n),
        }
    }

    /// Visits an IdentifierNode (leaf).
    fn visit_identifier(&mut self, _node: &IdentifierNode) {}

    /// Visits a TableNode.
    fn visit_table(&mut self, node: &TableNode) {
        if let Some(ref schema) = node.schema {
            self.visit_node(schema);
        }
        self.visit_node(&node.table);
    }

    /// Visits a ColumnNode.
    fn visit_column(&mut self, node: &ColumnNode) {
        self.visit_node(&node.column);
    }

    /// Visits a ReferenceNode.
    fn visit_reference(&mut self, node: &ReferenceNode) {
        if let Some(ref table) = node.table {
            self.visit_node(table);
        }
        self.visit_node(&node.column);
    }

    /// Visits a ValueNode (leaf).
    fn visit_value(&mut self, _node: &ValueNode) {}

    /// Visits a PrimitiveValueListNode (leaf).
    fn visit_primitive_value_list(&mut self, _node: &PrimitiveValueListNode) {}

    /// Visits a RawNode.
    fn visit_raw(&mut self, node: &RawNode) {
        for param in &node.parameters {
            self.visit_node(param);
        }
    }

    /// Visits a FromNode.
    fn visit_from(&mut self, node: &FromNode) {
        for from in &node.froms {
            self.visit_node(from);
        }
    }

    /// Visits a JoinNode.
    fn visit_join(&mut self, node: &JoinNode) {
        self.visit_node(&node.table);
        if let Some(ref on) = node.on {
            self.visit_node(on);
        }
    }

    /// Visits a SelectionNode.
    fn visit_selection(&mut self, node: &SelectionNode) {
        self.visit_node(&node.expr);
        if let Some(ref alias) = node.alias {
            self.visit_node(alias);
        }
    }

    /// Visits a WhereNode.
    fn visit_where(&mut self, node: &WhereNode) {
        self.visit_node(&node.where_expr);
    }

    /// Visits a HavingNode.
    fn visit_having(&mut self, node: &HavingNode) {
        self.visit_node(&node.having_expr);
    }

    /// Visits a GroupByNode.
    fn visit_group_by(&mut self, node: &GroupByNode) {
        for item in &node.items {
            self.visit_node(item);
        }
    }

    /// Visits an OrderByNode.
    fn visit_order_by(&mut self, node: &OrderByNode) {
        for item in &node.items {
            self.visit_node(item);
        }
    }

    /// Visits an OrderByItemNode.
    fn visit_order_by_item(&mut self, node: &OrderByItemNode) {
        self.visit_node(&node.expr);
    }

    /// Visits a LimitNode.
    fn visit_limit(&mut self, node: &LimitNode) {
        self.visit_node(&node.limit);
    }

    /// Visits an OffsetNode.
    fn visit_offset(&mut self, node: &OffsetNode) {
        self.visit_node(&node.offset);
    }

    /// Visits a WithNode.
    fn visit_with(&mut self, node: &WithNode) {
        for expr in &node.expressions {
            self.visit_node(expr);
        }
    }

    /// Visits a CommonTableExpressionNode.
    fn visit_common_table_expression(&mut self, node: &CommonTableExpressionNode) {
        self.visit_node(&node.name);
        self.visit_node(&node.expression);
    }

    /// Visits a SetOperationNode.
    fn visit_set_operation(&mut self, node: &SetOperationNode) {
        self.visit_node(&node.expression);
    }

    /// Visits a BinaryOperationNode.
    fn visit_binary(&mut self, node: &BinaryOperationNode) {
        self.visit_node(&node.left);
        self.visit_node(&node.right);
    }

    /// Visits a UnaryOperationNode.
    fn visit_unary(&mut self, node: &UnaryOperationNode) {
        self.visit_node(&node.operand);
    }

    /// Visits a CaseNode.
    fn visit_case(&mut self, node: &CaseNode) {
        if let Some(ref val) = node.value {
            self.visit_node(val);
        }
        for when in &node.when_clauses {
            self.visit_node(when);
        }
        if let Some(ref el) = node.else_clause {
            self.visit_node(el);
        }
    }

    /// Visits a WhenNode.
    fn visit_when(&mut self, node: &WhenNode) {
        self.visit_node(&node.condition);
        self.visit_node(&node.result);
    }

    /// Visits a CastNode.
    fn visit_cast(&mut self, node: &CastNode) {
        self.visit_node(&node.expr);
    }

    /// Visits an AliasNode.
    fn visit_alias(&mut self, node: &AliasNode) {
        self.visit_node(&node.node);
        self.visit_node(&node.alias);
    }

    /// Visits a ParensNode.
    fn visit_parens(&mut self, node: &ParensNode) {
        self.visit_node(&node.node);
    }

    /// Visits an AndNode.
    fn visit_and(&mut self, node: &AndNode) {
        self.visit_node(&node.left);
        self.visit_node(&node.right);
    }

    /// Visits an OrNode.
    fn visit_or(&mut self, node: &OrNode) {
        self.visit_node(&node.left);
        self.visit_node(&node.right);
    }

    /// Visits a FunctionNode.
    fn visit_function(&mut self, node: &FunctionNode) {
        for arg in &node.arguments {
            self.visit_node(arg);
        }
    }

    /// Visits a SelectQueryNode.
    fn visit_select_query(&mut self, node: &SelectQueryNode) {
        if let Some(ref with) = node.with {
            self.visit_node(with);
        }
        if let Some(ref distinct) = node.distinct_on {
            for d in distinct {
                self.visit_node(d);
            }
        }
        for sel in &node.selections {
            self.visit_node(sel);
        }
        if let Some(ref from) = node.from {
            self.visit_node(from);
        }
        for join in &node.joins {
            self.visit_node(join);
        }
        if let Some(ref whr) = node.where_clause {
            self.visit_node(whr);
        }
        if let Some(ref gb) = node.group_by {
            self.visit_node(gb);
        }
        if let Some(ref hav) = node.having {
            self.visit_node(hav);
        }
        if let Some(ref ob) = node.order_by {
            self.visit_node(ob);
        }
        if let Some(ref lim) = node.limit {
            self.visit_node(lim);
        }
        if let Some(ref off) = node.offset {
            self.visit_node(off);
        }
        for set_op in &node.set_operations {
            self.visit_node(set_op);
        }
    }

    /// Visits an InsertQueryNode.
    fn visit_insert_query(&mut self, node: &InsertQueryNode) {
        if let Some(ref with) = node.with {
            self.visit_node(with);
        }
        if let Some(ref into) = node.into {
            self.visit_node(into);
        }
        if let Some(ref cols) = node.columns {
            for col in cols {
                self.visit_node(col);
            }
        }
        if let Some(ref vals) = node.values {
            self.visit_node(vals);
        }
        if let Some(ref oc) = node.on_conflict {
            self.visit_node(oc);
        }
        if let Some(ref ret) = node.returning {
            for r in ret {
                self.visit_node(r);
            }
        }
    }

    /// Visits an UpdateQueryNode.
    fn visit_update_query(&mut self, node: &UpdateQueryNode) {
        if let Some(ref with) = node.with {
            self.visit_node(with);
        }
        if let Some(ref table) = node.table {
            self.visit_node(table);
        }
        if let Some(ref from) = node.from {
            self.visit_node(from);
        }
        for join in &node.joins {
            self.visit_node(join);
        }
        for update in &node.updates {
            self.visit_node(update);
        }
        if let Some(ref whr) = node.where_clause {
            self.visit_node(whr);
        }
        if let Some(ref ret) = node.returning {
            for r in ret {
                self.visit_node(r);
            }
        }
    }

    /// Visits a DeleteQueryNode.
    fn visit_delete_query(&mut self, node: &DeleteQueryNode) {
        if let Some(ref with) = node.with {
            self.visit_node(with);
        }
        if let Some(ref from) = node.from {
            self.visit_node(from);
        }
        for join in &node.joins {
            self.visit_node(join);
        }
        if let Some(ref whr) = node.where_clause {
            self.visit_node(whr);
        }
        if let Some(ref ret) = node.returning {
            for r in ret {
                self.visit_node(r);
            }
        }
    }

    /// Visits a ColumnUpdateNode.
    fn visit_column_update(&mut self, node: &ColumnUpdateNode) {
        self.visit_node(&node.column);
        self.visit_node(&node.value);
    }

    /// Visits a ValuesNode.
    fn visit_values(&mut self, node: &ValuesNode) {
        for row in &node.rows {
            self.visit_node(row);
        }
    }

    /// Visits an OnConflictNode.
    fn visit_on_conflict(&mut self, node: &OnConflictNode) {
        if let Some(ref cols) = node.columns {
            for col in cols {
                self.visit_node(col);
            }
        }
        if let Some(ref updates) = node.updates {
            for u in updates {
                self.visit_node(u);
            }
        }
        if let Some(ref whr) = node.where_clause {
            self.visit_node(whr);
        }
    }

    /// Visits a CreateTableNode.
    fn visit_create_table(&mut self, _node: &CreateTableNode) {}

    /// Visits an AlterTableNode.
    fn visit_alter_table(&mut self, _node: &AlterTableNode) {}

    /// Visits a DropTableNode.
    fn visit_drop_table(&mut self, _node: &DropTableNode) {}

    /// Visits a CreateIndexNode.
    fn visit_create_index(&mut self, _node: &CreateIndexNode) {}

    /// Visits a DropIndexNode.
    fn visit_drop_index(&mut self, _node: &DropIndexNode) {}

    /// Visits a CreateViewNode.
    fn visit_create_view(&mut self, _node: &CreateViewNode) {}

    /// Visits a DropViewNode.
    fn visit_drop_view(&mut self, _node: &DropViewNode) {}

    /// Visits a CreateTypeNode.
    fn visit_create_type(&mut self, _node: &CreateTypeNode) {}

    /// Visits a DropTypeNode.
    fn visit_drop_type(&mut self, _node: &DropTypeNode) {}
}

/// Visitor that automatically maintains a parent node stack during traversal.
// @spec REQ-AST-006
pub struct StackTrackingVisitor<V> {
    visitor: V,
    node_stack: Vec<*const OperationNode>,
}

impl<V: OperationNodeVisitor> StackTrackingVisitor<V> {
    /// Creates a new stack tracking visitor wrapping an inner visitor.
    pub fn new(visitor: V) -> Self {
        Self {
            visitor,
            node_stack: Vec::new(),
        }
    }

    /// Visits an AST node while tracking the parent stack.
    pub fn walk(&mut self, node: &OperationNode) {
        self.node_stack.push(node as *const OperationNode);
        self.visitor.visit_node(node);
        self.node_stack.pop();
    }

    /// Returns a reference to the parent node, if one exists.
    pub fn parent_node(&self) -> Option<&OperationNode> {
        if self.node_stack.len() >= 2 {
            let ptr = self.node_stack[self.node_stack.len() - 2];
            // Safety: The pointer was pushed from an active reference on the call stack
            // and is guaranteed valid during walk.
            unsafe { ptr.as_ref() }
        } else {
            None
        }
    }

    /// Returns the current visitor depth.
    pub fn depth(&self) -> usize {
        self.node_stack.len()
    }

    /// Returns the inner visitor.
    pub fn into_inner(self) -> V {
        self.visitor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct IdentifierCollector {
        names: Vec<String>,
    }

    impl OperationNodeVisitor for IdentifierCollector {
        fn visit_identifier(&mut self, node: &IdentifierNode) {
            self.names.push(node.name.to_string());
        }
    }

    #[test]
    fn test_visitor() {
        let mut select = SelectQueryNode::new();
        select
            .selections
            .push(Arc::new(OperationNode::Selection(SelectionNode::new(
                Arc::new(OperationNode::Identifier(IdentifierNode::new("id"))),
            ))));
        select
            .selections
            .push(Arc::new(OperationNode::Selection(SelectionNode::new(
                Arc::new(OperationNode::Identifier(IdentifierNode::new("name"))),
            ))));
        select.from = Some(Arc::new(OperationNode::From(FromNode::new(Arc::new(
            OperationNode::Identifier(IdentifierNode::new("users")),
        )))));

        let root = OperationNode::SelectQuery(select);
        let mut collector = IdentifierCollector { names: Vec::new() };
        collector.visit_node(&root);

        assert_eq!(collector.names, vec!["id", "name", "users"]);
    }
}
