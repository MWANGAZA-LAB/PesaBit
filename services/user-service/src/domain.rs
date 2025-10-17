/// Domain models and business logic for user management
/// 
/// This module defines the core business entities and rules for user operations.
/// All business logic and validation is centralized here.

use serde::{Deserialize, Serialize};
use shared_types::*;
use uuid::Uuid;
use validator::Validate;

/// Request to register a new user
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    /// Phone number in E.164 format (e.g., "+254712345678")
    #[validate(regex = "PHONE_REGEX")]
    pub phone_number: String,
}

/// Response after initiating registration (OTP sent)
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub message: String,
    /// Session token for OTP verification
    pub verification_token: String,
}

/// Request to verify OTP code
#[derive(Debug, Deserialize, Validate)]
pub struct VerifyOtpRequest {
    /// Token from registration response
    pub verification_token: String,
    /// 6-digit OTP code from SMS
    #[validate(length(min = 6, max = 6))]
    pub otp_code: String,
    /// User's chosen 4-6 digit PIN
    #[validate(length(min = 4, max = 6))]
    pub pin: String,
    /// User's full name (optional)
    #[validate(length(max = 100))]
    pub full_name: Option<String>,
    /// Preferred Lightning username (will become username@pesa.co.ke)
    #[validate(length(min = 3, max = 30), regex = "USERNAME_REGEX")]
    pub lightning_username: String,
}

/// Response after successful OTP verification
#[derive(Debug, Serialize)]
pub struct VerifyOtpResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user: UserProfile,
}

/// Request to login with existing credentials
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    /// User's phone number
    #[validate(regex = "PHONE_REGEX")]
    pub phone_number: String,
    /// User's PIN
    #[validate(length(min = 4, max = 6))]
    pub pin: String,
}

/// Response after successful login
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user: UserProfile,
}

/// Request to refresh access token
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Response with new access token
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
}

/// User profile information
#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: UserId,
    pub phone_number: PhoneNumber,
    pub lightning_username: String,
    pub lightning_address: LightningAddress,
    pub full_name: Option<String>,
    pub kyc_status: KycStatus,
    pub kyc_tier: KycTier,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request to update user profile
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    /// Updated full name
    #[validate(length(max = 100))]
    pub full_name: Option<String>,
}

/// Lightning address response
#[derive(Debug, Serialize)]
pub struct LightningAddressResponse {
    pub lightning_address: LightningAddress,
    pub username: String,
    pub qr_code_url: Option<String>, // URL to QR code image
}

/// Complete user entity (internal domain model)
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub phone_number: PhoneNumber,
    pub pin_hash: String,
    pub lightning_username: String,
    pub full_name: Option<String>,
    pub kyc_status: KycStatus,
    pub kyc_tier: KycTier,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// OTP verification record
#[derive(Debug, Clone)]
pub struct OtpCode {
    pub id: Uuid,
    pub phone_number: PhoneNumber,
    pub code_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub used: bool,
    pub attempts: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// User session for authentication
#[derive(Debug, Clone)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: UserId,
    pub refresh_token_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub device_fingerprint: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// Validation regex patterns
lazy_static::lazy_static! {
    /// Phone number validation (E.164 format)
    static ref PHONE_REGEX: regex::Regex = regex::Regex::new(r"^\+[1-9]\d{1,14}$").unwrap();
    
    /// Username validation (alphanumeric + underscore, no spaces)
    static ref USERNAME_REGEX: regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9_]{3,30}$").unwrap();
}

/// Business rules and validation
impl User {
    /// Create new user from registration data
    pub fn new(
        phone_number: PhoneNumber,
        pin_hash: String,
        lightning_username: String,
        full_name: Option<String>,
    ) -> Self {
        Self {
            id: UserId::new(),
            phone_number,
            pin_hash,
            lightning_username,
            full_name,
            kyc_status: KycStatus::None,
            kyc_tier: KycTier::Tier0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Get Lightning address for this user
    pub fn lightning_address(&self) -> LightningAddress {
        LightningAddress::new(&self.lightning_username, "pesa.co.ke")
    }

    /// Convert to public profile (removes sensitive data)
    pub fn to_profile(&self) -> UserProfile {
        UserProfile {
            id: self.id,
            phone_number: self.phone_number.clone(),
            lightning_username: self.lightning_username.clone(),
            lightning_address: self.lightning_address(),
            full_name: self.full_name.clone(),
            kyc_status: self.kyc_status.clone(),
            kyc_tier: self.kyc_tier.clone(),
            created_at: self.created_at,
        }
    }

    /// Check if username is available (business rule)
    pub fn is_valid_username(username: &str) -> bool {
        USERNAME_REGEX.is_match(username) && 
        !RESERVED_USERNAMES.contains(&username.to_lowercase().as_str())
    }
}

/// Reserved usernames that users cannot register
const RESERVED_USERNAMES: &[&str] = &[
    "admin", "support", "help", "api", "www", "mail", "ftp", 
    "root", "system", "pesa", "bitcoin", "lightning", "mpesa",
    "safaricom", "test", "demo", "null", "undefined"
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_number_validation() {
        assert!(PHONE_REGEX.is_match("+254712345678")); // Kenyan mobile
        assert!(PHONE_REGEX.is_match("+1234567890"));   // US number
        assert!(!PHONE_REGEX.is_match("254712345678"));  // Missing +
        assert!(!PHONE_REGEX.is_match("+0712345678"));   // Starts with 0
        assert!(!PHONE_REGEX.is_match("invalid"));       // Not a number
    }

    #[test]
    fn test_username_validation() {
        assert!(User::is_valid_username("john123"));
        assert!(User::is_valid_username("alice_doe"));
        assert!(!User::is_valid_username("admin"));      // Reserved
        assert!(!User::is_valid_username("ab"));         // Too short
        assert!(!User::is_valid_username("user name")); // Contains space
        assert!(!User::is_valid_username("user@name")); // Invalid character
    }

    #[test]
    fn test_lightning_address_generation() {
        let user = User::new(
            PhoneNumber::new("+254712345678".to_string()).unwrap(),
            "pin_hash".to_string(),
            "john".to_string(),
            Some("John Doe".to_string()),
        );
        
        assert_eq!(user.lightning_address().0, "john@pesa.co.ke");
    }
}