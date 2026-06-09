//! Health check endpoint.

use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use metasearch_core::engine::EngineCatalogSummary;

use crate::health::{
    ProviderProbeSummary, provider_config_ready, provider_pool_status,
    provider_probe_recently_proven,
};
use crate::state::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_check))
        .route("/livez", get(livez))
        .route("/readyz", get(readyz))
}

async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let unhealthy = state.health.unhealthy_engines();
    let config_warnings = state.runtime_warnings();
    let catalog = state.engine_registry.adapter_catalog();
    let provider_summary = EngineCatalogSummary::from_entries(&catalog);
    let provider_probe_summary = ProviderProbeSummary::from_catalog(
        &catalog,
        &state.health,
        state.settings.search.request_timeout_ms,
    );
    let provider_status = provider_pool_status(
        &provider_summary,
        &provider_probe_summary,
        state.settings.search.max_concurrent_engines,
    );
    let provider_config_ready = provider_config_ready(provider_status);
    let provider_probe_proven = provider_probe_recently_proven(&provider_probe_summary);
    let status = health_status(&state, &unhealthy, &config_warnings);
    let status_code = if status == "error" {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };
    let raw_health_counts = serde_json::json!({
        "tracked": state.health.tracked_engine_count(),
        "recently_healthy": state.health.recently_healthy_engine_count(),
        "unhealthy": unhealthy.len(),
    });

    (
        status_code,
        Json(serde_json::json!({
            "status": status,
            "provider_status": provider_status,
            "provider_config_ready": provider_config_ready,
            "provider_probe_recently_proven": provider_probe_proven,
            "version": env!("CARGO_PKG_VERSION"),
            "engine_count": state.engine_registry.count(),
            "provider_summary": provider_summary,
            "provider_probe_summary": provider_probe_summary.clone(),
            "recently_healthy_engine_count": provider_probe_summary.recently_healthy,
            "probe_window_secs": crate::health::RECENT_HEALTH_WINDOW.as_secs(),
            "tracked_engines": provider_probe_summary.tracked,
            "unhealthy_engine_count": provider_probe_summary.unhealthy,
            "unhealthy_engines": unhealthy,
            "raw_health_counts": raw_health_counts,
            "warning_count": config_warnings.len(),
            "config_warnings": config_warnings,
            "remote_autocomplete_enabled": state.settings.search.remote_autocomplete_enabled,
            "cache_enabled": state.settings.cache.enabled,
            "rate_limit_enabled": state.settings.rate_limit.enabled,
            "bot_detection_enabled": state.settings.bot_detection.enabled,
            "security_headers_enabled": state.settings.server.security_headers_enabled,
            "permissive_cors": state.settings.server.permissive_cors,
            "allowed_origins": state.settings.server.allowed_origins,
            "trust_forwarded_headers": state.settings.server.trust_forwarded_headers,
        })),
    )
}

async fn livez(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "kind": "live",
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}

async fn readyz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let unhealthy = state.health.unhealthy_engines();
    let config_warnings = state.runtime_warnings();
    let status = health_status(&state, &unhealthy, &config_warnings);
    let catalog = state.engine_registry.adapter_catalog();
    let provider_summary = EngineCatalogSummary::from_entries(&catalog);
    let provider_probe_summary = ProviderProbeSummary::from_catalog(
        &catalog,
        &state.health,
        state.settings.search.request_timeout_ms,
    );
    let provider_status = provider_pool_status(
        &provider_summary,
        &provider_probe_summary,
        state.settings.search.max_concurrent_engines,
    );
    let provider_config_ready = provider_config_ready(provider_status);
    let provider_probe_proven = provider_probe_recently_proven(&provider_probe_summary);
    let provider_ready = provider_config_ready && provider_probe_proven;
    let ready = status != "error" && provider_ready;
    let status_code = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(serde_json::json!({
            "status": if ready { "ok" } else { "error" },
            "kind": "ready",
            "ready": ready,
            "runtime_status": status,
            "provider_status": provider_status,
            "provider_ready": provider_ready,
            "provider_config_ready": provider_config_ready,
            "provider_probe_recently_proven": provider_probe_proven,
            "version": env!("CARGO_PKG_VERSION"),
            "engine_count": state.engine_registry.count(),
            "provider_summary": provider_summary,
            "provider_probe_summary": provider_probe_summary.clone(),
            "recently_healthy_engine_count": provider_probe_summary.recently_healthy,
            "probe_window_secs": crate::health::RECENT_HEALTH_WINDOW.as_secs(),
            "tracked_engines": provider_probe_summary.tracked,
            "unhealthy_engine_count": provider_probe_summary.unhealthy,
            "warning_count": config_warnings.len(),
        })),
    )
}

fn health_status(
    state: &AppState,
    unhealthy: &[String],
    config_warnings: &[String],
) -> &'static str {
    if state.engine_registry.count() == 0 {
        "error"
    } else if unhealthy.is_empty() && config_warnings.is_empty() {
        "ok"
    } else {
        "degraded"
    }
}
