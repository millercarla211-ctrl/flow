//! Search engine trait definition.

use std::borrow::Cow;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::category::SearchCategory;
use crate::error::Result;
use crate::query::SearchQuery;
use crate::result::SearchResult;

/// How this adapter is allowed or expected to access its upstream provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineAccessModel {
    NoKeyOpenEndpoint,
    NoKeyRateLimited,
    OptionalApiKey,
    RequiresApiKey,
    SelfHostedInstance,
    LocalConfigured,
    HtmlScraperBrittle,
    NotAcceptable,
    Unknown,
}

/// Configuration needed before an adapter can be queried.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineConfigRequirement {
    ApiKey,
    BaseUrl,
    Token,
    ClientCredentials,
    IndexName,
    Username,
    Model,
    DatabaseUrl,
    CargoFeature,
}

/// The primary implementation strategy for an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineImplementation {
    JsonApi,
    XmlApi,
    RssFeed,
    HtmlScraper,
    LocalIndex,
    Database,
    AiCompletion,
    Unknown,
}

/// Operational metadata for catalog and health reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineAdapterMetadata {
    pub enabled_by_default: bool,
    pub configured: bool,
    pub access_model: EngineAccessModel,
    pub implementation: EngineImplementation,
    pub config_requirements: Vec<EngineConfigRequirement>,
    pub docs_url: Option<Cow<'static, str>>,
    pub skip_reason: Option<Cow<'static, str>>,
    pub notes: Option<Cow<'static, str>>,
}

/// Catalog entry combining provider identity with operational metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCatalogEntry {
    pub metadata: EngineMetadata,
    pub adapter: EngineAdapterMetadata,
}

/// Aggregate registry counts used by operator-facing status surfaces.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineCatalogSummary {
    pub registered: usize,
    pub enabled_by_default: usize,
    pub configured: usize,
    pub effective_enabled: usize,
    pub skipped: usize,
    pub requires_api_key: usize,
    pub requires_base_url: usize,
    pub html_scraper_brittle: usize,
}

/// Metadata describing a search engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetadata {
    /// Unique identifier (e.g., "google", "duckduckgo").
    pub name: Cow<'static, str>,

    /// Human-readable display name.
    pub display_name: Cow<'static, str>,

    /// URL of the engine's homepage.
    pub homepage: Cow<'static, str>,

    /// Categories this engine supports (stack-allocated for ≤4 categories).
    pub categories: SmallVec<[SearchCategory; 4]>,

    /// Whether this engine is enabled by default.
    pub enabled: bool,

    /// Timeout for requests to this engine (ms).
    pub timeout_ms: u64,

    /// Weight for result scoring (higher = more trusted).
    pub weight: f64,
}

impl EngineAdapterMetadata {
    pub fn no_key_open(
        implementation: EngineImplementation,
        docs_url: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            enabled_by_default: true,
            configured: true,
            access_model: EngineAccessModel::NoKeyOpenEndpoint,
            implementation,
            config_requirements: Vec::new(),
            docs_url: Some(docs_url.into()),
            skip_reason: None,
            notes: None,
        }
    }

    pub fn no_key_rate_limited(
        implementation: EngineImplementation,
        docs_url: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            enabled_by_default: true,
            configured: true,
            access_model: EngineAccessModel::NoKeyRateLimited,
            implementation,
            config_requirements: Vec::new(),
            docs_url: Some(docs_url.into()),
            skip_reason: None,
            notes: None,
        }
    }

    pub fn html_scraper_brittle(docs_url: impl Into<Cow<'static, str>>) -> Self {
        Self {
            enabled_by_default: false,
            configured: true,
            access_model: EngineAccessModel::HtmlScraperBrittle,
            implementation: EngineImplementation::HtmlScraper,
            config_requirements: Vec::new(),
            docs_url: Some(docs_url.into()),
            skip_reason: Some(
                "disabled by default because this adapter scrapes upstream HTML".into(),
            ),
            notes: None,
        }
    }

    pub fn missing_config(
        access_model: EngineAccessModel,
        implementation: EngineImplementation,
        config_requirements: Vec<EngineConfigRequirement>,
        docs_url: impl Into<Cow<'static, str>>,
        skip_reason: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            enabled_by_default: false,
            configured: false,
            access_model,
            implementation,
            config_requirements,
            docs_url: Some(docs_url.into()),
            skip_reason: Some(skip_reason.into()),
            notes: None,
        }
    }

    pub fn from_metadata(metadata: &EngineMetadata) -> Self {
        Self {
            enabled_by_default: metadata.enabled,
            configured: metadata.enabled,
            access_model: EngineAccessModel::Unknown,
            implementation: EngineImplementation::Unknown,
            config_requirements: Vec::new(),
            docs_url: None,
            skip_reason: (!metadata.enabled).then(|| "disabled by default".into()),
            notes: None,
        }
    }

    pub fn effective_enabled(&self) -> bool {
        self.enabled_by_default && self.configured
    }
}

impl EngineCatalogSummary {
    pub fn from_entries(entries: &[EngineCatalogEntry]) -> Self {
        let mut summary = Self {
            registered: entries.len(),
            ..Self::default()
        };

        for entry in entries {
            let adapter = &entry.adapter;
            if adapter.enabled_by_default {
                summary.enabled_by_default += 1;
            }
            if adapter.configured {
                summary.configured += 1;
            }
            if entry.metadata.enabled && adapter.configured {
                summary.effective_enabled += 1;
            }
            if !entry.metadata.enabled || !adapter.configured {
                summary.skipped += 1;
            }
            if adapter
                .config_requirements
                .contains(&EngineConfigRequirement::ApiKey)
                || adapter
                    .config_requirements
                    .contains(&EngineConfigRequirement::Token)
                || adapter
                    .config_requirements
                    .contains(&EngineConfigRequirement::ClientCredentials)
            {
                summary.requires_api_key += 1;
            }
            if adapter
                .config_requirements
                .contains(&EngineConfigRequirement::BaseUrl)
            {
                summary.requires_base_url += 1;
            }
            if adapter.access_model == EngineAccessModel::HtmlScraperBrittle {
                summary.html_scraper_brittle += 1;
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EngineAccessModel, EngineAdapterMetadata, EngineCatalogEntry, EngineCatalogSummary,
        EngineConfigRequirement, EngineImplementation, EngineMetadata,
    };
    use crate::category::SearchCategory;
    use smallvec::smallvec;

    #[test]
    fn catalog_summary_counts_tokens_as_credential_requirements() {
        let metadata = EngineMetadata {
            name: "token_provider".into(),
            display_name: "Token Provider".into(),
            homepage: "https://example.com".into(),
            categories: smallvec![SearchCategory::General],
            enabled: false,
            timeout_ms: 1000,
            weight: 1.0,
        };
        let adapter = EngineAdapterMetadata::missing_config(
            EngineAccessModel::RequiresApiKey,
            EngineImplementation::JsonApi,
            vec![EngineConfigRequirement::Token],
            "https://example.com",
            "missing token",
        );
        let summary =
            EngineCatalogSummary::from_entries(&[EngineCatalogEntry { metadata, adapter }]);

        assert_eq!(summary.requires_api_key, 1);
    }
}

/// The trait that every search engine must implement.
#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// Return metadata about this engine.
    fn metadata(&self) -> EngineMetadata;

    /// Perform a search and return results.
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;

    /// Optional: return autocomplete suggestions.
    async fn autocomplete(&self, _partial: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}
