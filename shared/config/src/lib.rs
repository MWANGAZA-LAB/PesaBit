/// Configuration management for PesaBit
/// 
/// This library provides type-safe configuration loading from environment variables
/// with proper validation and default values for all services.

use serde::{Deserialize, Serialize};
use std::env;
use shared_errors::{AppError, Result};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub services: ServicesConfig,
    pub rate_limiting: RateLimitingConfig,
    pub mpesa: MpesaConfig,
    pub lightning: LightningConfig,
    pub exchange_rate: ExchangeRateConfig,
    pub sms: SmsConfig,
    pub security: SecurityConfig,
    pub ssl: SslConfig,
    pub monitoring: MonitoringConfig,
    pub app: ApplicationConfig,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub run_migrations: bool,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry_minutes: i64,
    pub refresh_token_expiry_days: i64,
}

/// Services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    pub user_service_url: String,
    pub payment_service_url: String,
    pub api_gateway_url: String,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

/// M-Pesa configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpesaConfig {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub shortcode: String,
    pub passkey: String,
    pub sandbox_url: String,
    pub callback_url: String,
}

/// Lightning Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningConfig {
    pub node_url: String,
    pub macaroon_path: String,
    pub tls_cert_path: String,
}

/// Exchange rate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRateConfig {
    pub api_url: String,
    pub api_key: String,
}

/// SMS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    pub provider_url: String,
    pub api_key: String,
    pub username: String,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub cors_allowed_origins: Vec<String>,
    pub cors_allowed_methods: Vec<String>,
    pub cors_allowed_headers: Vec<String>,
}

