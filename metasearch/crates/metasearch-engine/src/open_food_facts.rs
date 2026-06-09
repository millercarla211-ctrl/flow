//! Open Food Facts product search via the public JSON API.
//!
//! Reference: <https://openfoodfacts.github.io/openfoodfacts-server/api/>

use async_trait::async_trait;
use metasearch_core::{
    category::SearchCategory,
    engine::{EngineMetadata, SearchEngine},
    error::{MetasearchError, Result},
    query::SearchQuery,
    result::SearchResult,
};
use reqwest::Client;
use serde::Deserialize;
use smallvec::smallvec;

const USER_AGENT: &str =
    "metasearch-engine/1.0 (+https://github.com/najmus-sakib-hossain/metasearch)";

pub struct OpenFoodFacts {
    metadata: EngineMetadata,
    client: Client,
}

impl OpenFoodFacts {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "open_food_facts".to_string().into(),
                display_name: "Open Food Facts".to_string().into(),
                homepage: "https://world.openfoodfacts.org".to_string().into(),
                categories: smallvec![SearchCategory::General],
                enabled: false,
                timeout_ms: 7000,
                weight: 0.6,
            },
            client,
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenFoodFactsResponse {
    #[serde(default)]
    products: Vec<OpenFoodFactsProduct>,
}

#[derive(Debug, Deserialize)]
struct OpenFoodFactsProduct {
    product_name: Option<String>,
    code: Option<String>,
    brands: Option<String>,
    categories: Option<String>,
    url: Option<String>,
    image_front_thumb_url: Option<String>,
}

pub fn parse_open_food_facts_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: OpenFoodFactsResponse = serde_json::from_str(body)?;
    let results = response
        .products
        .into_iter()
        .enumerate()
        .filter_map(|(i, product)| {
            let title = product
                .product_name
                .filter(|value| !value.trim().is_empty())?;
            let url = product
                .url
                .filter(|value| !value.trim().is_empty())
                .or_else(|| {
                    product
                        .code
                        .filter(|value| !value.trim().is_empty())
                        .map(|code| format!("https://world.openfoodfacts.org/product/{code}"))
                })?;

            let mut parts = Vec::new();
            if let Some(brands) = product.brands.filter(|value| !value.trim().is_empty()) {
                parts.push(brands);
            }
            if let Some(categories) = product.categories.filter(|value| !value.trim().is_empty()) {
                parts.push(categories);
            }

            let mut result = SearchResult::new(title, url, parts.join(" - "), "open_food_facts");
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::General.to_string();
            result.thumbnail = product
                .image_front_thumb_url
                .filter(|value| !value.trim().is_empty());
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for OpenFoodFacts {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://world.openfoodfacts.org/cgi/search.pl?search_terms={}&search_simple=1&action=process&json=1&page_size=10&page={}",
            urlencoding::encode(&query.query),
            query.page.max(1),
        );

        let body = self
            .client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .timeout(std::time::Duration::from_millis(self.metadata.timeout_ms))
            .send()
            .await
            .map_err(|error| MetasearchError::HttpError(error.to_string()))?
            .error_for_status()
            .map_err(|error| MetasearchError::HttpError(error.to_string()))?
            .text()
            .await
            .map_err(|error| MetasearchError::ParseError(error.to_string()))?;

        parse_open_food_facts_results(&body)
    }
}
