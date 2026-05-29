.PHONY: help setup run check fmt lint \
        docker\:up docker\:down docker\:build docker\:logs \
        db\:up db\:down \
        g\:env g\:feature \
        ci

help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_:.-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

setup: ## Bootstrap environment and install cargo-watch
	$(MAKE) g\:env
	cargo install cargo-watch

# ─── App ──────────────────────────────────────────
run: ## Run the backend application with hot-reloading
	cargo watch -x run

check: ## Fast compilation check
	cargo check

fmt: ## Format code
	cargo fmt --all

lint: ## Run clippy lints
	cargo clippy --all-targets -- -D warnings

ci: ## Run full CI pipeline locally (fmt + lint)
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings

# ─── Docker ───────────────────────────────────────
docker\:up: ## Start all containers (app + postgres)
	docker compose up -d

docker\:down: ## Stop all containers and remove volumes
	docker compose down -v

docker\:build: ## Build the Docker image
	docker compose build

docker\:logs: ## Tail container logs
	docker compose logs -f

# ─── Database ─────────────────────────────────────
db\:up: ## Run database migrations via sea-orm-cli
	sea-orm-cli migrate -d db/migrations up

db\:down: ## Rollback the last migration via sea-orm-cli
	sea-orm-cli migrate -d db/migrations down

g\:migration: ## Create a new migration (usage: make g:migration name=xxx)
	@if [ -z "$(name)" ]; then echo "Error: name is required. Usage: make g:migration name=xxx"; exit 1; fi
	sea-orm-cli migrate -d db/migrations generate $(name)

g\:entity: ## Generate entity models from database
	@set -a && [ -f .env ] && . ./.env || true && set +a && \
	DATABASE_URL=$${DATABASE_URL:-postgres://$$POSTGRES_USER:$$POSTGRES_PASSWORD@$$POSTGRES_HOST:$$POSTGRES_PORT/$$POSTGRES_DB}; \
	echo "Generating entities from $$DATABASE_URL..."; \
	sea-orm-cli generate entity --database-url "$$DATABASE_URL" -o db/models

# ─── Generators ───────────────────────────────────
g\:env: ## Generate .env from .env.example
	@cp .env.example .env
	@echo "Created .env from .env.example"

g\:feature: ## Generate a new feature module (usage: make g:feature name=xxx)
	@if [ -z "$(name)" ]; then echo "Error: name is required. Usage: make g:feature name=xxx"; exit 1; fi
	cargo run -p g -- feature $(name)

g\:resource: ## Generate a NestJS-like CRUD resource (usage: make g:resource name=xxx)
	@if [ -z "$(name)" ]; then echo "Error: name is required. Usage: make g:resource name=xxx"; exit 1; fi
	cargo run -p g -- resource $(name)

g\:middleware: ## Generate a custom middleware (usage: make g:middleware name=xxx)
	@if [ -z "$(name)" ]; then echo "Error: name is required. Usage: make g:middleware name=xxx"; exit 1; fi
	cargo run -p g -- middleware $(name)

g\:extractor: ## Generate a custom extractor (usage: make g:extractor name=xxx)
	@if [ -z "$(name)" ]; then echo "Error: name is required. Usage: make g:extractor name=xxx"; exit 1; fi
	cargo run -p g -- extractor $(name)

