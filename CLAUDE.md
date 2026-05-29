# CLAUDE.md — Project Intelligence for rust-backend-boilerplate

## Project Overview

A minimal, production-ready Rust backend boilerplate built with **Axum + SeaORM + PostgreSQL**. It provides infrastructure scaffolding (error handling, config, validation, pagination, middleware, Swagger) without opinionated business logic. The `examples/user_auth_reference/` directory contains a full auth/user feature implementation that can be copied back in as a starting point.

## Workspace Structure

This is a **Cargo workspace** with three members:

```
.                          # Main app (axum server)
├── migration/             # SeaORM migration runner (separate crate)
├── g/                     # Feature scaffolding CLI tool (separate crate)
├── examples/              # Reference implementations (not compiled)
│   └── user_auth_reference/
├── src/
│   ├── main.rs            # Entry point: config → db → router → serve
│   ├── lib.rs             # Re-exports all modules
│   ├── db/                # Database connection setup
│   │   └── setup.rs       # connect_db(url, max, min) → DatabaseConnection
│   ├── extractors/         # Custom Axum extractors (ValidatedJson, custom extractors)
│   ├── features/          # Business logic grouped by domain
│   │   └── health/        # Example: handler.rs, router.rs (with utoipa annotations)
│   ├── infra/             # Cross-cutting infrastructure
│   │   ├── config.rs      # Config struct from env vars (dotenvy)
│   │   ├── error.rs       # AppError enum → IntoResponse, From<DbErr>
│   │   └── pagination.rs  # PaginationParams, PaginatedResponse<T>
│   ├── middleware/         # HTTP middleware
│   │   └── request_id.rs  # x-request-id propagation + tracing span
│   └── routes/            # Axum router assembly
│       ├── mod.rs          # AppState, create_router(), FromRef impls
│       └── swagger.rs      # utoipa OpenApi derive with ApiDoc
```

## Key Patterns — FOLLOW THESE

### Adding a New Feature, Resource, Middleware, or Extractor

Use the generator:
- **Full CRUD Resource**: `make g:resource name=my_resource` (or `cargo run -p g -- resource my_resource`)
  This scaffolds a database-backed CRUD resource (DTOs, handlers, services, and routes) and automatically registers them in routing and Swagger.
- **Simple Feature Placeholder**: `make g:feature name=my_feature` (or `cargo run -p g -- feature my_feature`)
  This creates `src/features/my_feature/{mod.rs, dto.rs, handler.rs, service.rs, router.rs}` and automatically registers it in routing and the feature listing.
- **HTTP Middleware**: `make g:middleware name=my_middleware` (or `cargo run -p g -- middleware my_middleware`)
  This creates `src/middleware/my_middleware.rs` and registers it in `src/middleware/mod.rs`.
- **Custom Extractor**: `make g:extractor name=my_extractor` (or `cargo run -p g -- extractor my_extractor`)
  This creates `src/extractors/my_extractor.rs` and registers/re-exports it in `src/extractors/mod.rs`.

### Handler Pattern

Handlers are async functions that return `Result<(StatusCode, Json<T>), AppError>`:

```rust
#[utoipa::path(
    get,
    path = "/api/things/{id}",
    responses(
        (status = 200, description = "Success", body = ThingResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
    ),
    params(("id" = i32, Path, description = "Thing ID")),
)]
pub async fn get_thing(
    State(db): State<DatabaseConnection>,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<(StatusCode, Json<ThingResponse>), AppError> {
    let thing = ThingService::find_by_id(&db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Thing not found".to_string()))?;
    Ok((StatusCode::OK, Json(ThingResponse::from(thing))))
}
```

### Validated Input

Use `ValidatedJson<T>` (from `extractors`) for request bodies that need validation. The DTO struct must derive `Validate` from the `validator` crate:

```rust
#[derive(Debug, Deserialize, Validate, ToSchema, Clone)]
pub struct CreateThingRequest {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}

pub async fn create(
    State(db): State<DatabaseConnection>,
    ValidatedJson(payload): ValidatedJson<CreateThingRequest>,
) -> Result<(StatusCode, Json<ThingResponse>), AppError> { ... }
```

### Error Handling

All errors go through `AppError` (in `src/infra/error.rs`). It implements `IntoResponse` and auto-converts from `sea_orm::DbErr` and `validator::ValidationErrors`. Use the existing variants:

