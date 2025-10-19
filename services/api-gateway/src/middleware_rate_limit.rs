/// Rate limiting middleware to prevent abuse and DDoS attacks
/// 
/// This implements a token bucket algorithm using Redis for distributed rate limiting.
/// Different limits apply based on authentication status and endpoint sensitivity.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::{AsyncCommands, Client};
use shared_errors::{AppError, Result};
use std::sync::Arc;
use tracing::{info, instrument, warn};

/// Rate limiter using Redis for distributed limiting across multiple gateway instances
pub struct RateLimiter {
    redis_client: Client,
}

/// Rate limiting configuration based on request type
#[derive(Debug)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub window_seconds: u32,
}

impl RateLimiter {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let redis_client = Client::open(redis_url)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis connection failed: {}", e)))?;

        // Test connection
        let mut conn = redis_client.get_async_connection().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis connection failed: {}", e)))?;
        
        let _: String = conn.ping().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis ping failed: {}", e)))?;

        info!("Rate limiter connected to Redis");

        Ok(Self { redis_client })
    }

    /// Check if request is within rate limits
    #[instrument(skip(self))]
    pub async fn check_rate_limit(
        &self,
        key: &str,
        limit: &RateLimit,
    ) -> Result<bool> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis connection failed: {}", e)))?;

        // Use sliding window log approach
        let now = chrono::Utc::now().timestamp();
        let window_start = now - limit.window_seconds as i64;

        // Remove expired entries
        let _: i32 = conn.zremrangebyscore(key, "-inf", window_start).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis operation failed: {}", e)))?;

        // Count current requests in window
        let current_count: i32 = conn.zcard(key).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis operation failed: {}", e)))?;

        if current_count >= limit.requests_per_minute as i32 {
            warn!("Rate limit exceeded for key: {}", key);
            return Ok(false);
        }

        // Add current request
        let _: i32 = conn.zadd(key, now, now).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis operation failed: {}", e)))?;

        // Set expiry for cleanup
        let _: bool = conn.expire(key, limit.window_seconds as usize).await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis operation failed: {}", e)))?;

        Ok(true)
    }

    /// Get rate limit configuration based on request
    pub fn get_rate_limit(&self, path: &str, is_authenticated: bool) -> RateLimit {
        match (path, is_authenticated) {
            // Authentication endpoints (more restrictive)
            (path, false) if path.starts_with("/auth/") => RateLimit {
                requests_per_minute: 5,   // 5 login attempts per minute
                window_seconds: 60,
            },
            
            // Payment endpoints (high security)
            (path, true) if path.contains("/deposits/") || path.contains("/withdrawals/") => RateLimit {
                requests_per_minute: 10,  // 10 financial transactions per minute
                window_seconds: 60,
            },
            
            // Lightning payments (medium security)
            (path, true) if path.contains("/lightning/") => RateLimit {
                requests_per_minute: 20,  // 20 Lightning payments per minute
                window_seconds: 60,
            },
            
            // General authenticated endpoints
            (_, true) => RateLimit {
                requests_per_minute: 100, // 100 requests per minute for logged-in users
                window_seconds: 60,
            },
            
            // Public endpoints (most restrictive)
            (_, false) => RateLimit {
                requests_per_minute: 10,  // 10 requests per minute for anonymous users
                window_seconds: 60,
            },
        }
    }

    /// Generate rate limiting key from request
    pub fn generate_key(&self, headers: &HeaderMap, path: &str) -> String {
        // Try to get user ID from Authorization header first
        if let Some(auth_header) = headers.get("authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    // In production, decode JWT to get user ID
                    // For now, use hash of token for key
                    let token_hash = format!("{:x}", md5::compute(&auth_str[7..]));
                    return format!("rate_limit:user:{}", token_hash);
                }
            }
        }

        // Fall back to IP address
        let ip = headers
            .get("x-forwarded-for")
            .or_else(|| headers.get("x-real-ip"))
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown");

        format!("rate_limit:ip:{}:{}", ip, path.replace('/', "_"))
    }
}

/// Middleware function to apply rate limiting to requests
#[instrument(skip(request, next))]
pub async fn rate_limit_middleware(
    State(state): State<crate::AppState>,
    request: Request,
    next: Next,
) -> Result<Response> {
    let path = request.uri().path();
    let headers = request.headers();
    
    // Skip rate limiting for health checks
    if path == "/health" {
        return Ok(next.run(request).await);
    }

    // Check if user is authenticated
    let is_authenticated = headers.get("authorization")
        .map(|h| h.to_str().unwrap_or("").starts_with("Bearer "))
        .unwrap_or(false);

    // Get rate limit config
    let rate_limit = state.rate_limiter.get_rate_limit(path, is_authenticated);
    
    // Generate rate limiting key
    let key = state.rate_limiter.generate_key(headers, path);
    
    // Check rate limit
    match state.rate_limiter.check_rate_limit(&key, &rate_limit).await {
        Ok(true) => {
            // Request allowed
            Ok(next.run(request).await)
        }
        Ok(false) => {
            // Rate limit exceeded
            let error_response = serde_json::json!({
                "error": "RATE_LIMIT_EXCEEDED",
                "message": "Too many requests. Please wait and try again.",
                "retry_after_seconds": rate_limit.window_seconds
            });

            Ok((
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", rate_limit.window_seconds.to_string())],
                axum::Json(error_response),
            ).into_response())
        }
        Err(e) => {
            // Redis error - allow request but log error
            warn!("Rate limiting failed: {:?}", e);
            Ok(next.run(request).await)
        }
    }
}