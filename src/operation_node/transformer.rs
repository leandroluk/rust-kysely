//! AST transformation trait enabling query rewriting plugins and optimizers.

use std::sync::Arc;
use smallvec::SmallVec;

use crate::operation_node::*;

/// Trait providing immutable AST transformation and node rewriting.
// @spec REQ-AST-007
pub trait OperationNodeTransformer {
    /// Transforms an AST node.
    fn transform_node(&mut self, node: Arc<OperationNode>) -> Arc<OperationNode> {
        match node.as_ref() {
            OperationNode::Identifier(n) => self.transform_identifier(n, &node),
            OperationNode::Table(n) => self.transform_table(n, &node),
            OperationNode::Column(n) => self.transform_column(n, &node),
            OperationNode::Reference(n) => self.transform_reference(n, &node),
            OperationNode::Value(n) => self.transform_value(n, &node),
            OperationNode::PrimitiveValueList(n) => self.transform_primitive_value_list(n, &node),
            OperationNode::Raw(n) => self.transform_raw(n, &node),
            OperationNode::From(n) => self.transform_from(n, &node),
            OperationNode::Join(n) => self.transform_join(n, &node),
            OperationNode::Selection(n) => self.transform_selection(n, &node),
            OperationNode::Where(n) => self.transform_where(n, &node),
            OperationNode::Having(n) => self.transform_having(n, &node),
            OperationNode::GroupBy(n) => self.transform_group_by(n, &node),
            OperationNode::OrderBy(n) => self.transform_order_by(n, &node),
            OperationNode::OrderByItem(n) => self.transform_order_by_item(n, &node),
            OperationNode::Limit(n) => self.transform_limit(n, &node),
            OperationNode::Offset(n) => self.transform_offset(n, &node),
            OperationNode::With(n) => self.transform_with(n, &node),
            OperationNode::CommonTableExpression(n) => {
                self.transform_common_table_expression(n, &node)
            }
            OperationNode::Binary(n) => self.transform_binary(n, &node),
            OperationNode::Unary(n) => self.transform_unary(n, &node),
            OperationNode::Case(n) => self.transform_case(n, &node),
            OperationNode::When(n) => self.transform_when(n, &node),
            OperationNode::Cast(n) => self.transform_cast(n, &node),
            OperationNode::Alias(n) => self.transform_alias(n, &node),
            OperationNode::Parens(n) => self.transform_parens(n, &node),
            OperationNode::And(n) => self.transform_and(n, &node),
            OperationNode::Or(n) => self.transform_or(n, &node),
            OperationNode::Function(n) => self.transform_function(n, &node),
            OperationNode::SelectQuery(n) => self.transform_select_query(n, &node),
            OperationNode::InsertQuery(n) => self.transform_insert_query(n, &node),
            OperationNode::UpdateQuery(n) => self.transform_update_query(n, &node),
            OperationNode::DeleteQuery(n) => self.transform_delete_query(n, &node),
            OperationNode::ColumnUpdate(n) => self.transform_column_update(n, &node),
            OperationNode::Values(n) => self.transform_values(n, &node),
            OperationNode::OnConflict(n) => self.transform_on_conflict(n, &node),
        }
    }

    /// Transforms an IdentifierNode (leaf).
    fn transform_identifier(
        &mut self,
        _node: &IdentifierNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        Arc::clone(orig)
    }

    /// Transforms a TableNode.
    fn transform_table(
        &mut self,
        node: &TableNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_schema = node
            .schema
            .as_ref()
            .map(|s| self.transform_node(Arc::clone(s)));
        let new_table = self.transform_node(Arc::clone(&node.table));

        let schema_changed = match (&node.schema, &new_schema) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        };

