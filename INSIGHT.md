# INSIGHT.md — Arquitetura, Portabilidade e Design Macro-Free do Kysely para Rust (`rust-kysely`)

Baseado na inspeção detalhada do código-fonte original do **Kysely** (`kysely-org/kysely`, clonado localmente para análise), este documento estabelece a fundação arquitetural, o inventário completo de recursos, o mapeamento idiomático e o plano de implementação do **`rust-kysely`**.

Seguindo a mesma filosofia aplicada com sucesso no **`rust-zod`**, o objetivo central é **preservar a elegância, a fluidez e a expressividade da API do Kysely**, eliminando totalmente a dependência de procedural macros e adotando as melhores práticas do ecossistema Rust.

---

## 1. O Manifesto: Por que o Kysely em Rust e Sem Macros?

### 1.1. O Cenário Atual no Ecossistema Rust
Hoje, desenvolvedores Rust que precisam interagir com bancos de dados relacionais enfrentam dilemas conhecidos:
- **`sqlx`**: É um driver e executor fantástico, mas **não é um query builder**. O macro `query!()` exige conexão viva com banco ou um arquivo JSON de cache offline em tempo de compilação, tornando a construção de queries dinâmicas (com filtros opcionais, joins condicionais ou paginação dinâmica) manual, frágil e sujeita a concatenação de strings.
- **`diesel`**: É extremamente poderoso, mas é um ORM pesado baseado em macros complexas (`table!`, `#[derive(Queryable)]`) e metaprogramação de tipos em nível extremo. Isso resulta em tempos de compilação lentos, mensagens de erro quase indecifráveis para iniciantes e inflexibilidade para queries ad-hoc.
- **`sea-query`**: Um excelente construtor dinâmico de queries, mas cuja ergonomia é orientada a enums e estruturas verbosas, exigindo muito boilerplate em comparação com a elegância de builders fluidos modernos.

### 1.2. O Diferencial do Kysely
No TypeScript, o **Kysely** conquistou a comunidade porque:
1. **Não é um ORM**: Não impõe entidades ativas, change-tracking ou relacionamentos mágicos em memória.
2. **É puramente orientado a dados e SQL**: Cada método do builder corresponde diretamente a uma cláusula ou operador SQL.
3. **AST Introspectável e Imutável**: Toda query é construída como uma árvore de nós (`OperationNode`) imutável, permitindo plugins, reescrita de queries e compilação para múltiplos dialetos.
4. **Ergonomia Incomparável**: Encadeamento fluente, subqueries naturais e suporte a expressões complexas via `eb` (ExpressionBuilder).

### 1.3. A Proposta do `rust-kysely`: Zero Macros!
No `rust-kysely`, **abolimos radicalmente o uso de macros** (`no syn`, `no quote`, `no proc-macro`, `no macro_rules!` herméticas):
- **Tempos de compilação ultrarrápidos**: Sem expansão pesada de macros, mantendo o workspace enxuto e o feedback do `cargo check` instantâneo.
- **Transparência total**: Erros de compilação apontam diretamente para a linha e tipo real do Rust, sem códigos gerados ocultos.
- **Autocompletion & IDE Inlay Hints perfeitos**: Toda a API é composta por structs, traits e métodos nativos com tipagem forte.
- **Controle estrito de footprint**: Configurado com `debug = 0`, `strip = "debuginfo"` e `incremental = false` para evitar o inchaço de gigabytes na pasta `target/` no Windows/MSVC.

---

## 2. Mapeamento de Idiomas: TypeScript vs. Rust Macro-Free

Como o TypeScript possui digitação estrutural (*structural typing*), tipos literais de strings (`'person.id'`) e objetos literais anônimos (`{ ... }`), precisamos mapear cada um desses conceitos para equivalentes idiomáticos e de alta performance no Rust.

