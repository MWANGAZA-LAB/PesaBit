/// Database connection and utilities for PesaBit
/// 
/// This library handles PostgreSQL connections, connection pooling, and database
/// configuration. All services use this to ensure consistent database access.

use shared_errors::{AppError, Result};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tracing::{info, warn};

/// Database configuration settings
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://pesa:pesa@localhost/pesa".to_string(),
            max_connections: 100,
            min_connections: 5,
            acquire_timeout: 30,
        }
    }
}

/// Create a PostgreSQL connection pool with optimized settings for financial applications
/// 
/// This pool handles:
/// - Connection reuse to avoid overhead
/// - Automatic reconnection on failures  
/// - Connection limits to prevent overwhelming the database
/// - Health monitoring
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool> {
    info!("Connecting to database...");

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout))
        .connect(&config.url)
        .await
        .map_err(|e| {
            AppError::Database(e)
        })?;

    // Test the connection
    test_connection(&pool).await?;
    
    info!("Database connection pool created successfully");
    Ok(pool)
}

/// Test database connectivity and return basic health info
async fn test_connection(pool: &PgPool) -> Result<()> {
    let row = sqlx::query("SELECT version(), now() as current_time")
        .fetch_one(pool)
        .await?;
    
    let version: String = row.get("version");
    let current_time: chrono::DateTime<chrono::Utc> = row.get("current_time");
    
    info!("Database connected - Version: {}, Time: {}", 
          version.split_whitespace().take(2).collect::<Vec<_>>().join(" "),
          current_time.format("%Y-%m-%d %H:%M:%S UTC"));
    
    Ok(())
}

/// Run database migrations on startup
/// This ensures the database schema is always up-to-date
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");
    
    sqlx::migrate!("../migrations")
        .run(pool)
        .await
        .map_err(AppError::Database)?;
        
    info!("Database migrations completed");
    Ok(())
}

/// Check database health for monitoring/health check endpoints
pub async fn health_check(pool: &PgPool) -> Result<DatabaseHealth> {
    let start = std::time::Instant::now();
    
    // Simple query to test responsiveness
    let row = sqlx::query("SELECT COUNT(*) as connection_count FROM pg_stat_activity WHERE datname = current_database()")
        .fetch_one(pool)
        .await?;
    
    let response_time = start.elapsed();
    let connection_count: i64 = row.get("connection_count");
    
    let status = if response_time.as_millis() < 100 {
        "healthy"
    } else if response_time.as_millis() < 1000 {
        "degraded" 
    } else {
        "unhealthy"
    };
    
    if status != "healthy" {
        warn!("Database health check: {} ({}ms response time)", status, response_time.as_millis());
    }
    
    Ok(DatabaseHealth {
        status: status.to_string(),
        response_time_ms: response_time.as_millis() as u64,
        connection_count: connection_count as u32,
    })
}

/// Database health information for monitoring
#[derive(Debug, serde::Serialize)]
pub struct DatabaseHealth {
    pub status: String,
    pub response_time_ms: u64,
    pub connection_count: u32,
}

/// Initialize database connection with environment variables
/// This is the main function services call to get a database connection
pub async fn init() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://pesa:pesa@localhost/pesa".to_string());
    
    let config = DatabaseConfig {
        url: database_url,
        max_connections: std::env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100),
        min_connections: std::env::var("DB_MIN_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5),
        acquire_timeout: std::env::var("DB_ACQUIRE_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30),
    };
    
    let pool = create_pool(&config).await?;
    
    // Run migrations if requested
    if std::env::var("RUN_MIGRATIONS").unwrap_or_default() == "true" {
        run_migrations(&pool).await?;
    }
    
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.min_connections, 5);
    }
}