//! Default ANSI SQL query compiler.

use crate::operation_node::*;
use crate::query_compiler::compiled_query::CompiledQuery;
use crate::util::QueryId;

/// Placeholder style used for query parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterStyle {
    /// ANSI standard / SQLite / MySQL `?` placeholder.
    QuestionMark,
    /// PostgreSQL sequential `$1, $2, ...` placeholder.
    Dollar,
    /// MSSQL `@p1, @p2, ...` placeholder.
    AtP,
}

/// Identifier quote style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierQuote {
    /// Bare unquoted identifiers.
    None,
    /// ANSI / PostgreSQL / SQLite double quotes (`"name"`).
    DoubleQuote,
    /// MySQL backticks (`` `name` ``).
    Backtick,
    /// MSSQL brackets (`[name]`).
    Bracket,
}

/// Base query compiler generating parameterized SQL from an `OperationNode` AST.
// @spec REQ-AST-008
#[derive(Debug, Clone)]
pub struct DefaultQueryCompiler {
    parameter_style: ParameterStyle,
    identifier_quote: IdentifierQuote,
}

impl Default for DefaultQueryCompiler {
    fn default() -> Self {
        Self {
            parameter_style: ParameterStyle::QuestionMark,
            identifier_quote: IdentifierQuote::DoubleQuote,
        }
    }
}

impl DefaultQueryCompiler {
    /// Creates a new compiler configured for standard ANSI SQL.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a custom compiler with specified placeholder and quoting styles.
    pub fn with_styles(parameter_style: ParameterStyle, identifier_quote: IdentifierQuote) -> Self {
        Self {
            parameter_style,
            identifier_quote,
        }
    }

    /// Compiles an AST node into a `CompiledQuery`.
    pub fn compile(&self, node: &OperationNode) -> CompiledQuery {
        let mut visitor = CompilerVisitor {
            compiler: self,
            sql: String::with_capacity(128),
            parameters: Vec::new(),
            query_id: QueryId::generate(),
        };
        visitor.visit_node(node);
        CompiledQuery::new(visitor.sql, visitor.parameters, visitor.query_id)
    }

    /// Formats an identifier according to the configured quoting style.
    pub fn format_identifier(&self, name: &str) -> String {
        match self.identifier_quote {
            IdentifierQuote::None => name.to_string(),
            IdentifierQuote::DoubleQuote => format!("\"{}\"", name.replace('"', "\"\"")),
            IdentifierQuote::Backtick => format!("`{}`", name.replace('`', "``")),
            IdentifierQuote::Bracket => format!("[{}]", name.replace(']', "]]")),
        }
    }

    /// Returns the placeholder string for a given parameter index (1-based).
    pub fn format_parameter_placeholder(&self, index: usize) -> String {
        match self.parameter_style {
            ParameterStyle::QuestionMark => "?".to_string(),
            ParameterStyle::Dollar => format!("${}", index),
            ParameterStyle::AtP => format!("@p{}", index),
        }
    }
}

struct CompilerVisitor<'a> {
    compiler: &'a DefaultQueryCompiler,
    sql: String,
    parameters: Vec<Value>,
    query_id: QueryId,
}

impl<'a> CompilerVisitor<'a> {
    fn append(&mut self, str: &str) {
        self.sql.push_str(str);
    }

    fn add_parameter(&mut self, val: Value) {
        self.parameters.push(val);
        let placeholder = self.compiler.format_parameter_placeholder(self.parameters.len());
        self.append(&placeholder);
    }
}

impl<'a> OperationNodeVisitor for CompilerVisitor<'a> {
    fn visit_identifier(&mut self, node: &IdentifierNode) {
        let formatted = self.compiler.format_identifier(node.name.as_str());
        self.append(&formatted);
    }

    fn visit_table(&mut self, node: &TableNode) {
        if let Some(ref schema) = node.schema {
            self.visit_node(schema);
            self.append(".");
        }
        self.visit_node(&node.table);
    }

