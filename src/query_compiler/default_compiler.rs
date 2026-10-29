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
        if name == "*" {
            return "*".to_string();
        }
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

    fn visit_table_constraint(&mut self, constraint: &TableConstraintNode) {
        match constraint {
            TableConstraintNode::PrimaryKey { name, columns } => {
                if let Some(n) = name {
                    self.append("CONSTRAINT ");
                    self.append(&self.compiler.format_identifier(n.as_str()));
                    self.append(" ");
                }
                self.append("PRIMARY KEY (");
                for (i, c) in columns.iter().enumerate() {
                    if i > 0 {
                        self.append(", ");
                    }
                    self.append(&self.compiler.format_identifier(c.as_str()));
                }
                self.append(")");
            }
            TableConstraintNode::ForeignKey {
                name,
                columns,
                target_table,
                target_columns,
                on_delete,
                on_update,
            } => {
                if let Some(n) = name {
                    self.append("CONSTRAINT ");
                    self.append(&self.compiler.format_identifier(n.as_str()));
                    self.append(" ");
                }
                self.append("FOREIGN KEY (");
                for (i, c) in columns.iter().enumerate() {
                    if i > 0 {
                        self.append(", ");
                    }
                    self.append(&self.compiler.format_identifier(c.as_str()));
                }
                self.append(") REFERENCES ");
                self.append(&self.compiler.format_identifier(target_table.as_str()));
                self.append(" (");
                for (i, c) in target_columns.iter().enumerate() {
                    if i > 0 {
                        self.append(", ");
                    }
                    self.append(&self.compiler.format_identifier(c.as_str()));
                }
                self.append(")");
                if let Some(on_del) = on_delete {
                    self.append(" ON DELETE ");
                    self.append(on_del.as_sql());
                }
                if let Some(on_upd) = on_update {
                    self.append(" ON UPDATE ");
                    self.append(on_upd.as_sql());
                }
            }
            TableConstraintNode::Unique { name, columns } => {
                if let Some(n) = name {
                    self.append("CONSTRAINT ");
                    self.append(&self.compiler.format_identifier(n.as_str()));
                    self.append(" ");
                }
                self.append("UNIQUE (");
                for (i, c) in columns.iter().enumerate() {
                    if i > 0 {
                        self.append(", ");
                    }
                    self.append(&self.compiler.format_identifier(c.as_str()));
                }
                self.append(")");
            }
            TableConstraintNode::Check { name, expr } => {
                if let Some(n) = name {
                    self.append("CONSTRAINT ");
                    self.append(&self.compiler.format_identifier(n.as_str()));
                    self.append(" ");
                }
                self.append("CHECK (");
                self.visit_node(expr);
                self.append(")");
            }
        }
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

    fn visit_set_operation(&mut self, node: &SetOperationNode) {
        self.append(" ");
        self.append(node.operator.as_sql());
        self.append(" ");
        self.visit_node(&node.expression);
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

    fn visit_function(&mut self, node: &FunctionNode) {
        self.append(node.name.as_str());
        self.append("(");
        for (i, arg) in node.arguments.iter().enumerate() {
            if i > 0 {
                self.append(", ");
            }
            self.visit_node(arg);
        }
        self.append(")");
    }

    fn visit_select_query(&mut self, node: &SelectQueryNode) {
        if let Some(ref with) = node.with {
            self.visit_node(with);
        }
        self.append("SELECT ");
        if let Some(ref distinct) = node.distinct_on {
            if distinct.is_empty() {
                self.append("DISTINCT ");
            } else {
                self.append("DISTINCT ON (");
                for (i, d) in distinct.iter().enumerate() {
                    if i > 0 {
                        self.append(", ");
                    }
                    self.visit_node(d);
                }
                self.append(") ");
            }
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

        for set_op in &node.set_operations {
            self.visit_node(set_op);
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

    fn visit_create_table(&mut self, node: &CreateTableNode) {
        self.append("CREATE ");
        if node.temporary {
            self.append("TEMPORARY ");
        }
        self.append("TABLE ");
        if node.if_not_exists {
            self.append("IF NOT EXISTS ");
        }
        self.visit_node(&node.name);
        self.append(" (");

        let mut first = true;
        for col in &node.columns {
            if !first {
                self.append(", ");
            }
            first = false;

            self.append(&self.compiler.format_identifier(col.name.as_str()));
            self.append(" ");
            self.append(col.data_type.as_str());

            if col.is_primary_key {
                self.append(" PRIMARY KEY");
            }
            if !col.is_nullable {
                self.append(" NOT NULL");
            }
            if col.is_unique {
                self.append(" UNIQUE");
            }
            if let Some(ref def) = col.default_to {
                self.append(" DEFAULT ");
                self.visit_node(def);
            }
            if let Some(ref chk) = col.check {
                self.append(" CHECK (");
                self.visit_node(chk);
                self.append(")");
            }
            if let Some(ref r) = col.references {
                self.append(" REFERENCES ");
                self.append(&self.compiler.format_identifier(r.target_table.as_str()));
                self.append(" (");
                self.append(&self.compiler.format_identifier(r.target_column.as_str()));
                self.append(")");
                if let Some(on_del) = r.on_delete {
                    self.append(" ON DELETE ");
                    self.append(on_del.as_sql());
                }
                if let Some(on_upd) = r.on_update {
                    self.append(" ON UPDATE ");
                    self.append(on_upd.as_sql());
                }
            }
        }

        for constraint in &node.constraints {
            if !first {
                self.append(", ");
            }
            first = false;

            self.visit_table_constraint(constraint);
        }

        self.append(")");
    }

    fn visit_alter_table(&mut self, node: &AlterTableNode) {
        self.append("ALTER TABLE ");
        self.visit_node(&node.table);
        self.append(" ");

        for (i, action) in node.actions.iter().enumerate() {
            if i > 0 {
                self.append(", ");
            }
            match action {
                AlterTableAction::AddColumn(col) => {
                    self.append("ADD COLUMN ");
                    self.append(&self.compiler.format_identifier(col.name.as_str()));
                    self.append(" ");
                    self.append(col.data_type.as_str());
                    if col.is_primary_key {
                        self.append(" PRIMARY KEY");
                    }
                    if !col.is_nullable {
                        self.append(" NOT NULL");
                    }
                    if col.is_unique {
                        self.append(" UNIQUE");
                    }
                    if let Some(ref def) = col.default_to {
                        self.append(" DEFAULT ");
                        self.visit_node(def);
                    }
                }
                AlterTableAction::DropColumn { name, if_exists } => {
                    self.append("DROP COLUMN ");
                    if *if_exists {
                        self.append("IF EXISTS ");
                    }
                    self.append(&self.compiler.format_identifier(name.as_str()));
                }
                AlterTableAction::AlterColumnType { name, data_type } => {
                    self.append("ALTER COLUMN ");
                    self.append(&self.compiler.format_identifier(name.as_str()));
                    self.append(" TYPE ");
                    self.append(data_type.as_str());
                }
                AlterTableAction::RenameColumn { old_name, new_name } => {
                    self.append("RENAME COLUMN ");
                    self.append(&self.compiler.format_identifier(old_name.as_str()));
                    self.append(" TO ");
                    self.append(&self.compiler.format_identifier(new_name.as_str()));
                }
                AlterTableAction::AddConstraint(c) => {
                    self.append("ADD ");
                    self.visit_table_constraint(c);
                }
                AlterTableAction::DropConstraint { name, if_exists } => {
                    self.append("DROP CONSTRAINT ");
                    if *if_exists {
                        self.append("IF EXISTS ");
                    }
                    self.append(&self.compiler.format_identifier(name.as_str()));
                }
            }
        }
    }

    fn visit_drop_table(&mut self, node: &DropTableNode) {
        self.append("DROP TABLE ");
        if node.if_exists {
            self.append("IF EXISTS ");
        }
        self.visit_node(&node.name);
        if node.cascade {
            self.append(" CASCADE");
        }
    }

    fn visit_create_index(&mut self, node: &CreateIndexNode) {
        self.append("CREATE ");
        if node.unique {
            self.append("UNIQUE ");
        }
        self.append("INDEX ");
        if node.if_not_exists {
            self.append("IF NOT EXISTS ");
        }
        self.append(&self.compiler.format_identifier(node.name.as_str()));
        self.append(" ON ");
        self.visit_node(&node.table);
        if let Some(ref method) = node.using_method {
            self.append(" USING ");
            self.append(method.as_str());
        }
        self.append(" (");
        for (i, col) in node.columns.iter().enumerate() {
            if i > 0 {
                self.append(", ");
            }
            self.append(&self.compiler.format_identifier(col.as_str()));
        }
        self.append(")");
        if let Some(ref whr) = node.where_clause {
            self.append(" WHERE ");
            self.visit_node(whr);
        }
    }

    fn visit_drop_index(&mut self, node: &DropIndexNode) {
        self.append("DROP INDEX ");
        if node.if_exists {
            self.append("IF EXISTS ");
        }
        self.append(&self.compiler.format_identifier(node.name.as_str()));
        if node.cascade {
            self.append(" CASCADE");
        }
    }

    fn visit_create_view(&mut self, node: &CreateViewNode) {
        self.append("CREATE ");
        if node.or_replace {
            self.append("OR REPLACE ");
        }
        if node.materialized {
            self.append("MATERIALIZED ");
        }
        self.append("VIEW ");
        if node.if_not_exists {
            self.append("IF NOT EXISTS ");
        }
        self.append(&self.compiler.format_identifier(node.name.as_str()));
        self.append(" AS ");
        self.visit_node(&node.query);
    }

    fn visit_drop_view(&mut self, node: &DropViewNode) {
        self.append("DROP ");
        if node.materialized {
            self.append("MATERIALIZED ");
        }
        self.append("VIEW ");
        if node.if_exists {
            self.append("IF EXISTS ");
        }
        self.append(&self.compiler.format_identifier(node.name.as_str()));
        if node.cascade {
            self.append(" CASCADE");
        }
    }

    fn visit_create_type(&mut self, node: &CreateTypeNode) {
        self.append("CREATE TYPE ");
        self.append(&self.compiler.format_identifier(node.name.as_str()));
        self.append(" AS ENUM (");
        for (i, val) in node.enum_values.iter().enumerate() {
            if i > 0 {
                self.append(", ");
            }
            self.append("'");
            self.append(&val.replace('\'', "''"));
            self.append("'");
        }
        self.append(")");
    }

    fn visit_drop_type(&mut self, node: &DropTypeNode) {
        self.append("DROP TYPE ");
        if node.if_exists {
            self.append("IF EXISTS ");
        }
        self.append(&self.compiler.format_identifier(node.name.as_str()));
        if node.cascade {
            self.append(" CASCADE");
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

    #[test]
    fn test_ddl_compilation() {
        let compiler = DefaultQueryCompiler::new();

        // 1. CREATE TABLE
        let mut create_table = CreateTableNode::new(Arc::new(OperationNode::Identifier(IdentifierNode::new("users"))));
        create_table.if_not_exists = true;

        let mut id_col = ColumnDefinitionNode::new("id", "serial");
        id_col.is_primary_key = true;
        create_table.columns.push(id_col);

        let mut email_col = ColumnDefinitionNode::new("email", "varchar(255)");
        email_col.is_nullable = false;
        email_col.is_unique = true;
        create_table.columns.push(email_col);

        let mut status_col = ColumnDefinitionNode::new("status", "varchar(50)");
        status_col.default_to = Some(Arc::new(OperationNode::Value(ValueNode::new("active"))));
        create_table.columns.push(status_col);

        create_table.constraints.push(TableConstraintNode::ForeignKey {
            name: Some("fk_user_org".into()),
            columns: smallvec::smallvec!["org_id".into()],
            target_table: "organizations".into(),
            target_columns: smallvec::smallvec!["id".into()],
            on_delete: Some(OnModifyForeignAction::Cascade),
            on_update: None,
        });

        let q1 = compiler.compile(&OperationNode::CreateTable(create_table));
        assert_eq!(
            q1.sql,
            "CREATE TABLE IF NOT EXISTS \"users\" (\"id\" serial PRIMARY KEY, \"email\" varchar(255) NOT NULL UNIQUE, \"status\" varchar(50) DEFAULT ?, CONSTRAINT \"fk_user_org\" FOREIGN KEY (\"org_id\") REFERENCES \"organizations\" (\"id\") ON DELETE CASCADE)"
        );

        // 2. ALTER TABLE
        let mut alter_table = AlterTableNode::new(Arc::new(OperationNode::Identifier(IdentifierNode::new("users"))));
        alter_table.actions.push(AlterTableAction::AddColumn(ColumnDefinitionNode::new("age", "int4")));
        alter_table.actions.push(AlterTableAction::DropColumn {
            name: "deprecated".into(),
            if_exists: true,
        });
        alter_table.actions.push(AlterTableAction::RenameColumn {
            old_name: "old".into(),
            new_name: "new".into(),
        });

        let q2 = compiler.compile(&OperationNode::AlterTable(alter_table));
        assert_eq!(
            q2.sql,
            "ALTER TABLE \"users\" ADD COLUMN \"age\" int4, DROP COLUMN IF EXISTS \"deprecated\", RENAME COLUMN \"old\" TO \"new\""
        );

        // 3. DROP TABLE
        let mut drop_table = DropTableNode::new(Arc::new(OperationNode::Identifier(IdentifierNode::new("users"))));
        drop_table.if_exists = true;
        drop_table.cascade = true;

        let q3 = compiler.compile(&OperationNode::DropTable(drop_table));
        assert_eq!(q3.sql, "DROP TABLE IF EXISTS \"users\" CASCADE");

        // 4. CREATE INDEX
        let mut create_idx = CreateIndexNode::new("users_email_idx", Arc::new(OperationNode::Identifier(IdentifierNode::new("users"))));
        create_idx.unique = true;
        create_idx.columns.push("email".into());
        create_idx.using_method = Some("btree".into());

        let q4 = compiler.compile(&OperationNode::CreateIndex(create_idx));
        assert_eq!(q4.sql, "CREATE UNIQUE INDEX \"users_email_idx\" ON \"users\" USING btree (\"email\")");

        // 5. CREATE VIEW
        let select_ast = Arc::new(OperationNode::SelectQuery(SelectQueryNode::new()));
        let mut create_view = CreateViewNode::new("active_users", select_ast);
        create_view.or_replace = true;
        create_view.materialized = true;

        let q5 = compiler.compile(&OperationNode::CreateView(create_view));
        assert_eq!(q5.sql, "CREATE OR REPLACE MATERIALIZED VIEW \"active_users\" AS SELECT *");

        // 6. CREATE TYPE
        let mut create_type = CreateTypeNode::new("user_status");
        create_type.enum_values.push("active".into());
        create_type.enum_values.push("pending".into());
        create_type.enum_values.push("deleted".into());

        let q6 = compiler.compile(&OperationNode::CreateType(create_type));
        assert_eq!(q6.sql, "CREATE TYPE \"user_status\" AS ENUM ('active', 'pending', 'deleted')");
    }
}