### 2.1. Arrow Functions e Subqueries $\to$ Closure Builders (`|eb| ...`)
No TypeScript, o Kysely usa arrow functions para aninhar subqueries, construir expressões booleanas complexas e configurar transações:
```ts
// TypeScript Kysely
db.selectFrom('person')
  .where((eb) => eb.or([
    eb('age', '>', 18),
    eb('first_name', '=', 'Bob')
  ]))
```
No Rust, mapeamos isso de forma 100% natural através de **Closures de Ordem Superior** (`FnOnce(&ExpressionBuilder) -> Expression`):
```rust
// Rust (rust-kysely)
db.select_from("person")
  .where_expr(|eb| eb.or([
      eb.cmp("age", ">", 18),
      eb.cmp("first_name", "=", "Bob"),
  ]))
```

### 2.2. Tagged Template Literals $\to$ Raw SQL Fluent Builder (`sql(...)`)
No TypeScript, o Kysely utiliza o operador de template string `sql`...``:
```ts
// TypeScript Kysely
const query = sql<Person[]>`select * from person where id = ${id}`
```
No Rust (sem macros como `sql!("...")`), fornecemos uma interface fluente, segura contra injeção de SQL e rica em helpers:
```rust
// Rust (rust-kysely)
let query = sql("select * from person where id = $1")
    .bind(id);

// Suporte a fragmentos e identificadores seguros:
let fragment = sql("concat(")
    .append(sql::id("first_name"))
    .append(", ' ', ")
    .append(sql::id("last_name"))
    .append(")");
```
Funções auxiliares do namespace `sql::*`:
- `sql::raw(str)`: Insere SQL bruto verbatim (sem bind de parâmetro).
- `sql::lit(val)`: Insere um valor literal escapado.
- `sql::id(ident)`: Trata a string como identificador de coluna/tabela (com aspas sanitizadas conforme o dialeto).
- `sql::table(name)`: Trata como nome de tabela.
- `sql::join(items, separator)`: Junta uma lista de expressões com delimitador.

### 2.3. Objeto de Valores no INSERT/UPDATE $\to$ Zero `.into()` via Traits e Closures
No TypeScript:
```ts
// TypeScript Kysely
db.insertInto('person').values({ first_name: 'John', age: 30 })
```

#### Por que um array literal exigiria `.into()` e como evitamos isso?
No Rust, arrays literais `[a, b]` são tipos homogêneos `[T; N]`. Se tentássemos passar `[("first_name", "John"), ("age", 30)]`, o compilador rejeitaria porque o primeiro elemento é `(&str, &str)` e o segundo é `(&str, i32)`.

Para que o desenvolvedor **nunca precise escrever `.into()` em cada campo**, o `rust-kysely` adota três padrões ergonômicos e tipados via trait `IntoValue`:

1. **Encadeamento Fluente Direto (`.set(col, val)`) — Zero Boilerplate**:
   Cada chamada a `.set()` é monomorfizada independentemente para o tipo real do valor (`val: impl IntoValue`), sem nenhuma necessidade de `.into()`:
   ```rust
   db.insert_into("person")
     .set("first_name", "John") // &str implementa IntoValue
     .set("age", 30)            // i32 implementa IntoValue
     .set("active", true)       // bool implementa IntoValue
     .returning_all()
     .execute()
     .await?;
   ```

2. **Closure Builder (`.values(|v| ...)`) — Agrupamento e Batch Inserts**:
   Perfeito para manter a semântica de objeto de valores e para inserções em lote (*batch*):
   ```rust
   // Inserção de uma linha:
   db.insert_into("person")
     .values(|v| {
         v.set("first_name", "John")
          .set("age", 30)
          .set("email", "john@example.com")
     })

   // Inserção em lote (batch insert):
   db.insert_into("person")
     .values_batch([
         |v: &mut RowValues| v.set("name", "Alice").set("age", 25),
         |v: &mut RowValues| v.set("name", "Bob").set("age", 30),
     ])
   ```

3. **Mapeamento Direto de Structs via Serde**:
   Quando os dados já residem em structs de domínio, passam-se diretamente sem nenhuma conversão manual:
   ```rust
   #[derive(serde::Serialize)] // Standard serde, sem macro customizada do Kysely
   struct NewPerson {
       first_name: &'static str,
       age: i32,
   }
   db.insert_into("person").values(&NewPerson { first_name: "John", age: 30 })
   ```