    fn visit_column(&mut self, node: &ColumnNode) {
        self.visit_node(&node.column);
    }

    fn visit_reference(&mut self, node: &ReferenceNode) {
        if let Some(ref table) = node.table {
            self.visit_node(table);
            self.append(".");
        }
        self.visit_node(&node.column);
    }

    fn visit_value(&mut self, node: &ValueNode) {
        self.add_parameter(node.value.clone());
    }

    fn visit_primitive_value_list(&mut self, node: &PrimitiveValueListNode) {
        self.append("(");
        for (i, val) in node.values.iter().enumerate() {
            if i > 0 {
                self.append(", ");
            }
            self.add_parameter(val.clone());
        }
        self.append(")");
    }

    fn visit_raw(&mut self, node: &RawNode) {
        let frag_len = node.sql_fragments.len();
        let param_len = node.parameters.len();
        let max_len = frag_len.max(param_len);

        for i in 0..max_len {
            if i < frag_len {
                self.append(node.sql_fragments[i].as_str());
            }
            if i < param_len {
                self.visit_node(&node.parameters[i]);
            }
        }
    }

    fn visit_from(&mut self, node: &FromNode) {
        for (i, from) in node.froms.iter().enumerate() {
            if i > 0 {
                self.append(", ");
            }
            self.visit_node(from);
        }
    }

    fn visit_join(&mut self, node: &JoinNode) {
        self.append(" ");
        self.append(node.join_type.as_sql());
        self.append(" ");
        self.visit_node(&node.table);
        if let Some(ref on) = node.on {
            self.append(" ON ");
            self.visit_node(on);
        }
    }

    fn visit_selection(&mut self, node: &SelectionNode) {
        self.visit_node(&node.expr);
        if let Some(ref alias) = node.alias {
            self.append(" AS ");
            self.visit_node(alias);
        }
    }

    fn visit_where(&mut self, node: &WhereNode) {
        self.append(" WHERE ");
        self.visit_node(&node.where_expr);
    }

    fn visit_having(&mut self, node: &HavingNode) {
        self.append(" HAVING ");
        self.visit_node(&node.having_expr);
    }

    fn visit_group_by(&mut self, node: &GroupByNode) {
        self.append(" GROUP BY ");
        for (i, item) in node.items.iter().enumerate() {
            if i > 0 {
                self.append(", ");
            }
            self.visit_node(item);
        }
    }

    fn visit_order_by(&mut self, node: &OrderByNode) {
        self.append(" ORDER BY ");
        for (i, item) in node.items.iter().enumerate() {
            if i > 0 {
                self.append(", ");
            }
            self.visit_node(item);
        }
    }

    fn visit_order_by_item(&mut self, node: &OrderByItemNode) {
        self.visit_node(&node.expr);
        if let Some(dir) = node.direction {
            match dir {
                OrderDirection::Asc => self.append(" ASC"),
                OrderDirection::Desc => self.append(" DESC"),
            }
        }
        if let Some(nulls) = node.nulls {
            match nulls {
                OrderNulls::First => self.append(" NULLS FIRST"),
                OrderNulls::Last => self.append(" NULLS LAST"),
            }
        }
    }

    fn visit_limit(&mut self, node: &LimitNode) {
        self.append(" LIMIT ");
        self.visit_node(&node.limit);
    }

    fn visit_offset(&mut self, node: &OffsetNode) {
        self.append(" OFFSET ");
        self.visit_node(&node.offset);
    }

    fn visit_with(&mut self, node: &WithNode) {
        self.append("WITH ");
        if node.recursive {
            self.append("RECURSIVE ");
        }
        for (i, expr) in node.expressions.iter().enumerate() {
            if i > 0 {
                self.append(", ");
            }
            self.visit_node(expr);
        }
        self.append(" ");
    }

    fn visit_common_table_expression(&mut self, node: &CommonTableExpressionNode) {
        self.visit_node(&node.name);
        self.append(" AS (");
        self.visit_node(&node.expression);
        self.append(")");
    }

