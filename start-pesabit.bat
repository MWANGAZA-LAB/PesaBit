@echo off
echo 🚀 Starting PesaBit...
echo.

REM Set environment variables
set APP_ENV=development
set SERVICE_PORT=3000
set DATABASE_URL=postgresql://pesabit:pesabit_dev_password@localhost:5432/pesabit
set REDIS_URL=redis://:redis_dev_password@localhost:6379
set JWT_SECRET=dev_jwt_secret_key_change_in_production
set USER_SERVICE_URL=http://localhost:8001
set PAYMENT_SERVICE_URL=http://localhost:8002
set RATE_LIMIT_REQUESTS_PER_MINUTE=100
set CORS_ORIGINS=http://localhost:5173,http://localhost:3000
set RUST_LOG=info,api_gateway=debug,user_service=debug,payment_service=debug

echo 📋 Environment variables set
echo.

echo 🔍 Checking for required services...
echo.

REM Check if PostgreSQL is running
powershell -Command "try { Invoke-WebRequest -Uri 'http://localhost:5432' -TimeoutSec 2 -ErrorAction Stop | Out-Null; Write-Host '✅ PostgreSQL is running' -ForegroundColor Green } catch { Write-Host '❌ PostgreSQL not running. Please start PostgreSQL on port 5432' -ForegroundColor Red; Write-Host '   You can use: docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=pesabit_dev_password postgres:16-alpine' -ForegroundColor Yellow }"

REM Check if Redis is running
powershell -Command "try { Invoke-WebRequest -Uri 'http://localhost:6379' -TimeoutSec 2 -ErrorAction Stop | Out-Null; Write-Host '✅ Redis is running' -ForegroundColor Green } catch { Write-Host '❌ Redis not running. Please start Redis on port 6379' -ForegroundColor Red; Write-Host '   You can use: docker run -d -p 6379:6379 redis:7-alpine' -ForegroundColor Yellow }"

echo.
echo 🎯 Starting PesaBit API Gateway...
echo    URL: http://localhost:3000
echo    Health: http://localhost:3000/health
echo    Docs: http://localhost:3000/docs
echo.

REM Start API Gateway
start "PesaBit API Gateway" cmd /k "cargo run --bin api-gateway"

echo ⏳ Waiting for API Gateway to start...
timeout /t 10 /nobreak > nul

REM Test API Gateway
powershell -Command "try { $response = Invoke-WebRequest -Uri 'http://localhost:3000/health' -TimeoutSec 5; Write-Host '✅ API Gateway is running!' -ForegroundColor Green; Write-Host '   Status:' $response.StatusCode -ForegroundColor Cyan } catch { Write-Host '❌ API Gateway failed to start' -ForegroundColor Red; Write-Host '   Check the logs above for errors' -ForegroundColor Yellow }"

echo.
echo 🎉 PesaBit is starting up!
echo.
echo 📚 Available endpoints:
echo    • Health Check: http://localhost:3000/health
echo    • API Documentation: http://localhost:3000/docs
echo    • User Service: http://localhost:3000/v1/users/*
echo    • Payment Service: http://localhost:3000/v1/payments/*
echo.
echo 🛑 To stop PesaBit, close the API Gateway window
echo.
echo 📖 For more information, see README.md
echo.
pause