### 2.4. Inferência de Linhas e Resultados (`Row` e Deserialização)
No TypeScript, o Kysely calcula o tipo de retorno da query a partir das colunas passadas no `.select(...)`. Em Rust, os tipos são nominais:
1. **Dynamic Type-Safe `Row` (Default)**:
   ```rust
   let rows: Vec<Row> = db.select_from("person")
       .select(["id", "first_name", "age"])
       .execute()
       .await?;

   for row in rows {
       let id: i64 = row.get("id")?;
       let name: String = row.get("first_name")?;
       let age: Option<i32> = row.get_opt("age")?;
   }
   ```
2. **Deserialização Tipada Direta**:
   ```rust
   let persons: Vec<Person> = db.select_from("person")
       .select_all()
       .execute_into::<Person>()
       .await?;
   ```
3. **Extração de Escalar Único**:
   ```rust
   let count: i64 = db.select_from("person")
       .select(sql::raw("count(*)"))
       .execute_take_first_val()
       .await?;
   ```

---

## 3. Arquitetura Interna: AST, Compilador e Execução

A arquitetura do `rust-kysely` espelha fielmente a separação limpa de camadas do upstream do Kysely:

```
┌─────────────────────────────────────────────────────────┐
│               Fluent Builders (API Layer)               │
│  SelectQueryBuilder, InsertQueryBuilder, SchemaModule   │
└────────────────────────────┬────────────────────────────┘
                             │ Constrói / Transforma
                             ▼
┌─────────────────────────────────────────────────────────┐
│                 OperationNode AST (Data)                │
│  Arc<OperationNode> (SelectQueryNode, WhereNode, ...)   │
└──────────────┬───────────────────────────┬──────────────┘
               │                           │
               │ Visita / Compila          │ Transforma (Plugins)
               ▼                           ▼
┌──────────────────────────────┐ ┌────────────────────────┐
│        QueryCompiler         │ │      KyselyPlugin      │
│  PostgresQueryCompiler       │ │ CamelCase, Deduplicate │
│  (Produz CompiledQuery)      │ └────────────────────────┘
└──────────────┬───────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────┐
│               Execution Engine & Driver                 │
│  QueryExecutor -> ConnectionProvider -> tokio-postgres   │
└─────────────────────────────────────────────────────────┘
```

### 3.1. A Árvore de Sintaxe Abstrata (`OperationNode`)
No Kysely, toda query é uma árvore pura de dados imutáveis. Em Rust, implementamos isso como uma hierarquia de enums e structs:
- **`OperationNode` (Enum unificado)**:
  - `SelectQuery(SelectQueryNode)`
  - `InsertQuery(InsertQueryNode)`
  - `UpdateQuery(UpdateQueryNode)`
  - `DeleteQuery(DeleteQueryNode)`
  - `Table(TableNode)`
  - `Column(ColumnNode)`
  - `Reference(ReferenceNode)`
  - `Join(JoinNode)`
  - `Where(WhereNode)`
  - `Having(HavingNode)`
  - `GroupBy(GroupByNode)`
  - `OrderBy(OrderByNode)`
  - `Limit(LimitNode)`
  - `Offset(OffsetNode)`
  - `OnConflict(OnConflictNode)`
  - `Returning(ReturningNode)`
  - `BinaryOperation(BinaryOperationNode)`
  - `UnaryOperation(UnaryOperationNode)`
  - `Value(ValueNode)`
  - `Raw(RawNode)`
  - *e mais de 60 nós adicionais que cobrem DDL, tipos, janelas, CTEs e JSON*.

#### Otimização de Memória e Zero Cópia
- **`Arc<Node>` com Copy-on-Write (`Arc::make_mut`)**: Enquanto o builder está sendo encadeado na mesma thread, modificamos o nó in-place sem alocação adicional. Ao clonar uma query base para reuso, a clonagem custa apenas um incremento atômico $O(1)$.
- **`compact_str::CompactStr`**: Para nomes de tabelas, aliases e colunas, strings de até 24 bytes são armazenadas inline na stack sem ponteiro para heap.
- **`smallvec::SmallVec<[T; 4]>`**: Listas de projeções e parâmetros comuns não provocam alocação no heap para queries com poucas colunas.

