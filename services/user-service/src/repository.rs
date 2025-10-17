/// Repository layer for user data access
/// 
/// This module handles all database operations for users, OTP codes, and sessions.
/// It abstracts the database implementation from the business logic.

use crate::domain::*;
use shared_errors::{AppError, Result};
use shared_types::*;
use sqlx::{PgPool, Row};
use tracing::{instrument, warn};
use uuid::Uuid;

/// User repository for database operations
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new user in the database
    #[instrument(skip(self, user))]
    pub async fn create(&self, user: &User) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, phone_number, pin_hash, lightning_username, full_name, kyc_status, kyc_tier)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            user.id.0,
            user.phone_number.0,
            user.pin_hash,
            user.lightning_username,
            user.full_name,
            user.kyc_status as _,
            user.kyc_tier as _,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("users_phone_number_key") {
                AppError::User {
                    message: "Phone number already registered".to_string(),
                }
            } else if e.to_string().contains("users_lightning_username_key") {
                AppError::User {
                    message: "Username already taken".to_string(),
                }
            } else {
                AppError::Database(e)
            }
        })?;

        Ok(())
    }

    /// Find user by phone number
    #[instrument(skip(self))]
    pub async fn find_by_phone(&self, phone_number: &PhoneNumber) -> Result<Option<User>> {
        let row = sqlx::query!(
            "SELECT * FROM users WHERE phone_number = $1",
            phone_number.0
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: UserId(r.id),
            phone_number: PhoneNumber(r.phone_number),
            pin_hash: r.pin_hash,
            lightning_username: r.lightning_username,
            full_name: r.full_name,
            kyc_status: r.kyc_status,
            kyc_tier: r.kyc_tier,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// Find user by ID
    #[instrument(skip(self))]
    pub async fn find_by_id(&self, user_id: UserId) -> Result<Option<User>> {
        let row = sqlx::query!(
            "SELECT * FROM users WHERE id = $1",
            user_id.0
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: UserId(r.id),
            phone_number: PhoneNumber(r.phone_number),
            pin_hash: r.pin_hash,
            lightning_username: r.lightning_username,
            full_name: r.full_name,
            kyc_status: r.kyc_status,
            kyc_tier: r.kyc_tier,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// Find user by Lightning username
    #[instrument(skip(self))]
    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query!(
            "SELECT * FROM users WHERE lightning_username = $1",
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: UserId(r.id),
            phone_number: PhoneNumber(r.phone_number),
            pin_hash: r.pin_hash,
            lightning_username: r.lightning_username,
            full_name: r.full_name,
            kyc_status: r.kyc_status,
            kyc_tier: r.kyc_tier,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    /// Update user profile
    #[instrument(skip(self))]
    pub async fn update(&self, user: &User) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users 
            SET full_name = $2, kyc_status = $3, kyc_tier = $4, updated_at = NOW()
            WHERE id = $1
            "#,
            user.id.0,
            user.full_name,
            user.kyc_status as _,
            user.kyc_tier as _,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if username is available
    #[instrument(skip(self))]
    pub async fn is_username_available(&self, username: &str) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE lightning_username = $1",
            username
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.unwrap_or(0) == 0)
    }
}

/// OTP code repository
pub struct OtpRepository {
    pool: PgPool,
}

impl OtpRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Store OTP code for verification
    #[instrument(skip(self, otp))]
    pub async fn create(&self, otp: &OtpCode) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO otp_codes (id, phone_number, code_hash, expires_at, used, attempts)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            otp.id,
            otp.phone_number.0,
            otp.code_hash,
            otp.expires_at,
            otp.used,
            otp.attempts,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Find valid OTP code for phone number
    #[instrument(skip(self))]
    pub async fn find_valid_code(&self, phone_number: &PhoneNumber) -> Result<Option<OtpCode>> {
        let row = sqlx::query!(
            r#"
            SELECT * FROM otp_codes 
            WHERE phone_number = $1 AND used = false AND expires_at > NOW()
            ORDER BY created_at DESC 
            LIMIT 1
            "#,
            phone_number.0
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| OtpCode {
            id: r.id,
            phone_number: PhoneNumber(r.phone_number),
            code_hash: r.code_hash,
            expires_at: r.expires_at,
            used: r.used,
            attempts: r.attempts,
            created_at: r.created_at,
        }))
    }

    /// Mark OTP code as used
    #[instrument(skip(self))]
    pub async fn mark_used(&self, otp_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE otp_codes SET used = true WHERE id = $1",
            otp_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Increment attempt counter
    #[instrument(skip(self))]
    pub async fn increment_attempts(&self, otp_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE otp_codes SET attempts = attempts + 1 WHERE id = $1",
            otp_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clean up expired OTP codes (called periodically)
    #[instrument(skip(self))]
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let result = sqlx::query!(
            "DELETE FROM otp_codes WHERE expires_at < NOW() - INTERVAL '1 day'"
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            warn!("Cleaned up {} expired OTP codes", result.rows_affected());
        }

        Ok(result.rows_affected())
    }
}

/// User session repository
pub struct SessionRepository {
    pool: PgPool,
}

impl SessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create or update user session (single device login)
    #[instrument(skip(self, session))]
    pub async fn create_or_update(&self, session: &UserSession) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO sessions (id, user_id, refresh_token_hash, expires_at, device_fingerprint)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id) 
            DO UPDATE SET 
                id = $1,
                refresh_token_hash = $3,
                expires_at = $4,
                device_fingerprint = $5,
                created_at = NOW()
            "#,
            session.id,
            session.user_id.0,
            session.refresh_token_hash,
            session.expires_at,
            session.device_fingerprint,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Find session by user ID
    #[instrument(skip(self))]
    pub async fn find_by_user_id(&self, user_id: UserId) -> Result<Option<UserSession>> {
        let row = sqlx::query!(
            "SELECT * FROM sessions WHERE user_id = $1 AND expires_at > NOW()",
            user_id.0
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| UserSession {
            id: r.id,
            user_id: UserId(r.user_id),
            refresh_token_hash: r.refresh_token_hash,
            expires_at: r.expires_at,
            device_fingerprint: r.device_fingerprint,
            created_at: r.created_at,
        }))
    }

    /// Find session by refresh token hash
    #[instrument(skip(self))]
    pub async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<UserSession>> {
        let row = sqlx::query!(
            "SELECT * FROM sessions WHERE refresh_token_hash = $1 AND expires_at > NOW()",
            token_hash
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| UserSession {
            id: r.id,
            user_id: UserId(r.user_id),
            refresh_token_hash: r.refresh_token_hash,
            expires_at: r.expires_at,
            device_fingerprint: r.device_fingerprint,
            created_at: r.created_at,
        }))
    }

    /// Delete user session (logout)
    #[instrument(skip(self))]
    pub async fn delete_by_user_id(&self, user_id: UserId) -> Result<()> {
        sqlx::query!(
            "DELETE FROM sessions WHERE user_id = $1",
            user_id.0
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clean up expired sessions
    #[instrument(skip(self))]
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let result = sqlx::query!(
            "DELETE FROM sessions WHERE expires_at < NOW()"
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            warn!("Cleaned up {} expired sessions", result.rows_affected());
        }

        Ok(result.rows_affected())
    }
}