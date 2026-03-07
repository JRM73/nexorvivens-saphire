// =============================================================================
// api/static_files.rs — Static file handlers (lite version)
//
// This module serves the embedded web UI assets (HTML, CSS, JavaScript, SVG
// favicon, and i18n translation files). All files are compiled into the binary
// via `include_str!`, eliminating the need for a separate static file directory
// at runtime.
// =============================================================================

/// Embedded HTML content for the main web UI page.
pub const INDEX_HTML: &str = include_str!("../../static/index.html");
/// Embedded CSS stylesheet for the web UI.
pub const STYLE_CSS: &str = include_str!("../../static/style.css");
/// Embedded JavaScript application code for the web UI.
pub const APP_JS: &str = include_str!("../../static/app.js");
/// Embedded SVG favicon.
pub const FAVICON_SVG: &str = include_str!("../../static/favicon.svg");
/// Embedded internationalization (i18n) JavaScript module.
pub const I18N_JS: &str = include_str!("../../static/i18n.js");

/// Embedded French translation file.
pub const I18N_FR: &str = include_str!("../../static/i18n/fr.json");
/// Embedded English translation file.
pub const I18N_EN: &str = include_str!("../../static/i18n/en.json");

/// GET / -- Serves the main HTML page of the web UI.
pub async fn index_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(INDEX_HTML)
}

/// GET /style.css -- Serves the CSS stylesheet with the correct Content-Type.
pub async fn css_handler() -> impl axum::response::IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "text/css")], STYLE_CSS)
}

/// GET /app.js -- Serves the JavaScript application with the correct Content-Type.
pub async fn js_handler() -> impl axum::response::IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "application/javascript")], APP_JS)
}

/// GET /favicon.svg -- Serves the SVG favicon with the correct Content-Type.
pub async fn favicon_handler() -> impl axum::response::IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "image/svg+xml")], FAVICON_SVG)
}

/// GET /i18n.js -- Serves the i18n JavaScript module with the correct Content-Type.
pub async fn i18n_js_handler() -> impl axum::response::IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "application/javascript")], I18N_JS)
}

/// GET /i18n/:lang -- Serves a translation JSON file based on the language code.
///
/// # Path parameters
/// * `lang` - Filename of the translation (e.g. "fr.json", "en.json").
///
/// Falls back to French if the requested language is not available.
pub async fn i18n_handler(
    axum::extract::Path(lang): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    let content = match lang.as_str() {
        "fr.json" => I18N_FR,
        "en.json" => I18N_EN,
        _ => I18N_FR, // Default fallback to French
    };
    ([(axum::http::header::CONTENT_TYPE, "application/json; charset=utf-8")], content)
}