/// SSL/TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub prometheus_enabled: bool,
    pub grafana_enabled: bool,
    pub log_level: String,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub rust_env: String,
    pub service_port: u16,
    pub frontend_url: String,
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(AppConfig {
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://pesabit:pesabit_dev_password@localhost:5432/pesabit".to_string()),
                max_connections: env::var("DB_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(100),
                min_connections: env::var("DB_MIN_CONNECTIONS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5),
                acquire_timeout: env::var("DB_ACQUIRE_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30),
                run_migrations: env::var("RUN_MIGRATIONS")
                    .unwrap_or_default() == "true",
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://:redis_dev_password@localhost:6379".to_string()),
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production-minimum-32-characters".to_string()),
                access_token_expiry_minutes: env::var("JWT_ACCESS_TOKEN_EXPIRY_MINUTES")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(15),
                refresh_token_expiry_days: env::var("JWT_REFRESH_TOKEN_EXPIRY_DAYS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(7),
            },
            services: ServicesConfig {
                user_service_url: env::var("USER_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:8001".to_string()),
                payment_service_url: env::var("PAYMENT_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:8002".to_string()),
                api_gateway_url: env::var("API_GATEWAY_URL")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            },
            rate_limiting: RateLimitingConfig {
                requests_per_minute: env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(100),
                burst_size: env::var("RATE_LIMIT_BURST_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(20),
            },
            mpesa: MpesaConfig {
                consumer_key: env::var("MPESA_CONSUMER_KEY")
                    .unwrap_or_else(|_| "your_mpesa_consumer_key".to_string()),
                consumer_secret: env::var("MPESA_CONSUMER_SECRET")
                    .unwrap_or_else(|_| "your_mpesa_consumer_secret".to_string()),
                shortcode: env::var("MPESA_SHORTCODE")
                    .unwrap_or_else(|_| "174379".to_string()),
                passkey: env::var("MPESA_PASSKEY")
                    .unwrap_or_else(|_| "your_mpesa_passkey".to_string()),
                sandbox_url: env::var("MPESA_SANDBOX_URL")
                    .unwrap_or_else(|_| "https://sandbox.safaricom.co.ke".to_string()),
                callback_url: env::var("MPESA_CALLBACK_URL")
                    .unwrap_or_else(|_| "https://your-domain.com/api/v1/deposits/mpesa/callback".to_string()),
            },
            lightning: LightningConfig {
                node_url: env::var("LIGHTNING_NETWORK_NODE")
                    .unwrap_or_else(|_| "http://localhost:9735".to_string()),
                macaroon_path: env::var("LIGHTNING_NETWORK_MACAROON_PATH")
                    .unwrap_or_else(|_| "/path/to/admin.macaroon".to_string()),
                tls_cert_path: env::var("LIGHTNING_NETWORK_TLS_CERT_PATH")
                    .unwrap_or_else(|_| "/path/to/tls.cert".to_string()),
            },
            exchange_rate: ExchangeRateConfig {
                api_url: env::var("EXCHANGE_RATE_API_URL")
                    .unwrap_or_else(|_| "https://api.coingecko.com/api/v3".to_string()),
                api_key: env::var("EXCHANGE_RATE_API_KEY")
                    .unwrap_or_else(|_| "your_api_key_here".to_string()),
            },
            sms: SmsConfig {
                provider_url: env::var("SMS_PROVIDER_URL")
                    .unwrap_or_else(|_| "https://api.africastalking.com/version1/messaging".to_string()),
                api_key: env::var("SMS_API_KEY")
                    .unwrap_or_else(|_| "your_sms_api_key".to_string()),
                username: env::var("SMS_USERNAME")
                    .unwrap_or_else(|_| "your_sms_username".to_string()),
            },
            security: SecurityConfig {
                cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:5173,https://pesa.co.ke".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                cors_allowed_methods: env::var("CORS_ALLOWED_METHODS")
                    .unwrap_or_else(|_| "GET,POST,PUT,DELETE,OPTIONS".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                cors_allowed_headers: env::var("CORS_ALLOWED_HEADERS")
                    .unwrap_or_else(|_| "Content-Type,Authorization,X-Requested-With".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            ssl: SslConfig {
                enabled: env::var("SSL_ENABLED")
                    .unwrap_or_default() == "true",
                cert_path: env::var("SSL_CERT_PATH")
                    .unwrap_or_else(|_| "/path/to/cert.pem".to_string()),
                key_path: env::var("SSL_KEY_PATH")
                    .unwrap_or_else(|_| "/path/to/key.pem".to_string()),
            },
            monitoring: MonitoringConfig {
                prometheus_enabled: env::var("PROMETHEUS_ENABLED")
                    .unwrap_or_default() == "true",
                grafana_enabled: env::var("GRAFANA_ENABLED")
                    .unwrap_or_default() == "true",
                log_level: env::var("LOG_LEVEL")
                    .unwrap_or_else(|_| "info".to_string()),
            },
            app: ApplicationConfig {
                rust_env: env::var("RUST_ENV")
                    .unwrap_or_else(|_| "development".to_string()),
                service_port: env::var("SERVICE_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(3000),
                frontend_url: env::var("FRONTEND_URL")
                    .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            },
        })
    }

    /// Validate configuration for production readiness
    pub fn validate_production(&self) -> Result<()> {
        // Check for development secrets
        if self.jwt.secret == "your-super-secret-jwt-key-change-in-production-minimum-32-characters" {
            return Err(AppError::Validation {
                message: "JWT secret must be changed for production".to_string(),
            });
        }

        if self.mpesa.consumer_key == "your_mpesa_consumer_key" {
            return Err(AppError::Validation {
                message: "M-Pesa credentials must be configured for production".to_string(),
            });
        }

        if self.sms.api_key == "your_sms_api_key" {
            return Err(AppError::Validation {
                message: "SMS credentials must be configured for production".to_string(),
            });
        }

        // Check JWT secret length
        if self.jwt.secret.len() < 32 {
            return Err(AppError::Validation {
                message: "JWT secret must be at least 32 characters long".to_string(),
            });
        }

        // Check for HTTPS in production
        if self.app.rust_env == "production" && !self.ssl.enabled {
            return Err(AppError::Validation {
                message: "SSL must be enabled for production".to_string(),
            });
        }

        Ok(())
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.app.rust_env == "production"
    }

    /// Check if running in development
    pub fn is_development(&self) -> bool {
        self.app.rust_env == "development"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        let config = AppConfig::from_env().unwrap();
        assert!(!config.jwt.secret.is_empty());
        assert!(config.database.max_connections > 0);
    }

    #[test]
    fn test_production_validation() {
        let mut config = AppConfig::from_env().unwrap();
        config.app.rust_env = "production".to_string();
        
        // Should fail with default secrets
        assert!(config.validate_production().is_err());
        
        // Should pass with proper secrets
        config.jwt.secret = "a-very-long-secret-key-for-production-use-only-32-chars-minimum".to_string();
        config.mpesa.consumer_key = "real_consumer_key".to_string();
        config.sms.api_key = "real_sms_key".to_string();
        config.ssl.enabled = true;
        
        assert!(config.validate_production().is_ok());
    }
}
