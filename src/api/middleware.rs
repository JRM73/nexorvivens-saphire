// =============================================================================
// api/middleware.rs — Middlewares de securite (auth, rate limit, CORS)
//
// Role : Authentification par Bearer token, limitation de debit par IP,
// construction du layer CORS configurable.
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

// ─── Authentification Bearer Token ──────────────────────────────────────────

/// Middleware d'authentification : verifie le header Authorization: Bearer <key>.
/// Si aucune api_key n'est configuree, toutes les requetes sont autorisees.
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

/// Limiteur de debit par IP (fenetre glissante de 60 secondes).
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

    /// Verifie si l'IP est sous la limite. Retourne true si autorise.
    pub fn check(&self, ip: IpAddr) -> bool {
        if self.max_per_minute == 0 { return true; }

        let mut windows = self.windows.lock().unwrap();
        let now = Instant::now();
        let entry = windows.entry(ip).or_insert((now, 0));

        if now.duration_since(entry.0).as_secs() >= 60 {
            // Nouvelle fenetre
            *entry = (now, 1);
            true
        } else {
            entry.1 += 1;
            entry.1 <= self.max_per_minute
        }
    }

    /// Nettoie les entrees expirees (appeler periodiquement)
    pub fn cleanup(&self) {
        let mut windows = self.windows.lock().unwrap();
        let now = Instant::now();
        windows.retain(|_, (start, _)| now.duration_since(*start).as_secs() < 120);
    }
}

/// Middleware de rate limiting : extrait l'IP depuis les headers ou fallback localhost.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if state.rate_limiter.max_per_minute == 0 {
        return next.run(request).await;
    }

    // Extraire l'IP : X-Forwarded-For > X-Real-IP > localhost
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

/// Construit le layer CORS en fonction des origines autorisees.
/// - Vide : meme origine uniquement (pas de cross-origin)
/// - ["*"] : toutes les origines (dev/debug)
/// - ["http://example.com"] : origines specifiques
pub fn build_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    if allowed_origins.iter().any(|o| o == "*") {
        base.allow_origin(tower_http::cors::Any)
    } else if allowed_origins.is_empty() {
        // Pas d'origines = pas de cross-origin autorise (defaut securise)
        base
    } else {
        let origins: Vec<HeaderValue> = allowed_origins.iter()
            .filter_map(|o| HeaderValue::from_str(o).ok())
            .collect();
        base.allow_origin(origins)
    }
}

// ─── WebSocket Origin Check ─────────────────────────────────────────────────

/// Verifie que l'origine WebSocket est autorisee.
/// Retourne true si autorise (pas d'origines configurees = tout autorise en local).
pub fn check_ws_origin(headers: &axum::http::HeaderMap, allowed: &[String]) -> bool {
    if allowed.is_empty() {
        return true; // Pas de restriction configuree
    }
    if allowed.iter().any(|o| o == "*") {
        return true;
    }
    match headers.get("origin").and_then(|v| v.to_str().ok()) {
        Some(origin) => allowed.iter().any(|o| o == origin),
        None => true, // Pas de header Origin = meme origine (navigateur local)
    }
}
