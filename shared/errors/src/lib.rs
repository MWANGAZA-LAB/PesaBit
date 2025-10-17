/// Centralized error handling for PesaBit
/// 
/// This library provides consistent error types and HTTP response formatting
/// across all services. Each error includes user-friendly messages and proper
/// HTTP status codes.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard API error response format
/// This ensures all services return errors in the same JSON structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Main application error type that covers all possible errors
/// Each variant handles a different category of problems
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// User-related errors (invalid phone number, user not found, etc.)
    #[error("User error: {message}")]
    User { message: String },

    /// Authentication errors (invalid token, expired session, etc.) 
    #[error("Authentication error: {message}")]
    Auth { message: String },

    /// Payment processing errors (insufficient funds, Lightning failures, etc.)
    #[error("Payment error: {message}")]
    Payment { message: String },

    /// M-Pesa integration errors (API failures, invalid responses, etc.)
    #[error("M-Pesa error: {message}")]
    Mpesa { message: String },

    /// Lightning Network errors (routing failures, channel issues, etc.)
    #[error("Lightning error: {message}")]
    Lightning { message: String },

    /// Database operation errors (connection failures, constraint violations, etc.)
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    /// External service errors (exchange rate APIs, SMS providers, etc.)
    #[error("External service error: {message}")]
    ExternalService { message: String },

    /// Input validation errors (invalid amounts, malformed requests, etc.)
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// Rate limiting errors (too many requests)
    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String },

    /// Internal server errors (unexpected failures)
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    /// Get appropriate HTTP status code for each error type
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::User { .. } => StatusCode::BAD_REQUEST,
            AppError::Auth { .. } => StatusCode::UNAUTHORIZED,
            AppError::Payment { .. } => StatusCode::BAD_REQUEST,
            AppError::Mpesa { .. } => StatusCode::BAD_GATEWAY,
            AppError::Lightning { .. } => StatusCode::BAD_GATEWAY,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalService { .. } => StatusCode::BAD_GATEWAY,
            AppError::Validation { .. } => StatusCode::BAD_REQUEST,
            AppError::RateLimit { .. } => StatusCode::TOO_MANY_REQUESTS,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get error code for client-side error handling
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::User { .. } => "USER_ERROR",
            AppError::Auth { .. } => "AUTH_ERROR", 
            AppError::Payment { .. } => "PAYMENT_ERROR",
            AppError::Mpesa { .. } => "MPESA_ERROR",
            AppError::Lightning { .. } => "LIGHTNING_ERROR",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::ExternalService { .. } => "EXTERNAL_SERVICE_ERROR",
            AppError::Validation { .. } => "VALIDATION_ERROR",
            AppError::RateLimit { .. } => "RATE_LIMIT_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Get user-friendly error message (safe to show to end users)
    pub fn user_message(&self) -> String {
        match self {
            AppError::User { message } => message.clone(),
            AppError::Auth { .. } => "Authentication failed. Please log in again.".to_string(),
            AppError::Payment { message } => message.clone(),
            AppError::Mpesa { .. } => "M-Pesa service temporarily unavailable. Please try again.".to_string(),
            AppError::Lightning { .. } => "Lightning payment failed. Please try again.".to_string(),
            AppError::Database(_) => "Service temporarily unavailable. Please try again.".to_string(),
            AppError::ExternalService { .. } => "External service unavailable. Please try again.".to_string(),
            AppError::Validation { message } => message.clone(),
            AppError::RateLimit { message } => message.clone(),
            AppError::Internal(_) => "Internal server error. Please contact support.".to_string(),
        }
    }
}

/// Convert AppError to HTTP response
/// This allows errors to be returned directly from Axum handlers
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            error: self.error_code().to_string(),
            message: self.user_message(),
            code: self.error_code().to_string(),
            details: None,
        };

        // Log the error for debugging (but don't expose internal details to users)
        tracing::error!("API Error: {:?}", self);

        (status_code, Json(error_response)).into_response()
    }
}

/// Convenient result type for all operations
pub type Result<T> = std::result::Result<T, AppError>;

/// Helper functions to create specific error types quickly
impl AppError {
    pub fn user_not_found() -> Self {
        AppError::User {
            message: "User not found".to_string(),
        }
    }

    pub fn invalid_phone_number() -> Self {
        AppError::User {
            message: "Invalid phone number format".to_string(),
        }
    }

    pub fn insufficient_balance(required: i64, available: i64) -> Self {
        AppError::Payment {
            message: format!(
                "Insufficient balance. Required: {} sats, Available: {} sats",
                required, available
            ),
        }
    }

    pub fn invalid_pin() -> Self {
        AppError::Auth {
            message: "Invalid PIN".to_string(),
        }
    }

    pub fn expired_token() -> Self {
        AppError::Auth {
            message: "Token expired".to_string(),
        }
    }

    pub fn mpesa_timeout() -> Self {
        AppError::Mpesa {
            message: "M-Pesa transaction timed out".to_string(),
        }
    }

    pub fn lightning_route_not_found() -> Self {
        AppError::Lightning {
            message: "No route found for Lightning payment".to_string(),
        }
    }

    pub fn invalid_amount() -> Self {
        AppError::Validation {
            message: "Invalid amount".to_string(),
        }
    }

    pub fn rate_limit_exceeded() -> Self {
        AppError::RateLimit {
            message: "Too many requests. Please wait and try again.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            AppError::user_not_found().status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::invalid_pin().status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::rate_limit_exceeded().status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[test]
    fn test_user_messages() {
        let error = AppError::insufficient_balance(1000, 500);
        assert!(error.user_message().contains("Insufficient balance"));
        assert!(error.user_message().contains("1000"));
        assert!(error.user_message().contains("500"));
    }
}