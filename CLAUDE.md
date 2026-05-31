# CLAUDE.md — rust-backend-boilerplate

## Overview

Minimal Rust backend: **Axum + SeaORM + SQLite**.
Infrastructure scaffolding only — no business logic ships.

## Structure

2-crate workspace:

```
.                              # Main app (Axum server)
├── database/
│   ├── mod.rs                 # Re-exports models + setup
│   ├── setup.rs               # connect_db(url, max, min) → DatabaseConnection
│   ├── models/
│   │   └── mod.rs             # Auto-generated SeaORM entities (empty initially)
│   └── migrations/            # SeaORM migration crate
│       └── src/
│           ├── lib.rs         # Migrator struct
│           └── main.rs        # dotenvy + run_cli(Migrator)
├── src/
│   ├── main.rs                # Config → DB → Router → serve + graceful shutdown
│   ├── lib.rs                 # Re-exports all modules (database via path-mapping)
│   ├── extractors/            # ValidatedJson, ValidatedPath, ValidatedQuery
│   ├── features/              # Domain modules (handler → service → repository)
│   │   └── health/            # handler.rs, service.rs, repository.rs
│   ├── infra/
│   │   ├── config.rs          # Config struct from env (dotenvy)
│   │   └── routes.rs          # AppState + create_router() + ApiDoc (utoipa) + FromRef
│   ├── middleware/
│   │   └── mod.rs             # request_id_middleware (x-request-id + tracing span)
│   └── types/
│       ├── error.rs           # AppError → IntoResponse + From<DbErr> + From<ValidationErrors>
│       └── pagination.rs      # PaginationParams + PaginatedResponse<T>
├── Makefile                   # CLI automation
├── Dockerfile                 # Multi-stage Docker build
└── docker-compose.yml         # Single service setup
```

## Key Patterns

### Add New Feature

1. Create `src/features/my_feature/` with:
   - `mod.rs` — exports handler, service, repository; re-exports `router()`
   - `handler.rs` — handlers + DTO structs + `pub fn router() -> Router<AppState>`
   - `service.rs` — business logic (unit struct, async methods, calls repository)
   - `repository.rs` — raw DB access (unit struct, returns `Result<T, DbErr>`)

2. Register `pub mod my_feature;` in `src/features/mod.rs`

3. Mount `.nest("/my-feature", crate::features::my_feature::router())` in `src/infra/routes.rs`

4. Register paths + schemas in `#[openapi(...)]` on `ApiDoc` in `src/infra/routes.rs`

### Handler

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

### Router (inside handler.rs, not separate file)

```rust
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_by_id))
}
```

### Validation

`ValidatedJson<T>` for bodies. DTO derives `Validate`:

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

Also: `ValidatedPath<T>`, `ValidatedQuery<T>`.

### Error Handling

`AppError` in `src/types/error.rs`. Variants:

- `BadRequest(msg)` → 400
- `Unauthorized(msg)` → 401
- `Forbidden(msg)` → 403
- `NotFound(msg)` → 404
- `Conflict(msg)` → 409
- `ValidationError(errors)` → 422
- `Internal(msg)` → 500
- `ServiceUnavailable(msg)` → 503

`From<DbErr>`: unique violation → `Conflict`, FK violation → `BadRequest`, `RecordNotFound` → `NotFound`, rest → `Internal`.

### Service (calls repository, business logic)

```rust
pub struct ThingService;

impl ThingService {
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<thing::Model>, AppError> {
        ThingRepository::find_by_id(db, id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }
}
```

### Repository (raw DB, returns `Result<T, DbErr>`)

```rust
pub struct ThingRepository;

impl ThingRepository {
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<thing::Model>, sea_orm::DbErr> {
        thing::Entity::find_by_id(id).one(db).await
    }
}
```

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

Place in `database/models/`. Auto-generate: `make db:entity`.

```rust
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "things")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, DeriveRelation, EnumIter)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
```

Register `pub mod my_model;` in `database/models/mod.rs`.

### Config

Add env vars → `Config` in `src/infra/config.rs` + update `.env.example`.
New state types → `FromRef<AppState>` in `src/infra/routes.rs`.

### DB Path-Mapping

`database/` lives outside `src/`, included via:
```rust
#[path = "../database/mod.rs"]
pub mod database;
```

## Commands

| Command | Purpose |
|---|---|
| `make run` | Hot-reload dev server (`cargo watch -x run`) |
| `make db:up` | Run pending migrations |
| `make db:down` | Rollback last migration |
| `make db:migration name=xxx` | Generate new migration |
| `make db:entity` | Auto-generate entity models from DB |
| `make env:setup` | Copy `.env.example` → `.env` |

## Rules

1. Every handler needs `#[utoipa::path]` — register in `ApiDoc` in `src/infra/routes.rs`.
2. No `unwrap()` in handlers/services/repos — `?` with `AppError`.
3. Handler returns: `Result<(StatusCode, Json<T>), AppError>` or `Result<StatusCode, AppError>`.
4. Feature names `snake_case`, URL paths `kebab-case`.
5. `sea_orm::sqlx::Error` — no direct sqlx dep.
6. Architecture: handler → service → repository. Handlers never call repos directly.
7. Migrations in `database/migrations/` — decoupled compile.
8. No business deps in root `Cargo.toml` unless core infra needs.
9. DB module path-mapped from `database/` — don't move into `src/`.
