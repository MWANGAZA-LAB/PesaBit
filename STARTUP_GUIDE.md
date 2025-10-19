# PesaBit Startup Guide

## Quick Start

PesaBit is now ready to run! Here's how to start it:

### Prerequisites
1. **Rust** - Make sure Rust is installed: `rustup --version`
2. **PostgreSQL** - Start PostgreSQL on port 5432
3. **Redis** - Start Redis on port 6379

### Option 1: Using Docker (Recommended)
```bash
# Start PostgreSQL
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=pesabit_dev_password postgres:16-alpine

# Start Redis  
docker run -d -p 6379:6379 redis:7-alpine

# Start PesaBit
docker-compose -f docker-compose.staging.yml up --build
```

### Option 2: Direct Rust Execution
```bash
# Set environment variables
export APP_ENV=development
export SERVICE_PORT=3000
export DATABASE_URL=postgresql://pesabit:pesabit_dev_password@localhost:5432/pesabit
export REDIS_URL=redis://:redis_dev_password@localhost:6379
export JWT_SECRET=dev_jwt_secret_key_change_in_production

# Start API Gateway
cargo run --bin api-gateway
```

### Option 3: PowerShell (Windows)
```powershell
# Run the startup script
.\start-pesabit-simple.ps1
```

## Access Points

Once running, PesaBit will be available at:

- **API Gateway**: http://localhost:3000
- **Health Check**: http://localhost:3000/health
- **API Documentation**: http://localhost:3000/docs
- **User Service**: http://localhost:3000/v1/users/*
- **Payment Service**: http://localhost:3000/v1/payments/*

## Services

PesaBit consists of these microservices:

1. **API Gateway** (Port 3000) - Main entry point
2. **User Service** (Port 8001) - User management and authentication
3. **Payment Service** (Port 8002) - M-Pesa and Lightning Network payments

## Troubleshooting

### If services won't start:
1. Check if PostgreSQL is running: `docker ps | grep postgres`
2. Check if Redis is running: `docker ps | grep redis`
3. Check Rust compilation: `cargo check --workspace`
4. Check logs: `cargo run --bin api-gateway --verbose`

### If you get connection errors:
1. Ensure database is accessible
2. Check environment variables
3. Verify port availability

## Development

For development, you can run individual services:

```bash
# API Gateway only
cargo run --bin api-gateway

# User Service only  
cargo run --bin user-service

# Payment Service only
cargo run --bin payment-service
```

## Production

For production deployment, use the Kubernetes scripts:

```bash
# Deploy to production
./scripts/deploy-production-k8s.sh deploy

# Verify deployment
./scripts/deploy-production-k8s.sh verify
```

## Support

If you encounter issues:
1. Check the logs
2. Verify all prerequisites are installed
3. Ensure ports are available
4. Check the troubleshooting section above

PesaBit is ready to revolutionize fintech in Kenya! 🇰🇪