    fn visit_binary(&mut self, node: &BinaryOperationNode) {
        self.visit_node(&node.left);
        self.append(" ");
        self.append(node.operator.as_str());
        self.append(" ");
        self.visit_node(&node.right);
    }

    fn visit_unary(&mut self, node: &UnaryOperationNode) {
        self.append(node.operator.as_str());
        self.append(" ");
        self.visit_node(&node.operand);
    }

    fn visit_case(&mut self, node: &CaseNode) {
        self.append("CASE");
        if let Some(ref val) = node.value {
            self.append(" ");
            self.visit_node(val);
        }
        for when in &node.when_clauses {
            self.append(" ");
            self.visit_node(when);
        }
        if let Some(ref el) = node.else_clause {
            self.append(" ELSE ");
            self.visit_node(el);
        }
        self.append(" END");
    }

    fn visit_when(&mut self, node: &WhenNode) {
        self.append("WHEN ");
        self.visit_node(&node.condition);
        self.append(" THEN ");
        self.visit_node(&node.result);
    }

    fn visit_cast(&mut self, node: &CastNode) {
        self.append("CAST(");
        self.visit_node(&node.expr);
        self.append(" AS ");
        self.append(node.data_type.as_str());
        self.append(")");
    }

    fn visit_alias(&mut self, node: &AliasNode) {
        self.visit_node(&node.node);
        self.append(" AS ");
        self.visit_node(&node.alias);
    }

    fn visit_parens(&mut self, node: &ParensNode) {
        self.append("(");
        self.visit_node(&node.node);
        self.append(")");
    }

    fn visit_and(&mut self, node: &AndNode) {
        self.visit_node(&node.left);
        self.append(" AND ");
        self.visit_node(&node.right);
    }

    fn visit_or(&mut self, node: &OrNode) {
        self.visit_node(&node.left);
        self.append(" OR ");
        self.visit_node(&node.right);
    }

    fn visit_select_query(&mut self, node: &SelectQueryNode) {
        if let Some(ref with) = node.with {
            self.visit_node(with);
        }
        self.append("SELECT ");
        if let Some(ref distinct) = node.distinct_on {
            self.append("DISTINCT ON (");
            for (i, d) in distinct.iter().enumerate() {
                if i > 0 {
                    self.append(", ");
                }
                self.visit_node(d);
            }
            self.append(") ");
        }

        if node.selections.is_empty() {
            self.append("*");
        } else {
            for (i, sel) in node.selections.iter().enumerate() {
                if i > 0 {
                    self.append(", ");
                }
                self.visit_node(sel);
            }
        }

        if let Some(ref from) = node.from {
            self.append(" FROM ");
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
    }

    fn visit_insert_query(&mut self, node: &InsertQueryNode) {
        if let Some(ref with) = node.with {
            self.visit_node(with);
        }
        self.append("INSERT INTO ");
        if let Some(ref into) = node.into {
            self.visit_node(into);
        }

        if let Some(ref cols) = node.columns {
            self.append(" (");
            for (i, col) in cols.iter().enumerate() {
                if i > 0 {
                    self.append(", ");
                }
                self.visit_node(col);
            }
            self.append(")");
        }

        if let Some(ref vals) = node.values {
            match vals.as_ref() {
                OperationNode::SelectQuery(_) => {
                    self.append(" ");
                    self.visit_node(vals);
                }
                _ => {
                    self.append(" VALUES ");
                    self.visit_node(vals);
                }
            }
        }

        if let Some(ref oc) = node.on_conflict {
            self.visit_node(oc);
        }

        if let Some(ref ret) = node.returning {
            self.append(" RETURNING ");
            for (i, r) in ret.iter().enumerate() {
                if i > 0 {
                    self.append(", ");
                }
                self.visit_node(r);
            }
        }
    }

    fn visit_update_query(&mut self, node: &UpdateQueryNode) {
        if let Some(ref with) = node.with {
            self.visit_node(with);
        }
        self.append("UPDATE ");
        if let Some(ref table) = node.table {
            self.visit_node(table);
        }

        if !node.updates.is_empty() {
            self.append(" SET ");
            for (i, update) in node.updates.iter().enumerate() {
                if i > 0 {
                    self.append(", ");
                }
                self.visit_node(update);
            }
        }

        if let Some(ref from) = node.from {
            self.append(" FROM ");
            self.visit_node(from);
        }

        for join in &node.joins {
            self.visit_node(join);
        }

        if let Some(ref whr) = node.where_clause {
            self.visit_node(whr);
        }

        if let Some(ref ret) = node.returning {
            self.append(" RETURNING ");
            for (i, r) in ret.iter().enumerate() {
                if i > 0 {
                    self.append(", ");
                }
                self.visit_node(r);
            }
        }
    }

    fn visit_delete_query(&mut self, node: &DeleteQueryNode) {
        if let Some(ref with) = node.with {
            self.visit_node(with);
        }
        self.append("DELETE FROM ");
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
            self.append(" RETURNING ");
            for (i, r) in ret.iter().enumerate() {
                if i > 0 {
                    self.append(", ");
                }
                self.visit_node(r);
            }
        }
    }