        if !schema_changed && Arc::ptr_eq(&node.table, &new_table) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Table(TableNode {
                schema: new_schema,
                table: new_table,
            }))
        }
    }

    /// Transforms a ColumnNode.
    fn transform_column(
        &mut self,
        node: &ColumnNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_column = self.transform_node(Arc::clone(&node.column));
        if Arc::ptr_eq(&node.column, &new_column) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Column(ColumnNode { column: new_column }))
        }
    }

    /// Transforms a ReferenceNode.
    fn transform_reference(
        &mut self,
        node: &ReferenceNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_table = node
            .table
            .as_ref()
            .map(|t| self.transform_node(Arc::clone(t)));
        let new_column = self.transform_node(Arc::clone(&node.column));

        let table_changed = match (&node.table, &new_table) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        };

        if !table_changed && Arc::ptr_eq(&node.column, &new_column) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Reference(ReferenceNode {
                table: new_table,
                column: new_column,
            }))
        }
    }

    /// Transforms a ValueNode (leaf).
    fn transform_value(
        &mut self,
        _node: &ValueNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        Arc::clone(orig)
    }

    /// Transforms a PrimitiveValueListNode (leaf).
    fn transform_primitive_value_list(
        &mut self,
        _node: &PrimitiveValueListNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        Arc::clone(orig)
    }

    /// Transforms a RawNode.
    fn transform_raw(&mut self, node: &RawNode, orig: &Arc<OperationNode>) -> Arc<OperationNode> {
        let mut changed = false;
        let new_params: SmallVec<[Arc<OperationNode>; 2]> = node
            .parameters
            .iter()
            .map(|p| {
                let transformed = self.transform_node(Arc::clone(p));
                if !Arc::ptr_eq(p, &transformed) {
                    changed = true;
                }
                transformed
            })
            .collect();

        if !changed {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Raw(RawNode {
                sql_fragments: node.sql_fragments.clone(),
                parameters: new_params,
            }))
        }
    }

    /// Transforms a FromNode.
    fn transform_from(&mut self, node: &FromNode, orig: &Arc<OperationNode>) -> Arc<OperationNode> {
        let mut changed = false;
        let new_froms: SmallVec<[Arc<OperationNode>; 2]> = node
            .froms
            .iter()
            .map(|f| {
                let transformed = self.transform_node(Arc::clone(f));
                if !Arc::ptr_eq(f, &transformed) {
                    changed = true;
                }
                transformed
            })
            .collect();

        if !changed {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::From(FromNode { froms: new_froms }))
        }
    }

    /// Transforms a JoinNode.
    fn transform_join(&mut self, node: &JoinNode, orig: &Arc<OperationNode>) -> Arc<OperationNode> {
        let new_table = self.transform_node(Arc::clone(&node.table));
        let new_on = node.on.as_ref().map(|o| self.transform_node(Arc::clone(o)));

        let on_changed = match (&node.on, &new_on) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        };

        if !on_changed && Arc::ptr_eq(&node.table, &new_table) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Join(JoinNode {
                join_type: node.join_type,
                table: new_table,
                on: new_on,
            }))
        }
    }

    /// Transforms a SelectionNode.
    fn transform_selection(
        &mut self,
        node: &SelectionNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_expr = self.transform_node(Arc::clone(&node.expr));
        let new_alias = node
            .alias
            .as_ref()
            .map(|a| self.transform_node(Arc::clone(a)));

        let alias_changed = match (&node.alias, &new_alias) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        };

        if !alias_changed && Arc::ptr_eq(&node.expr, &new_expr) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Selection(SelectionNode {
                expr: new_expr,
                alias: new_alias,
            }))
        }
    }

    /// Transforms a WhereNode.
    fn transform_where(
        &mut self,
        node: &WhereNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_expr = self.transform_node(Arc::clone(&node.where_expr));
        if Arc::ptr_eq(&node.where_expr, &new_expr) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Where(WhereNode {
                where_expr: new_expr,
            }))
        }
    }

    /// Transforms a HavingNode.
    fn transform_having(
        &mut self,
        node: &HavingNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_expr = self.transform_node(Arc::clone(&node.having_expr));
        if Arc::ptr_eq(&node.having_expr, &new_expr) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Having(HavingNode {
                having_expr: new_expr,
            }))
        }
    }

    /// Transforms a GroupByNode.
    fn transform_group_by(
        &mut self,
        node: &GroupByNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let mut changed = false;
        let new_items: SmallVec<[Arc<OperationNode>; 4]> = node
            .items
            .iter()
            .map(|i| {
                let transformed = self.transform_node(Arc::clone(i));
                if !Arc::ptr_eq(i, &transformed) {
                    changed = true;
                }
                transformed
            })
            .collect();

        if !changed {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::GroupBy(GroupByNode { items: new_items }))
        }
    }

    /// Transforms an OrderByNode.
    fn transform_order_by(
        &mut self,
        node: &OrderByNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let mut changed = false;
        let new_items: SmallVec<[Arc<OperationNode>; 4]> = node
            .items
            .iter()
            .map(|i| {
                let transformed = self.transform_node(Arc::clone(i));
                if !Arc::ptr_eq(i, &transformed) {
                    changed = true;
                }
                transformed
            })
            .collect();

        if !changed {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::OrderBy(OrderByNode { items: new_items }))
        }
    }

    /// Transforms an OrderByItemNode.
    fn transform_order_by_item(
        &mut self,
        node: &OrderByItemNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_expr = self.transform_node(Arc::clone(&node.expr));
        if Arc::ptr_eq(&node.expr, &new_expr) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::OrderByItem(OrderByItemNode {
                expr: new_expr,
                direction: node.direction,
                nulls: node.nulls,
            }))
        }
    }

    /// Transforms a LimitNode.
    fn transform_limit(
        &mut self,
        node: &LimitNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_limit = self.transform_node(Arc::clone(&node.limit));
        if Arc::ptr_eq(&node.limit, &new_limit) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Limit(LimitNode { limit: new_limit }))
        }
    }

    /// Transforms an OffsetNode.
    fn transform_offset(
        &mut self,
        node: &OffsetNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_offset = self.transform_node(Arc::clone(&node.offset));
        if Arc::ptr_eq(&node.offset, &new_offset) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Offset(OffsetNode { offset: new_offset }))
        }
    }

    /// Transforms a WithNode.
    fn transform_with(
        &mut self,
        node: &WithNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let mut changed = false;
        let new_exprs: SmallVec<[Arc<OperationNode>; 2]> = node
            .expressions
            .iter()
            .map(|e| {
                let transformed = self.transform_node(Arc::clone(e));
                if !Arc::ptr_eq(e, &transformed) {
                    changed = true;
                }
                transformed
            })
            .collect();

        if !changed {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::With(WithNode {
                recursive: node.recursive,
                expressions: new_exprs,
            }))
        }
    }

    /// Transforms a CommonTableExpressionNode.
    fn transform_common_table_expression(
        &mut self,
        node: &CommonTableExpressionNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_name = self.transform_node(Arc::clone(&node.name));
        let new_expr = self.transform_node(Arc::clone(&node.expression));

        if Arc::ptr_eq(&node.name, &new_name) && Arc::ptr_eq(&node.expression, &new_expr) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::CommonTableExpression(
                CommonTableExpressionNode {
                    name: new_name,
                    expression: new_expr,
                },
            ))
        }
    }

    /// Transforms a BinaryOperationNode.
    fn transform_binary(
        &mut self,
        node: &BinaryOperationNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_left = self.transform_node(Arc::clone(&node.left));
        let new_right = self.transform_node(Arc::clone(&node.right));

        if Arc::ptr_eq(&node.left, &new_left) && Arc::ptr_eq(&node.right, &new_right) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Binary(BinaryOperationNode {
                left: new_left,
                operator: node.operator.clone(),
                right: new_right,
            }))
        }
    }

    /// Transforms a UnaryOperationNode.
    fn transform_unary(
        &mut self,
        node: &UnaryOperationNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_operand = self.transform_node(Arc::clone(&node.operand));
        if Arc::ptr_eq(&node.operand, &new_operand) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Unary(UnaryOperationNode {
                operator: node.operator.clone(),
                operand: new_operand,
            }))
        }
    }

    /// Transforms a CaseNode.
    fn transform_case(
        &mut self,
        node: &CaseNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_value = node
            .value
            .as_ref()
            .map(|v| self.transform_node(Arc::clone(v)));

        let mut changed = match (&node.value, &new_value) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        };

        let new_whens: SmallVec<[Arc<OperationNode>; 4]> = node
            .when_clauses
            .iter()
            .map(|w| {
                let transformed = self.transform_node(Arc::clone(w));
                if !Arc::ptr_eq(w, &transformed) {
                    changed = true;
                }
                transformed
            })
            .collect();

        let new_else = node
            .else_clause
            .as_ref()
            .map(|e| self.transform_node(Arc::clone(e)));
        if match (&node.else_clause, &new_else) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        } {
            changed = true;
        }

        if !changed {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Case(CaseNode {
                value: new_value,
                when_clauses: new_whens,
                else_clause: new_else,
            }))
        }
    }

    /// Transforms a WhenNode.
    fn transform_when(
        &mut self,
        node: &WhenNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_cond = self.transform_node(Arc::clone(&node.condition));
        let new_res = self.transform_node(Arc::clone(&node.result));

        if Arc::ptr_eq(&node.condition, &new_cond) && Arc::ptr_eq(&node.result, &new_res) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::When(WhenNode {
                condition: new_cond,
                result: new_res,
            }))
        }
    }

    /// Transforms a CastNode.
    fn transform_cast(
        &mut self,
        node: &CastNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_expr = self.transform_node(Arc::clone(&node.expr));
        if Arc::ptr_eq(&node.expr, &new_expr) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Cast(CastNode {
                expr: new_expr,
                data_type: node.data_type.clone(),
            }))
        }
    }

    /// Transforms an AliasNode.
    fn transform_alias(
        &mut self,
        node: &AliasNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_node = self.transform_node(Arc::clone(&node.node));
        let new_alias = self.transform_node(Arc::clone(&node.alias));

        if Arc::ptr_eq(&node.node, &new_node) && Arc::ptr_eq(&node.alias, &new_alias) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Alias(AliasNode {
                node: new_node,
                alias: new_alias,
            }))
        }
    }

    /// Transforms a ParensNode.
    fn transform_parens(
        &mut self,
        node: &ParensNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_node = self.transform_node(Arc::clone(&node.node));
        if Arc::ptr_eq(&node.node, &new_node) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Parens(ParensNode { node: new_node }))
        }
    }

    /// Transforms an AndNode.
    fn transform_and(
        &mut self,
        node: &AndNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_left = self.transform_node(Arc::clone(&node.left));
        let new_right = self.transform_node(Arc::clone(&node.right));

        if Arc::ptr_eq(&node.left, &new_left) && Arc::ptr_eq(&node.right, &new_right) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::And(AndNode {
                left: new_left,
                right: new_right,
            }))
        }
    }

    /// Transforms an OrNode.
    fn transform_or(&mut self, node: &OrNode, orig: &Arc<OperationNode>) -> Arc<OperationNode> {
        let new_left = self.transform_node(Arc::clone(&node.left));
        let new_right = self.transform_node(Arc::clone(&node.right));

        if Arc::ptr_eq(&node.left, &new_left) && Arc::ptr_eq(&node.right, &new_right) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Or(OrNode {
                left: new_left,
                right: new_right,
            }))
        }
    }

    /// Transforms a FunctionNode.
    fn transform_function(
        &mut self,
        node: &FunctionNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let mut changed = false;
        let new_args: SmallVec<[Arc<OperationNode>; 2]> = node
            .arguments
            .iter()
            .map(|a| {
                let transformed = self.transform_node(Arc::clone(a));
                if !Arc::ptr_eq(a, &transformed) {
                    changed = true;
                }
                transformed
            })
            .collect();

        if !changed {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::Function(FunctionNode {
                name: node.name.clone(),
                arguments: new_args,
            }))
        }
    }

    /// Transforms a SelectQueryNode.
    fn transform_select_query(
        &mut self,
        node: &SelectQueryNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_from = node.from.as_ref().map(|f| self.transform_node(Arc::clone(f)));
        let mut changed = match (&node.from, &new_from) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        };

        let new_selections: SmallVec<[Arc<OperationNode>; 4]> = node
            .selections
            .iter()
            .map(|s| {
                let transformed = self.transform_node(Arc::clone(s));
                if !Arc::ptr_eq(s, &transformed) {
                    changed = true;
                }
                transformed
            })
            .collect();

        let new_where = node
            .where_clause
            .as_ref()
            .map(|w| self.transform_node(Arc::clone(w)));
        if match (&node.where_clause, &new_where) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        } {
            changed = true;
        }

        if !changed {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::SelectQuery(SelectQueryNode {
                with: node.with.clone(),
                selections: new_selections,
                distinct_on: node.distinct_on.clone(),
                from: new_from,
                joins: node.joins.clone(),
                where_clause: new_where,
                group_by: node.group_by.clone(),
                having: node.having.clone(),
                order_by: node.order_by.clone(),
                limit: node.limit.clone(),
                offset: node.offset.clone(),
            }))
        }
    }

    /// Transforms an InsertQueryNode.
    fn transform_insert_query(
        &mut self,
        _node: &InsertQueryNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        Arc::clone(orig)
    }

    /// Transforms an UpdateQueryNode.
    fn transform_update_query(
        &mut self,
        _node: &UpdateQueryNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        Arc::clone(orig)
    }

    /// Transforms a DeleteQueryNode.
    fn transform_delete_query(
        &mut self,
        _node: &DeleteQueryNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        Arc::clone(orig)
    }

    /// Transforms a ColumnUpdateNode.
    fn transform_column_update(
        &mut self,
        node: &ColumnUpdateNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        let new_col = self.transform_node(Arc::clone(&node.column));
        let new_val = self.transform_node(Arc::clone(&node.value));
        if Arc::ptr_eq(&node.column, &new_col) && Arc::ptr_eq(&node.value, &new_val) {
            Arc::clone(orig)
        } else {
            Arc::new(OperationNode::ColumnUpdate(ColumnUpdateNode {
                column: new_col,
                value: new_val,
            }))
        }
    }

    /// Transforms a ValuesNode.
    fn transform_values(
        &mut self,
        _node: &ValuesNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        Arc::clone(orig)
    }

    /// Transforms an OnConflictNode.
    fn transform_on_conflict(
        &mut self,
        _node: &OnConflictNode,
        orig: &Arc<OperationNode>,
    ) -> Arc<OperationNode> {
        Arc::clone(orig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TableRenamer {
        from: &'static str,
        to: &'static str,
    }

    impl OperationNodeTransformer for TableRenamer {
        fn transform_identifier(
            &mut self,
            node: &IdentifierNode,
            orig: &Arc<OperationNode>,
        ) -> Arc<OperationNode> {
            if node.name.as_str() == self.from {
                Arc::new(OperationNode::Identifier(IdentifierNode::new(self.to)))
            } else {
                Arc::clone(orig)
            }
        }
    }

    #[test]
    fn test_transformer() {
        let root = Arc::new(OperationNode::From(FromNode::new(Arc::new(
            OperationNode::Identifier(IdentifierNode::new("old_table")),
        ))));

        let mut renamer = TableRenamer {
            from: "old_table",
            to: "new_table",
        };

        let transformed = renamer.transform_node(root);
        if let OperationNode::From(ref f) = *transformed {
            if let OperationNode::Identifier(ref id) = *f.froms[0] {
                assert_eq!(id.name.as_str(), "new_table");
            } else {
                panic!("Expected Identifier node");
            }
        } else {
            panic!("Expected From node");
        }
    }
}
