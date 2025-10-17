/// Authentication and authorization for PesaBit
/// 
/// This library handles JWT tokens, PIN hashing, session management, and auth middleware.
/// It ensures secure user authentication across all services.

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    async_trait,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use shared_errors::{AppError, Result};
use shared_types::{KycTier, PhoneNumber, UserId};
use uuid::Uuid;

/// JWT token claims structure
/// Contains user information needed by all services
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// User's phone number
    pub phone: String,
    /// User's KYC verification tier (affects transaction limits)
    pub kyc_tier: KycTier,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expires at (Unix timestamp)  
    pub exp: i64,
}

/// Authentication token pair (access + refresh tokens)
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64, // seconds until access token expires
    pub token_type: String, // "Bearer"
}

/// JWT token service for creating and verifying tokens
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
}

impl JwtService {
    /// Create new JWT service with secret key
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_token_expiry: Duration::minutes(15), // Short-lived access tokens
            refresh_token_expiry: Duration::days(7),    // Longer refresh tokens
        }
    }

    /// Generate access and refresh token pair for authenticated user
    pub fn generate_tokens(
        &self,
        user_id: UserId,
        phone: &PhoneNumber,
        kyc_tier: KycTier,
    ) -> Result<TokenResponse> {
        let now = Utc::now();

        // Access token (short-lived)
        let access_claims = Claims {
            sub: user_id.to_string(),
            phone: phone.0.clone(),
            kyc_tier: kyc_tier.clone(),
            iat: now.timestamp(),
            exp: (now + self.access_token_expiry).timestamp(),
        };

        let access_token = encode(&Header::default(), &access_claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Token generation failed: {}", e)))?;

        // Refresh token (longer-lived, simpler claims)
        let refresh_claims = Claims {
            sub: user_id.to_string(),
            phone: phone.0.clone(),
            kyc_tier,
            iat: now.timestamp(),
            exp: (now + self.refresh_token_expiry).timestamp(),
        };

        let refresh_token = encode(&Header::default(), &refresh_claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Token generation failed: {}", e)))?;

        Ok(TokenResponse {
            access_token,
            refresh_token,
            expires_in: self.access_token_expiry.num_seconds(),
            token_type: "Bearer".to_string(),
        })
    }

    /// Verify and decode JWT token
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::default();
        
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::expired_token(),
                _ => AppError::Auth {
                    message: "Invalid token".to_string(),
                },
            })?;

        Ok(token_data.claims)
    }

    /// Generate new access token from valid refresh token
    pub fn refresh_access_token(&self, refresh_token: &str) -> Result<String> {
        let claims = self.verify_token(refresh_token)?;
        
        // Generate new access token with fresh expiry
        let new_claims = Claims {
            exp: (Utc::now() + self.access_token_expiry).timestamp(),
            ..claims
        };

        let access_token = encode(&Header::default(), &new_claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Token refresh failed: {}", e)))?;

        Ok(access_token)
    }
}

/// PIN hashing service using Argon2id (memory-hard, GPU-resistant)
pub struct PinService;

impl PinService {
    /// Hash a user's PIN for secure storage
    /// Uses Argon2id which is resistant to GPU attacks
    pub fn hash_pin(pin: &str) -> Result<String> {
        let salt: [u8; 32] = rand::thread_rng().gen();
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(pin.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("PIN hashing failed: {}", e)))?;

        Ok(password_hash.to_string())
    }

    /// Verify a PIN against stored hash
    pub fn verify_pin(pin: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid hash format: {}", e)))?;

        let argon2 = Argon2::default();
        
        match argon2.verify_password(pin.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Authenticated user context extracted from JWT
/// This is available in all protected endpoints
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: UserId,
    pub phone: PhoneNumber,
    pub kyc_tier: KycTier,
}

/// Axum extractor to get authenticated user from Authorization header
/// Usage: async fn handler(auth_user: AuthUser) -> impl IntoResponse
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        // Extract Bearer token from Authorization header
        let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
            .await
            .map_err(|_| AppError::Auth {
                message: "Missing or invalid authorization header".to_string(),
            })?;

        // Get JWT service from environment (in real app, inject via state)
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key".to_string());
        let jwt_service = JwtService::new(&jwt_secret);

        // Verify token and extract claims
        let claims = jwt_service.verify_token(bearer.token())?;

        // Parse user ID
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Auth {
                message: "Invalid user ID in token".to_string(),
            })?;

        // Parse phone number
        let phone = PhoneNumber::new(claims.phone)
            .map_err(|_| AppError::Auth {
                message: "Invalid phone number in token".to_string(),
            })?;

        Ok(AuthUser {
            user_id: UserId(user_id),
            phone,
            kyc_tier: claims.kyc_tier,
        })
    }
}

/// Session record for refresh token management
#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub user_id: UserId,
    pub refresh_token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub device_fingerprint: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// OTP (One-Time Password) service for phone verification
pub struct OtpService;

impl OtpService {
    /// Generate a 6-digit OTP code
    pub fn generate_code() -> String {
        format!("{:06}", rand::thread_rng().gen_range(100000..999999))
    }

    /// Verify OTP code (in real implementation, check against stored code + expiry)
    pub fn verify_code(submitted: &str, stored: &str, expires_at: DateTime<Utc>) -> bool {
        Utc::now() < expires_at && submitted == stored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_hashing() {
        let pin = "1234";
        let hash = PinService::hash_pin(pin).unwrap();
        
        assert!(PinService::verify_pin(pin, &hash).unwrap());
        assert!(!PinService::verify_pin("5678", &hash).unwrap());
    }

    #[test]
    fn test_jwt_generation_and_verification() {
        let jwt_service = JwtService::new("test-secret");
        let user_id = UserId::new();
        let phone = PhoneNumber::new("+254712345678".to_string()).unwrap();
        
        let tokens = jwt_service.generate_tokens(user_id, &phone, KycTier::Tier1).unwrap();
        let claims = jwt_service.verify_token(&tokens.access_token).unwrap();
        
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.phone, phone.0);
    }

    #[test]
    fn test_otp_generation() {
        let code = OtpService::generate_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }
}