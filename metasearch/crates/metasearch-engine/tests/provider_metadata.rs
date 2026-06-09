use metasearch_core::engine::{EngineAccessModel, EngineConfigRequirement, EngineImplementation};
use metasearch_engine::EngineRegistry;
use reqwest::Client;

#[test]
fn adapter_catalog_distinguishes_configured_and_skipped_engines() {
    let registry = EngineRegistry::with_defaults(Client::new());
    let catalog = registry.adapter_catalog();

    assert_eq!(catalog.len(), registry.count());

    let wikipedia = catalog
        .iter()
        .find(|entry| entry.metadata.name == "wikipedia")
        .expect("wikipedia should be registered");
    assert!(wikipedia.metadata.enabled);
    assert!(wikipedia.adapter.enabled_by_default);
    assert!(wikipedia.adapter.configured);
    assert_eq!(
        wikipedia.adapter.access_model,
        EngineAccessModel::NoKeyRateLimited
    );
    assert_eq!(
        wikipedia.adapter.implementation,
        EngineImplementation::JsonApi
    );
    assert!(wikipedia.adapter.config_requirements.is_empty());
    assert!(wikipedia.adapter.skip_reason.is_none());

    let brave_api = catalog
        .iter()
        .find(|entry| entry.metadata.name == "braveapi")
        .expect("braveapi should be registered");
    assert!(!brave_api.metadata.enabled);
    assert!(!brave_api.adapter.enabled_by_default);
    assert!(!brave_api.adapter.configured);
    assert_eq!(
        brave_api.adapter.access_model,
        EngineAccessModel::RequiresApiKey
    );
    assert_eq!(
        brave_api.adapter.implementation,
        EngineImplementation::JsonApi
    );
    assert!(
        brave_api
            .adapter
            .config_requirements
            .contains(&EngineConfigRequirement::ApiKey)
    );
    assert!(
        brave_api
            .adapter
            .skip_reason
            .as_deref()
            .is_some_and(|reason| { reason.contains("API key") })
    );

    let searxng = catalog
        .iter()
        .find(|entry| entry.metadata.name == "searx_engine")
        .expect("searx_engine should be registered");
    assert!(!searxng.metadata.enabled);
    assert!(!searxng.adapter.enabled_by_default);
    assert!(!searxng.adapter.configured);
    assert_eq!(
        searxng.adapter.access_model,
        EngineAccessModel::SelfHostedInstance
    );
    assert!(
        searxng
            .adapter
            .config_requirements
            .contains(&EngineConfigRequirement::BaseUrl)
    );
}

#[test]
fn reliable_no_key_provider_metadata_is_registered() {
    let registry = EngineRegistry::with_defaults(Client::new());
    let catalog = registry.adapter_catalog();

    for name in ["doaj", "internet_archive", "packagist", "rubygems"] {
        let entry = catalog
            .iter()
            .find(|entry| entry.metadata.name == name)
            .unwrap_or_else(|| panic!("{name} should be registered"));

        assert!(
            entry.metadata.enabled,
            "{name} should be enabled by default"
        );
        assert!(
            entry.adapter.enabled_by_default,
            "{name} should be enabled by default"
        );
        assert!(
            entry.adapter.configured,
            "{name} should need no local config"
        );
        assert!(
            matches!(
                entry.adapter.access_model,
                EngineAccessModel::NoKeyOpenEndpoint | EngineAccessModel::NoKeyRateLimited
            ),
            "{name} should be no-key, not an HTML scraper or key-required adapter"
        );
        assert_eq!(entry.adapter.implementation, EngineImplementation::JsonApi);
        assert!(
            entry
                .adapter
                .docs_url
                .as_deref()
                .is_some_and(|url| url.starts_with("https://")),
            "{name} should carry a docs receipt URL"
        );
    }
}

#[test]
fn duplicate_ads_adapter_is_not_registered_by_default() {
    let registry = EngineRegistry::with_defaults(Client::new());
    let catalog = registry.adapter_catalog();

    let ads_count = catalog
        .iter()
        .filter(|entry| entry.metadata.name == "ads")
        .count();
    assert_eq!(ads_count, 1, "ADS should have one canonical default id");
    assert!(
        catalog
            .iter()
            .all(|entry| entry.metadata.name != "astrophysics_data_system"),
        "legacy ADS id should not be registered beside ads"
    );
}

#[test]
fn html_scraper_adapters_are_not_marked_as_json_apis() {
    let registry = EngineRegistry::with_defaults(Client::new());
    let catalog = registry.adapter_catalog();

    for name in ["pypi", "sourcehut"] {
        let entry = catalog
            .iter()
            .find(|entry| entry.metadata.name == name)
            .unwrap_or_else(|| panic!("{name} should be registered"));

        assert_eq!(
            entry.adapter.access_model,
            EngineAccessModel::HtmlScraperBrittle,
            "{name} should be disclosed as brittle HTML parsing"
        );
        assert_eq!(
            entry.adapter.implementation,
            EngineImplementation::HtmlScraper,
            "{name} should not be reported as a JSON API"
        );
    }

    let sourcehut = catalog
        .iter()
        .find(|entry| entry.metadata.name == "sourcehut")
        .expect("sourcehut should be registered");
    assert!(
        !sourcehut.metadata.enabled,
        "sourcehut should be disabled by default because it is a brittle HTML scraper"
    );
    assert!(
        sourcehut
            .adapter
            .skip_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("HTML")),
        "sourcehut should explain why it is skipped"
    );
}

