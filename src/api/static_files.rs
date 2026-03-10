// =============================================================================
// api/static_files.rs — Handlers de fichiers statiques
//
// Role : Sert les fichiers HTML, CSS, JS et SVG embarques dans le binaire
// via include_str!() pour l'interface web et le dashboard.
// =============================================================================

/// Fichiers statiques embarques dans le binaire a la compilation.
/// Cela evite de dependre de fichiers externes au moment de l'execution.
pub const INDEX_HTML: &str = include_str!("../../static/index.html");
pub const STYLE_CSS: &str = include_str!("../../static/style.css");
pub const APP_JS: &str = include_str!("../../static/app.js");
pub const AUTH_JS: &str = include_str!("../../static/auth.js");
pub const DASHBOARD_HTML: &str = include_str!("../../static/dashboard.html");
pub const PIPELINE_EDITOR_HTML: &str = include_str!("../../static/pipeline-editor.html");
pub const BRAIN_MAP_HTML: &str = include_str!("../../static/brain-map.html");
pub const FAVICON_SVG: &str = include_str!("../../static/favicon.svg");
pub const I18N_JS: &str = include_str!("../../static/i18n.js");

// ─── Fichiers de traduction i18n embarques ──────────────────────────────────
pub const I18N_FR: &str = include_str!("../../static/i18n/fr.json");
pub const I18N_EN: &str = include_str!("../../static/i18n/en.json");
pub const I18N_DE: &str = include_str!("../../static/i18n/de.json");
pub const I18N_ES: &str = include_str!("../../static/i18n/es.json");
pub const I18N_IT: &str = include_str!("../../static/i18n/it.json");
pub const I18N_PT: &str = include_str!("../../static/i18n/pt.json");
pub const I18N_RU: &str = include_str!("../../static/i18n/ru.json");
pub const I18N_ZH: &str = include_str!("../../static/i18n/zh.json");
pub const I18N_JA: &str = include_str!("../../static/i18n/ja.json");
pub const I18N_AR: &str = include_str!("../../static/i18n/ar.json");

/// Sert la page HTML principale de l'interface web.
pub async fn index_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(INDEX_HTML)
}

/// Sert la feuille de style CSS (Cascading Style Sheets) de l'interface web.
pub async fn css_handler() -> impl axum::response::IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "text/css")], STYLE_CSS)
}

/// Sert le fichier JavaScript de l'interface web.
pub async fn js_handler() -> impl axum::response::IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "application/javascript")], APP_JS)
}

/// Sert le module JavaScript d'authentification.
pub async fn auth_js_handler() -> impl axum::response::IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "application/javascript")], AUTH_JS)
}

/// Sert le favicon SVG (icone d'onglet du navigateur).
pub async fn favicon_handler() -> impl axum::response::IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "image/svg+xml")], FAVICON_SVG)
}

/// Sert le module JavaScript i18n.
pub async fn i18n_js_handler() -> impl axum::response::IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "application/javascript")], I18N_JS)
}

/// GET /dashboard — Sert la page HTML du dashboard de monitoring.
pub async fn dashboard_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(DASHBOARD_HTML)
}

/// GET /pipeline-editor — Sert l'editeur visuel du pipeline cognitif.
pub async fn pipeline_editor_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(PIPELINE_EDITOR_HTML)
}

/// GET /brain-map — Sert la carte architecturale du cerveau (mindmap radiale).
pub async fn brain_map_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(BRAIN_MAP_HTML)
}

/// GET /i18n/:lang.json — Sert un fichier de traduction embarque.
pub async fn i18n_handler(
    axum::extract::Path(lang): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    let content = match lang.as_str() {
        "fr.json" => I18N_FR,
        "en.json" => I18N_EN,
        "de.json" => I18N_DE,
        "es.json" => I18N_ES,
        "it.json" => I18N_IT,
        "pt.json" => I18N_PT,
        "ru.json" => I18N_RU,
        "zh.json" => I18N_ZH,
        "ja.json" => I18N_JA,
        "ar.json" => I18N_AR,
        _ => I18N_FR,
    };
    ([(axum::http::header::CONTENT_TYPE, "application/json; charset=utf-8")], content)
}
