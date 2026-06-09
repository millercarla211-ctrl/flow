//! Metasearch CLI — entry point for the application.

// mimalloc: 2-6x faster than system allocator, critical on musl targets.
// Works on all platforms (Windows, Linux, macOS, Alpine/musl).
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{path::Path, sync::Arc};

use clap::{Parser, Subcommand};
use metasearch_core::{
    engine::{EngineAdapterMetadata, EngineMetadata},
    query::SearchQuery,
};
use tracing_subscriber::EnvFilter;

use metasearch_core::config::Settings;
use metasearch_engine::EngineRegistry;
use metasearch_server::{
    app, cache::SearchCache, health::EngineHealthTracker, orchestrator::SearchOrchestrator,
    state::AppState, templates::Templates,
};

#[derive(Parser)]
#[command(name = "metasearch")]
#[command(about = "A blazing-fast, privacy-respecting metasearch engine")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Host to bind to
    #[arg(long)]
    host: Option<String>,

    /// Port to listen on
    #[arg(short, long)]
    port: Option<u16>,

    /// Path to templates directory
    #[arg(long)]
    templates: Option<String>,

    /// Path to static assets directory
    #[arg(long)]
    static_dir: Option<String>,

    /// Optional path to a TOML configuration file
    #[arg(long)]
    config: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the web server (default)
    Serve,
    /// List all registered engines
    Engines,
    /// Probe selected engines with one live query
    Probe {
        /// Comma-separated engine ids to probe
        #[arg(long)]
        engines: String,
        /// Query to send to each selected engine
        #[arg(short, long, default_value = "rust programming")]
        query: String,
        /// Per-engine timeout in milliseconds
        #[arg(long, default_value_t = 6000)]
        timeout_ms: u64,
        /// Also call disabled or unconfigured adapters
        #[arg(long)]
        include_disabled: bool,
        /// Required opt-in for live network calls
        #[arg(long)]
        allow_network: bool,
    },
    /// Print the effective configuration
    Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("metasearch=info".parse()?))
        .init();

    let cli = Cli::parse();

    // Build settings, preferring an explicit config path or the root config file.
    let mut settings = match cli.config.as_deref() {
        Some(path) => load_settings_from_path(path)?,
        None if Path::new("config.toml").exists() => load_settings_from_path("config.toml")?,
        None => Settings::default(),
    };
    let original_host = settings.server.host.clone();
    let original_port = settings.server.port;
    let original_base_url = settings.server.base_url.clone();

    if let Some(host) = cli.host {
        settings.server.host = host;
    }
    if let Some(port) = cli.port {
        settings.server.port = port;
    }
    if let Some(templates_dir) = cli.templates.as_ref() {
        settings.server.templates_dir = templates_dir.clone();
    }
    if let Some(static_dir) = cli.static_dir.as_ref() {
        settings.server.static_dir = static_dir.clone();
    }

    let original_base_host = if original_host == "0.0.0.0" {
        "localhost"
    } else {
        original_host.as_str()
    };
    let original_derived_base_url = format!("http://{}:{}", original_base_host, original_port);
    let current_base_host = if settings.server.host == "0.0.0.0" {
        "localhost"
    } else {
        settings.server.host.as_str()
    };

    if original_base_url.trim().is_empty()
        || original_base_url == "http://localhost:8888"
        || original_base_url == original_derived_base_url
    {
        settings.server.base_url = format!("http://{}:{}", current_base_host, settings.server.port);
    }

    // Build optimized HTTP client with connection pooling
    let http_client = reqwest::Client::builder()
        .user_agent("Metasearch/0.1 (https://github.com/najmus-sakib-hossain/metasearch)")
        .timeout(std::time::Duration::from_secs(10))
        .connect_timeout(std::time::Duration::from_secs(3))
        .pool_max_idle_per_host(50) // More connections per host
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .tcp_nodelay(true) // Disable Nagle's algorithm for lower latency
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .http2_adaptive_window(true) // Better HTTP/2 performance
        .http2_keep_alive_interval(std::time::Duration::from_secs(30))
        .http2_keep_alive_timeout(std::time::Duration::from_secs(10))
        .http2_keep_alive_while_idle(true)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .build()?;

    // Register all built-in engines using with_defaults()
    // Clone the client so we keep one for the registry and one for general use (autocomplete, etc.)
    let shared_client = http_client.clone();
    let registry = EngineRegistry::with_defaults(http_client);
    let engine_count = registry.count();
    let registry = Arc::new(registry);

    validate_asset_dir(&settings.server.templates_dir, "template")?;
    validate_asset_dir(&settings.server.static_dir, "static")?;

    // Load templates
    let templates = Templates::new(&settings.server.templates_dir)?;

    // Build the performance stack
    let cache = SearchCache::new(settings.cache.max_entries, settings.cache.ttl_secs);
    let health = Arc::new(EngineHealthTracker::new());
    let max_engines = settings.search.max_concurrent_engines;
    let orchestrator = Arc::new(SearchOrchestrator::new(
        Arc::clone(&registry),
        cache.clone(),
        Arc::clone(&health),
        max_engines,
    ));

    let state = Arc::new(AppState {
        cache,
        engine_registry: registry,
        template_dir: settings.server.templates_dir.clone(),
        static_dir: settings.server.static_dir.clone(),
        templates: Arc::new(templates),
        orchestrator,
        health,
        settings,
        http_client: shared_client,
    });

    match cli.command.unwrap_or(Commands::Serve) {
        Commands::Serve => {
            tracing::info!("Registered {} search engines", engine_count);
            app::run(state).await?;
        }
        Commands::Engines => {
            println!("Registered engines ({}):", engine_count);
            let catalog = state.engine_registry.adapter_catalog();
            for entry in catalog {
                let status = if entry.metadata.enabled && entry.adapter.configured {
                    "enabled"
                } else {
                    "skipped"
                };
                println!(
                    "  - {:<28} {:<8} {}",
                    entry.metadata.name, status, entry.metadata.display_name
                );
            }
        }
        Commands::Probe {
            engines,
            query,
            timeout_ms,
            include_disabled,
            allow_network,
        } => {
            if !allow_network {
                anyhow::bail!(
                    "Provider probes make live network calls. Re-run with --allow-network."
                );
            }
            run_probe(
                state.engine_registry.as_ref(),
                &engines,
                &query,
                timeout_ms,
                include_disabled,
            )
            .await?;
        }
        Commands::Config => {
            println!("{}", serde_json::to_string_pretty(&state.settings)?);
        }
    }

    Ok(())
}