#[test]
fn pypi_html_scraper_policy_is_disabled_but_configured() {
    let registry = EngineRegistry::with_defaults(Client::new());
    let catalog = registry.adapter_catalog();
    let pypi = catalog
        .iter()
        .find(|entry| entry.metadata.name == "pypi")
        .expect("pypi should be registered");

    assert!(!pypi.metadata.enabled);
    assert!(!pypi.adapter.enabled_by_default);
    assert!(pypi.adapter.configured);
    assert_eq!(
        pypi.adapter.access_model,
        EngineAccessModel::HtmlScraperBrittle
    );
    assert_eq!(
        pypi.adapter.implementation,
        EngineImplementation::HtmlScraper
    );
    assert!(pypi.adapter.config_requirements.is_empty());
    assert!(
        pypi.adapter
            .skip_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("HTML")),
        "pypi should explain that broad package search is skipped because it scrapes HTML"
    );
}

#[test]
fn next_open_provider_batch_metadata_is_registered() {
    let registry = EngineRegistry::with_defaults(Client::new());
    let catalog = registry.adapter_catalog();

    let expected = [
        ("maven_central", EngineAccessModel::NoKeyRateLimited, true),
        ("nuget", EngineAccessModel::NoKeyOpenEndpoint, true),
        ("europe_pmc", EngineAccessModel::NoKeyOpenEndpoint, true),
        ("dbpedia", EngineAccessModel::NoKeyOpenEndpoint, true),
        (
            "open_food_facts",
            EngineAccessModel::NoKeyRateLimited,
            false,
        ),
        ("met_museum", EngineAccessModel::NoKeyRateLimited, true),
        ("datacite", EngineAccessModel::NoKeyRateLimited, true),
        ("zenodo", EngineAccessModel::NoKeyRateLimited, true),
        ("artifact_hub", EngineAccessModel::NoKeyRateLimited, true),
        ("nvd", EngineAccessModel::OptionalApiKey, true),
        ("pubchem", EngineAccessModel::NoKeyRateLimited, true),
        ("loc", EngineAccessModel::NoKeyRateLimited, true),
        ("openalex", EngineAccessModel::NoKeyRateLimited, true),
        ("openverse", EngineAccessModel::NoKeyRateLimited, true),
    ];

    for (name, access_model, enabled_by_default) in expected {
        let entry = catalog
            .iter()
            .find(|entry| entry.metadata.name == name)
            .unwrap_or_else(|| panic!("{name} should be registered"));

        assert_eq!(
            entry.metadata.enabled, enabled_by_default,
            "{name} default-enabled state"
        );
        assert!(
            entry.adapter.configured,
            "{name} should need no local config"
        );
        assert_eq!(entry.adapter.access_model, access_model, "{name}");
        assert_eq!(entry.adapter.implementation, EngineImplementation::JsonApi);
        assert!(
            entry
                .adapter
                .docs_url
                .as_deref()
                .is_some_and(|url| url.starts_with("https://")),
            "{name} should carry an HTTPS documentation reference"
        );
    }
}

#[test]
fn key_required_adapters_are_not_reported_as_unknown_or_no_key() {
    let registry = EngineRegistry::with_defaults(Client::new());
    let catalog = registry.adapter_catalog();

    for name in ["deepl", "github_code"] {
        let entry = catalog
            .iter()
            .find(|entry| entry.metadata.name == name)
            .unwrap_or_else(|| panic!("{name} should be registered"));

        assert!(
            !entry.metadata.enabled,
            "{name} should be disabled without config"
        );
        assert!(!entry.adapter.configured, "{name} should be unconfigured");
        assert_eq!(
            entry.adapter.access_model,
            EngineAccessModel::RequiresApiKey
        );
        assert!(
            entry.adapter.config_requirements.iter().any(|requirement| {
                matches!(
                    requirement,
                    EngineConfigRequirement::ApiKey | EngineConfigRequirement::Token
                )
            }),
            "{name} should disclose its credential requirement"
        );
    }
}

#[test]
fn self_hosted_adapters_report_exact_local_requirements() {
    let registry = EngineRegistry::with_defaults(Client::new());
    let catalog = registry.adapter_catalog();

    let expected = [
        (
            "Elasticsearch",
            vec![
                EngineConfigRequirement::BaseUrl,
                EngineConfigRequirement::IndexName,
            ],
        ),
        (
            "MeiliSearch",
            vec![
                EngineConfigRequirement::BaseUrl,
                EngineConfigRequirement::IndexName,
            ],
        ),
        (
            "tubearchivist",
            vec![
                EngineConfigRequirement::BaseUrl,
                EngineConfigRequirement::Token,
            ],
        ),
    ];

    for (name, requirements) in expected {
        let entry = catalog
            .iter()
            .find(|entry| entry.metadata.name == name)
            .unwrap_or_else(|| panic!("{name} should be registered"));

        assert_eq!(
            entry.adapter.access_model,
            EngineAccessModel::SelfHostedInstance,
            "{name}"
        );
        for requirement in requirements {
            assert!(
                entry.adapter.config_requirements.contains(&requirement),
                "{name} should disclose {requirement:?}"
            );
        }
    }
}

