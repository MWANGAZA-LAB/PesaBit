# PesaBit⚡ - Lightning-M-Pesa Bridge

> Lightning-fast Bitcoin payments for Kenya. Send money globally in 2 minutes for <1% fee.

## What is PesaBit?

PesaBit connects Kenya's M-Pesa mobile money system with Bitcoin's Lightning Network, allowing anyone with a phone to:

- **Receive global payments instantly** - Get tips from anywhere in the world to your phone
- **Send money internationally** - 2-minute transfers for under 1% fee vs 8-15% traditional remittance
- **No crypto knowledge needed** - Uses familiar M-Pesa interface
- **Lightning address included** - Get your own `yourname@pesa.co.ke` payment address

## Quick Start

### Prerequisites
- Rust 1.70+
- Docker & Docker Compose
- PostgreSQL 15+
- Node.js 18+ (for frontend)

### Development Setup

```bash
# Clone and setup
git clone https://github.com/MWANGAZA-LAB/PesaBit.git
cd PesaBit

# Start database and Redis
docker-compose up -d postgres redis

# Run database migrations
cargo install sqlx-cli
sqlx database create
sqlx migrate run

# Start all services
cargo run --bin api-gateway
```

### Project Structure

```
PesaBit/
├── services/                 # Microservices
│   ├── api-gateway/         # Main API entry point - routes requests
│   ├── user-service/        # Handles user accounts, auth, profiles
│   ├── payment-service/     # M-Pesa and Lightning payments
│   └── notification-service/ # SMS and push notifications
├── shared/                  # Common libraries
│   ├── database/           # DB connections and utilities
│   ├── auth/               # JWT tokens and session management
│   ├── types/              # Common data types (UserId, Amount, etc.)
│   ├── errors/             # Error handling across services
│   └── tracing/            # Logging and monitoring setup
├── frontend/               # React PWA (Progressive Web App)
├── migrations/             # Database schema changes
└── infrastructure/         # Docker, Kubernetes configs
```

## How It Works

1. **User signs up** with phone number (like M-Pesa)
2. **Deposit money** via M-Pesa STK Push → automatically converts to Bitcoin Lightning
3. **Send payments** using Lightning addresses (like email) or QR codes
4. **Receive money** from anywhere in the world instantly  
5. **Withdraw to M-Pesa** anytime - converts back to Kenyan Shillings

## API Documentation

- **Base URL:** `https://api.pesa.co.ke/v1`
- **Authentication:** Bearer JWT tokens
- **Docs:** Available at `/docs` when running locally

### Key Endpoints

```bash
# User Management
POST /auth/register     # Sign up with phone number
POST /auth/verify-otp   # Verify SMS code
POST /auth/login        # Login with phone + PIN

# Payments  
POST /deposits/mpesa    # Add money via M-Pesa
POST /lightning/pay     # Send Lightning payment
GET  /balance          # Check wallet balance
POST /withdrawals/mpesa # Cash out to M-Pesa
```

## Technology Stack

### Backend (Rust)
- **Axum** - Fast, type-safe web framework
- **PostgreSQL** - Reliable database for financial data
- **Redis** - Session storage and caching
- **LDK** - Lightning Development Kit for Bitcoin payments
- **SQLx** - Compile-time checked SQL queries

### Frontend (React)
- **React 18** - Modern UI framework
- **Tailwind CSS** - Utility-first styling
- **PWA** - Works like native mobile app
- **Zustand** - Simple state management

### Infrastructure
- **Docker** - Containerized services
- **Kubernetes** - Production deployment
- **GitHub Actions** - CI/CD pipeline
- **Prometheus + Grafana** - Monitoring

## Security & Compliance

- **Bank-level security** - AES-256 encryption, HSMs for keys
- **KYC compliance** - Tiered verification (ID scan, proof of address)
- **Regulatory compliant** - Follows Kenyan CMA and CBK guidelines
- **Audit logs** - All transactions tracked for compliance

## Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Make changes and add tests
4. Run tests: `cargo test`
5. Submit pull request

## License

Licensed under MIT License. See [LICENSE](LICENSE) for details.

## Support

- **Email:** support@pesa.co.ke
- **Telegram:** @PesaBitSupport  
- **Documentation:** [docs.pesa.co.ke](https://docs.pesa.co.ke)

---

**Disclaimer:** PesaBit is experimental software. Only send amounts you can afford to lose. Not financial advice.