fn load_settings_from_path(path: &str) -> anyhow::Result<Settings> {
    let contents = std::fs::read_to_string(path)?;
    let settings = toml::from_str::<Settings>(&contents)?;
    Ok(settings)
}

fn validate_asset_dir(path: &str, label: &str) -> anyhow::Result<()> {
    let metadata = std::fs::metadata(path).map_err(|error| {
        anyhow::anyhow!(
            "{} directory `{}` is not accessible: {}",
            label,
            path,
            error
        )
    })?;
    if !metadata.is_dir() {
        anyhow::bail!("{} directory `{}` is not a directory", label, path);
    }
    Ok(())
}

async fn run_probe(
    registry: &EngineRegistry,
    selected_engines: &str,
    query: &str,
    timeout_ms: u64,
    include_disabled: bool,
) -> anyhow::Result<()> {
    let engine_names: Vec<String> = selected_engines
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    if engine_names.is_empty() {
        anyhow::bail!("At least one engine id is required.");
    }

    let catalog = registry.adapter_catalog();
    let mut search_query = SearchQuery::new(query);
    search_query.engines = engine_names.clone();
    let timeout = std::time::Duration::from_millis(timeout_ms.max(1));

    println!(
        "{:<28} {:<22} {:<12} {:>7} {:>10} reason",
        "engine", "display", "status", "results", "latency_ms"
    );
    println!("{}", "-".repeat(92));

    for name in engine_names {
        let Some(engine) = registry.get(&name) else {
            println!(
                "{:<28} {:<22} {:<12} {:>7} {:>10} not registered",
                name, "-", "unknown", 0, 0
            );
            continue;
        };

        let metadata = engine.metadata();
        let adapter = catalog
            .iter()
            .find(|entry| entry.metadata.name == metadata.name)
            .map(|entry| &entry.adapter);

        if !include_disabled {
            if let Some((status, reason)) = probe_skip_status(&metadata, adapter) {
                println!(
                    "{:<28} {:<22} {:<12} {:>7} {:>10} {}",
                    metadata.name, metadata.display_name, status, 0, 0, reason
                );
                continue;
            }
        }

        let started = std::time::Instant::now();
        let result = tokio::time::timeout(timeout, engine.search(&search_query)).await;
        let latency_ms = started.elapsed().as_millis();

        match result {
            Ok(Ok(results)) => {
                let malformed = results.iter().any(|result| {
                    result.title.trim().is_empty()
                        || result.url.trim().is_empty()
                        || result.engine.trim().is_empty()
                });
                let status = if malformed {
                    "malformed"
                } else if results.is_empty() {
                    "empty"
                } else {
                    "ok"
                };
                println!(
                    "{:<28} {:<22} {:<12} {:>7} {:>10} {}",
                    metadata.name,
                    metadata.display_name,
                    status,
                    results.len(),
                    latency_ms,
                    if malformed {
                        "missing title/url/engine"
                    } else {
                        ""
                    }
                );
            }
            Ok(Err(error)) => {
                println!(
                    "{:<28} {:<22} {:<12} {:>7} {:>10} {}",
                    metadata.name,
                    metadata.display_name,
                    "error",
                    0,
                    latency_ms,
                    short_error(&error.to_string())
                );
            }
            Err(_) => {
                println!(
                    "{:<28} {:<22} {:<12} {:>7} {:>10} timeout after {}ms",
                    metadata.name, metadata.display_name, "timeout", 0, latency_ms, timeout_ms
                );
            }
        }
    }

    Ok(())
}

