# Web API CRUD with Axum and Tokio

This example demonstrates how to integrate `rust-kysely` with an [Axum](https://github.com/tokio-rs/axum) web service.

---

## Architecture

In an Axum application, `Kysely` can be wrapped in an `Arc` (or cloned directly, since its internal handles are already `Arc`-backed) and passed into router state:

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use rust_kysely::dialect::postgres::PostgresDialect;
use rust_kysely::Kysely;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: Arc<Kysely>,
}

#[derive(Serialize, Deserialize)]
struct User {
    id: i64,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUserDto {
    name: String,
    email: String,
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<User>, StatusCode> {
    let result = state
        .db
        .select_from("users")
        .select(["id", "name", "email"])
        .where_("id", "=", id)
        .execute_take_first()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match result {
        Some(row) => Ok(Json(User {
            id: row.get("id").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            name: row.get("name").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            email: row.get("email").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    let result = state
        .db
        .insert_into("users")
        .value("name", payload.name)
        .value("email", payload.email)
        .returning(["id", "name", "email"])
        .execute_take_first()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = User {
        id: result.get("id").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        name: result.get("name").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        email: result.get("email").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    };

    Ok((StatusCode::CREATED, Json(user)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dialect = PostgresDialect::from_connection_string("host=localhost user=postgres dbname=app")?;
    let db = Arc::new(Kysely::new(dialect));
    let state = AppState { db };

    let app = Router::new()
        .route("/users/:id", get(get_user))
        .route("/users", post(create_user))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
```
