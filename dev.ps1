# PesaBit Development Script for Windows PowerShell
# Provides convenient commands for development workflow

param(
    [Parameter(Position=0)]
    [string]$Command,
    [Parameter(Position=1)]
    [string]$Service
)

# Colors for output
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Check if Docker is running
function Test-Docker {
    try {
        docker info | Out-Null
        return $true
    }
    catch {
        Write-Error "Docker is not running. Please start Docker and try again."
        exit 1
    }
}

# Show help
function Show-Help {
    Write-Host "PesaBit Development Script" -ForegroundColor Cyan
    Write-Host "=========================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\dev.ps1 <command> [service]"
    Write-Host ""
    Write-Host "Commands:"
    Write-Host "  setup           - Initial project setup"
    Write-Host "  start           - Start development environment"
    Write-Host "  stop            - Stop all services"
    Write-Host "  restart         - Restart all services"
    Write-Host "  logs [service]  - Show logs (optionally for specific service)"
    Write-Host "  test [service]  - Run tests (optionally for specific service)"
    Write-Host "  lint            - Run code linting"
    Write-Host "  format          - Format all code"
    Write-Host "  clean           - Clean up containers and volumes"
    Write-Host "  shell <service> - Open shell in service container"
    Write-Host "  db              - Open PostgreSQL shell"
    Write-Host "  redis           - Open Redis CLI"
    Write-Host "  status          - Show service status"
    Write-Host "  monitoring      - Start monitoring stack"
    Write-Host "  build           - Build all services"
    Write-Host ""
    Write-Host "Services: api-gateway, user-service, payment-service, frontend"
}

# Initial setup
function Initialize-Setup {
    Write-Info "Setting up PesaBit development environment..."
    
    Test-Docker
    
    # Create .env file if it doesn't exist
    if (!(Test-Path ".env")) {
        Write-Info "Creating .env file from template..."
        Copy-Item ".env.template" ".env"
        Write-Success ".env file created. Please review and update as needed."
    }
    
    # Build and start services
    Write-Info "Building Docker images..."
    docker-compose build
    
    Write-Info "Starting services..."
    docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d
    
    Write-Info "Waiting for services to be ready..."
    Start-Sleep -Seconds 30
    
    Write-Success "Setup complete! Services available at:"
    Write-Host "  - Frontend: http://localhost:5173"
    Write-Host "  - API Gateway: http://localhost:3000"
    Write-Host "  - PgAdmin: http://localhost:5050"
    Write-Host "  - Redis Commander: http://localhost:8081"
}

# Start development environment
function Start-Environment {
    Write-Info "Starting PesaBit development environment..."
    Test-Docker
    docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d
    Write-Success "Development environment started!"
}

# Stop all services
function Stop-Services {
    Write-Info "Stopping PesaBit services..."
    docker-compose down
    Write-Success "All services stopped."
}

# Restart services
function Restart-Services {
    Write-Info "Restarting PesaBit services..."
    Stop-Services
    Start-Environment
}

# Show logs
function Show-Logs {
    param([string]$ServiceName)
    
    if ([string]::IsNullOrEmpty($ServiceName)) {
        docker-compose logs -f
    }
    else {
        docker-compose logs -f $ServiceName
    }
}

# Run tests
function Invoke-Tests {
    param([string]$ServiceName)
    
    if ([string]::IsNullOrEmpty($ServiceName)) {
        Write-Info "Running all tests..."
        docker-compose exec user-service cargo test
        docker-compose exec payment-service cargo test  
        docker-compose exec api-gateway cargo test
        docker-compose exec frontend npm test
    }
    else {
        switch ($ServiceName) {
            { $_ -in @("user-service", "payment-service", "api-gateway") } {
                Write-Info "Running tests for $ServiceName..."
                docker-compose exec $ServiceName cargo test
                break
            }
            "frontend" {
                Write-Info "Running tests for frontend..."
                docker-compose exec frontend npm test
                break
            }
            default {
                Write-Error "Unknown service: $ServiceName"
                exit 1
            }
        }
    }
}

# Run linting
function Invoke-Lint {
    Write-Info "Running linting..."
    docker-compose exec user-service cargo clippy -- -D warnings
    docker-compose exec payment-service cargo clippy -- -D warnings
    docker-compose exec api-gateway cargo clippy -- -D warnings
    docker-compose exec frontend npm run lint
    Write-Success "Linting complete!"
}

# Format code
function Format-Code {
    Write-Info "Formatting code..."
    docker-compose exec user-service cargo fmt
    docker-compose exec payment-service cargo fmt
    docker-compose exec api-gateway cargo fmt
    try {
        docker-compose exec frontend npm run format
    }
    catch {
        # Ignore if format script doesn't exist
    }
    Write-Success "Code formatting complete!"
}

# Clean up
function Remove-Everything {
    $response = Read-Host "This will remove all containers, images, and volumes. Are you sure? (y/N)"
    if ($response -match "^[Yy]$") {
        Write-Info "Cleaning up..."
        docker-compose down --remove-orphans
        docker system prune -af
        docker volume prune -f
        Write-Success "Cleanup complete!"
    }
    else {
        Write-Info "Cleanup cancelled."
    }
}

# Open shell in service
function Open-Shell {
    param([string]$ServiceName)
    
    if ([string]::IsNullOrEmpty($ServiceName)) {
        Write-Error "Please specify a service name"
        exit 1
    }
    
    switch ($ServiceName) {
        { $_ -in @("user-service", "payment-service", "api-gateway") } {
            docker-compose exec $ServiceName /bin/bash
            break
        }
        "frontend" {
            docker-compose exec $ServiceName /bin/sh
            break
        }
        default {
            Write-Error "Unknown service: $ServiceName"
            exit 1
        }
    }
}

# Open database shell
function Open-Database {
    Write-Info "Opening PostgreSQL shell..."
    docker-compose exec postgres psql -U pesabit -d pesabit
}

# Open Redis CLI
function Open-Redis {
    Write-Info "Opening Redis CLI..."
    docker-compose exec redis redis-cli -a redis_dev_password
}

# Show status
function Show-Status {
    Write-Info "Service Status:"
    docker-compose ps
}

# Start monitoring
function Start-Monitoring {
    Write-Info "Starting monitoring stack..."
    docker-compose -f docker-compose.monitoring.yml up -d
    Write-Success "Monitoring stack started!"
    Write-Host "  - Prometheus: http://localhost:9090"
    Write-Host "  - Grafana: http://localhost:3001 (admin/admin)"
    Write-Host "  - Jaeger: http://localhost:16686"
}

# Build services
function Build-Services {
    Write-Info "Building all services..."
    docker-compose build
    Write-Success "Build complete!"
}

# Main command handler
switch ($Command.ToLower()) {
    "setup" { Initialize-Setup }
    "start" { Start-Environment }
    "stop" { Stop-Services }
    "restart" { Restart-Services }
    "logs" { Show-Logs -ServiceName $Service }
    "test" { Invoke-Tests -ServiceName $Service }
    "lint" { Invoke-Lint }
    "format" { Format-Code }
    "clean" { Remove-Everything }
    "shell" { Open-Shell -ServiceName $Service }
    "db" { Open-Database }
    "redis" { Open-Redis }
    "status" { Show-Status }
    "monitoring" { Start-Monitoring }
    "build" { Build-Services }
    { $_ -in @("help", "--help", "-h", "") } { Show-Help }
    default {
        Write-Error "Unknown command: $Command"
        Show-Help
        exit 1
    }
}