### 3.2. O Compilador de Queries (`QueryCompiler`)
O compilador de queries percorre a AST aplicando o padrão `OperationNodeVisitor`:
1. Gera a string SQL parametrizada.
2. Coleta os valores dos parâmetros dinâmicos de forma ordenada.
3. Produz um `CompiledQuery`:
   ```rust
   pub struct CompiledQuery {
       pub query_id: QueryId,
       pub sql: String,
       pub parameters: Vec<Value>,
   }
   ```

#### Especificidades do Dialeto PostgreSQL:
- **Placeholders numerados**: `$1, $2, $3, ...` (gerenciados pelo contador de parâmetros).
- **Sanitização de Identificadores**: `identifier.replace('"', "\"\"")` envolto em `""`.
- **Literais de String**: Substituição de `'` por `''`.
- **Suporte a RETURNING**: Ativado nativamente em inserts, updates e deletes.
- **Postgres Advisory Locks**: Para orquestração de migrações concorrentes seguras.

### 3.3. O Motor de Execução (`QueryExecutor`)
O executor orquestra o pipeline completo:
1. Passa o nó da query por todos os plugins configurados (`transform_query`).
2. Compila a query com o compilador do dialeto ativo.
3. Adquire uma conexão via `ConnectionProvider`.
4. Executa a query através do driver (`PostgresDriver` sobre `tokio-postgres`).
5. Passa o resultado pelos plugins (`transform_result`).
6. Retorna o `QueryResult` com as linhas afetadas e o vetor de linhas decodificadas.

---

## 4. Inventário Completo de Módulos e Recursos a Portar

A tabela abaixo lista detalhadamente cada módulo do upstream do Kysely (`kysely-org/kysely`), sua responsabilidade, complexidade e estratégia de implementação no Rust:

| Módulo Upstream (`src/`) | Arquivos / Componentes Chave | Responsabilidade | Complexidade | Estratégia no `rust-kysely` |
| :--- | :--- | :--- | :---: | :--- |
| **`operation-node/`** | 80+ nós (`select-query-node`, `where-node`, `join-node`, `binary-operation-node`, etc.) | Representação de dados da AST de SQL | Média-Alta | Enums e structs Rust com `CompactStr`, `SmallVec` e `Arc`. |
| **`query-compiler/`** | `default-query-compiler.ts`, `compiled-query.ts` | Visitor que traduz a AST para SQL parametrizado | Média | Trait `OperationNodeVisitor` e `DefaultQueryCompiler` base. |
| **`dialect/postgres/`** | `postgres-query-compiler.ts`, `postgres-adapter.ts`, `postgres-driver.ts`, `postgres-introspector.ts` | Compilador, driver e introspecção PostgreSQL | Média-Alta | Integração com `tokio-postgres`, placeholders `$n`, advisory locks. |
| **`expression/`** | `expression-builder.ts`, `expression-wrapper.ts`, `expression.ts` | Construtor de expressões (`eb.or`, `eb.fn`, comparações, subqueries) | Média | `ExpressionBuilder` com métodos de alta ordem e closures `\|eb\|`. |
| **`raw-builder/`** | `sql.ts`, `raw-builder.ts` | Construção de fragmentos SQL manuais e binds seguros | Baixa-Média | Struct `Sql` fluente com `sql::raw`, `sql::id`, `sql::lit`, `sql::join`. |
| **`query-builder/select`** | `select-query-builder.ts`, `join-builder.ts`, `case-builder.ts`, `cte-builder.ts` | SELECT queries: joins, wheres, order, group, limit, CTEs | Alta | `SelectQueryBuilder` fluente com suporte a subqueries e merges. |
| **`query-builder/insert`** | `insert-query-builder.ts`, `on-conflict-builder.ts`, `insert-result.ts` | INSERT queries: values, batch, upsert (`on_conflict`), returning | Média | `InsertQueryBuilder` com closure para `on_conflict` e returning. |
| **`query-builder/update`** | `update-query-builder.ts`, `update-result.ts` | UPDATE queries: set, joins, where, returning | Média | `UpdateQueryBuilder` com suporte a updates dinâmicos e estáticos. |
| **`query-builder/delete`** | `delete-query-builder.ts`, `delete-result.ts` | DELETE queries: where, using, returning | Baixa-Média | `DeleteQueryBuilder` completo espelhando upstream. |
| **`schema/`** | `create-table-builder.ts`, `alter-table-builder.ts`, `create-index-builder.ts`, etc. | DDL: criação e alteração de tabelas, índices e tipos | Alta | `SchemaModule` com builders tipados para constraints e tipos. |
| **`migration/`** | `migrator.ts`, `migration-provider.ts` | Controle de versão de banco, lock advisory e execução | Média | `Migrator` assíncrono com tabela de controle e migrações modulares. |
| **`plugin/`** | `camel-case-plugin.ts`, `deduplicate-joins-plugin.ts`, `with-schema-plugin.ts` | Interceptação e transformação de AST e resultados | Média | Trait `KyselyPlugin` (`transform_query`, `transform_result`). |
| **`driver/`** | `database-connection.ts`, `driver.ts`, `connection-provider.ts` | Abstração de conexões assíncronas e transações | Média | Traits assíncronas com `#[async_trait]` e controle de transações. |
| **`util/`** | `query-id.ts`, `log.ts`, `compilable.ts`, `executable.ts` | Utilitários de ID, logging e traits de compilação | Baixa | Structs e traits em `src/util/`. |

---

## 5. Exemplos de Código: Comparativo Direto (TypeScript vs. Rust)

### 5.1. Consulta SELECT com Joins, Filtro Condicional e Agregação
#### TypeScript Original:
```ts
const users = await db
  .selectFrom('users as u')
  .innerJoin('orders as o', 'o.user_id', 'u.id')
  .where('u.active', '=', true)
  .where((eb) => eb.or([
    eb('u.role', '=', 'admin'),
    eb('u.points', '>=', 1000)
  ]))
  .select([
    'u.id',
    'u.name',
    db.fn.count('o.id').as('order_count')
  ])
  .groupBy(['u.id', 'u.name'])
  .orderBy('order_count', 'desc')
  .limit(10)
  .execute()
```

#### Rust (`rust-kysely`):
```rust
let users = db
    .select_from("users as u")
    .inner_join("orders as o", "o.user_id", "u.id")
    .where_("u.active", "=", true)
    .where_expr(|eb| eb.or([
        eb.cmp("u.role", "=", "admin"),
        eb.cmp("u.points", ">=", 1000),
    ]))
    .select([
        "u.id",
        "u.name",
    ])
    .select(db.fn().count("o.id").as_("order_count"))
    .group_by(["u.id", "u.name"])
    .order_by("order_count", "desc")
    .limit(10)
    .execute()
    .await?;
```

### 5.2. Inserção com Upsert (`ON CONFLICT DO UPDATE`) e `RETURNING`
#### TypeScript Original:
```ts
const inserted = await db
  .insertInto('person')
  .values({ id: 1, name: 'Alice', visits: 1 })
  .onConflict((oc) => oc
    .column('id')
    .doUpdateSet((eb) => ({
      visits: eb('person.visits', '+', 1),
      name: eb.ref('excluded.name')
    }))
  )
  .returning(['id', 'name', 'visits'])
  .executeTakeFirstOrThrow()
```

#### Rust (`rust-kysely`):
```rust
let inserted = db
    .insert_into("person")
    .values(|v| {
        v.set("id", 1)
         .set("name", "Alice")
         .set("visits", 1)
    })
    .on_conflict(|oc| oc
        .column("id")
        .do_update_set(|eb| {
            eb.set("visits", eb.expr("person.visits", "+", 1))
              .set("name", eb.ref_("excluded.name"))
        })
    )
    .returning(["id", "name", "visits"])
    .execute_take_first_or_throw()
    .await?;
```

