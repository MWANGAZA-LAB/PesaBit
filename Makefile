.PHONY: help dev build start stop clean logs test migrate format lint check docs

# Default target
help: ## Show this help message
	@echo "PesaBit Development Commands"
	@echo "============================"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

# Development
dev: ## Start development environment with hot reload
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml up --build

dev-detached: ## Start development environment in background
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d --build

# Production-like build
build: ## Build all services for production
	docker-compose build

start: ## Start all services
	docker-compose up -d

stop: ## Stop all services
	docker-compose down

restart: ## Restart all services
	docker-compose restart

# Database operations
migrate: ## Run database migrations
	docker-compose exec postgres psql -U pesabit -d pesabit -c "SELECT 'Database ready for migrations';"
	# Note: In a real setup, you'd run sqlx migrate here

migrate-test: ## Run migrations on test database
	docker-compose exec postgres psql -U pesabit -d pesabit_test -c "SELECT 'Test database ready';"

reset-db: ## Reset the database (WARNING: destroys data)
	docker-compose down -v postgres
	docker-compose up -d postgres
	sleep 10
	$(MAKE) migrate

# Logs and monitoring
logs: ## Show logs from all services
	docker-compose logs -f

logs-api: ## Show API gateway logs
	docker-compose logs -f api-gateway

logs-user: ## Show user service logs
	docker-compose logs -f user-service

logs-payment: ## Show payment service logs
	docker-compose logs -f payment-service

logs-frontend: ## Show frontend logs
	docker-compose logs -f frontend

# Testing
test: ## Run all tests
	docker-compose exec user-service cargo test
	docker-compose exec payment-service cargo test
	docker-compose exec api-gateway cargo test
	docker-compose exec frontend npm test

test-user: ## Run user service tests
	docker-compose exec user-service cargo test

test-payment: ## Run payment service tests
	docker-compose exec payment-service cargo test

test-api: ## Run API gateway tests
	docker-compose exec api-gateway cargo test

test-frontend: ## Run frontend tests
	docker-compose exec frontend npm test

# Code quality
format: ## Format all Rust code
	docker-compose exec user-service cargo fmt
	docker-compose exec payment-service cargo fmt
	docker-compose exec api-gateway cargo fmt

lint: ## Run linting on all code
	docker-compose exec user-service cargo clippy -- -D warnings
	docker-compose exec payment-service cargo clippy -- -D warnings
	docker-compose exec api-gateway cargo clippy -- -D warnings
	docker-compose exec frontend npm run lint

check: ## Run all checks (format, lint, test)
	$(MAKE) format
	$(MAKE) lint
	$(MAKE) test

# Utilities
clean: ## Clean up containers and images
	docker-compose down --remove-orphans
	docker system prune -f
	docker volume prune -f

shell-postgres: ## Open PostgreSQL shell
	docker-compose exec postgres psql -U pesabit -d pesabit

shell-redis: ## Open Redis CLI
	docker-compose exec redis redis-cli -a redis_dev_password

shell-user: ## Open shell in user service container
	docker-compose exec user-service /bin/bash

shell-payment: ## Open shell in payment service container
	docker-compose exec payment-service /bin/bash

shell-api: ## Open shell in API gateway container
	docker-compose exec api-gateway /bin/bash

shell-frontend: ## Open shell in frontend container
	docker-compose exec frontend /bin/sh

# Documentation
docs: ## Generate and serve documentation
	@echo "Opening development services..."
	@echo "Frontend:      http://localhost:5173"
	@echo "API Gateway:   http://localhost:3000"
	@echo "PostgreSQL:    localhost:5432 (pesabit/pesabit_dev_password)"
	@echo "Redis:         localhost:6379 (password: redis_dev_password)"
	@echo "PgAdmin:       http://localhost:5050 (admin@pesabit.dev/admin)"
	@echo "Redis Commander: http://localhost:8081"

status: ## Show status of all services
	docker-compose ps

# Environment setup
setup: ## Initial setup for development
	@echo "Setting up PesaBit development environment..."
	@echo "1. Building Docker images..."
	$(MAKE) build
	@echo "2. Starting services..."
	$(MAKE) start
	@echo "3. Waiting for services to be ready..."
	sleep 30
	@echo "4. Running initial setup..."
	$(MAKE) migrate
	@echo ""
	@echo "✅ Setup complete!"
	$(MAKE) docs