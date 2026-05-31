run:
	cargo watch -x run

db\:up:
	sea-orm-cli migrate -d database/migrations up

db\:down:
	sea-orm-cli migrate -d database/migrations down

db\:migration:
	sea-orm-cli migrate -d database/migrations generate $(name)

db\:entity:
	@if [ -f .env ]; then \
		export $$(grep -v '^#' .env | xargs); \
	fi; \
	if [ -z "$$DATABASE_URL" ]; then \
		echo "DATABASE_URL is not set"; \
		exit 1; \
	fi; \
	echo "Generating entities from $$DATABASE_URL..."; \
	sea-orm-cli generate entity --database-url "$$DATABASE_URL" -o database/models

env\:setup:
	@if [ ! -f .env ]; then \
		cp .env.example .env; \
		echo "Created .env from .env.example"; \
	else \
		echo ".env already exists"; \
	fi

.PHONY: run db\:up db\:down db\:migration db\:entity env\:setup
