/// API Gateway for PesaBit
/// 
/// This is the main entry point for all client requests. It handles:
/// - Request routing to appropriate microservices
/// - Authentication and authorization
/// - Rate limiting and DDoS protection  
/// - Request/response logging and monitoring
/// - CORS handling for web clients
/// - Load balancing between service instances

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get},
    Router,
};
use shared_auth::AuthUser;
use shared_config::AppConfig;
use shared_errors::{AppError, Result};
use shared_security::{create_cors_layer, request_validation_middleware, security_headers_middleware};
use shared_tracing::init_tracing;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info, instrument, warn};

mod middleware_auth;
mod middleware_rate_limit;
mod service_client;

use middleware_auth::*;
use middleware_rate_limit::*;
use service_client::*;

/// Application state for the API gateway
#[derive(Clone)]
pub struct AppState {
    pub user_service_client: Arc<UserServiceClient>,
    pub payment_service_client: Arc<PaymentServiceClient>,
    pub rate_limiter: Arc<RateLimiter>,
    pub config: AppConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_tracing("api-gateway");

    // Load configuration
    let config = AppConfig::from_env()?;
    
    // Validate configuration for production
    if config.is_production() {
        config.validate_production()?;
    }

    // Create service clients
    let user_service_client = Arc::new(UserServiceClient::new(&config.services.user_service_url));
    let payment_service_client = Arc::new(PaymentServiceClient::new(&config.services.payment_service_url));
    
    // Create rate limiter
    let rate_limiter = Arc::new(RateLimiter::new(&config.redis.url).await?);

    let state = AppState {
        user_service_client,
        payment_service_client,
        rate_limiter,
        config: config.clone(),
    };

    // Build router with security middleware
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/v1/*path", any(route_to_services))
        .layer(create_cors_layer(&config))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .layer(middleware::from_fn(request_validation_middleware))
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(shared_tracing::trace_id_layer())
        .with_state(state);

    // Start server
    let addr = format!("0.0.0.0:{}", config.app.service_port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("🚀 PesaBit API Gateway listening on {}", addr);
    info!("📋 API Documentation available at http://{}/docs", addr);
    
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Server error: {}", e)))?;

    Ok(())
}

/// Health check endpoint with service status
#[instrument]
async fn health_check(State(state): State<AppState>) -> Result<impl IntoResponse> {
    // Check if downstream services are healthy
    let user_service_health = state.user_service_client.health_check().await;
    let payment_service_health = state.payment_service_client.health_check().await;
    
    let overall_status = if user_service_health.is_ok() && payment_service_health.is_ok() {
        "healthy"
    } else {
        "degraded"
    };
    
    let response = serde_json::json!({
        "status": overall_status,
        "service": "api-gateway",
        "timestamp": chrono::Utc::now(),
        "services": {
            "user_service": user_service_health.is_ok(),
            "payment_service": payment_service_health.is_ok()
        },
        "version": env!("CARGO_PKG_VERSION")
    });

    Ok((StatusCode::OK, axum::Json(response)))
}

/// Main routing function that forwards requests to appropriate services
#[instrument(skip(state, request))]
async fn route_to_services(
    State(state): State<AppState>,
    mut request: Request<Body>,
) -> Result<Response> {
    let path = request.uri().path();
    
    // Determine which service to route to based on path
    let (service_client, service_path) = match path {
        // User service routes
        path if path.starts_with("/v1/auth/") => {
            (&state.user_service_client as &dyn ServiceClient, path.strip_prefix("/v1").unwrap())
        }
        path if path.starts_with("/v1/users/") => {
            (&state.user_service_client as &dyn ServiceClient, path.strip_prefix("/v1").unwrap())
        }
        
        // Payment service routes
        path if path.starts_with("/v1/balance") => {
            (&state.payment_service_client as &dyn ServiceClient, path.strip_prefix("/v1").unwrap())
        }
        path if path.starts_with("/v1/deposits/") => {
            (&state.payment_service_client as &dyn ServiceClient, path.strip_prefix("/v1").unwrap())
        }
        path if path.starts_with("/v1/withdrawals/") => {
            (&state.payment_service_client as &dyn ServiceClient, path.strip_prefix("/v1").unwrap())
        }
        path if path.starts_with("/v1/lightning/") => {
            (&state.payment_service_client as &dyn ServiceClient, path.strip_prefix("/v1").unwrap())
        }
        path if path.starts_with("/v1/transactions") => {
            (&state.payment_service_client as &dyn ServiceClient, path.strip_prefix("/v1").unwrap())
        }
        path if path.starts_with("/v1/exchange-rates/") => {
            (&state.payment_service_client as &dyn ServiceClient, path.strip_prefix("/v1").unwrap())
        }
        
        _ => {
            warn!("Unknown route: {}", path);
            return Ok((
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({
                    "error": "NOT_FOUND",
                    "message": "API endpoint not found",
                    "path": path
                }))
            ).into_response());
        }
    };

    // Update request URI to remove /v1 prefix
    let new_uri = format!("{}{}", 
        service_path,
        request.uri().query().map(|q| format!("?{}", q)).unwrap_or_default()
    );
    *request.uri_mut() = new_uri.parse().map_err(|e| {
        error!("Failed to parse URI: {}", e);
        AppError::Internal(anyhow::anyhow!("Invalid URI: {}", e))
    })?;

    // Forward request to service
    match service_client.forward_request(request).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("Service request failed: {:?}", e);
            Ok((
                StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::json!({
                    "error": "SERVICE_UNAVAILABLE", 
                    "message": "Upstream service temporarily unavailable"
                }))
            ).into_response())
        }
    }
}