use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    SerdeSerialize,
    SerdeDeserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
pub enum SearchVertical {
    Web,
    News,
    Code,
    Academic,
    Images,
    Video,
    Models,
    Packages,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    SerdeSerialize,
    SerdeDeserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
pub enum SearchIntent {
    AgentGrounding,
    AnswerSearch,
    DeepResearch,
    ProviderDiscovery,
    ModelDiscovery,
    CodeResearch,
    UserSearch,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    SerdeSerialize,
    SerdeDeserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
pub struct SearchRequestPlan {
    pub query: String,
    pub intent: SearchIntent,
    pub verticals: Vec<SearchVertical>,
    pub use_adjacent_metasearch: bool,
    pub notes: Vec<String>,
}

pub struct MetasearchBridge;

impl MetasearchBridge {
    pub fn for_friday_answer_search(query: impl Into<String>) -> SearchRequestPlan {
        SearchRequestPlan {
            query: query.into(),
            intent: SearchIntent::AnswerSearch,
            verticals: vec![
                SearchVertical::Web,
                SearchVertical::News,
                SearchVertical::Academic,
                SearchVertical::Code,
            ],
            use_adjacent_metasearch: true,
            notes: vec![
                "Use the adjacent metasearch Rust crate for Friday answer search.".to_string(),
                "Do not route through Perplexity Computer or browser-control dependencies."
                    .to_string(),
                "Return answer-ready result groups with preserved source provenance and citations."
                    .to_string(),
            ],
        }
    }

    pub fn for_friday_research(query: impl Into<String>) -> SearchRequestPlan {
        SearchRequestPlan {
            query: query.into(),
            intent: SearchIntent::DeepResearch,
            verticals: vec![
                SearchVertical::Web,
                SearchVertical::News,
                SearchVertical::Academic,
                SearchVertical::Code,
                SearchVertical::Models,
                SearchVertical::Packages,
            ],
            use_adjacent_metasearch: true,
            notes: vec![
                "Plan multi-pass Friday research through metasearch before synthesis.".to_string(),
                "Separate discovery, source scoring, citation extraction, and final report generation."
                    .to_string(),
                "Prefer local/RLM/serializer context packing before any remote provider is considered."
                    .to_string(),
            ],
        }
    }

    pub fn for_agent_grounding(query: impl Into<String>) -> SearchRequestPlan {
        SearchRequestPlan {
            query: query.into(),
            intent: SearchIntent::AgentGrounding,
            verticals: vec![SearchVertical::Web, SearchVertical::Code, SearchVertical::Academic],
            use_adjacent_metasearch: true,
            notes: vec![
                "Prefer the adjacent metasearch project when it is available.".to_string(),
                "Use multiple verticals so agents can ground against docs, code, and general web results."
                    .to_string(),
            ],
        }
    }

    pub fn for_model_discovery(query: impl Into<String>) -> SearchRequestPlan {
        SearchRequestPlan {
            query: query.into(),
            intent: SearchIntent::ModelDiscovery,
            verticals: vec![
                SearchVertical::Models,
                SearchVertical::Web,
                SearchVertical::Code,
            ],
            use_adjacent_metasearch: true,
            notes: vec![
                "Use model and web verticals together for faster provider and runtime discovery."
                    .to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_grounding_plan_uses_multiple_verticals() {
        let plan = MetasearchBridge::for_agent_grounding("best local stt");
        assert!(plan.verticals.contains(&SearchVertical::Web));
        assert!(plan.verticals.contains(&SearchVertical::Code));
    }

    #[test]
    fn friday_search_never_depends_on_perplexity_computer() {
        let plan = MetasearchBridge::for_friday_answer_search("latest local stt");
        assert_eq!(plan.intent, SearchIntent::AnswerSearch);
        assert!(plan.use_adjacent_metasearch);
        assert!(
            plan.notes
                .iter()
                .any(|note| note.contains("Do not route through Perplexity Computer"))
        );
    }

    #[test]
    fn friday_research_uses_broad_source_verticals() {
        let plan = MetasearchBridge::for_friday_research("compare Claude and Gemini");
        assert_eq!(plan.intent, SearchIntent::DeepResearch);
        assert!(plan.verticals.contains(&SearchVertical::Academic));
        assert!(plan.verticals.contains(&SearchVertical::Models));
        assert!(plan.verticals.contains(&SearchVertical::Packages));
    }
}
