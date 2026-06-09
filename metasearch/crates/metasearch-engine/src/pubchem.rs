//! PubChem compound lookup via the public PUG REST API.
//!
//! Reference: <https://pubchem.ncbi.nlm.nih.gov/docs/pug-rest>

use async_trait::async_trait;
use metasearch_core::{
    category::SearchCategory,
    engine::{EngineMetadata, SearchEngine},
    error::{MetasearchError, Result},
    query::SearchQuery,
    result::SearchResult,
};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use smallvec::smallvec;

const USER_AGENT: &str =
    "metasearch-engine/1.0 (+https://github.com/najmus-sakib-hossain/metasearch)";
const PROPERTY_LIST: &str = "MolecularFormula,MolecularWeight,CanonicalSMILES,InChIKey,IUPACName";

pub struct PubChem {
    metadata: EngineMetadata,
    client: Client,
}

impl PubChem {
    pub fn new(client: Client) -> Self {
        Self {
            metadata: EngineMetadata {
                name: "pubchem".to_string().into(),
                display_name: "PubChem".to_string().into(),
                homepage: "https://pubchem.ncbi.nlm.nih.gov".to_string().into(),
                categories: smallvec![SearchCategory::Science],
                enabled: true,
                timeout_ms: 7000,
                weight: 0.8,
            },
            client,
        }
    }
}

#[derive(Debug, Deserialize)]
struct PubChemResponse {
    #[serde(rename = "PropertyTable")]
    property_table: PubChemPropertyTable,
}

#[derive(Debug, Deserialize)]
struct PubChemPropertyTable {
    #[serde(rename = "Properties", default)]
    properties: Vec<PubChemCompound>,
}

#[derive(Debug, Deserialize)]
struct PubChemCompound {
    #[serde(rename = "CID")]
    cid: Option<u64>,
    #[serde(rename = "MolecularFormula")]
    molecular_formula: Option<JsonScalar>,
    #[serde(rename = "MolecularWeight")]
    molecular_weight: Option<JsonScalar>,
    #[serde(rename = "CanonicalSMILES", alias = "ConnectivitySMILES")]
    smiles: Option<String>,
    #[serde(rename = "InChIKey")]
    inchi_key: Option<String>,
    #[serde(rename = "IUPACName")]
    iupac_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum JsonScalar {
    String(String),
    Number(f64),
}

impl JsonScalar {
    fn into_text(self) -> String {
        match self {
            Self::String(value) => value,
            Self::Number(value) => value.to_string(),
        }
    }
}

pub fn parse_pubchem_results(body: &str) -> Result<Vec<SearchResult>> {
    let response: PubChemResponse = serde_json::from_str(body)?;
    let results = response
        .property_table
        .properties
        .into_iter()
        .enumerate()
        .filter_map(|(i, compound)| {
            let cid = compound.cid?;
            let title = non_empty(compound.iupac_name).unwrap_or_else(|| format!("CID {cid}"));
            let mut parts = Vec::new();

            if let Some(formula) = compound
                .molecular_formula
                .map(JsonScalar::into_text)
                .and_then(non_empty_value)
            {
                parts.push(formula);
            }
            if let Some(weight) = compound
                .molecular_weight
                .map(JsonScalar::into_text)
                .and_then(non_empty_value)
            {
                parts.push(format!("molecular weight {weight}"));
            }
            if let Some(inchi_key) = non_empty(compound.inchi_key) {
                parts.push(format!("InChIKey {inchi_key}"));
            }
            if let Some(smiles) = non_empty(compound.smiles) {
                parts.push(format!("SMILES {smiles}"));
            }

            let mut result = SearchResult::new(
                title,
                format!("https://pubchem.ncbi.nlm.nih.gov/compound/{cid}"),
                parts.join(" - "),
                "pubchem",
            );
            result.engine_rank = (i + 1) as u32;
            result.category = SearchCategory::Science.to_string();
            Some(result)
        })
        .collect();

    Ok(results)
}

#[async_trait]
impl SearchEngine for PubChem {
    fn metadata(&self) -> EngineMetadata {
        self.metadata.clone()
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let url = if looks_like_formula(&query.query) {
            format!(
                "https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/fastformula/{}/property/{PROPERTY_LIST}/JSON?MaxRecords=10",
                urlencoding::encode(&query.query),
            )
        } else {
            format!(
                "https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/name/{}/property/{PROPERTY_LIST}/JSON",
                urlencoding::encode(&query.query),
            )
        };

        let response = self
            .client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Accept", "application/json")
            .timeout(std::time::Duration::from_millis(self.metadata.timeout_ms))
            .send()
            .await
            .map_err(|error| MetasearchError::HttpError(error.to_string()))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }

        let body = response
            .error_for_status()
            .map_err(|error| MetasearchError::HttpError(error.to_string()))?
            .text()
            .await
            .map_err(|error| MetasearchError::ParseError(error.to_string()))?;

        parse_pubchem_results(&body)
    }
}

fn looks_like_formula(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.chars().any(|ch| ch.is_ascii_digit())
        && trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '(' | ')' | '+' | '-'))
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(non_empty_value)
}

fn non_empty_value(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}
