.PHONY: help dev build test clean docker-up docker-down migrate-up migrate-down

help: ## Display this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

dev: ## Run the application in development mode
	cargo run -p api

build: ## Build the application in release mode
	cargo build --release

test: ## Run all tests
	cargo test --workspace

clean: ## Clean build artifacts
	cargo clean

docker-up: ## Start PostgreSQL using docker-compose
	docker-compose up -d

docker-down: ## Stop PostgreSQL
	docker-compose down

migrate-up: ## Run database migrations (optional, migrations run automatically on startup)
	sqlx migrate run

migrate-down: ## Revert last database migration
	sqlx migrate revert

init: ## Initialize and start the project (start DB, create database, run migrations, start server)
	@echo "🚀 正在初始化项目..."
	@echo "📦 启动 PostgreSQL..."
	$(MAKE) docker-up
	@echo "⏳ 等待 PostgreSQL 就绪..."
	@sleep 3
	@echo "✅ PostgreSQL 已就绪"
	@echo "🎯 启动应用（自动创建数据库和运行迁移）..."
	$(MAKE) dev

run: ## Build and run the application
	cargo build && cargo run -p api

