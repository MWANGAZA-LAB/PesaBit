/// Security middleware and utilities for PesaBit
/// 
/// This library provides comprehensive security features including:
/// - Security headers (CSP, HSTS, X-Frame-Options, etc.)
/// - CORS configuration
/// - Request validation
/// - Security monitoring

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use shared_config::AppConfig;
use shared_errors::{AppError, Result};
use std::collections::HashSet;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

/// Security headers middleware
pub async fn security_headers_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let response = next.run(request).await;
    
    let mut response = response;
    let headers = response.headers_mut();
    
    // Content Security Policy - Prevent XSS attacks
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             font-src 'self' data:; \
             connect-src 'self' https:; \
             frame-ancestors 'none'; \
             base-uri 'self'; \
             form-action 'self'"
        ),
    );
    
    // HTTP Strict Transport Security - Force HTTPS
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );
    
    // Prevent clickjacking
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );
    
    // Prevent MIME type sniffing
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    
    // XSS Protection
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    
    // Referrer Policy
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    
    // Permissions Policy (formerly Feature Policy)
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static(
            "camera=(), microphone=(), geolocation=(), \
             payment=(), usb=(), magnetometer=(), \
             accelerometer=(), gyroscope=()"
        ),
    );
    
    // Remove server information
    headers.remove("Server");
    headers.remove("X-Powered-By");
    
    Ok(response)
}

/// Create CORS layer with proper security configuration
pub fn create_cors_layer(config: &AppConfig) -> CorsLayer {
    let allowed_origins = config.security.cors_allowed_origins.clone();
    let allowed_methods = config.security.cors_allowed_methods.clone();
    let allowed_headers = config.security.cors_allowed_headers.clone();
    
    // Convert string vectors to proper types
    let origins: Vec<_> = allowed_origins
        .iter()
        .map(|origin| origin.parse().unwrap())
        .collect();
    
    let methods: Vec<Method> = allowed_methods
        .iter()
        .filter_map(|method| method.parse().ok())
        .collect();
    
    let headers: Vec<_> = allowed_headers
        .iter()
        .filter_map(|header| header.parse().ok())
        .collect();
    
    CorsLayer::new()
        .allow_origin(origins.into())
        .allow_methods(methods)
        .allow_headers(headers)
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(86400)) // 24 hours
}