    fn visit_column_update(&mut self, node: &ColumnUpdateNode) {
        self.visit_node(&node.column);
        self.append(" = ");
        self.visit_node(&node.value);
    }

    fn visit_values(&mut self, node: &ValuesNode) {
        for (i, row) in node.rows.iter().enumerate() {
            if i > 0 {
                self.append(", ");
            }
            self.visit_node(row);
        }
    }

    fn visit_on_conflict(&mut self, node: &OnConflictNode) {
        self.append(" ON CONFLICT");
        if let Some(ref cols) = node.columns {
            self.append(" (");
            for (i, col) in cols.iter().enumerate() {
                if i > 0 {
                    self.append(", ");
                }
                self.visit_node(col);
            }
            self.append(")");
        }

        if node.do_nothing {
            self.append(" DO NOTHING");
        } else if let Some(ref updates) = node.updates {
            self.append(" DO UPDATE SET ");
            for (i, u) in updates.iter().enumerate() {
                if i > 0 {
                    self.append(", ");
                }
                self.visit_node(u);
            }
            if let Some(ref whr) = node.where_clause {
                match whr.as_ref() {
                    OperationNode::Where(_) => self.visit_node(whr),
                    _ => {
                        self.append(" WHERE ");
                        self.visit_node(whr);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_default_compiler() {
        let compiler = DefaultQueryCompiler::new();

        // Test SELECT "id", "name" FROM "users" WHERE "age" > ?
        let mut select = SelectQueryNode::new();
        select.selections.push(Arc::new(OperationNode::Selection(SelectionNode::new(
            Arc::new(OperationNode::Identifier(IdentifierNode::new("id"))),
        ))));
        select.selections.push(Arc::new(OperationNode::Selection(SelectionNode::new(
            Arc::new(OperationNode::Identifier(IdentifierNode::new("name"))),
        ))));
        select.from = Some(Arc::new(OperationNode::From(FromNode::new(Arc::new(
            OperationNode::Identifier(IdentifierNode::new("users")),
        )))));
        select.where_clause = Some(Arc::new(OperationNode::Where(WhereNode::new(Arc::new(
            OperationNode::Binary(BinaryOperationNode::new(
                Arc::new(OperationNode::Identifier(IdentifierNode::new("age"))),
                ">",
                Arc::new(OperationNode::Value(ValueNode::new(18))),
            )),
        )))));

        let query = compiler.compile(&OperationNode::SelectQuery(select));
        assert_eq!(
            query.sql,
            "SELECT \"id\", \"name\" FROM \"users\" WHERE \"age\" > ?"
        );
        assert_eq!(query.parameters, vec![Value::I64(18)]);
        assert!(query.query_id.raw() > 0);
    }
}
