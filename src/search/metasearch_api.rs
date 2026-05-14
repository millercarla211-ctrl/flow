use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::search::{SearchRequestPlan, SearchVertical};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetasearchServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_ms: u64,
}

impl Default for MetasearchServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8888,
            timeout_ms: 8_000,
        }
    }
}

impl MetasearchServerConfig {
    pub fn api_path_for_plan(&self, plan: &SearchRequestPlan) -> String {
        let categories = metasearch_categories(&plan.verticals).join(",");
        format!(
            "/api/v1/search?format=json&q={}&categories={}",
            percent_encode_query(&plan.query),
            percent_encode_query(&categories)
        )
    }

    pub fn search_blocking(&self, plan: &SearchRequestPlan) -> Result<MetasearchApiResponse> {
        let path = self.api_path_for_plan(plan);
        let timeout = Duration::from_millis(self.timeout_ms);
        let address = (self.host.as_str(), self.port)
            .to_socket_addrs()
            .with_context(|| format!("Could not resolve metasearch host {}", self.host))?
            .next()
            .with_context(|| format!("No socket address for metasearch host {}", self.host))?;

        let mut stream = TcpStream::connect_timeout(&address, timeout).with_context(|| {
            format!(
                "Could not connect to local metasearch at {}:{}",
                self.host, self.port
            )
        })?;
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;

        let request = format!(
            "GET {path} HTTP/1.1\r\nHost: {}:{}\r\nAccept: application/json\r\nConnection: close\r\n\r\n",
            self.host, self.port
        );
        stream.write_all(request.as_bytes())?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        let (headers, body) = response
            .split_once("\r\n\r\n")
            .context("Metasearch returned a malformed HTTP response")?;

        if !(headers.starts_with("HTTP/1.1 200") || headers.starts_with("HTTP/1.0 200")) {
            bail!("Metasearch request failed: {}", first_header_line(headers));
        }

        serde_json::from_str(body).context("Metasearch returned invalid JSON")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetasearchApiResponse {
    pub query: String,
    #[serde(default)]
    pub results: Vec<MetasearchApiResult>,
    #[serde(default)]
    pub number_of_results: usize,
    #[serde(default)]
    pub engines_used: Vec<String>,
    #[serde(default)]
    pub engines_failed: Vec<String>,
    #[serde(default)]
    pub search_time_ms: u64,
    #[serde(default)]
    pub cached: bool,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetasearchApiResult {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub engine: String,
    #[serde(default)]
    pub engine_rank: u32,
    #[serde(default)]
    pub score: f64,
    #[serde(default)]
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub published_date: Option<String>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

pub fn metasearch_categories(verticals: &[SearchVertical]) -> Vec<String> {
    let mut categories = Vec::new();
    for vertical in verticals {
        let category = match vertical {
            SearchVertical::Web => "general",
            SearchVertical::News => "news",
            SearchVertical::Code | SearchVertical::Models | SearchVertical::Packages => "it",
            SearchVertical::Academic => "science",
            SearchVertical::Images => "images",
            SearchVertical::Video => "videos",
        };
        if !categories.contains(&category) {
            categories.push(category);
        }
    }
    categories.into_iter().map(str::to_string).collect()
}

fn percent_encode_query(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn first_header_line(headers: &str) -> &str {
    headers.lines().next().unwrap_or(headers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::MetasearchBridge;

    #[test]
    fn api_path_maps_friday_verticals_to_metasearch_categories() {
        let plan = MetasearchBridge::for_friday_research("AI search features");
        let path = MetasearchServerConfig::default().api_path_for_plan(&plan);

        assert!(path.contains("/api/v1/search?format=json"));
        assert!(path.contains("q=AI%20search%20features"));
        assert!(path.contains("categories=general%2Cnews%2Cscience%2Cit"));
    }

    #[test]
    fn api_response_parses_metasearch_json_shape() {
        let json = r#"{
            "query": "local ai search",
            "results": [
                {
                    "title": "Example",
                    "url": "https://example.com",
                    "content": "Source snippet",
                    "engine": "duckduckgo",
                    "engine_rank": 1,
                    "score": 0.9,
                    "category": "general"
                }
            ],
            "number_of_results": 1,
            "engines_used": ["duckduckgo"],
            "engines_failed": [],
            "search_time_ms": 42,
            "cached": false,
            "categories": ["general"]
        }"#;

        let response: MetasearchApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.number_of_results, 1);
        assert_eq!(response.results[0].engine, "duckduckgo");
    }
}
