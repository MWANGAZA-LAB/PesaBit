/// Structured logging and tracing for PesaBit
/// 
/// This library configures consistent logging across all services with:
/// - JSON structured logs for production
/// - Pretty console logs for development  
/// - Request tracing with correlation IDs
/// - Performance monitoring

use serde_json::json;
use tracing::{info, Span};
use tracing_subscriber::{
    fmt::{format::FmtSpan, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};
use uuid::Uuid;

/// Initialize logging for a service
/// Call this once at startup of each service
pub fn init_tracing(service_name: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_timer(UtcTime::rfc_3339());

    // Use JSON format in production, pretty format in development
    if is_production() {
        // JSON structured logging for production (easier for log aggregation)
        let json_layer = fmt_layer
            .json()
            .with_current_span(true)
            .with_span_list(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(json_layer)
            .init();
    } else {
        // Pretty console logging for development
        let console_layer = fmt_layer.pretty();

        tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .init();
    }

    info!(service = service_name, "Tracing initialized");
}

/// Check if running in production environment
fn is_production() -> bool {
    std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        .to_lowercase()
        == "production"
}

/// Generate a unique trace ID for request correlation
/// Each HTTP request gets a unique ID that follows it through all services
pub fn generate_trace_id() -> String {
    Uuid::new_v4().to_string()
}

/// Add structured fields to current span for better log analysis
/// Usage: add_span_fields(&[("user_id", &user_id.to_string()), ("amount", &amount.to_string())])
pub fn add_span_fields(fields: &[(&str, &str)]) {
    let current_span = Span::current();
    for (key, value) in fields {
        current_span.record(key, value);
    }
}

/// Log a business event with structured data
/// These logs are important for business analytics and monitoring
pub fn log_business_event(event_type: &str, data: serde_json::Value) {
    info!(
        event_type = event_type,
        event_data = %data,
        "Business event"
    );
}

/// Log a financial transaction with all relevant details
/// Critical for audit trails and compliance
pub fn log_financial_event(
    transaction_id: &str,
    user_id: &str,
    transaction_type: &str,
    amount_kes: Option<&str>,
    amount_sats: Option<&str>,
    status: &str,
) {
    info!(
        transaction_id = transaction_id,
        user_id = user_id,
        transaction_type = transaction_type,
        amount_kes = amount_kes,
        amount_sats = amount_sats,
        status = status,
        "Financial transaction"
    );
}

/// Log external API calls for debugging integration issues
pub fn log_external_api_call(
    service: &str,
    endpoint: &str,
    method: &str,
    status_code: Option<u16>,
    duration_ms: u64,
    error: Option<&str>,
) {
    if let Some(error) = error {
        tracing::error!(
            service = service,
            endpoint = endpoint,
            method = method,
            duration_ms = duration_ms,
            error = error,
            "External API call failed"
        );
    } else {
        info!(
            service = service,
            endpoint = endpoint,
            method = method,
            status_code = status_code,
            duration_ms = duration_ms,
            "External API call completed"
        );
    }
}

/// Middleware to add trace ID to all requests
pub fn trace_id_layer() -> tower_http::trace::TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsFailureClass>,
    impl Fn(&http::Request<axum::body::Body>) -> tracing::Span + Clone,
> {
    tower_http::trace::TraceLayer::new_for_http().make_span_with(|request: &http::Request<_>| {
        let trace_id = generate_trace_id();
        
        tracing::info_span!(
            "http_request",
            method = %request.method(),
            uri = %request.uri(),
            trace_id = %trace_id,
            status_code = tracing::field::Empty,
            duration_ms = tracing::field::Empty,
        )
    })
}

/// Performance monitoring helpers
pub struct Timer {
    start: std::time::Instant,
    operation: String,
}

impl Timer {
    pub fn new(operation: &str) -> Self {
        Self {
            start: std::time::Instant::now(),
            operation: operation.to_string(),
        }
    }

    pub fn finish(self) {
        let duration = self.start.elapsed();
        info!(
            operation = %self.operation,
            duration_ms = duration.as_millis(),
            "Operation completed"
        );
    }
}

/// Macros for convenient structured logging
#[macro_export]
macro_rules! log_user_action {
    ($user_id:expr, $action:expr, $details:expr) => {
        tracing::info!(
            user_id = %$user_id,
            action = $action,
            details = ?$details,
            "User action"
        );
    };
}

#[macro_export]
macro_rules! log_security_event {
    ($event_type:expr, $user_id:expr, $details:expr) => {
        tracing::warn!(
            event_type = $event_type,
            user_id = %$user_id,
            details = ?$details,
            "Security event"
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id_generation() {
        let trace_id1 = generate_trace_id();
        let trace_id2 = generate_trace_id();
        
        assert_ne!(trace_id1, trace_id2);
        assert_eq!(trace_id1.len(), 36); // UUID format
    }

    #[test]
    fn test_environment_detection() {
        std::env::set_var("ENVIRONMENT", "production");
        assert!(is_production());
        
        std::env::set_var("ENVIRONMENT", "development");
        assert!(!is_production());
    }
}