fn probe_skip_status(
    metadata: &EngineMetadata,
    adapter: Option<&EngineAdapterMetadata>,
) -> Option<(&'static str, String)> {
    let reason = || {
        adapter
            .and_then(|adapter| adapter.skip_reason.as_ref())
            .map(|reason| reason.to_string())
    };

    if adapter.is_some_and(|adapter| !adapter.configured) {
        return Some((
            "skipped_unconfigured",
            reason().unwrap_or_else(|| "missing required configuration".to_string()),
        ));
    }

    if !metadata.enabled {
        return Some((
            "disabled",
            reason().unwrap_or_else(|| "disabled by default".to_string()),
        ));
    }

    None
}

fn short_error(message: &str) -> String {
    const MAX_LEN: usize = 96;
    let one_line = message.split_whitespace().collect::<Vec<_>>().join(" ");
    if one_line.len() <= MAX_LEN {
        one_line
    } else {
        format!("{}...", one_line.chars().take(MAX_LEN).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use metasearch_core::engine::{
        EngineAccessModel, EngineAdapterMetadata, EngineConfigRequirement, EngineImplementation,
        EngineMetadata,
    };

    use super::probe_skip_status;

    fn metadata(enabled: bool) -> EngineMetadata {
        EngineMetadata {
            name: "candidate".into(),
            display_name: "Candidate".into(),
            homepage: "https://example.com".into(),
            categories: Default::default(),
            enabled,
            timeout_ms: 1000,
            weight: 1.0,
        }
    }

    #[test]
    fn probe_skip_status_distinguishes_disabled_from_unconfigured() {
        let html_scraper = EngineAdapterMetadata::html_scraper_brittle("https://example.com/docs");
        let disabled = probe_skip_status(&metadata(false), Some(&html_scraper))
            .expect("disabled HTML scraper should be skipped");
        assert_eq!(disabled.0, "disabled");
        assert!(disabled.1.contains("HTML"));

        let missing_config = EngineAdapterMetadata::missing_config(
            EngineAccessModel::SelfHostedInstance,
            EngineImplementation::JsonApi,
            vec![EngineConfigRequirement::BaseUrl],
            "https://example.com/docs",
            "missing base URL",
        );
        let unconfigured = probe_skip_status(&metadata(false), Some(&missing_config))
            .expect("missing config should be skipped");
        assert_eq!(unconfigured.0, "skipped_unconfigured");
        assert_eq!(unconfigured.1, "missing base URL");

        assert!(probe_skip_status(&metadata(true), None).is_none());
    }
}
