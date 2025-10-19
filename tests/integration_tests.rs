/// Integration tests for PesaBit
/// 
/// These tests verify that all services work together correctly
/// and that the API endpoints function as expected.

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use serde_json::json;
use shared_config::AppConfig;
use shared_errors::Result;
use std::collections::HashMap;
use tower::ServiceExt;

mod common;

use common::*;

#[tokio::test]
async fn test_health_check_endpoints() -> Result<()> {
    let test_app = create_test_app().await?;
    
    // Test API Gateway health check
    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .body(Body::empty())?;
    
    let response = test_app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body(), usize::MAX).await?;
    let health: serde_json::Value = serde_json::from_slice(&body)?;
    
    assert_eq!(health["service"], "api-gateway");
    assert_eq!(health["status"], "healthy");
    
    Ok(())
}

#[tokio::test]
async fn test_user_registration_flow() -> Result<()> {
    let test_app = create_test_app().await?;
    
    // Step 1: Register user
    let register_request = json!({
        "phone_number": "+254712345678",
        "full_name": "Test User"
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&register_request)?))?;
    
    let response = test_app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body(), usize::MAX).await?;
    let register_response: serde_json::Value = serde_json::from_slice(&body)?;
    
    assert!(register_response["verification_token"].is_string());
    assert_eq!(register_response["message"], "OTP sent to your phone");
    
    // Step 2: Verify OTP
    let verify_request = json!({
        "verification_token": register_response["verification_token"],
        "otp_code": "123456",
        "pin": "1234"
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/verify-otp")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&verify_request)?))?;
    
    let response = test_app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body(), usize::MAX).await?;
    let verify_response: serde_json::Value = serde_json::from_slice(&body)?;
    
    assert!(verify_response["access_token"].is_string());
    assert!(verify_response["refresh_token"].is_string());
    assert_eq!(verify_response["token_type"], "Bearer");
    
    Ok(())
}

#[tokio::test]
async fn test_user_login_flow() -> Result<()> {
    let test_app = create_test_app().await?;
    
    // First register a user
    let register_request = json!({
        "phone_number": "+254712345679",
        "full_name": "Login Test User"
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&register_request)?))?;
    
    let response = test_app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body(), usize::MAX).await?;
    let register_response: serde_json::Value = serde_json::from_slice(&body)?;
    
    // Verify OTP
    let verify_request = json!({
        "verification_token": register_response["verification_token"],
        "otp_code": "123456",
        "pin": "1234"
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/verify-otp")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&verify_request)?))?;
    
    let response = test_app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);
    
    // Now test login
    let login_request = json!({
        "phone_number": "+254712345679",
        "pin": "1234"
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_request)?))?;
    
    let response = test_app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body(), usize::MAX).await?;
    let login_response: serde_json::Value = serde_json::from_slice(&body)?;
    
    assert!(login_response["access_token"].is_string());
    assert!(login_response["refresh_token"].is_string());
    
    Ok(())
}

#[tokio::test]
async fn test_protected_endpoints_require_auth() -> Result<()> {
    let test_app = create_test_app().await?;
    
    // Try to access protected endpoint without auth
    let request = Request::builder()
        .method(Method::GET)
        .uri("/v1/balance")
        .body(Body::empty())?;
    
    let response = test_app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    Ok(())
}

#[tokio::test]
async fn test_rate_limiting() -> Result<()> {
    let test_app = create_test_app().await?;
    
    // Make many requests quickly to trigger rate limiting
    for i in 0..150 {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(Body::empty())?;
        
        let response = test_app.clone().oneshot(request).await?;
        
        if i < 100 {
            assert_eq!(response.status(), StatusCode::OK);
        } else {
            // Should be rate limited
            assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_security_headers() -> Result<()> {
    let test_app = create_test_app().await?;
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .body(Body::empty())?;
    
    let response = test_app.clone().oneshot(request).await?;
    
    let headers = response.headers();
    
    // Check security headers are present
    assert!(headers.contains_key("content-security-policy"));
    assert!(headers.contains_key("strict-transport-security"));
    assert!(headers.contains_key("x-frame-options"));
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("x-xss-protection"));
    assert!(headers.contains_key("referrer-policy"));
    assert!(headers.contains_key("permissions-policy"));
    
    // Check that server information is hidden
    assert!(!headers.contains_key("server"));
    assert!(!headers.contains_key("x-powered-by"));
    
    Ok(())
}

#[tokio::test]
async fn test_cors_headers() -> Result<()> {
    let test_app = create_test_app().await?;
    
    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/v1/auth/register")
        .header("origin", "http://localhost:5173")
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())?;
    
    let response = test_app.clone().oneshot(request).await?;
    
    let headers = response.headers();
    
    // Check CORS headers
    assert!(headers.contains_key("access-control-allow-origin"));
    assert!(headers.contains_key("access-control-allow-methods"));
    assert!(headers.contains_key("access-control-allow-headers"));
    assert!(headers.contains_key("access-control-max-age"));
    
    Ok(())
}

#[tokio::test]
async fn test_invalid_requests_are_rejected() -> Result<()> {
    let test_app = create_test_app().await?;
    
    // Test with suspicious user agent
    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .header("user-agent", "sqlmap/1.0")
        .body(Body::empty())?;
    
    let response = test_app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test with suspicious content type
    let request = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/register")
        .header("content-type", "application/x-php")
        .body(Body::from("malicious content"))?;
    
    let response = test_app.clone().oneshot(request).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    Ok(())
}

#[tokio::test]
async fn test_configuration_validation() -> Result<()> {
    // Test development config
    let dev_config = AppConfig::from_env()?;
    assert!(dev_config.is_development());
    assert!(!dev_config.is_production());
    
    // Test production config validation
    let mut prod_config = dev_config.clone();
    prod_config.app.rust_env = "production".to_string();
    
    // Should fail with default secrets
    assert!(prod_config.validate_production().is_err());
    
    // Should pass with proper secrets
    prod_config.jwt.secret = "a-very-long-secret-key-for-production-use-only-32-chars-minimum".to_string();
    prod_config.mpesa.consumer_key = "real_consumer_key".to_string();
    prod_config.sms.api_key = "real_sms_key".to_string();
    prod_config.ssl.enabled = true;
    
    assert!(prod_config.validate_production().is_ok());
    
    Ok(())
}
