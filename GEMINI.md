# Antigravity / Gemini Skill — rust-backend-boilerplate

## Project Overview

A minimal, production-ready Rust backend boilerplate: **Axum + SeaORM + PostgreSQL**.
Provides infrastructure scaffolding only — no opinionated business logic ships by default.

## Workspace Layout

Cargo workspace with three members:

| Crate | Path | Purpose |
|---|---|---|
| `rust-backend-boilerplate` | `.` | Main Axum app server |
| `migration` | `db/migrations/` | SeaORM migration runner |
| `g` | `g/` | Feature scaffolding CLI |

```
.
├── Cargo.toml           # Workspace declaration & backend dependencies
├── Makefile             # CLI automation scripts via sea-orm-cli and g
├── db/
│   ├── mod.rs           # setup + models exports
│   ├── setup.rs         # connect_db(url, max_conn, min_conn)
│   ├── models/          # Auto-generated SeaORM entities
│   └── migrations/      # SeaORM migration runner crate
├── g/                   # Feature scaffolding CLI crate (renamed from generator)
├── src/
│   ├── main.rs          # Config → DB → Router → serve with graceful shutdown
│   ├── lib.rs           # Re-exports features, infra, middleware, routes, db (mapped path)
│   ├── features/        # Domain modules (handler + service + dto pattern)
│   │   └── health/
│   ├── infra/
│   │   ├── config.rs    # Config with automatic DATABASE_URL resolution
│   │   ├── error.rs     # AppError enum → IntoResponse + From<DbErr>
│   │   ├── extractor.rs # ValidatedJson, ValidatedQuery, and ValidatedPath Axum extractors
│   │   └── pagination.rs# PaginationParams + PaginatedResponse<T>
│   ├── middleware/
│   │   └── request_id.rs# x-request-id propagation with tracing span
│   └── routes/
│       ├── mod.rs       # AppState + create_router() + FromRef impls
│       ├── health.rs    # Route wiring for health feature
│       └── swagger.rs   # utoipa OpenApi derive (ApiDoc)
```

## How to Add a New Feature (step-by-step)

### Step 1: Scaffold

```bash
make g:feature name=my_feature
```

This generates `src/features/my_feature/{mod.rs, dto.rs, handler.rs, service.rs}` and auto-registers in `src/features/mod.rs`.

### Step 2: Create Route File

Create `src/routes/my_feature.rs`:

```rust
use axum::{routing::{get, post}, Router};
use crate::{features::my_feature::handler as my_feature_handler, routes::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(my_feature_handler::list).post(my_feature_handler::create))
        .route("/{id}", get(my_feature_handler::get_by_id))
}
```

### Step 3: Register in `src/routes/mod.rs`

Add `pub mod my_feature;` and `.nest("/my-feature", my_feature::router())` inside `create_router()`.

### Step 4: Register in `src/routes/swagger.rs`

Add handler paths to `paths(...)` and DTO schemas to `components(schemas(...))` in the `#[openapi(...)]` attribute on `ApiDoc`.

## Canonical Code Patterns

### Handler Signature

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

### Request Validation

Use `ValidatedJson<T>` for bodies that need validation. DTOs must derive `validator::Validate`:

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

### Service Pattern

Services are unit structs with associated methods taking `&DatabaseConnection` first:

```rust
pub struct ThingService;

impl ThingService {
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<thing::Model>, sea_orm::DbErr> {
        thing::Entity::find_by_id(id).one(db).await
    }
}
```

### Error Handling

`AppError` variants: `BadRequest(400)`, `Unauthorized(401)`, `Forbidden(403)`, `NotFound(404)`, `Conflict(409)`, `ValidationError(422)`, `Internal(500)`.

`?` on `sea_orm::DbErr` auto-maps PG 23505 → `Conflict`, PG 23503 → `BadRequest`.

### Pagination

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

### SeaORM Entity

```rust
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

Place in `db/models/`. Auto-generated by running `make db:entity`.

### Config

Add new env vars to `Config` struct + `Config::init()` in `src/infra/config.rs`. Update `.env.example`.
Implement `FromRef<AppState>` for new state types in `src/routes/mod.rs`.

## Commands

| Command | Purpose |
|---|---|
| `make run` | Hot-reload dev server |
| `make check` | Fast compilation check |
| `make fmt` | Format code |
| `make lint` | Clippy with `-D warnings` |
| `make ci` | Full CI pipeline (fmt + lint) |
| `make db:up` | Run pending migrations |
| `make db:down` | Rollback last migration |
| `make db:migration name=xxx` | Generate a new migration script |
| `make db:entity` | Auto-generate/update database models/entities |
| `make g:feature name=xxx` | Scaffold a new feature module |
| `make docker:up` | Start app + postgres |

## Rules

1. **Every handler must have `#[utoipa::path]` annotations** — register in `swagger.rs` `ApiDoc`.
2. **No `unwrap()` in handlers/services** — use `?` with `AppError`.
3. **Handler return types**: `Result<(StatusCode, Json<T>), AppError>` or `Result<StatusCode, AppError>`.
4. **Naming**: feature names = `snake_case`, URL paths = `kebab-case`.
5. **Use `sea_orm::sqlx::Error`**, not direct `sqlx` — no direct sqlx dependency.
6. **Decoupled Migrations**: Migrations live inside `db/migrations/` crate and do not affect application compile times.
7. **Scaffolder**: `g/` is a separate workspace member — does not affect main app compile times.
8. **No business-specific deps in root `Cargo.toml`** unless used by core infra.
