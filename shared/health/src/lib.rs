use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::SystemTime};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub timestamp: DateTime<Utc>,
    pub service: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: HashMap<String, ComponentHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "degraded")]
    Degraded,
    #[serde(rename = "unhealthy")]
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
    pub last_checked: DateTime<Utc>,
}

pub struct HealthChecker {
    service_name: String,
    version: String,
    start_time: SystemTime,
    checks: Vec<Box<dyn HealthCheckProvider>>,
}

#[async_trait::async_trait]
pub trait HealthCheckProvider: Send + Sync {
    async fn name(&self) -> String;
    async fn check(&self) -> ComponentHealth;
}

impl HealthChecker {
    pub fn new(service_name: String, version: String) -> Self {
        Self {
            service_name,
            version,
            start_time: SystemTime::now(),
            checks: Vec::new(),
        }
    }

    pub fn add_check(mut self, check: Box<dyn HealthCheckProvider>) -> Self {
        self.checks.push(check);
        self
    }

    pub async fn check_health(&self) -> HealthCheck {
        let mut checks = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        for check in &self.checks {
            let name = check.name().await;
            let component_health = check.check().await;

            match component_health.status {
                HealthStatus::Unhealthy => overall_status = HealthStatus::Unhealthy,
                HealthStatus::Degraded if matches!(overall_status, HealthStatus::Healthy) => {
                    overall_status = HealthStatus::Degraded;
                }
                _ => {}
            }

            checks.insert(name, component_health);
        }

        let uptime = self
            .start_time
            .elapsed()
            .unwrap_or_default()
            .as_secs();

        HealthCheck {
            status: overall_status,
            timestamp: Utc::now(),
            service: self.service_name.clone(),
            version: self.version.clone(),
            uptime_seconds: uptime,
            checks,
        }
    }

    pub fn router(self) -> Router<Arc<Self>> {
        Router::new()
            .route("/health", get(health_handler))
            .route("/health/ready", get(readiness_handler))
            .route("/health/live", get(liveness_handler))
            .with_state(Arc::new(self))
    }
}

async fn health_handler(State(checker): State<Arc<HealthChecker>>) -> Json<HealthCheck> {
    Json(checker.check_health().await)
}

async fn readiness_handler(State(checker): State<Arc<HealthChecker>>) -> Json<HealthCheck> {
    Json(checker.check_health().await)
}

async fn liveness_handler(State(checker): State<Arc<HealthChecker>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "alive",
        "timestamp": Utc::now(),
        "service": checker.service_name
    }))
}

// Database health check
pub struct DatabaseHealthCheck {
    pool: sqlx::PgPool,
}

impl DatabaseHealthCheck {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl HealthCheckProvider for DatabaseHealthCheck {
    async fn name(&self) -> String {
        "database".to_string()
    }

    async fn check(&self) -> ComponentHealth {
        let start = SystemTime::now();
        
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => ComponentHealth {
                status: HealthStatus::Healthy,
                message: Some("Database connection successful".to_string()),
                response_time_ms: Some(start.elapsed().unwrap_or_default().as_millis() as u64),
                last_checked: Utc::now(),
            },
            Err(e) => ComponentHealth {
                status: HealthStatus::Unhealthy,
                message: Some(format!("Database connection failed: {}", e)),
                response_time_ms: Some(start.elapsed().unwrap_or_default().as_millis() as u64),
                last_checked: Utc::now(),
            },
        }
    }
}

// Redis health check
pub struct RedisHealthCheck {
    client: redis::Client,
}

impl RedisHealthCheck {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl HealthCheckProvider for RedisHealthCheck {
    async fn name(&self) -> String {
        "redis".to_string()
    }

    async fn check(&self) -> ComponentHealth {
        let start = SystemTime::now();
        
        match self.client.get_async_connection().await {
            Ok(mut conn) => {
                match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                    Ok(_) => ComponentHealth {
                        status: HealthStatus::Healthy,
                        message: Some("Redis connection successful".to_string()),
                        response_time_ms: Some(start.elapsed().unwrap_or_default().as_millis() as u64),
                        last_checked: Utc::now(),
                    },
                    Err(e) => ComponentHealth {
                        status: HealthStatus::Unhealthy,
                        message: Some(format!("Redis ping failed: {}", e)),
                        response_time_ms: Some(start.elapsed().unwrap_or_default().as_millis() as u64),
                        last_checked: Utc::now(),
                    },
                }
            }
            Err(e) => ComponentHealth {
                status: HealthStatus::Unhealthy,
                message: Some(format!("Redis connection failed: {}", e)),
                response_time_ms: Some(start.elapsed().unwrap_or_default().as_millis() as u64),
                last_checked: Utc::now(),
            },
        }
    }
}