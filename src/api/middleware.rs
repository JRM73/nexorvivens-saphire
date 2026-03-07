// =============================================================================
// api/middleware.rs — Security middleware (authentication, rate limiting, CORS)
//
// This module provides three security layers for the API:
// 1. Bearer token authentication middleware
// 2. Per-IP rate limiting middleware (sliding 60-second window)
// 3. Configurable CORS layer builder
// 4. WebSocket origin verification helper
// =============================================================================

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex as StdMutex;
use std::time::Instant;

use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tower_http::cors::CorsLayer;

use super::state::AppState;

// ─── Bearer Token Authentication ───────────────────────────────────────────

/// Authentication middleware that validates the `Authorization: Bearer <key>` header.
///
/// If no `api_key` is configured in `AppState` (i.e. it is `None`), all requests
/// are allowed through without any authentication check. Otherwise, the middleware
/// extracts the Bearer token from the request and compares it against the expected key.
///
/// # Returns
/// - The next handler's response if authorized.
/// - HTTP 401 Unauthorized with a JSON error body if the key is missing or invalid.
pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if let Some(ref expected_key) = state.api_key {
        let authorized = request.headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(|h| {
                if let Some(token) = h.strip_prefix("Bearer ") {
                    token == expected_key.as_str()
                } else {
                    false
                }
            })
            .unwrap_or(false);

        if !authorized {
            return (
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({
                    "error": "unauthorized",
                    "message": "Cle API manquante ou invalide"
                })),
            ).into_response();
        }
    }
    next.run(request).await
}

// ─── Rate Limiter ───────────────────────────────────────────────────────────

/// Per-IP rate limiter using a sliding 60-second window.
///
/// Each IP address is tracked with its window start time and request count.
/// When the window expires (>= 60 seconds elapsed), the counter resets.
/// A `max_per_minute` of 0 disables rate limiting entirely.
pub struct RateLimiter {
    /// Map from IP address to (window_start_time, request_count).
    /// Uses a standard (blocking) mutex since lock hold times are very short.
    windows: StdMutex<HashMap<IpAddr, (Instant, u32)>>,
    /// Maximum number of requests allowed per IP per 60-second window.
    /// A value of 0 means rate limiting is disabled.
    pub max_per_minute: u32,
}

impl RateLimiter {
    pub fn new(max_per_minute: u32) -> Self {
        Self {
            windows: StdMutex::new(HashMap::new()),
            max_per_minute,
        }
    }

    /// Checks whether the given IP address is within the rate limit.
    ///
    /// # Arguments
    /// * `ip` - The client IP address to check.
    ///
    /// # Returns
    /// `true` if the request is allowed, `false` if the rate limit has been exceeded.
    pub fn check(&self, ip: IpAddr) -> bool {
        if self.max_per_minute == 0 { return true; }

        let mut windows = self.windows.lock().unwrap();
        let now = Instant::now();
        let entry = windows.entry(ip).or_insert((now, 0));

        if now.duration_since(entry.0).as_secs() >= 60 {
            // Window expired: start a new 60-second window with count = 1
            *entry = (now, 1);
            true
        } else {
            // Within current window: increment count and check limit
            entry.1 += 1;
            entry.1 <= self.max_per_minute
        }
    }

    /// Removes expired entries from the rate limiter map.
    /// Entries older than 120 seconds are purged. Should be called periodically
    /// (e.g. from a background task) to prevent unbounded memory growth.
    pub fn cleanup(&self) {
        let mut windows = self.windows.lock().unwrap();
        let now = Instant::now();
        windows.retain(|_, (start, _)| now.duration_since(*start).as_secs() < 120);
    }
}

/// Rate limiting middleware: extracts the client IP from proxy headers and enforces the limit.
///
/// IP resolution order: `X-Forwarded-For` (first entry) > `X-Real-IP` > fallback to 127.0.0.1.
/// If `max_per_minute` is 0 in the rate limiter, this middleware is a no-op.
///
/// # Returns
/// - The next handler's response if within the rate limit.
/// - HTTP 429 Too Many Requests with a JSON error body if the limit is exceeded.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if state.rate_limiter.max_per_minute == 0 {
        return next.run(request).await;
    }

    // Extract client IP: X-Forwarded-For > X-Real-IP > fallback to localhost
    let ip = request.headers().get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
        .or_else(|| {
            request.headers().get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

    if !state.rate_limiter.check(ip) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(serde_json::json!({
                "error": "too_many_requests",
                "message": "Trop de requetes, reessayez dans une minute"
            })),
        ).into_response();
    }

    next.run(request).await
}

// ─── CORS ───────────────────────────────────────────────────────────────────

/// Builds the CORS layer based on the configured allowed origins.
///
/// # Behavior
/// - Empty list: same-origin only (no cross-origin requests allowed -- secure default).
/// - `["*"]`: all origins allowed (suitable for development/debugging only).
/// - Specific origins (e.g. `["http://example.com"]`): only those origins are allowed.
///
/// # Arguments
/// * `allowed_origins` - Slice of origin strings from the application configuration.
///
/// # Returns
/// A configured `CorsLayer` that allows GET/POST methods and Content-Type/Authorization headers.
pub fn build_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    if allowed_origins.iter().any(|o| o == "*") {
        base.allow_origin(tower_http::cors::Any)
    } else if allowed_origins.is_empty() {
        // No origins configured = no cross-origin allowed (secure default)
        base
    } else {
        let origins: Vec<HeaderValue> = allowed_origins.iter()
            .filter_map(|o| HeaderValue::from_str(o).ok())
            .collect();
        base.allow_origin(origins)
    }
}

// ─── WebSocket Origin Check ─────────────────────────────────────────────────

/// Checks whether the WebSocket connection's Origin header is allowed.
///
/// # Arguments
/// * `headers` - The HTTP request headers from the WebSocket upgrade request.
/// * `allowed` - The list of allowed origin strings from the application configuration.
///
/// # Returns
/// `true` if the connection is allowed. Rules:
/// - No origins configured (empty list): all connections allowed (local development).
/// - `"*"` in the list: all origins allowed.
/// - Specific origins: the request's `Origin` header must match one of them.
/// - No `Origin` header present: allowed (same-origin browser request).
pub fn check_ws_origin(headers: &axum::http::HeaderMap, allowed: &[String]) -> bool {
    if allowed.is_empty() {
        return true; // No restriction configured
    }
    if allowed.iter().any(|o| o == "*") {
        return true;
    }
    match headers.get("origin").and_then(|v| v.to_str().ok()) {
        Some(origin) => allowed.iter().any(|o| o == origin),
        None => true, // No Origin header = same-origin request (local browser)
    }
}