/// Request validation middleware
pub async fn request_validation_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    
    // Check for suspicious headers
    if let Some(user_agent) = headers.get("User-Agent") {
        let ua = user_agent.to_str().unwrap_or("");
        if is_suspicious_user_agent(ua) {
            warn!("Suspicious User-Agent detected: {}", ua);
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    // Check for oversized requests
    if let Some(content_length) = headers.get("Content-Length") {
        if let Ok(length) = content_length.to_str().unwrap_or("0").parse::<usize>() {
            if length > 10 * 1024 * 1024 { // 10MB limit
                warn!("Request too large: {} bytes", length);
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }
        }
    }
    
    // Check for suspicious content types
    if let Some(content_type) = headers.get("Content-Type") {
        let ct = content_type.to_str().unwrap_or("");
        if is_suspicious_content_type(ct) {
            warn!("Suspicious Content-Type detected: {}", ct);
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    Ok(next.run(request).await)
}

/// Check if user agent is suspicious
fn is_suspicious_user_agent(user_agent: &str) -> bool {
    let suspicious_patterns = [
        "sqlmap",
        "nikto",
        "nmap",
        "masscan",
        "zap",
        "burp",
        "w3af",
        "acunetix",
        "nessus",
        "openvas",
        "qualys",
        "rapid7",
        "metasploit",
        "havij",
        "pangolin",
        "sqlninja",
        "absinthe",
        "bsqlbf",
        "fimap",
        "golismero",
        "skipfish",
        "wafw00f",
        "whatweb",
        "dirb",
        "dirbuster",
        "gobuster",
        "wfuzz",
        "ffuf",
        "feroxbuster",
        "dirsearch",
    ];
    
    let ua_lower = user_agent.to_lowercase();
    suspicious_patterns.iter().any(|pattern| ua_lower.contains(pattern))
}

/// Check if content type is suspicious
fn is_suspicious_content_type(content_type: &str) -> bool {
    let suspicious_types = [
        "application/x-php",
        "application/x-httpd-php",
        "text/x-php",
        "application/x-executable",
        "application/x-msdownload",
        "application/x-msdos-program",
        "application/x-winexe",
        "application/x-msi",
        "application/x-ms-shortcut",
        "application/x-java-archive",
        "application/x-java-applet",
        "application/x-java-bean",
        "application/x-java-vm",
        "application/x-java-serialized-object",
        "application/x-java-jnlp-file",
        "application/x-java-jnlp-file",
        "application/x-java-jnlp-file",
    ];
    
    suspicious_types.contains(&content_type)
}

/// Security monitoring and alerting
pub struct SecurityMonitor {
    suspicious_ips: HashSet<String>,
    failed_attempts: std::collections::HashMap<String, u32>,
}

impl SecurityMonitor {
    pub fn new() -> Self {
        Self {
            suspicious_ips: HashSet::new(),
            failed_attempts: std::collections::HashMap::new(),
        }
    }
    
    /// Record a failed authentication attempt
    pub fn record_failed_auth(&mut self, ip: &str) {
        let count = self.failed_attempts.entry(ip.to_string()).or_insert(0);
        *count += 1;
        
        if *count >= 5 {
            self.suspicious_ips.insert(ip.to_string());
            warn!("IP {} marked as suspicious after {} failed attempts", ip, count);
        }
    }
    
    /// Check if IP is suspicious
    pub fn is_suspicious_ip(&self, ip: &str) -> bool {
        self.suspicious_ips.contains(ip)
    }
    
    /// Reset failed attempts for IP
    pub fn reset_failed_attempts(&mut self, ip: &str) {
        self.failed_attempts.remove(ip);
    }
}

/// Rate limiting based on IP and user
pub struct SecurityRateLimiter {
    ip_limits: std::collections::HashMap<String, (u32, std::time::Instant)>,
    user_limits: std::collections::HashMap<String, (u32, std::time::Instant)>,
}

impl SecurityRateLimiter {
    pub fn new() -> Self {
        Self {
            ip_limits: std::collections::HashMap::new(),
            user_limits: std::collections::HashMap::new(),
        }
    }
    
    /// Check if IP is rate limited
    pub fn check_ip_limit(&mut self, ip: &str, limit: u32, window: std::time::Duration) -> bool {
        let now = std::time::Instant::now();
        let entry = self.ip_limits.entry(ip.to_string()).or_insert((0, now));
        
        if now.duration_since(entry.1) > window {
            *entry = (0, now);
        }
        
        if entry.0 >= limit {
            return false;
        }
        
        entry.0 += 1;
        true
    }
    
    /// Check if user is rate limited
    pub fn check_user_limit(&mut self, user_id: &str, limit: u32, window: std::time::Duration) -> bool {
        let now = std::time::Instant::now();
        let entry = self.user_limits.entry(user_id.to_string()).or_insert((0, now));
        
        if now.duration_since(entry.1) > window {
            *entry = (0, now);
        }
        
        if entry.0 >= limit {
            return false;
        }
        
        entry.0 += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspicious_user_agent() {
        assert!(is_suspicious_user_agent("sqlmap/1.0"));
        assert!(is_suspicious_user_agent("Mozilla/5.0 (compatible; Nmap Scripting Engine)"));
        assert!(!is_suspicious_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"));
    }

    #[test]
    fn test_suspicious_content_type() {
        assert!(is_suspicious_content_type("application/x-php"));
        assert!(is_suspicious_content_type("application/x-executable"));
        assert!(!is_suspicious_content_type("application/json"));
        assert!(!is_suspicious_content_type("text/html"));
    }

    #[test]
    fn test_security_monitor() {
        let mut monitor = SecurityMonitor::new();
        let ip = "192.168.1.1";
        
        // Should not be suspicious initially
        assert!(!monitor.is_suspicious_ip(ip));
        
        // Record failed attempts
        for _ in 0..5 {
            monitor.record_failed_auth(ip);
        }
        
        // Should be suspicious after 5 attempts
        assert!(monitor.is_suspicious_ip(ip));
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = SecurityRateLimiter::new();
        let ip = "192.168.1.1";
        let limit = 10;
        let window = std::time::Duration::from_secs(60);
        
        // Should allow requests within limit
        for _ in 0..limit {
            assert!(limiter.check_ip_limit(ip, limit, window));
        }
        
        // Should block requests beyond limit
        assert!(!limiter.check_ip_limit(ip, limit, window));
    }
}
