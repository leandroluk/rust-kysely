//! Projections and selection list parser implementing IntoSelections.

use compact_str::CompactString;
use smallvec::SmallVec;
use std::sync::Arc;

use crate::expression::expression_builder::{parse_column_reference, ExpressionBuilder};
use crate::expression::{Expression, ExpressionWrapper};
use crate::operation_node::{OperationNode, SelectionNode};
use crate::raw_builder::RawBuilder;

/// Trait converting a single item into a selection AST node.
// @spec REQ-SEL-005
pub trait IntoSelectionItem {
    /// Converts this item into a selection `OperationNode`.
    fn into_selection_item(self) -> Arc<OperationNode>;
}

impl IntoSelectionItem for &str {
    fn into_selection_item(self) -> Arc<OperationNode> {
        let node = parse_column_reference(self);
        Arc::new(OperationNode::Selection(SelectionNode::new(node)))
    }
}

impl IntoSelectionItem for String {
    fn into_selection_item(self) -> Arc<OperationNode> {
        self.as_str().into_selection_item()
    }
}

impl IntoSelectionItem for CompactString {
    fn into_selection_item(self) -> Arc<OperationNode> {
        self.as_str().into_selection_item()
    }
}

impl<T> IntoSelectionItem for ExpressionWrapper<T> {
    fn into_selection_item(self) -> Arc<OperationNode> {
        let node = self.to_operation_node();
        match node.as_ref() {
            OperationNode::Selection(_) | OperationNode::Alias(_) => node,
            _ => Arc::new(OperationNode::Selection(SelectionNode::new(node))),
        }
    }
}

impl<T> IntoSelectionItem for RawBuilder<T> {
    fn into_selection_item(self) -> Arc<OperationNode> {
        let node = self.to_operation_node();
        Arc::new(OperationNode::Selection(SelectionNode::new(node)))
    }
}

impl IntoSelectionItem for Arc<OperationNode> {
    fn into_selection_item(self) -> Arc<OperationNode> {
        match self.as_ref() {
            OperationNode::Selection(_) | OperationNode::Alias(_) => self,
            _ => Arc::new(OperationNode::Selection(SelectionNode::new(self))),
        }
    }
}

/// Trait for converting single columns, homogeneous arrays, heterogeneous tuples,
/// or closure callbacks into an AST selection list.
// @spec REQ-SEL-005
pub trait IntoSelections {
    /// Converts the target into a collection of selection nodes.
    fn into_selections(self) -> SmallVec<[Arc<OperationNode>; 4]>;
}

// 1. Single items
impl IntoSelections for &str {
    fn into_selections(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        let mut v = SmallVec::new();
        v.push(self.into_selection_item());
        v
    }
}

impl IntoSelections for String {
    fn into_selections(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        self.as_str().into_selections()
    }
}

impl IntoSelections for CompactString {
    fn into_selections(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        self.as_str().into_selections()
    }
}

impl<T> IntoSelections for ExpressionWrapper<T> {
    fn into_selections(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        let mut v = SmallVec::new();
        v.push(self.into_selection_item());
        v
    }
}

impl<T> IntoSelections for RawBuilder<T> {
    fn into_selections(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        let mut v = SmallVec::new();
        v.push(self.into_selection_item());
        v
    }
}

impl IntoSelections for Arc<OperationNode> {
    fn into_selections(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        let mut v = SmallVec::new();
        v.push(self.into_selection_item());
        v
    }
}

// 2. Arrays
impl<T: IntoSelectionItem, const N: usize> IntoSelections for [T; N] {
    fn into_selections(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        self.into_iter()
            .map(|item| item.into_selection_item())
            .collect()
    }
}

impl<T: IntoSelectionItem> IntoSelections for Vec<T> {
    fn into_selections(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        self.into_iter()
            .map(|item| item.into_selection_item())
            .collect()
    }
}

// 3. Closures
impl<F, S> IntoSelections for F
where
    F: FnOnce(&ExpressionBuilder) -> S,
    S: IntoSelections,
{
    fn into_selections(self) -> SmallVec<[Arc<OperationNode>; 4]> {
        let eb = ExpressionBuilder::new();
        (self)(&eb).into_selections()
    }
}

// 4. Heterogeneous Tuples (up to 12 elements)
macro_rules! impl_into_selections_tuple {
    ($($idx:tt: $T:ident),+) => {
        impl<$($T: IntoSelectionItem),+> IntoSelections for ($($T,)+) {
            fn into_selections(self) -> SmallVec<[Arc<OperationNode>; 4]> {
                let mut v = SmallVec::new();
                $(
                    v.push(self.$idx.into_selection_item());
                )+
                v
            }
        }
    };
}

impl_into_selections_tuple!(0: A);
impl_into_selections_tuple!(0: A, 1: B);
impl_into_selections_tuple!(0: A, 1: B, 2: C);
impl_into_selections_tuple!(0: A, 1: B, 2: C, 3: D);
impl_into_selections_tuple!(0: A, 1: B, 2: C, 3: D, 4: E);
impl_into_selections_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
impl_into_selections_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
impl_into_selections_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);
impl_into_selections_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I);
impl_into_selections_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J);
impl_into_selections_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K);
impl_into_selections_tuple!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_selections() {
        let eb = ExpressionBuilder::new();

        // 1. Single string
        let s1 = "users.id".into_selections();
        assert_eq!(s1.len(), 1);

        // 2. Array of strings
        let s2 = ["id", "name", "email"].into_selections();
        assert_eq!(s2.len(), 3);

        // 3. Heterogeneous tuple (strings + functions)
        let count_fn = eb.fn_().count("o.id").as_("order_count");
        let s3 = ("u.id", "u.name", count_fn).into_selections();
        assert_eq!(s3.len(), 3);

        // 4. Closure callback
        let s4 =
            (|eb: &ExpressionBuilder| ("id", eb.fn_().count_all().as_("total"))).into_selections();
        assert_eq!(s4.len(), 2);
    }
}