### 5.3. Transações Atômicas
#### TypeScript Original:
```ts
await db.transaction().execute(async (trx) => {
  await trx.updateTable('account').set({ balance: 50 }).where('id', '=', 1).execute()
  await trx.updateTable('account').set({ balance: 150 }).where('id', '=', 2).execute()
})
```

#### Rust (`rust-kysely`):
```rust
db.transaction().execute(|trx| async move {
    trx.update_table("account")
       .set("balance", 50)
       .where_("id", "=", 1)
       .execute()
       .await?;

    trx.update_table("account")
       .set("balance", 150)
       .where_("id", "=", 2)
       .execute()
       .await?;

    Ok(())
}).await?;
```

### 5.4. Schema DDL e Migrações
#### TypeScript Original:
```ts
await db.schema.createTable('pet')
  .addColumn('id', 'serial', (col) => col.primaryKey())
  .addColumn('name', 'varchar(255)', (col) => col.notNull())
  .addColumn('owner_id', 'integer', (col) => col.references('person.id').onDelete('cascade'))
  .execute()
```

#### Rust (`rust-kysely`):
```rust
db.schema().create_table("pet")
  .add_column("id", "serial", |col| col.primary_key())
  .add_column("name", "varchar(255)", |col| col.not_null())
  .add_column("owner_id", "integer", |col| col.references("person.id").on_delete("cascade"))
  .execute()
  .await?;
```

---

## 6. Plano de Fases e Estimativa de Esforço

| Fase / Milestone | Escopo & Entregáveis | Dias Est. | Dependências |
| :--- | :--- | :---: | :--- |
| **M1: Core AST & Compilador Postgres** | Nós da AST (`OperationNode`), Visitor base, `PostgresQueryCompiler`, sanitização de identificadores e placeholders `$n`. Testes unitários com compilação de SQL puro. | 3–4 dias | Nenhuma |
| **M2: Expressions & Raw SQL (`sql`)** | Struct `Sql`, métodos auxiliares (`sql::id`, `sql::lit`, `sql::join`), `ExpressionBuilder` com lógica booleana (`or`, `and`, `not`) e operadores binários. | 2–3 dias | M1 |
| **M3: Select Query Builder** | `SelectQueryBuilder`: FROM, INNER/LEFT/RIGHT/FULL JOIN, WHERE, WHERE_REF, GROUP BY, HAVING, ORDER BY, LIMIT, OFFSET, CTEs (`with`), UNIONS. | 3–4 dias | M1, M2 |
| **M4: Mutation Builders & RETURNING** | `InsertQueryBuilder` (batch, on_conflict / upsert), `UpdateQueryBuilder`, `DeleteQueryBuilder`, cláusula `RETURNING` e `RETURNING ALL`. | 3–4 dias | M1, M2, M3 |
| **M5: Driver Postgres & Execução** | Camada de conexão com `tokio-postgres`, `QueryExecutor`, abstração de `Row`, `QueryResult`, `.execute()`, `.execute_take_first()`, transações e savepoints. | 3–4 dias | M1 a M4 |
| **M6: Schema DDL & Introspecção** | `SchemaModule` (`create_table`, `alter_table`, `drop_table`, `create_index`, `create_view`, `create_type`), introspector consultando `pg_catalog`. | 3–4 dias | M1, M5 |
| **M7: Migrações & Advisory Locks** | `Migrator`, `MigrationProvider`, migração atômica com locks consultando `pg_advisory_lock` e tabela de histórico. | 2–3 dias | M5, M6 |
| **M8: Sistema de Plugins & Polimento** | `KyselyPlugin`, `CamelCasePlugin`, documentação completa, benchmarks e paridade com testes do upstream. | 2–3 dias | M1 a M7 |

---

## 7. Próximos Passos Imediatos

1. Validar e formalizar o `INSIGHT.md` como diretriz mestre do repositório.
2. Iniciar a **Fase 1 (M1: Core AST & Compilador Postgres)** com especificação de requisitos formais (`REQ-001` a `REQ-010`) em `.specs/features/01-core-ast-compiler/spec.md`.
3. Implementar a árvore de nós `OperationNode` e o compilador de PostgreSQL validando contra dezenas de casos de teste de SQL do upstream.
