//! Per-engine health tracking with adaptive timeouts and circuit-breaker pattern.
//!
//! Tracks P95 latency per engine over a rolling window of 100 requests and
//! automatically adjusts timeouts.  Unhealthy engines (>50% failure rate) are
//! skipped for 60 s before being retried.

use std::time::{Duration, Instant};

use dashmap::DashMap;
use metasearch_core::engine::{EngineCatalogEntry, EngineCatalogSummary};
use parking_lot::RwLock;
use serde::Serialize;

pub const RECENT_HEALTH_WINDOW: Duration = Duration::from_secs(15 * 60);

/// Rolling statistics for a single engine.
pub struct EngineStats {
    /// Ring buffer of last 100 response times (milliseconds).
    latencies: [u32; 100],
    cursor: usize,
    total_requests: u64,
    total_failures: u64,
    last_success: Option<Instant>,
    last_failure: Option<Instant>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineHealthSnapshot {
    pub name: String,
    pub probed: bool,
    pub total_requests: u64,
    pub total_failures: u64,
    pub failure_rate: f32,
    pub healthy: bool,
    pub recently_healthy: bool,
    pub adaptive_timeout_ms: u64,
    pub last_success_age_secs: Option<u64>,
    pub last_failure_age_secs: Option<u64>,
}

impl Default for EngineStats {
    fn default() -> Self {
        Self {
            latencies: [0u32; 100],
            cursor: 0,
            total_requests: 0,
            total_failures: 0,
            last_success: None,
            last_failure: None,
        }
    }
}

impl EngineStats {
    /// Record a successful response with the given latency.
    pub fn record_success(&mut self, ms: u32) {
        self.latencies[self.cursor % 100] = ms;
        self.cursor += 1;
        self.total_requests += 1;
        self.last_success = Some(Instant::now());
    }

    /// Record a failure (timeout or HTTP error).
    pub fn record_failure(&mut self) {
        // Store a high latency sentinel so it influences P95
        let sentinel = 30_000u32;
        self.latencies[self.cursor % 100] = sentinel;
        self.cursor += 1;
        self.total_requests += 1;
        self.total_failures += 1;
        self.last_failure = Some(Instant::now());
    }

    /// Compute adaptive timeout: P95 latency × 1.5, clamped to [1 s, 10 s].
    /// Returns the configured static timeout when there is insufficient data.
    pub fn adaptive_timeout(&self, static_ms: u64) -> Duration {
        let count = self.cursor.min(100);
        if count < 5 {
            return Duration::from_millis(static_ms);
        }

        let mut sorted = self.latencies[..count].to_vec();
        sorted.sort_unstable();
        let p95_idx = ((count as f32 * 0.95) as usize).min(count - 1);
        let p95_ms = sorted[p95_idx] as u64;
        let timeout_ms = (p95_ms as f64 * 1.5) as u64;

        Duration::from_millis(timeout_ms.clamp(1_000, 10_000))
    }

    /// Recent failure rate over tracked requests.
    pub fn failure_rate(&self) -> f32 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.total_failures as f32 / self.total_requests as f32
    }

    /// Circuit-breaker: skip if failure rate > 50% (with > 10 requests sampled),
    /// but retry automatically after 60 s.
    pub fn is_healthy(&self) -> bool {
        if self.total_requests > 10 && self.failure_rate() > 0.5 {
            if let Some(last_fail) = self.last_failure {
                return last_fail.elapsed() > Duration::from_secs(60);
            }
            return false;
        }
        true
    }

    pub fn snapshot(&self, name: &str, static_ms: u64) -> EngineHealthSnapshot {
        let now = Instant::now();
        let last_success_age_secs = self
            .last_success
            .map(|instant| now.saturating_duration_since(instant).as_secs());
        let recently_healthy = self
            .last_success
            .is_some_and(|instant| now.saturating_duration_since(instant) <= RECENT_HEALTH_WINDOW);

        EngineHealthSnapshot {
            name: name.to_string(),
            probed: self.total_requests > 0,
            total_requests: self.total_requests,
            total_failures: self.total_failures,
            failure_rate: self.failure_rate(),
            healthy: self.is_healthy(),
            recently_healthy,
            adaptive_timeout_ms: self.adaptive_timeout(static_ms).as_millis() as u64,
            last_success_age_secs,
            last_failure_age_secs: self
                .last_failure
                .map(|instant| now.saturating_duration_since(instant).as_secs()),
        }
    }
}

/// Tracks health stats for all registered engines.  Lock-free per engine.
#[derive(Default)]
pub struct EngineHealthTracker {
    stats: DashMap<String, RwLock<EngineStats>>,
}

