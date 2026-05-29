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
│   ├── extractors/      # Custom Axum extractors (ValidatedJson, ValidatedQuery, ValidatedPath, etc.)
│   ├── features/        # Domain modules (handler + service + dto + router pattern)
│   │   └── health/
│   │       ├── mod.rs
│   │       ├── handler.rs
│   │       └── router.rs
│   ├── infra/
│   │   ├── config.rs    # Config with automatic DATABASE_URL resolution
│   │   ├── error.rs     # AppError enum → IntoResponse + From<DbErr>
│   │   └── pagination.rs# PaginationParams + PaginatedResponse<T>
│   ├── middleware/
│   │   └── request_id.rs# x-request-id propagation with tracing span
│   └── routes/
│       ├── mod.rs       # AppState + create_router() + FromRef impls
│       └── swagger.rs   # utoipa OpenApi derive (ApiDoc)
```

## How to Add a New Feature or Resource (step-by-step)

### Method A: Scaffold a Clean Resource

To generate a NestJS-like CRUD resource (with placeholder service returning mock DTO responses, CRUD handlers, and CRUD routes automatically registered in Swagger and routes):

```bash
make g:resource name=my_resource
```

### Method B: Scaffold a Clean Feature Placeholder

To generate a clean feature skeleton (empty handler, empty service, empty dto, empty route, registered in features/routes module):

```bash
make g:feature name=my_feature
```

`make g:feature` generates empty shell files (with `#[allow(unused_imports)]` to compile cleanly), while `make g:resource` generates CRUD files using mock placeholder data so it compiles out-of-the-box without needing any database tables or SeaORM models first. Both commands automatically wire up the routes to `src/features/mod.rs` and `src/routes/mod.rs`. `make g:resource` also registers schemas and paths in `src/routes/swagger.rs`.


### Swagger Registration (if using custom endpoints or schemas)

Add handler paths to `paths(...)` and DTO schemas to `components(schemas(...))` in the `#[openapi(...)]` attribute on `ApiDoc` inside `src/routes/swagger.rs`.


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
| `make g:migration name=xxx` | Generate a new migration script |
| `make g:entity` | Auto-generate/update database models/entities |
| `make g:feature name=xxx` | Scaffold a new feature module |
| `make g:resource name=xxx` | Scaffold a NestJS-like CRUD resource module |
| `make g:middleware name=xxx` | Scaffold a new HTTP middleware |
| `make g:extractor name=xxx` | Scaffold a new custom Axum extractor |
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
