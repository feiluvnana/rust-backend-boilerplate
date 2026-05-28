# 🦀 Rust Backend Boilerplate

[![CI](https://github.com/YOUR_USERNAME/rust-backend-boilerplate/actions/workflows/ci.yml/badge.svg)](https://github.com/YOUR_USERNAME/rust-backend-boilerplate/actions/workflows/ci.yml)

A minimal, production-ready Rust backend boilerplate built with **Axum**, **SeaORM**, and **PostgreSQL**.

Ships only infrastructure scaffolding — no opinionated business logic. Add your own features on top of a clean foundation.

## ✨ What's Included

| Category | What you get |
|---|---|
| **Web Framework** | [Axum](https://github.com/tokio-rs/axum) with typed extractors and layered middleware |
| **Database** | [SeaORM](https://github.com/SeaQL/sea-orm) + PostgreSQL with connection pooling and migrations |
| **Validation** | `ValidatedJson<T>` custom extractor using [validator](https://github.com/Keats/validator) |
| **Error Handling** | Structured `AppError` → JSON responses with auto PG error code mapping |
| **Pagination** | Built-in `PaginatedResponse<T>` with page/per_page query params |
| **API Docs** | Auto-generated Swagger UI at `/swagger-ui` via [utoipa](https://github.com/juhaku/utoipa) |
| **Observability** | Structured tracing + `x-request-id` propagation middleware |
| **Security Headers** | `X-Content-Type-Options`, `X-Frame-Options`, `X-XSS-Protection` |
| **CORS** | Configurable via `CORS_ORIGIN` env var |
| **Code Generator** | `make g:feature name=xxx` scaffolds handler + service + DTO |
| **Docker** | Multi-stage build producing a ~30MB Alpine image |
| **CI** | GitHub Actions workflow (fmt → clippy → test) |

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [PostgreSQL](https://www.postgresql.org/) (or Docker)
- [cargo-watch](https://github.com/watchexec/cargo-watch) (`cargo install cargo-watch`)

### 1. Clone & configure

```bash
git clone https://github.com/YOUR_USERNAME/rust-backend-boilerplate.git
cd rust-backend-boilerplate
make g:env          # creates .env from .env.example
```

### 2. Start the database

```bash
make docker:up      # starts PostgreSQL via docker-compose
```

### 3. Run migrations & start the server

```bash
make db:up          # run pending migrations
make run            # hot-reload dev server on :3000
```

Open **http://localhost:3000/swagger-ui** to explore the API.

## 📁 Project Structure

```
.
├── src/
│   ├── main.rs                 # Entry point: config → db → router → serve
│   ├── lib.rs                  # Module re-exports
│   ├── features/               # Business logic (handler + service + dto)
│   │   └── health/             # Example feature: health checks
│   │       └── handler.rs
│   ├── infra/                  # Cross-cutting infrastructure
│   │   ├── config.rs           # Env-based configuration
│   │   ├── error.rs            # AppError → JSON error responses
│   │   ├── extractor.rs        # ValidatedJson<T> extractor
│   │   └── pagination.rs       # PaginationParams + PaginatedResponse<T>
│   ├── middleware/
│   │   └── request_id.rs       # x-request-id tracing middleware
│   └── routes/
│       ├── mod.rs              # AppState + create_router()
│       ├── health.rs           # Route definitions for /health
│       └── swagger.rs          # OpenAPI spec (utoipa)
├── db/
│   ├── setup.rs                # Database connection with pooling
│   ├── models/                 # Auto-generated database entities
│   └── migrations/             # SeaORM migrations crate
├── g/                          # Feature scaffolding CLI
├── Dockerfile                  # Multi-stage production build
├── docker-compose.yml          # App + PostgreSQL
└── Makefile                    # Developer commands
```

## 🛠 Adding a New Feature

### Step 1 — Scaffold

```bash
make g:feature name=product
```

This generates `src/features/product/{mod.rs, dto.rs, handler.rs, service.rs}` and registers the module.

### Step 2 — Wire up routes

Create `src/routes/product.rs`:

```rust
use axum::{routing::{get, post}, Router};
use crate::{features::product::handler as product_handler, routes::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(product_handler::list).post(product_handler::create))
        .route("/{id}", get(product_handler::get_by_id))
}
```

### Step 3 — Register

In `src/routes/mod.rs`:

```rust
pub mod product;

// inside create_router():
.nest("/products", product::router())
```

### Step 4 — Add to Swagger

In `src/routes/swagger.rs`, add your handler paths and DTO schemas to `ApiDoc`.

### Step 5 — Database (if needed)

1. Create a migration: `make db:migration name=xxx`
2. Register it in `db/migrations/src/lib.rs`
3. Create a SeaORM entity in `db/models/`
4. Run `make db:up`

## 📋 Commands

Run `make help` to see all available commands.

| Command | Description |
|---|---|
| `make run` | Start dev server with hot-reload |
| `make check` | Fast compilation check |
| `make fmt` | Format code |
| `make lint` | Run Clippy with `-D warnings` |
| `make ci` | Full CI pipeline (fmt → lint) |
| `make db:up` | Run pending migrations |
| `make db:down` | Rollback last migration |
| `make g:env` | Generate `.env` from `.env.example` |
| `make g:feature name=xxx` | Scaffold a new feature module |
| `make docker:up` | Start app + PostgreSQL containers |
| `make docker:down` | Stop containers and remove volumes |
| `make docker:build` | Build the Docker image |
| `make docker:logs` | Tail container logs |

## ⚙️ Configuration

All configuration is read from environment variables (loaded from `.env` via [dotenvy](https://github.com/allan2/dotenvy)):

| Variable | Default | Description |
|---|---|---|
| `POSTGRES_USER` | `postgres` | Database user |
| `POSTGRES_PASSWORD` | `password` | Database password |
| `POSTGRES_DB` | `backend_db` | Database name |
| `POSTGRES_HOST` | `localhost` | Database host |
| `POSTGRES_PORT` | `5432` | Database port |
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `3000` | Server port |
| `CORS_ORIGIN` | `*` | Allowed CORS origin |
| `RUST_LOG` | `info,sqlx=warn` | Tracing filter |

Add new config fields to `src/infra/config.rs` and update `.env.example`.

## 🏗 Architecture

```
Request
  │
  ▼
┌─────────────────────────────────┐
│  Tower Layers (main.rs)         │
│  CORS → Body Limit → Compress  │
│  → Trace → Catch Panic         │
└─────────────┬───────────────────┘
              │
              ▼
┌─────────────────────────────────┐
│  API Middleware (routes/mod.rs) │
│  Request ID → Security Headers │
└─────────────┬───────────────────┘
              │
              ▼
┌─────────────────────────────────┐
│  Router                        │
│  /api/health → health::router  │
│  /swagger-ui → SwaggerUi       │
└─────────────┬───────────────────┘
              │
              ▼
┌─────────────────────────────────┐
│  Handler                       │
│  Extracts State, Path, Query,  │
│  ValidatedJson from request    │
│  Returns Result<Json, AppError>│
└─────────────┬───────────────────┘
              │
              ▼
┌─────────────────────────────────┐
│  Service                       │
│  Business logic + DB queries   │
│  via SeaORM                    │
└─────────────────────────────────┘
```

## 🐳 Docker

Build and run with Docker Compose:

```bash
make docker:build   # build the image
make docker:up      # start app + postgres
make docker:logs    # watch logs
```

The Dockerfile uses a multi-stage build:
1. **Build stage**: Compiles with `rust:alpine`, caches cargo registry and build artifacts
2. **Runtime stage**: Copies only the binary to `alpine:3.18` (~30MB final image)

## License

MIT
