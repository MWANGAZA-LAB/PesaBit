/// Common test utilities for PesaBit integration tests

use axum::{
    body::Body,
    http::{Method, Request},
    Router,
};
use shared_config::AppConfig;
use shared_errors::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceExt;

/// Create a test application for integration tests
pub async fn create_test_app() -> Result<Router> {
    // Load test configuration
    let config = AppConfig::from_env()?;
    
    // Create test database connection
    let db = shared_database::init().await?;
    
    // Create test Redis connection
    let redis_client = redis::Client::open(config.redis.url.as_str())?;
    let redis_conn = redis_client.get_async_connection().await?;
    
    // Create test services
    let user_service_client = Arc::new(TestUserServiceClient::new());
    let payment_service_client = Arc::new(TestPaymentServiceClient::new());
    let rate_limiter = Arc::new(TestRateLimiter::new());
    
    // Build test router
    let app = Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/v1/*path", axum::routing::any(route_to_services))
        .with_state(TestAppState {
            user_service_client,
            payment_service_client,
            rate_limiter,
            config,
            db,
            redis_conn,
        });
    
    Ok(app)
}

/// Test application state
#[derive(Clone)]
pub struct TestAppState {
    pub user_service_client: Arc<TestUserServiceClient>,
    pub payment_service_client: Arc<TestPaymentServiceClient>,
    pub rate_limiter: Arc<TestRateLimiter>,
    pub config: AppConfig,
    pub db: sqlx::PgPool,
    pub redis_conn: redis::aio::Connection,
}

/// Test user service client
#[derive(Clone)]
pub struct TestUserServiceClient;

impl TestUserServiceClient {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn health_check(&self) -> Result<()> {
        Ok(())
    }
    
    pub async fn forward_request(&self, _request: Request<Body>) -> Result<axum::response::Response> {
        // Mock implementation for testing
        Ok(axum::response::Response::builder()
            .status(200)
            .body(Body::from("OK"))
            .unwrap())
    }
}

/// Test payment service client
#[derive(Clone)]
pub struct TestPaymentServiceClient;

impl TestPaymentServiceClient {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn health_check(&self) -> Result<()> {
        Ok(())
    }
    
    pub async fn forward_request(&self, _request: Request<Body>) -> Result<axum::response::Response> {
        // Mock implementation for testing
        Ok(axum::response::Response::builder()
            .status(200)
            .body(Body::from("OK"))
            .unwrap())
    }
}

/// Test rate limiter
#[derive(Clone)]
pub struct TestRateLimiter;

impl TestRateLimiter {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn check_rate_limit(&self, _key: &str, _limit: &TestRateLimit) -> Result<bool> {
        Ok(true)
    }
}

/// Test rate limit configuration
#[derive(Clone)]
pub struct TestRateLimit {
    pub requests_per_minute: u32,
    pub window_seconds: u32,
}

/// Health check endpoint for testing
pub async fn health_check() -> Result<axum::Json<serde_json::Value>> {
    Ok(axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "test-service",
        "timestamp": chrono::Utc::now()
    })))
}

/// Route to services for testing
pub async fn route_to_services(
    axum::extract::State(_state): axum::extract::State<TestAppState>,
    request: Request<Body>,
) -> Result<axum::response::Response> {
    // Mock routing for testing
    Ok(axum::response::Response::builder()
        .status(200)
        .body(Body::from("Mock response"))
        .unwrap())
}
