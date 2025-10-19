/// Service client for making HTTP requests to other microservices
/// 
/// This module handles communication between the API gateway and the backend services
/// (user-service and payment-service). It provides a clean interface for forwarding
/// requests and handling service-to-service communication.

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri},
    response::Response,
};
use reqwest::Client;
use shared_errors::{AppError, Result};
use std::collections::HashMap;
use tracing::{error, info, instrument, warn};

/// Trait for service clients to enable polymorphism
#[async_trait::async_trait]
pub trait ServiceClient: Send + Sync {
    async fn forward_request(&self, request: Request<Body>) -> Result<Response>;
    async fn health_check(&self) -> Result<serde_json::Value>;
}

/// Client for communicating with user service
pub struct UserServiceClient {
    base_url: String,
    http_client: Client,
}

impl UserServiceClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http_client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl ServiceClient for UserServiceClient {
    #[instrument(skip(self, request))]
    async fn forward_request(&self, request: Request<Body>) -> Result<Response> {
        forward_request_impl(&self.http_client, &self.base_url, request).await
    }

    async fn health_check(&self) -> Result<serde_json::Value> {
        health_check_impl(&self.http_client, &self.base_url).await
    }
}

/// Client for communicating with payment service
pub struct PaymentServiceClient {
    base_url: String,
    http_client: Client,
}

impl PaymentServiceClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http_client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl ServiceClient for PaymentServiceClient {
    #[instrument(skip(self, request))]
    async fn forward_request(&self, request: Request<Body>) -> Result<Response> {
        forward_request_impl(&self.http_client, &self.base_url, request).await
    }

    async fn health_check(&self) -> Result<serde_json::Value> {
        health_check_impl(&self.http_client, &self.base_url).await
    }
}

/// Generic request forwarding implementation
async fn forward_request_impl(
    http_client: &Client,
    base_url: &str,
    request: Request<Body>,
) -> Result<Response> {
    let (parts, body) = request.into_parts();
    
    // Convert Axum request to reqwest request
    let method = convert_method(&parts.method);
    let url = format!("{}{}", base_url, parts.uri.path_and_query().unwrap_or(&parts.uri.path().parse().unwrap()));
    
    // Convert headers
    let mut req_headers = reqwest::header::HeaderMap::new();
    for (name, value) in parts.headers.iter() {
        if let (Ok(name), Ok(value)) = (
            reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes()),
            reqwest::header::HeaderValue::from_bytes(value.as_bytes())
        ) {
            req_headers.insert(name, value);
        }
    }

    // Extract body
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return Err(AppError::Internal(anyhow::anyhow!("Body read error: {}", e)));
        }
    };

    // Make request to service
    let response = http_client
        .request(method, &url)
        .headers(req_headers)
        .body(body_bytes.to_vec())
        .send()
        .await
        .map_err(|e| {
            error!("Service request failed: {}", e);
            AppError::ExternalService {
                message: format!("Service unavailable: {}", e),
            }
        })?;

    // Convert response back to Axum response
    let status = StatusCode::from_u16(response.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    
    let mut response_builder = Response::builder().status(status);
    
    // Copy headers
    for (name, value) in response.headers().iter() {
        if let Ok(header_name) = axum::http::HeaderName::from_bytes(name.as_str().as_bytes()) {
            if let Ok(header_value) = axum::http::HeaderValue::from_bytes(value.as_bytes()) {
                response_builder = response_builder.header(header_name, header_value);
            }
        }
    }

    // Get response body
    let response_body = response.bytes().await.map_err(|e| {
        error!("Failed to read response body: {}", e);
        AppError::Internal(anyhow::anyhow!("Response body read error: {}", e))
    })?;

    let axum_response = response_builder
        .body(Body::from(response_body))
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Response build error: {}", e)))?;

    Ok(axum_response)
}

/// Generic health check implementation
async fn health_check_impl(
    http_client: &Client,
    base_url: &str,
) -> Result<serde_json::Value> {
    let url = format!("{}/health", base_url);
    
    let response = http_client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| {
            warn!("Health check failed for {}: {}", base_url, e);
            AppError::ExternalService {
                message: format!("Health check failed: {}", e),
            }
        })?;

    if response.status().is_success() {
        let health_data: serde_json::Value = response.json().await.map_err(|e| {
            AppError::ExternalService {
                message: format!("Invalid health check response: {}", e),
            }
        })?;
        Ok(health_data)
    } else {
        Err(AppError::ExternalService {
            message: format!("Service unhealthy: {}", response.status()),
        })
    }
}

/// Convert Axum method to reqwest method
fn convert_method(method: &Method) -> reqwest::Method {
    match *method {
        Method::GET => reqwest::Method::GET,
        Method::POST => reqwest::Method::POST,
        Method::PUT => reqwest::Method::PUT,
        Method::DELETE => reqwest::Method::DELETE,
        Method::PATCH => reqwest::Method::PATCH,
        Method::HEAD => reqwest::Method::HEAD,
        Method::OPTIONS => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET, // Default fallback
    }
}

/// Circuit breaker for handling service failures gracefully
pub struct CircuitBreaker {
    failure_count: std::sync::atomic::AtomicU32,
    last_failure_time: std::sync::Mutex<Option<std::time::Instant>>,
    failure_threshold: u32,
    recovery_time: std::time::Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_time: std::time::Duration) -> Self {
        Self {
            failure_count: std::sync::atomic::AtomicU32::new(0),
            last_failure_time: std::sync::Mutex::new(None),
            failure_threshold,
            recovery_time,
        }
    }

    /// Check if circuit breaker should allow requests
    pub fn should_allow_request(&self) -> bool {
        let current_failures = self.failure_count.load(std::sync::atomic::Ordering::Relaxed);
        
        if current_failures < self.failure_threshold {
            return true; // Circuit closed, allow requests
        }

        // Circuit is open, check if enough time has passed for recovery
        if let Ok(guard) = self.last_failure_time.lock() {
            if let Some(last_failure) = *guard {
                if last_failure.elapsed() >= self.recovery_time {
                    // Try half-open state
                    return true;
                }
            }
        }

        false // Circuit open, block requests
    }

    /// Record successful request
    pub fn record_success(&self) {
        self.failure_count.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record failed request  
    pub fn record_failure(&self) {
        self.failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        if let Ok(mut guard) = self.last_failure_time.lock() {
            *guard = Some(std::time::Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(3, std::time::Duration::from_secs(60));
        
        // Initially should allow requests
        assert!(breaker.should_allow_request());
        
        // Record failures
        breaker.record_failure();
        breaker.record_failure();
        assert!(breaker.should_allow_request()); // Still under threshold
        
        breaker.record_failure(); // Hit threshold
        assert!(!breaker.should_allow_request()); // Circuit open
        
        // Success should reset
        breaker.record_success();
        assert!(breaker.should_allow_request());
    }
}