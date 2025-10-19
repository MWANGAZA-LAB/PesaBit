#!/usr/bin/env bash

# PesaBit Development Script
# Provides convenient commands for development workflow

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        log_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
}

# Show help
show_help() {
    echo "PesaBit Development Script"
    echo "========================="
    echo ""
    echo "Usage: ./dev.sh <command>"
    echo ""
    echo "Commands:"
    echo "  setup           - Initial project setup"
    echo "  start           - Start development environment"
    echo "  stop            - Stop all services"
    echo "  restart         - Restart all services"
    echo "  logs [service]  - Show logs (optionally for specific service)"
    echo "  test [service]  - Run tests (optionally for specific service)"
    echo "  lint            - Run code linting"
    echo "  format          - Format all code"
    echo "  clean           - Clean up containers and volumes"
    echo "  shell <service> - Open shell in service container"
    echo "  db              - Open PostgreSQL shell"
    echo "  redis           - Open Redis CLI"
    echo "  status          - Show service status"
    echo "  monitoring      - Start monitoring stack"
    echo "  build           - Build all services"
    echo ""
    echo "Services: api-gateway, user-service, payment-service, frontend"
}

# Initial setup
setup() {
    log_info "Setting up PesaBit development environment..."
    
    check_docker
    
    # Create .env file if it doesn't exist
    if [ ! -f .env ]; then
        log_info "Creating .env file from template..."
        cp .env.template .env
        log_success ".env file created. Please review and update as needed."
    fi
    
    # Build and start services
    log_info "Building Docker images..."
    docker-compose build
    
    log_info "Starting services..."
    docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d
    
    log_info "Waiting for services to be ready..."
    sleep 30
    
    log_success "Setup complete! Services available at:"
    echo "  - Frontend: http://localhost:5173"
    echo "  - API Gateway: http://localhost:3000"
    echo "  - PgAdmin: http://localhost:5050"
    echo "  - Redis Commander: http://localhost:8081"
}

# Start development environment
start() {
    log_info "Starting PesaBit development environment..."
    check_docker
    docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d
    log_success "Development environment started!"
}

# Stop all services
stop() {
    log_info "Stopping PesaBit services..."
    docker-compose down
    log_success "All services stopped."
}

# Restart services
restart() {
    log_info "Restarting PesaBit services..."
    stop
    start
}

# Show logs
show_logs() {
    local service=$1
    if [ -z "$service" ]; then
        docker-compose logs -f
    else
        docker-compose logs -f "$service"
    fi
}

# Run tests
run_tests() {
    local service=$1
    
    if [ -z "$service" ]; then
        log_info "Running all tests..."
        docker-compose exec user-service cargo test
        docker-compose exec payment-service cargo test  
        docker-compose exec api-gateway cargo test
        docker-compose exec frontend npm test
    else
        case $service in
            "user-service"|"payment-service"|"api-gateway")
                log_info "Running tests for $service..."
                docker-compose exec "$service" cargo test
                ;;
            "frontend")
                log_info "Running tests for frontend..."
                docker-compose exec frontend npm test
                ;;
            *)
                log_error "Unknown service: $service"
                exit 1
                ;;
        esac
    fi
}

# Run linting
run_lint() {
    log_info "Running linting..."
    docker-compose exec user-service cargo clippy -- -D warnings
    docker-compose exec payment-service cargo clippy -- -D warnings
    docker-compose exec api-gateway cargo clippy -- -D warnings
    docker-compose exec frontend npm run lint
    log_success "Linting complete!"
}

# Format code
format_code() {
    log_info "Formatting code..."
    docker-compose exec user-service cargo fmt
    docker-compose exec payment-service cargo fmt
    docker-compose exec api-gateway cargo fmt
    docker-compose exec frontend npm run format 2>/dev/null || true
    log_success "Code formatting complete!"
}

# Clean up
cleanup() {
    log_warning "This will remove all containers, images, and volumes. Are you sure? (y/N)"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        log_info "Cleaning up..."
        docker-compose down --remove-orphans
        docker system prune -af
        docker volume prune -f
        log_success "Cleanup complete!"
    else
        log_info "Cleanup cancelled."
    fi
}

# Open shell in service
open_shell() {
    local service=$1
    if [ -z "$service" ]; then
        log_error "Please specify a service name"
        exit 1
    fi
    
    case $service in
        "user-service"|"payment-service"|"api-gateway")
            docker-compose exec "$service" /bin/bash
            ;;
        "frontend")
            docker-compose exec "$service" /bin/sh
            ;;
        *)
            log_error "Unknown service: $service"
            exit 1
            ;;
    esac
}

# Open database shell
open_db() {
    log_info "Opening PostgreSQL shell..."
    docker-compose exec postgres psql -U pesabit -d pesabit
}

# Open Redis CLI
open_redis() {
    log_info "Opening Redis CLI..."
    docker-compose exec redis redis-cli -a redis_dev_password
}

# Show status
show_status() {
    log_info "Service Status:"
    docker-compose ps
}

# Start monitoring
start_monitoring() {
    log_info "Starting monitoring stack..."
    docker-compose -f docker-compose.monitoring.yml up -d
    log_success "Monitoring stack started!"
    echo "  - Prometheus: http://localhost:9090"
    echo "  - Grafana: http://localhost:3001 (admin/admin)"
    echo "  - Jaeger: http://localhost:16686"
}

# Build services
build_services() {
    log_info "Building all services..."
    docker-compose build
    log_success "Build complete!"
}

# Main command handler
case "${1:-}" in
    "setup")
        setup
        ;;
    "start")
        start
        ;;
    "stop")
        stop
        ;;
    "restart")
        restart
        ;;
    "logs")
        show_logs "$2"
        ;;
    "test")
        run_tests "$2"
        ;;
    "lint")
        run_lint
        ;;
    "format")
        format_code
        ;;
    "clean")
        cleanup
        ;;
    "shell")
        open_shell "$2"
        ;;
    "db")
        open_db
        ;;
    "redis")
        open_redis
        ;;
    "status")
        show_status
        ;;
    "monitoring")
        start_monitoring
        ;;
    "build")
        build_services
        ;;
    "help"|"--help"|"-h"|"")
        show_help
        ;;
    *)
        log_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac