# Gemini Skill — rust-backend-boilerplate

## Overview

Minimal Rust backend boilerplate: **Axum + SeaORM + SQLite**.
Infrastructure scaffolding only — no business logic ships.

## Workspace

2 crates:

| Crate | Path | Purpose |
|---|---|---|
| `rust-backend-boilerplate` | `.` | Axum app server |
| `migration` | `database/migrations/` | SeaORM migration runner |

```
.
├── Cargo.toml                # Workspace + backend deps
├── Makefile                  # CLI automation (make commands)
├── database/
│   ├── mod.rs                # Re-exports models + setup
│   ├── setup.rs              # connect_db(url, max, min)
│   ├── models/
│   │   └── mod.rs            # Auto-generated SeaORM entities (empty initially)
│   └── migrations/           # SeaORM migration crate
│       └── src/
│           ├── lib.rs        # Migrator struct
│           └── main.rs       # dotenvy + run_cli(Migrator)
├── src/
│   ├── main.rs               # Config → DB → Router → serve + graceful shutdown
│   ├── lib.rs                # Re-exports via path-mapping: database, extractors, middleware, types, infra, features
│   ├── extractors/           # ValidatedJson, ValidatedPath, ValidatedQuery
│   ├── features/             # Domain modules (handler → service → repository)
│   │   └── health/           # handler.rs, service.rs, repository.rs
│   ├── infra/
│   │   ├── config.rs         # Config struct from env (dotenvy)
│   │   └── routes.rs         # AppState + create_router() + ApiDoc (utoipa) + FromRef
│   ├── middleware/
│   │   └── mod.rs            # request_id_middleware (x-request-id + tracing span)
│   └── types/
│       ├── error.rs          # AppError → IntoResponse + From<DbErr> + From<ValidationErrors>
│       └── pagination.rs     # PaginationParams + PaginatedResponse<T>
```

## Add New Feature

1. Create `src/features/my_feature/`:
   - `mod.rs` — exports handler, service, repository; re-exports `router()`
   - `handler.rs` — handlers + `pub fn router() -> Router<AppState>` + DTO structs
   - `service.rs` — business logic (unit struct, async methods)
   - `repository.rs` — DB access (unit struct, returns `Result<T, DbErr>`)

2. Register in `src/features/mod.rs`:
   ```rust
   pub mod my_feature;
   ```

3. Mount in `src/infra/routes.rs` `create_router()`:
   ```rust
   .nest("/my-feature", crate::features::my_feature::router())
   ```

4. Register paths + schemas in `#[openapi(...)]` on `ApiDoc` in `src/infra/routes.rs`.

## Code Patterns

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

### Router (inside handler.rs)

```rust
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_by_id))
}
```

### Validation

`ValidatedJson<T>` for bodies. DTO derives `validator::Validate`:

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

### Service (calls repository, contains business logic)

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

### Error

Variants: `BadRequest(400)`, `Unauthorized(401)`, `Forbidden(403)`, `NotFound(404)`, `Conflict(409)`, `ValidationError(422)`, `Internal(500)`, `ServiceUnavailable(503)`.

`From<DbErr>` maps: unique violation → `Conflict`, FK violation → `BadRequest`, `RecordNotFound` → `NotFound`, rest → `Internal`.

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

### Config

Add env vars → `Config` struct in `src/infra/config.rs` + `Config::init()`. Update `.env.example`.
New state types → `FromRef<AppState>` in `src/infra/routes.rs`.

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
2. No `unwrap()` in handlers/services/repos — use `?` with `AppError`.
3. Handler returns: `Result<(StatusCode, Json<T>), AppError>` or `Result<StatusCode, AppError>`.
4. Feature names `snake_case`, URL paths `kebab-case`.
5. Use `sea_orm::sqlx::Error` — no direct sqlx dep.
6. Architecture: handler → service → repository. Handlers never call repos directly.
7. Migrations in `database/migrations/` crate — decoupled compile.
8. No business deps in root `Cargo.toml` unless core infra needs.
9. DB module path-mapped: `#[path = "../database/mod.rs"] pub mod database;` in `lib.rs`.
