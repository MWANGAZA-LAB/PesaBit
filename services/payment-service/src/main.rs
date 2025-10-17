/// Payment Service for PesaBit
/// 
/// This service handles all financial operations:
/// - M-Pesa deposits (KES → Bitcoin)
/// - M-Pesa withdrawals (Bitcoin → KES)  
/// - Lightning Network payments (send/receive)
/// - Wallet balance management
/// - Exchange rate conversions

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use shared_auth::AuthUser;
use shared_database::DatabaseConfig;
use shared_errors::{AppError, Result};
use shared_tracing::init_tracing;
use shared_types::*;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, instrument};

mod domain;
mod repository;
mod service;
mod integrations;

use domain::*;
use repository::*;
use service::*;
use integrations::*;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub payment_service: Arc<PaymentService>,
    pub wallet_service: Arc<WalletService>,
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_tracing("payment-service");

    // Connect to database
    let db = shared_database::init().await?;
    
    // Create repositories
    let wallet_repository = Arc::new(WalletRepository::new(db.clone()));
    let transaction_repository = Arc::new(TransactionRepository::new(db.clone()));
    let exchange_rate_repository = Arc::new(ExchangeRateRepository::new(db.clone()));
    
    // Create external service clients
    let mpesa_client = Arc::new(MpesaClient::new());
    let lightning_client = Arc::new(LightningClient::new());
    let exchange_rate_client = Arc::new(ExchangeRateClient::new());
    
    // Create services
    let wallet_service = Arc::new(WalletService::new(wallet_repository.clone()));
    
    let payment_service = Arc::new(PaymentService::new(
        wallet_repository,
        transaction_repository,
        exchange_rate_repository,
        mpesa_client,
        lightning_client,
        exchange_rate_client,
    ));

    let state = AppState {
        payment_service,
        wallet_service,
        db,
    };

    // Build router with all endpoints
    let app = Router::new()
        .route("/health", get(health_check))
        
        // Wallet endpoints
        .route("/balance", get(get_balance))
        .route("/wallets/:user_id", post(create_wallet))
        
        // Deposit endpoints (M-Pesa → Bitcoin)
        .route("/deposits/mpesa", post(initiate_mpesa_deposit))
        .route("/deposits/mpesa/callback", post(mpesa_deposit_callback))
        
        // Withdrawal endpoints (Bitcoin → M-Pesa)
        .route("/withdrawals/mpesa", post(initiate_mpesa_withdrawal))
        
        // Lightning payments
        .route("/lightning/invoice", post(create_lightning_invoice))
        .route("/lightning/pay", post(pay_lightning_invoice))
        
        // Transaction history
        .route("/transactions", get(get_transaction_history))
        .route("/transactions/:id", get(get_transaction))
        
        // Exchange rates
        .route("/exchange-rates/current", get(get_current_exchange_rate))
        
        .layer(CorsLayer::permissive())
        .layer(shared_tracing::trace_id_layer())
        .with_state(state);

    // Start server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3002".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("Payment service listening on {}", addr);
    
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Server error: {}", e)))?;

    Ok(())
}

/// Health check endpoint
#[instrument]
async fn health_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
    let db_health = shared_database::health_check(&state.db).await?;
    
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "payment-service",
        "database": db_health,
        "timestamp": chrono::Utc::now()
    })))
}

/// Get user's wallet balance
#[instrument(skip(state))]
async fn get_balance(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<WalletBalance>> {
    let balance = state.wallet_service.get_balance(auth_user.user_id).await?;
    Ok(Json(balance))
}

/// Create wallet for new user (internal endpoint called by user service)
#[instrument(skip(state))]
async fn create_wallet(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let user_id = user_id.parse::<uuid::Uuid>()
        .map_err(|_| AppError::Validation { message: "Invalid user ID".to_string() })?;
    
    state.wallet_service.create_wallet(UserId(user_id)).await?;
    
    Ok(Json(serde_json::json!({"status": "created"})))
}

/// Initiate M-Pesa deposit (user adds money via M-Pesa)
#[instrument(skip(state))]
async fn initiate_mpesa_deposit(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<MpesaDepositRequest>,
) -> Result<Json<MpesaDepositResponse>> {
    let response = state.payment_service
        .initiate_mpesa_deposit(auth_user.user_id, request)
        .await?;
    Ok(Json(response))
}

/// M-Pesa callback webhook (called by Safaricom when payment completes)
#[instrument(skip(state))]
async fn mpesa_deposit_callback(
    State(state): State<AppState>,
    Json(callback): Json<MpesaCallback>,
) -> Result<Json<serde_json::Value>> {
    state.payment_service.process_mpesa_callback(callback).await?;
    Ok(Json(serde_json::json!({"status": "processed"})))
}

/// Initiate M-Pesa withdrawal (user cashes out Bitcoin to M-Pesa)
#[instrument(skip(state))]
async fn initiate_mpesa_withdrawal(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<MpesaWithdrawalRequest>,
) -> Result<Json<MpesaWithdrawalResponse>> {
    let response = state.payment_service
        .initiate_mpesa_withdrawal(auth_user.user_id, request)
        .await?;
    Ok(Json(response))
}

/// Create Lightning invoice for receiving payment
#[instrument(skip(state))]
async fn create_lightning_invoice(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<CreateInvoiceRequest>,
) -> Result<Json<CreateInvoiceResponse>> {
    let response = state.payment_service
        .create_lightning_invoice(auth_user.user_id, request)
        .await?;
    Ok(Json(response))
}

/// Pay Lightning invoice (user sends money via Lightning)
#[instrument(skip(state))]
async fn pay_lightning_invoice(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<PayInvoiceRequest>,
) -> Result<Json<PayInvoiceResponse>> {
    let response = state.payment_service
        .pay_lightning_invoice(auth_user.user_id, request)
        .await?;
    Ok(Json(response))
}

/// Get user's transaction history
#[instrument(skip(state))]
async fn get_transaction_history(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(params): Query<TransactionHistoryParams>,
) -> Result<Json<TransactionHistoryResponse>> {
    let response = state.payment_service
        .get_transaction_history(auth_user.user_id, params)
        .await?;
    Ok(Json(response))
}

/// Get specific transaction details
#[instrument(skip(state))]
async fn get_transaction(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(transaction_id): Path<String>,
) -> Result<Json<Transaction>> {
    let transaction_id = transaction_id.parse::<uuid::Uuid>()
        .map_err(|_| AppError::Validation { message: "Invalid transaction ID".to_string() })?;
    
    let transaction = state.payment_service
        .get_transaction(auth_user.user_id, transaction_id)
        .await?;
    Ok(Json(transaction))
}

/// Get current BTC/KES exchange rate
#[instrument(skip(state))]
async fn get_current_exchange_rate(
    State(state): State<AppState>,
) -> Result<Json<ExchangeRate>> {
    let rate = state.payment_service.get_current_exchange_rate().await?;
    Ok(Json(rate))
}