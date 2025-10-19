/// Authentication middleware for the API gateway
/// 
/// This module handles JWT token validation and user authentication for protected routes.
/// It extracts user information from tokens and makes it available to downstream services.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use shared_auth::{AuthUser, JwtService};
use shared_errors::{AppError, Result};
use tracing::{info, instrument, warn};

/// Authentication middleware that validates JWT tokens
#[instrument(skip(request, next))]
pub async fn auth_middleware(
    State(state): State<crate::AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response> {
    let path = request.uri().path();
    
    // Skip authentication for public endpoints
    if is_public_endpoint(path) {
        return Ok(next.run(request).await);
    }

    // Extract and validate JWT token
    match extract_and_validate_token(request.headers()) {
        Ok(user) => {
            // Add user info to request headers for downstream services
            add_user_headers(&mut request, &user);
            Ok(next.run(request).await)
        }
        Err(e) => {
            warn!("Authentication failed for {}: {:?}", path, e);
            Ok(create_auth_error_response())
        }
    }
}

/// Check if endpoint is public (doesn't require authentication)
fn is_public_endpoint(path: &str) -> bool {
    matches!(path, 
        "/health" |
        "/v1/auth/register" |
        "/v1/auth/verify-otp" |
        "/v1/auth/login" |
        "/v1/exchange-rates/current" |
        "/docs" |
        "/docs/"
    )
}

/// Extract JWT token from Authorization header and validate it
fn extract_and_validate_token(headers: &HeaderMap) -> Result<AuthUser> {
    // Get Authorization header
    let auth_header = headers.get("authorization")
        .ok_or_else(|| AppError::Auth {
            message: "Missing authorization header".to_string(),
        })?;

    // Extract Bearer token
    let auth_str = auth_header.to_str()
        .map_err(|_| AppError::Auth {
            message: "Invalid authorization header format".to_string(),
        })?;

    if !auth_str.starts_with("Bearer ") {
        return Err(AppError::Auth {
            message: "Invalid authorization header format".to_string(),
        });
    }

    let token = &auth_str[7..]; // Remove "Bearer " prefix

    // Validate JWT token
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key".to_string());
    let jwt_service = JwtService::new(&jwt_secret);

    let claims = jwt_service.verify_token(token)?;

    // Parse user information from claims
    let user_id = claims.sub.parse()
        .map_err(|_| AppError::Auth {
            message: "Invalid user ID in token".to_string(),
        })?;

    let phone = shared_types::PhoneNumber::new(claims.phone)
        .map_err(|_| AppError::Auth {
            message: "Invalid phone number in token".to_string(),
        })?;

    Ok(AuthUser {
        user_id: shared_types::UserId(user_id),
        phone,
        kyc_tier: claims.kyc_tier,
    })
}

/// Add user information to request headers for downstream services
fn add_user_headers(request: &mut Request, user: &AuthUser) {
    let headers = request.headers_mut();
    
    // Add user ID header
    if let Ok(user_id_header) = user.user_id.to_string().parse() {
        headers.insert("x-user-id", user_id_header);
    }
    
    // Add phone number header
    if let Ok(phone_header) = user.phone.0.parse() {
        headers.insert("x-user-phone", phone_header);
    }
    
    // Add KYC tier header
    let kyc_tier = match user.kyc_tier {
        shared_types::KycTier::Tier0 => "tier0",
        shared_types::KycTier::Tier1 => "tier1", 
        shared_types::KycTier::Tier2 => "tier2",
    };
    if let Ok(kyc_header) = kyc_tier.parse() {
        headers.insert("x-user-kyc-tier", kyc_header);
    }
}

/// Create standardized authentication error response
fn create_auth_error_response() -> Response {
    let error_response = serde_json::json!({
        "error": "UNAUTHORIZED",
        "message": "Authentication required. Please provide a valid access token.",
        "code": "AUTH_REQUIRED"
    });

    (
        StatusCode::UNAUTHORIZED,
        axum::Json(error_response),
    ).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_endpoint_detection() {
        assert!(is_public_endpoint("/health"));
        assert!(is_public_endpoint("/v1/auth/register"));
        assert!(is_public_endpoint("/v1/auth/login"));
        assert!(!is_public_endpoint("/v1/balance"));
        assert!(!is_public_endpoint("/v1/transactions"));
    }

    #[test]
    fn test_bearer_token_extraction() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer test-token-123".parse().unwrap());
        
        // This would need a valid JWT token to fully test
        // In real tests, you'd create a valid token and verify extraction
    }
}