impl EngineHealthTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful engine response.
    pub fn record_success(&self, engine: &str, latency_ms: u32) {
        self.stats
            .entry(engine.to_string())
            .or_default()
            .write()
            .record_success(latency_ms);
    }

    /// Record an engine failure.
    pub fn record_failure(&self, engine: &str) {
        self.stats
            .entry(engine.to_string())
            .or_default()
            .write()
            .record_failure();
    }

    /// Get the adaptive timeout for an engine, falling back to `static_ms`.
    pub fn timeout_for(&self, engine: &str, static_ms: u64) -> Duration {
        match self.stats.get(engine) {
            Some(s) => s.read().adaptive_timeout(static_ms),
            None => Duration::from_millis(static_ms),
        }
    }

    /// Returns `false` if the engine is circuit-broken.
    pub fn is_healthy(&self, engine: &str) -> bool {
        match self.stats.get(engine) {
            Some(s) => s.read().is_healthy(),
            None => true,
        }
    }

    pub fn tracked_engine_count(&self) -> usize {
        self.stats.len()
    }

    pub fn recently_healthy_engine_count(&self) -> usize {
        self.stats
            .iter()
            .filter(|entry| {
                entry
                    .value()
                    .read()
                    .last_success
                    .is_some_and(|instant| instant.elapsed() <= RECENT_HEALTH_WINDOW)
            })
            .count()
    }

    pub fn unhealthy_engines(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .stats
            .iter()
            .filter_map(|entry| {
                if entry.value().read().is_healthy() {
                    None
                } else {
                    Some(entry.key().clone())
                }
            })
            .collect();
        names.sort();
        names
    }

    pub fn snapshot(&self, engine: &str, static_ms: u64) -> Option<EngineHealthSnapshot> {
        self.stats
            .get(engine)
            .map(|stats| stats.read().snapshot(engine, static_ms))
    }

    pub fn snapshots(&self, default_timeout_ms: u64) -> Vec<EngineHealthSnapshot> {
        let mut snapshots: Vec<EngineHealthSnapshot> = self
            .stats
            .iter()
            .map(|entry| {
                entry
                    .value()
                    .read()
                    .snapshot(entry.key(), default_timeout_ms)
            })
            .collect();
        snapshots.sort_by(|left, right| left.name.cmp(&right.name));
        snapshots
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProviderProbeSummary {
    pub effective_enabled: usize,
    pub tracked: usize,
    pub recently_healthy: usize,
    pub unhealthy: usize,
    pub never_probed: usize,
    pub not_recently_healthy: usize,
}

impl ProviderProbeSummary {
    pub fn from_catalog(
        catalog: &[EngineCatalogEntry],
        health: &EngineHealthTracker,
        default_timeout_ms: u64,
    ) -> Self {
        let mut summary = Self::default();

        for entry in catalog
            .iter()
            .filter(|entry| entry.metadata.enabled && entry.adapter.configured)
        {
            summary.effective_enabled += 1;
            if let Some(snapshot) =
                health.snapshot(entry.metadata.name.as_ref(), default_timeout_ms)
            {
                summary.tracked += 1;
                if snapshot.healthy && snapshot.recently_healthy {
                    summary.recently_healthy += 1;
                }
                if !snapshot.healthy {
                    summary.unhealthy += 1;
                }
            }
        }

        summary.never_probed = summary.effective_enabled.saturating_sub(summary.tracked);
        summary.not_recently_healthy = summary
            .effective_enabled
            .saturating_sub(summary.recently_healthy);

        summary
    }
}

pub fn provider_config_ready(provider_status: &str) -> bool {
    !matches!(provider_status, "no_registered" | "unavailable")
}

pub fn provider_probe_recently_proven(probe_summary: &ProviderProbeSummary) -> bool {
    probe_summary.recently_healthy > 0
}

pub fn provider_pool_status(
    catalog_summary: &EngineCatalogSummary,
    probe_summary: &ProviderProbeSummary,
    max_concurrent_engines: usize,
) -> &'static str {
    if catalog_summary.registered == 0 {
        "no_registered"
    } else if catalog_summary.effective_enabled == 0 || max_concurrent_engines == 0 {
        "unavailable"
    } else if probe_summary.tracked == 0 {
        "cold"
    } else if probe_summary.unhealthy >= catalog_summary.effective_enabled {
        "unavailable"
    } else if probe_summary.unhealthy > 0 {
        "degraded"
    } else if probe_summary.tracked >= catalog_summary.effective_enabled
        && probe_summary.recently_healthy >= catalog_summary.effective_enabled
    {
        "healthy"
    } else if probe_summary.recently_healthy > 0 {
        "partially_healthy"
    } else {
        "degraded"
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EngineHealthTracker, EngineStats, ProviderProbeSummary, provider_pool_status,
        provider_probe_recently_proven,
    };
    use metasearch_core::{
        category::SearchCategory,
        engine::{
            EngineAccessModel, EngineAdapterMetadata, EngineCatalogEntry, EngineCatalogSummary,
            EngineImplementation, EngineMetadata,
        },
    };
    use smallvec::smallvec;

    #[test]
    fn health_snapshot_marks_recent_success() {
        let mut stats = EngineStats::default();
        stats.record_success(42);

        let snapshot = stats.snapshot("example", 5000);

        assert!(snapshot.probed);
        assert!(snapshot.healthy);
        assert!(snapshot.recently_healthy);
        assert_eq!(snapshot.total_requests, 1);
        assert_eq!(snapshot.total_failures, 0);
    }

    #[test]
    fn provider_status_never_reports_healthy_without_enough_effective_recent_probes() {
        let summary = EngineCatalogSummary {
            registered: 4,
            enabled_by_default: 4,
            configured: 4,
            effective_enabled: 4,
            skipped: 0,
            requires_api_key: 0,
            requires_base_url: 0,
            html_scraper_brittle: 0,
        };
        let probes = ProviderProbeSummary {
            effective_enabled: 4,
            tracked: 2,
            recently_healthy: 2,
            unhealthy: 0,
            never_probed: 2,
            not_recently_healthy: 2,
        };

        assert_eq!(
            provider_pool_status(&summary, &probes, 4),
            "partially_healthy"
        );
    }

    #[test]
    fn provider_probe_summary_counts_unprobed_and_not_recently_healthy_effective_engines() {
        let catalog = vec![
            catalog_entry("fresh", true, true),
            catalog_entry("failed", true, true),
            catalog_entry("cold", true, true),
            catalog_entry("disabled", false, true),
        ];
        let health = EngineHealthTracker::new();
        health.record_success("fresh", 42);
        health.record_failure("failed");

        let summary = ProviderProbeSummary::from_catalog(&catalog, &health, 5000);

        assert_eq!(summary.effective_enabled, 3);
        assert_eq!(summary.tracked, 2);
        assert_eq!(summary.recently_healthy, 1);
        assert_eq!(summary.never_probed, 1);
        assert_eq!(summary.not_recently_healthy, 2);
    }

    #[test]
    fn provider_probe_recently_proven_requires_recent_success() {
        let cold = ProviderProbeSummary {
            effective_enabled: 2,
            tracked: 0,
            recently_healthy: 0,
            unhealthy: 0,
            never_probed: 2,
            not_recently_healthy: 2,
        };
        let partial = ProviderProbeSummary {
            recently_healthy: 1,
            tracked: 1,
            never_probed: 1,
            not_recently_healthy: 1,
            ..cold.clone()
        };

        assert!(!provider_probe_recently_proven(&cold));
        assert!(provider_probe_recently_proven(&partial));
    }

    #[test]
    fn provider_probe_summary_does_not_count_circuit_broken_recent_success_as_healthy() {
        let catalog = vec![catalog_entry("flaky", true, true)];
        let health = EngineHealthTracker::new();
        health.record_success("flaky", 42);
        for _ in 0..11 {
            health.record_failure("flaky");
        }

        let summary = ProviderProbeSummary::from_catalog(&catalog, &health, 5000);

        assert_eq!(summary.tracked, 1);
        assert_eq!(summary.recently_healthy, 0);
        assert_eq!(summary.unhealthy, 1);
        assert_eq!(summary.not_recently_healthy, 1);
    }

    fn catalog_entry(name: &'static str, enabled: bool, configured: bool) -> EngineCatalogEntry {
        EngineCatalogEntry {
            metadata: EngineMetadata {
                name: name.into(),
                display_name: name.into(),
                homepage: "https://example.com".into(),
                categories: smallvec![SearchCategory::General],
                enabled,
                timeout_ms: 1000,
                weight: 1.0,
            },
            adapter: EngineAdapterMetadata {
                enabled_by_default: enabled,
                configured,
                access_model: EngineAccessModel::NoKeyOpenEndpoint,
                implementation: EngineImplementation::JsonApi,
                config_requirements: Vec::new(),
                docs_url: Some("https://example.com/docs".into()),
                skip_reason: None,
                notes: None,
            },
        }
    }
}
