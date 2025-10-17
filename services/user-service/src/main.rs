/// User Service for PesaBit
/// 
/// This service handles all user-related operations:
/// - User registration with phone number
/// - SMS OTP verification  
/// - PIN-based authentication
/// - Profile management
/// - Lightning address creation

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, patch, post},
    Router,
};
use shared_auth::{AuthUser, JwtService, OtpService, PinService};
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

use domain::*;
use repository::*;
use service::*;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging first
    init_tracing("user-service");

    // Connect to database
    let db = shared_database::init().await?;
    
    // Create services
    let user_repository = Arc::new(UserRepository::new(db.clone()));
    let otp_repository = Arc::new(OtpRepository::new(db.clone()));
    let session_repository = Arc::new(SessionRepository::new(db.clone()));
    let sms_client = Arc::new(SmsClient::new());
    
    let user_service = Arc::new(UserService::new(
        user_repository,
        otp_repository, 
        session_repository,
        sms_client,
    ));

    let state = AppState {
        user_service,
        db,
    };

    // Build router with all endpoints
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/auth/register", post(register))
        .route("/auth/verify-otp", post(verify_otp))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh_token))
        .route("/users/me", get(get_profile))
        .route("/users/me", patch(update_profile))
        .route("/users/:user_id/lightning-address", get(get_lightning_address))
        .layer(CorsLayer::permissive()) // Allow cross-origin requests
        .layer(shared_tracing::trace_id_layer()) // Add trace IDs to requests
        .with_state(state);

    // Start server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("User service listening on {}", addr);
    
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Server error: {}", e)))?;

    Ok(())
}

/// Health check endpoint for monitoring
#[instrument]
async fn health_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
    // Check database health
    let db_health = shared_database::health_check(&state.db).await?;
    
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "user-service",
        "database": db_health,
        "timestamp": chrono::Utc::now()
    })))
}

/// Register new user with phone number
#[instrument(skip(state))]
async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>> {
    let response = state.user_service.register(request).await?;
    Ok(Json(response))
}

/// Verify SMS OTP code and complete registration
#[instrument(skip(state))]
async fn verify_otp(
    State(state): State<AppState>,
    Json(request): Json<VerifyOtpRequest>,
) -> Result<Json<VerifyOtpResponse>> {
    let response = state.user_service.verify_otp(request).await?;
    Ok(Json(response))
}

/// Login with phone number and PIN
#[instrument(skip(state))]
async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    let response = state.user_service.login(request).await?;
    Ok(Json(response))
}

/// Refresh access token using refresh token
#[instrument(skip(state))]
async fn refresh_token(
    State(state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>> {
    let response = state.user_service.refresh_token(request).await?;
    Ok(Json(response))
}

/// Get current user profile (requires authentication)
#[instrument(skip(state))]
async fn get_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UserProfile>> {
    let profile = state.user_service.get_profile(auth_user.user_id).await?;
    Ok(Json(profile))
}

/// Update user profile (requires authentication)
#[instrument(skip(state))]
async fn update_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>> {
    let profile = state.user_service.update_profile(auth_user.user_id, request).await?;
    Ok(Json(profile))
}

/// Get user's Lightning address
#[instrument(skip(state))]
async fn get_lightning_address(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<LightningAddressResponse>> {
    let user_id = user_id.parse::<uuid::Uuid>()
        .map_err(|_| AppError::Validation { message: "Invalid user ID".to_string() })?;
    
    let response = state.user_service.get_lightning_address(UserId(user_id)).await?;
    Ok(Json(response))
}