# PesaBit Quick Start Script
# This script starts PesaBit services for development

Write-Host "🚀 Starting PesaBit..." -ForegroundColor Green

# Set environment variables
$env:APP_ENV = "development"
$env:SERVICE_PORT = "3000"
$env:DATABASE_URL = "postgresql://pesabit:pesabit_dev_password@localhost:5432/pesabit"
$env:REDIS_URL = "redis://:redis_dev_password@localhost:6379"
$env:JWT_SECRET = "dev_jwt_secret_key_change_in_production"
$env:USER_SERVICE_URL = "http://localhost:8001"
$env:PAYMENT_SERVICE_URL = "http://localhost:8002"
$env:RATE_LIMIT_REQUESTS_PER_MINUTE = "100"
$env:CORS_ORIGINS = "http://localhost:5173,http://localhost:3000"
$env:RUST_LOG = "info,api_gateway=debug,user_service=debug,payment_service=debug"

Write-Host "📋 Environment variables set" -ForegroundColor Blue

# Check if required services are running
Write-Host "🔍 Checking for required services..." -ForegroundColor Blue

# Check PostgreSQL
try {
    $pgTest = Invoke-WebRequest -Uri "http://localhost:5432" -TimeoutSec 2 -ErrorAction Stop
    Write-Host "✅ PostgreSQL is running" -ForegroundColor Green
} catch {
    Write-Host "❌ PostgreSQL not running. Please start PostgreSQL on port 5432" -ForegroundColor Red
    Write-Host "   You can use: docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=pesabit_dev_password postgres:16-alpine" -ForegroundColor Yellow
}

# Check Redis
try {
    $redisTest = Invoke-WebRequest -Uri "http://localhost:6379" -TimeoutSec 2 -ErrorAction Stop
    Write-Host "✅ Redis is running" -ForegroundColor Green
} catch {
    Write-Host "❌ Redis not running. Please start Redis on port 6379" -ForegroundColor Red
    Write-Host "   You can use: docker run -d -p 6379:6379 redis:7-alpine" -ForegroundColor Yellow
}

Write-Host "`n🎯 Starting PesaBit API Gateway..." -ForegroundColor Green
Write-Host "   URL: http://localhost:3000" -ForegroundColor Cyan
Write-Host "   Health: http://localhost:3000/health" -ForegroundColor Cyan
Write-Host "   Docs: http://localhost:3000/docs" -ForegroundColor Cyan

# Start API Gateway
Start-Process -FilePath "cargo" -ArgumentList "run", "--bin", "api-gateway" -NoNewWindow

Write-Host "`n⏳ Waiting for API Gateway to start..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# Test API Gateway
try {
    $response = Invoke-WebRequest -Uri "http://localhost:3000/health" -TimeoutSec 5
    Write-Host "✅ API Gateway is running!" -ForegroundColor Green
    Write-Host "   Status: $($response.StatusCode)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ API Gateway failed to start" -ForegroundColor Red
    Write-Host "   Check the logs above for errors" -ForegroundColor Yellow
}

Write-Host "`n🎉 PesaBit is starting up!" -ForegroundColor Green
Write-Host "`n📚 Available endpoints:" -ForegroundColor Blue
Write-Host "   • Health Check: http://localhost:3000/health" -ForegroundColor White
Write-Host "   • API Documentation: http://localhost:3000/docs" -ForegroundColor White
Write-Host "   • User Service: http://localhost:3000/v1/users/*" -ForegroundColor White
Write-Host "   • Payment Service: http://localhost:3000/v1/payments/*" -ForegroundColor White

Write-Host "`n🛑 To stop PesaBit, press Ctrl+C" -ForegroundColor Yellow
Write-Host "`n📖 For more information, see README.md" -ForegroundColor Blue
