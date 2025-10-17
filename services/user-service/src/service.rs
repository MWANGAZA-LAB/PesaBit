/// Business logic service for user operations
/// 
/// This module orchestrates the user registration, authentication, and profile
/// management workflows, coordinating between repositories and external services.

use crate::domain::*;
use crate::repository::*;
use shared_auth::{JwtService, OtpService, PinService, TokenResponse};
use shared_errors::{AppError, Result};
use shared_types::*;
use std::sync::Arc;
use tracing::{info, instrument, warn};
use uuid::Uuid;

/// Main user service coordinating all user operations
pub struct UserService {
    user_repository: Arc<UserRepository>,
    otp_repository: Arc<OtpRepository>,
    session_repository: Arc<SessionRepository>,
    sms_client: Arc<SmsClient>,
    jwt_service: JwtService,
}

impl UserService {
    pub fn new(
        user_repository: Arc<UserRepository>,
        otp_repository: Arc<OtpRepository>,
        session_repository: Arc<SessionRepository>,
        sms_client: Arc<SmsClient>,
    ) -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key".to_string());

        Self {
            user_repository,
            otp_repository,
            session_repository,
            sms_client,
            jwt_service: JwtService::new(&jwt_secret),
        }
    }

    /// Register new user - sends OTP for verification
    #[instrument(skip(self), fields(phone = %request.phone_number))]
    pub async fn register(&self, request: RegisterRequest) -> Result<RegisterResponse> {
        // Validate phone number format
        let phone_number = PhoneNumber::new(request.phone_number)
            .map_err(|_| AppError::invalid_phone_number())?;

        // Check if user already exists
        if let Some(_existing_user) = self.user_repository.find_by_phone(&phone_number).await? {
            return Err(AppError::User {
                message: "Phone number already registered".to_string(),
            });
        }

        // Generate and send OTP
        let otp_code = OtpService::generate_code();
        let verification_token = self.send_otp_code(&phone_number, &otp_code).await?;

        info!("Registration initiated for phone {}", phone_number.0);

        Ok(RegisterResponse {
            message: "Verification code sent to your phone".to_string(),
            verification_token,
        })
    }

    /// Verify OTP and complete user registration
    #[instrument(skip(self, request), fields(username = %request.lightning_username))]
    pub async fn verify_otp(&self, request: VerifyOtpRequest) -> Result<VerifyOtpResponse> {
        // Parse verification token to get phone number
        let phone_number = self.parse_verification_token(&request.verification_token)?;

        // Verify OTP code
        self.verify_otp_code(&phone_number, &request.otp_code).await?;

        // Validate username availability
        if !User::is_valid_username(&request.lightning_username) {
            return Err(AppError::Validation {
                message: "Invalid username format".to_string(),
            });
        }

        if !self.user_repository.is_username_available(&request.lightning_username).await? {
            return Err(AppError::User {
                message: "Username already taken".to_string(),
            });
        }

        // Hash the PIN securely
        let pin_hash = PinService::hash_pin(&request.pin)?;

        // Create new user
        let user = User::new(
            phone_number,
            pin_hash,
            request.lightning_username,
            request.full_name,
        );

        // Save user to database
        self.user_repository.create(&user).await?;

        // Create initial wallet for the user
        self.create_initial_wallet(user.id).await?;

        // Generate authentication tokens
        let tokens = self.jwt_service.generate_tokens(
            user.id,
            &user.phone_number,
            user.kyc_tier.clone(),
        )?;

        // Create session
        self.create_user_session(&user, &tokens).await?;

        info!("User registration completed for {}", user.lightning_username);

        Ok(VerifyOtpResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
            user: user.to_profile(),
        })
    }

    /// Login with phone number and PIN
    #[instrument(skip(self, request), fields(phone = %request.phone_number))]
    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse> {
        // Validate phone number
        let phone_number = PhoneNumber::new(request.phone_number)
            .map_err(|_| AppError::invalid_phone_number())?;

        // Find user by phone number
        let user = self.user_repository.find_by_phone(&phone_number).await?
            .ok_or_else(|| AppError::User {
                message: "Invalid phone number or PIN".to_string(),
            })?;

        // Verify PIN
        if !PinService::verify_pin(&request.pin, &user.pin_hash)? {
            warn!("Failed login attempt for user {}", user.id);
            return Err(AppError::invalid_pin());
        }

        // Generate authentication tokens
        let tokens = self.jwt_service.generate_tokens(
            user.id,
            &user.phone_number,
            user.kyc_tier.clone(),
        )?;

        // Update session
        self.create_user_session(&user, &tokens).await?;

        info!("User logged in: {}", user.lightning_username);

        Ok(LoginResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
            user: user.to_profile(),
        })
    }

    /// Refresh access token using refresh token
    #[instrument(skip(self, request))]
    pub async fn refresh_token(&self, request: RefreshTokenRequest) -> Result<RefreshTokenResponse> {
        // Generate new access token
        let access_token = self.jwt_service.refresh_access_token(&request.refresh_token)?;

        Ok(RefreshTokenResponse {
            access_token,
            expires_in: 15 * 60, // 15 minutes
        })
    }

    /// Get user profile by ID
    #[instrument(skip(self))]
    pub async fn get_profile(&self, user_id: UserId) -> Result<UserProfile> {
        let user = self.user_repository.find_by_id(user_id).await?
            .ok_or_else(|| AppError::user_not_found())?;

        Ok(user.to_profile())
    }

    /// Update user profile
    #[instrument(skip(self))]
    pub async fn update_profile(
        &self,
        user_id: UserId,
        request: UpdateProfileRequest,
    ) -> Result<UserProfile> {
        let mut user = self.user_repository.find_by_id(user_id).await?
            .ok_or_else(|| AppError::user_not_found())?;

        // Update fields
        user.full_name = request.full_name;

        // Save changes
        self.user_repository.update(&user).await?;

        info!("Profile updated for user {}", user_id);

        Ok(user.to_profile())
    }

    /// Get Lightning address for user
    #[instrument(skip(self))]
    pub async fn get_lightning_address(&self, user_id: UserId) -> Result<LightningAddressResponse> {
        let user = self.user_repository.find_by_id(user_id).await?
            .ok_or_else(|| AppError::user_not_found())?;

        let lightning_address = user.lightning_address();
        
        Ok(LightningAddressResponse {
            lightning_address: lightning_address.clone(),
            username: user.lightning_username,
            qr_code_url: Some(format!(
                "https://api.qrserver.com/v1/create-qr-code/?size=300x300&data={}",
                urlencoding::encode(&lightning_address.0)
            )),
        })
    }

    /// Send OTP code via SMS
    #[instrument(skip(self))]
    async fn send_otp_code(&self, phone_number: &PhoneNumber, code: &str) -> Result<String> {
        // Hash the OTP code for secure storage
        let code_hash = PinService::hash_pin(code)?;

        // Create OTP record
        let otp = OtpCode {
            id: Uuid::new_v4(),
            phone_number: phone_number.clone(),
            code_hash,
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(5),
            used: false,
            attempts: 0,
            created_at: chrono::Utc::now(),
        };

        // Store in database
        self.otp_repository.create(&otp).await?;

        // Send SMS
        self.sms_client.send_otp(phone_number, code).await?;

        // Create verification token (contains phone number)
        let verification_token = base64::encode(format!("{}:{}", phone_number.0, otp.id));

        Ok(verification_token)
    }

    /// Verify OTP code
    #[instrument(skip(self))]
    async fn verify_otp_code(&self, phone_number: &PhoneNumber, submitted_code: &str) -> Result<()> {
        // Find valid OTP for this phone number
        let mut otp = self.otp_repository.find_valid_code(phone_number).await?
            .ok_or_else(|| AppError::User {
                message: "Invalid or expired verification code".to_string(),
            })?;

        // Check attempt limit
        if otp.attempts >= 5 {
            return Err(AppError::User {
                message: "Too many verification attempts. Please request a new code.".to_string(),
            });
        }

        // Verify the code
        if !PinService::verify_pin(submitted_code, &otp.code_hash)? {
            // Increment attempts
            self.otp_repository.increment_attempts(otp.id).await?;
            return Err(AppError::User {
                message: "Invalid verification code".to_string(),
            });
        }

        // Mark OTP as used
        self.otp_repository.mark_used(otp.id).await?;

        Ok(())
    }

    /// Parse verification token to extract phone number
    fn parse_verification_token(&self, token: &str) -> Result<PhoneNumber> {
        let decoded = base64::decode(token)
            .map_err(|_| AppError::User {
                message: "Invalid verification token".to_string(),
            })?;

        let token_str = String::from_utf8(decoded)
            .map_err(|_| AppError::User {
                message: "Invalid verification token".to_string(),
            })?;

        let phone_number = token_str.split(':').next()
            .ok_or_else(|| AppError::User {
                message: "Invalid verification token".to_string(),
            })?;

        PhoneNumber::new(phone_number.to_string())
            .map_err(|_| AppError::User {
                message: "Invalid phone number in token".to_string(),
            })
    }

    /// Create user session for authentication
    #[instrument(skip(self, user, tokens))]
    async fn create_user_session(&self, user: &User, tokens: &TokenResponse) -> Result<()> {
        let refresh_token_hash = PinService::hash_pin(&tokens.refresh_token)?;

        let session = UserSession {
            id: Uuid::new_v4(),
            user_id: user.id,
            refresh_token_hash,
            expires_at: chrono::Utc::now() + chrono::Duration::days(7),
            device_fingerprint: serde_json::json!({}), // TODO: Add device fingerprinting
            created_at: chrono::Utc::now(),
        };

        self.session_repository.create_or_update(&session).await?;

        Ok(())
    }

    /// Create initial wallet for new user (calls payment service)
    #[instrument(skip(self))]
    async fn create_initial_wallet(&self, user_id: UserId) -> Result<()> {
        // In a real implementation, this would call the payment service
        // to create a wallet for the user. For now, we'll just log it.
        info!("Creating initial wallet for user {}", user_id);
        
        // TODO: Make HTTP request to payment service
        // POST /internal/wallets { "user_id": user_id }
        
        Ok(())
    }
}

/// SMS client for sending OTP codes
pub struct SmsClient {
    // In production, this would contain Twilio/Africa's Talking credentials
}

impl SmsClient {
    pub fn new() -> Self {
        Self {}
    }

    /// Send OTP code via SMS
    #[instrument(skip(self))]
    pub async fn send_otp(&self, phone_number: &PhoneNumber, code: &str) -> Result<()> {
        // In development, just log the code
        if std::env::var("ENVIRONMENT").unwrap_or_default() != "production" {
            info!("📱 SMS OTP for {}: {}", phone_number.0, code);
            return Ok(());
        }

        // In production, integrate with SMS provider (Twilio, Africa's Talking, etc.)
        let message = format!("Your PesaBit verification code is: {}. Valid for 5 minutes.", code);
        
        // TODO: Implement actual SMS sending
        info!("Sending SMS to {}: {}", phone_number.0, message);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use mockall::mock;

    // Mock implementations for testing would go here
    // This shows how to structure tests without external dependencies

    #[tokio::test]
    async fn test_username_validation() {
        assert!(User::is_valid_username("john123"));
        assert!(!User::is_valid_username("admin")); // Reserved
        assert!(!User::is_valid_username("ab"));   // Too short
    }
}