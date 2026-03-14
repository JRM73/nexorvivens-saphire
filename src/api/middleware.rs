// =============================================================================
// api/middleware.rs — Security middlewares (auth, rate limit, CORS)
//
// Role: Bearer token authentication, IP-based rate limiting,
// configurable CORS layer construction.
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

// ─── Bearer Token Authentication ──────────────────────────────────────────
/// Authentication middleware: verifies the Authorization: Bearer <key> header.
/// If no api_key is configured, all requests are allowed.
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
/// IP-based rate limiter (60-second sliding window).
pub struct RateLimiter {
    windows: StdMutex<HashMap<IpAddr, (Instant, u32)>>,
    pub max_per_minute: u32,
}

impl RateLimiter {
    pub fn new(max_per_minute: u32) -> Self {
        Self {
            windows: StdMutex::new(HashMap::new()),
            max_per_minute,
        }
    }

    /// Checks if the IP is under the limit. Returns true if allowed.
    pub fn check(&self, ip: IpAddr) -> bool {
        if self.max_per_minute == 0 { return true; }

        let mut windows = self.windows.lock().unwrap();
        let now = Instant::now();
        let entry = windows.entry(ip).or_insert((now, 0));

        if now.duration_since(entry.0).as_secs() >= 60 {
            // New window
            *entry = (now, 1);
            true
        } else {
            entry.1 += 1;
            entry.1 <= self.max_per_minute
        }
    }

    /// Cleans up expired entries (call periodically)
    pub fn cleanup(&self) {
        let mut windows = self.windows.lock().unwrap();
        let now = Instant::now();
        windows.retain(|_, (start, _)| now.duration_since(*start).as_secs() < 120);
    }
}

/// Rate limiting middleware: extracts IP from headers or falls back to localhost.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if state.rate_limiter.max_per_minute == 0 {
        return next.run(request).await;
    }

    // Extract IP: X-Forwarded-For > X-Real-IP > localhost
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
/// Builds the CORS layer based on allowed origins.
/// - Empty: same origin only (no cross-origin)
/// - ["*"]: all origins (dev/debug)
/// - ["http://example.com"]: specific origins
pub fn build_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    if allowed_origins.iter().any(|o| o == "*") {
        base.allow_origin(tower_http::cors::Any)
    } else if allowed_origins.is_empty() {
        // No origins = no cross-origin allowed (secure default)
        base
    } else {
        let origins: Vec<HeaderValue> = allowed_origins.iter()
            .filter_map(|o| HeaderValue::from_str(o).ok())
            .collect();
        base.allow_origin(origins)
    }
}

// ─── WebSocket Origin Check ─────────────────────────────────────────────────
/// Checks that the WebSocket origin is allowed.
/// Returns true if allowed (no configured origins = everything allowed locally).
pub fn check_ws_origin(headers: &axum::http::HeaderMap, allowed: &[String]) -> bool {
    if allowed.is_empty() {
        return true; // No restriction configured    }
    if allowed.iter().any(|o| o == "*") {
        return true;
    }
    match headers.get("origin").and_then(|v| v.to_str().ok()) {
        Some(origin) => allowed.iter().any(|o| o == origin),
        None => true, // No Origin header = same origin (local browser)    }
}