#[test]
fn community_and_developer_api_metadata_is_honest() {
    let registry = EngineRegistry::with_defaults(Client::new());
    let catalog = registry.adapter_catalog();

    let expected = [
        (
            "hackernews",
            EngineAccessModel::NoKeyRateLimited,
            EngineImplementation::JsonApi,
            true,
        ),
        (
            "gitlab",
            EngineAccessModel::NoKeyRateLimited,
            EngineImplementation::JsonApi,
            true,
        ),
        (
            "stackexchange",
            EngineAccessModel::NoKeyRateLimited,
            EngineImplementation::JsonApi,
            true,
        ),
        (
            "semantic_scholar",
            EngineAccessModel::OptionalApiKey,
            EngineImplementation::JsonApi,
            true,
        ),
        (
            "sourcehut",
            EngineAccessModel::HtmlScraperBrittle,
            EngineImplementation::HtmlScraper,
            false,
        ),
    ];

    for (name, access_model, implementation, enabled_by_default) in expected {
        let entry = catalog
            .iter()
            .find(|entry| entry.metadata.name == name)
            .unwrap_or_else(|| panic!("{name} should be registered"));

        assert_eq!(
            entry.metadata.enabled, enabled_by_default,
            "{name} default-enabled state"
        );
        assert_eq!(entry.adapter.enabled_by_default, enabled_by_default);
        assert_eq!(entry.adapter.access_model, access_model, "{name}");
        assert_eq!(entry.adapter.implementation, implementation, "{name}");
        assert!(
            entry
                .adapter
                .docs_url
                .as_deref()
                .is_some_and(|url| url.starts_with("https://")),
            "{name} should carry an HTTPS documentation reference"
        );
    }
}

#[test]
fn legacy_openalex_and_openverse_ids_remain_lookup_aliases() {
    let registry = EngineRegistry::with_defaults(Client::new());

    assert!(registry.get("openalex").is_some());
    assert!(registry.get("OpenAlex").is_some());
    assert!(registry.get("openverse").is_some());
    assert!(registry.get("Openverse").is_some());
}

#[test]
fn searchcode_duplicate_is_exposed_as_single_canonical_adapter() {
    let registry = EngineRegistry::with_defaults(Client::new());
    let catalog = registry.adapter_catalog();

    assert!(registry.get("searchcode_code").is_some());
    assert!(registry.get("searchcode").is_some());
    assert!(registry.get("Searchcode").is_some());

    let canonical_count = catalog
        .iter()
        .filter(|entry| entry.metadata.name == "searchcode_code")
        .count();
    assert_eq!(canonical_count, 1);
    assert!(
        catalog
            .iter()
            .all(|entry| entry.metadata.name != "Searchcode"),
        "legacy Searchcode id should not be registered beside searchcode_code"
    );

    let searchcode = catalog
        .iter()
        .find(|entry| entry.metadata.name == "searchcode_code")
        .expect("canonical Searchcode adapter should be registered");
    assert!(!searchcode.metadata.enabled);
    assert!(!searchcode.adapter.enabled_by_default);
    assert_eq!(
        searchcode.adapter.access_model,
        EngineAccessModel::NotAcceptable
    );
    assert_eq!(
        searchcode.adapter.implementation,
        EngineImplementation::JsonApi
    );
}

#[test]
fn legacy_ads_id_resolves_to_canonical_ads_without_catalog_duplicate() {
    let registry = EngineRegistry::with_defaults(Client::new());
    let catalog = registry.adapter_catalog();

    assert!(registry.get("ads").is_some());
    assert!(registry.get("astrophysics_data_system").is_some());
    assert_eq!(
        catalog
            .iter()
            .filter(|entry| entry.metadata.name == "ads")
            .count(),
        1
    );
    assert!(
        catalog
            .iter()
            .all(|entry| entry.metadata.name != "astrophysics_data_system"),
        "legacy ADS id should remain an alias, not a second registered adapter"
    );
}

#[test]
fn google_display_ids_have_api_safe_lookup_aliases() {
    let registry = EngineRegistry::with_defaults(Client::new());

    assert!(registry.get("Google News").is_some());
    assert!(registry.get("google_news").is_some());
    assert!(registry.get("Google Videos").is_some());
    assert!(registry.get("google_videos").is_some());

    let news = registry
        .adapter_catalog()
        .into_iter()
        .find(|entry| entry.metadata.name == "Google News")
        .expect("Google News should be registered");

    assert_eq!(
        news.adapter.access_model,
        EngineAccessModel::NoKeyRateLimited
    );
    assert_eq!(news.adapter.implementation, EngineImplementation::RssFeed);
}