- `AppError::BadRequest(msg)` → 400
- `AppError::Unauthorized(msg)` → 401
- `AppError::Forbidden(msg)` → 403
- `AppError::NotFound(msg)` → 404
- `AppError::Conflict(msg)` → 409
- `AppError::ValidationError(errors)` → 422
- `AppError::Internal(msg)` → 500

The `?` operator on `sea_orm::DbErr` automatically maps unique constraint violations (PG 23505) to `Conflict` and foreign key violations (PG 23503) to `BadRequest`.

### Service Pattern

Services are unit structs with associated async methods. They take `&DatabaseConnection` as the first argument and return `Result<T, sea_orm::DbErr>`:

```rust
pub struct ThingService;

impl ThingService {
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: i32,
    ) -> Result<Option<thing::Model>, sea_orm::DbErr> {
        thing::Entity::find_by_id(id).one(db).await
    }
}
```

### Pagination

Use `PaginationParams` from query string and return `PaginatedResponse<T>`:

```rust
pub async fn list(
    State(db): State<DatabaseConnection>,
    Query(params): Query<PaginationParams>,
) -> Result<(StatusCode, Json<PaginatedResponse<ThingResponse>>), AppError> {
    let page = params.page();
    let per_page = params.per_page();
    let (items, total) = ThingService::list(&db, page, per_page).await?;
    let data = items.into_iter().map(ThingResponse::from).collect();
    Ok((StatusCode::OK, Json(PaginatedResponse::new(data, page, per_page, total))))
}
```

### Route Definitions

Each feature has a router file in `src/features/my_feature/router.rs` returning `Router<AppState>`:

```rust
use axum::{routing::{get, post}, Router};
use crate::routes::AppState;
use super::handler as handler;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::list).post(handler::create))
        .route("/{id}", get(handler::get_by_id))
}
```

### Database Models (SeaORM entities)

Define in `src/db/models/`. Each model file uses `DeriveEntityModel`:

```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "things")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

Remember to register with `pub mod my_model;` in `src/db/models/mod.rs` (create this if it doesn't exist, and add `pub mod models;` to `src/db/mod.rs`).

### Config

All configuration comes from environment variables loaded via `dotenvy`. Add new config fields to `Config` struct in `src/infra/config.rs`, read them in `Config::init()`, and add them to `.env.example`.

### AppState

`AppState` (in `src/routes/mod.rs`) holds `db: DatabaseConnection` and `config: Config`. If you add new fields, implement `FromRef<AppState>` for the new type so handlers can extract it directly with `State(x): State<X>`.

## Commands

| Command | Purpose |
|---|---|
| `make run` | Hot-reload dev server (`cargo watch -x run`) |
| `make check` | Fast compilation check |
| `make fmt` | Format code |
| `make lint` | Clippy with `-D warnings` |
| `make ci` | Full CI: fmt check → clippy |
| `make g:migration name=xxx` | Generate a new migration script |
| `make g:entity` | Auto-generate/update database models/entities |
| `make db:up` | Run pending migrations |
| `make db:down` | Rollback last migration |
| `make g:env` | Copy `.env.example` → `.env` |
| `make g:feature name=xxx` | Scaffold a new feature module |
| `make g:resource name=xxx` | Scaffold a NestJS-like CRUD resource module |
| `make g:middleware name=xxx` | Scaffold a new HTTP middleware |
| `make g:extractor name=xxx` | Scaffold a new custom Axum extractor |
| `make docker:up` | Start app + postgres via docker-compose |

## Critical Rules

1. **Every handler must have `#[utoipa::path(...)]` annotations** for Swagger. Add paths and schemas to `src/routes/swagger.rs` `ApiDoc`.
2. **Never use `unwrap()` in handler/service code.** Use `?` with `AppError` or `.ok_or_else(|| AppError::...)`.
3. **Return types are always `Result<(StatusCode, Json<T>), AppError>`** for handlers that return data, or `Result<StatusCode, AppError>` for empty responses.
4. **Feature names must be `snake_case`**, route URL paths must be `kebab-case`.
5. **Use `sea_orm::sqlx::Error`** (not direct `sqlx`) when matching database errors — the crate does not have a direct `sqlx` dependency.
6. **Migrations** go in the `migration/` crate. Add new migration modules to `migration/src/lib.rs`.
8. **The `g/` crate is a workspace member** — it compiles separately and does not affect main app build times.
9. **Do NOT add business-specific dependencies to root `Cargo.toml`** unless they're used by the core infrastructure. Feature-specific deps should be evaluated for necessity.
10. **`examples/user_auth_reference/`** is the canonical reference for how to build auth, JWT, user CRUD, and custom extractors. Consult it when building similar features.
