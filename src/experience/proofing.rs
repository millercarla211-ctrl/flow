use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofingGoal {
    Grammar,
    Clarity,
    Tone,
    Concision,
    CitationSupport,
    FactCheck,
    PlagiarismScreen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofingSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofingIssue {
    pub goal: ProofingGoal,
    pub severity: ProofingSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcademicClaimStatus {
    Supported,
    NeedsCitation,
    Verify,
    SourceOverlap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademicSource {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub excerpt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademicCitationNeed {
    pub sentence: String,
    pub reason: String,
    pub suggested_query: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademicClaimReview {
    pub claim: String,
    pub status: AcademicClaimStatus,
    pub evidence_source_ids: Vec<String>,
    pub suggestion: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademicReviewRequest {
    pub text: String,
    pub sources: Vec<AcademicSource>,
    pub strict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademicReviewReport {
    pub source_count: usize,
    pub issues: Vec<ProofingIssue>,
    pub citation_needs: Vec<AcademicCitationNeed>,
    pub claim_reviews: Vec<AcademicClaimReview>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowProofingPlanner {
    pub profile_name: &'static str,
    pub goals: Vec<ProofingGoal>,
    pub strict_mode: bool,
}

impl FlowProofingPlanner {
    pub fn business_default() -> Self {
        Self {
            profile_name: "business-default",
            goals: vec![
                ProofingGoal::Grammar,
                ProofingGoal::Clarity,
                ProofingGoal::Tone,
                ProofingGoal::Concision,
            ],
            strict_mode: false,
        }
    }

    pub fn academic_default() -> Self {
        Self {
            profile_name: "academic-default",
            goals: vec![
                ProofingGoal::Grammar,
                ProofingGoal::Clarity,
                ProofingGoal::CitationSupport,
                ProofingGoal::FactCheck,
                ProofingGoal::PlagiarismScreen,
            ],
            strict_mode: true,
        }
    }

    pub fn inspect(&self, text: &str) -> Vec<ProofingIssue> {
        let mut issues = Vec::new();

        if text.contains("  ") {
            issues.push(ProofingIssue {
                goal: ProofingGoal::Grammar,
                severity: ProofingSeverity::Low,
                message: "Repeated spaces detected.".to_string(),
                suggestion: Some("Collapse consecutive spaces into one.".to_string()),
            });
        }

        if text.contains(" i ") {
            issues.push(ProofingIssue {
                goal: ProofingGoal::Grammar,
                severity: ProofingSeverity::Medium,
                message: "Standalone lowercase 'i' detected.".to_string(),
                suggestion: Some("Replace it with uppercase 'I'.".to_string()),
            });
        }

        if max_sentence_words(text) > 34 {
            issues.push(ProofingIssue {
                goal: ProofingGoal::Clarity,
                severity: ProofingSeverity::Medium,
                message: "At least one sentence is likely too long for instant reading."
                    .to_string(),
                suggestion: Some("Split the sentence into two shorter sentences.".to_string()),
            });
        }

        if missing_terminal_punctuation(text) {
            issues.push(ProofingIssue {
                goal: ProofingGoal::Grammar,
                severity: ProofingSeverity::Low,
                message: "The draft ends without terminal punctuation.".to_string(),
                suggestion: Some("Add a period, question mark, or exclamation mark.".to_string()),
            });
        }

        if self.goals.contains(&ProofingGoal::CitationSupport) && mentions_claim_language(text) {
            issues.push(ProofingIssue {
                goal: ProofingGoal::CitationSupport,
                severity: ProofingSeverity::Medium,
                message: "The draft contains claim language that may need a source citation."
                    .to_string(),
                suggestion: Some("Attach a source or quote before publishing.".to_string()),
            });
        }

        if self.goals.contains(&ProofingGoal::FactCheck) && mentions_absolute_language(text) {
            issues.push(ProofingIssue {
                goal: ProofingGoal::FactCheck,
                severity: ProofingSeverity::High,
                message: "Absolute phrasing detected; verify the claim before sending.".to_string(),
                suggestion: Some("Replace absolutes with evidence-backed wording.".to_string()),
            });
        }

        issues
    }

    pub fn review_academic(&self, request: AcademicReviewRequest) -> AcademicReviewReport {
        let mut issues = self.inspect(&request.text);
        let mut citation_needs = Vec::new();
        let mut claim_reviews = Vec::new();
        let sentences = extract_sentences(&request.text);

        for sentence in sentences {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }

            let source_matches = matching_sources(sentence, &request.sources);
            let has_citation = has_inline_citation(sentence);
            let claim_like = mentions_claim_language(sentence)
                || mentions_absolute_language(sentence)
                || mentions_measurement_language(sentence);
            let source_overlap = source_matches.iter().any(|source| {
                normalized_contains(&source.excerpt, sentence) && sentence.chars().count() >= 32
            });

            if source_overlap && self.goals.contains(&ProofingGoal::PlagiarismScreen) {
                issues.push(ProofingIssue {
                    goal: ProofingGoal::PlagiarismScreen,
                    severity: ProofingSeverity::High,
                    message: "A sentence closely overlaps supplied source material.".to_string(),
                    suggestion: Some(
                        "Quote it directly or rewrite it with attribution.".to_string(),
                    ),
                });
                claim_reviews.push(AcademicClaimReview {
                    claim: sentence.to_string(),
                    status: AcademicClaimStatus::SourceOverlap,
                    evidence_source_ids: source_matches
                        .iter()
                        .map(|source| source.id.clone())
                        .collect(),
                    suggestion: "Quote, paraphrase, or attribute this sentence before publishing."
                        .to_string(),
                });
                continue;
            }

            if !claim_like {
                continue;
            }

            if source_matches.is_empty()
                && !has_citation
                && (request.strict || self.goals.contains(&ProofingGoal::CitationSupport))
            {
                citation_needs.push(AcademicCitationNeed {
                    sentence: sentence.to_string(),
                    reason: "Claim-like language appears without a citation or matching supplied source."
                        .to_string(),
                    suggested_query: suggested_source_query(sentence),
                });
                claim_reviews.push(AcademicClaimReview {
                    claim: sentence.to_string(),
                    status: AcademicClaimStatus::NeedsCitation,
                    evidence_source_ids: Vec::new(),
                    suggestion:
                        "Attach a source, quote, dataset, or reference before using this claim."
                            .to_string(),
                });
                continue;
            }

            if mentions_absolute_language(sentence) || mentions_measurement_language(sentence) {
                claim_reviews.push(AcademicClaimReview {
                    claim: sentence.to_string(),
                    status: AcademicClaimStatus::Verify,
                    evidence_source_ids: source_matches
                        .iter()
                        .map(|source| source.id.clone())
                        .collect(),
                    suggestion:
                        "Verify the exact value or absolute phrasing against the cited source."
                            .to_string(),
                });
            } else {
                claim_reviews.push(AcademicClaimReview {
                    claim: sentence.to_string(),
                    status: AcademicClaimStatus::Supported,
                    evidence_source_ids: source_matches
                        .iter()
                        .map(|source| source.id.clone())
                        .collect(),
                    suggestion: "Keep the citation attached when sharing this claim.".to_string(),
                });
            }
        }

        let mut notes = vec![
            "Academic review is local-first and only uses supplied text and sources.".to_string(),
        ];
        if request.strict {
            notes.push(
                "Strict mode treats unsupported claim-like sentences as citation needs."
                    .to_string(),
            );
        }
        if request.sources.is_empty() {
            notes.push(
                "No sources were supplied, so support checks can only flag missing evidence."
                    .to_string(),
            );
        }

        AcademicReviewReport {
            source_count: request.sources.len(),
            issues,
            citation_needs,
            claim_reviews,
            notes,
        }
    }
}

fn max_sentence_words(text: &str) -> usize {
    text.split(['.', '!', '?'])
        .map(|sentence| sentence.split_whitespace().count())
        .max()
        .unwrap_or(0)
}

fn missing_terminal_punctuation(text: &str) -> bool {
    text.chars()
        .rev()
        .find(|ch| !ch.is_whitespace())
        .map(|ch| !matches!(ch, '.' | '!' | '?'))
        .unwrap_or(false)
}

fn mentions_claim_language(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "according to",
        "studies show",
        "research proves",
        "reported that",
    ]
    .iter()
    .any(|pattern| lower.contains(pattern))
}

fn mentions_absolute_language(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    ["always", "never", "everyone", "nobody", "guaranteed"]
        .iter()
        .any(|pattern| lower.contains(pattern))
}

fn mentions_measurement_language(text: &str) -> bool {
    text.chars().any(|ch| ch.is_ascii_digit())
        || [
            "percent", "%", "million", "billion", "majority", "increase", "decrease",
        ]
        .iter()
        .any(|pattern| text.to_ascii_lowercase().contains(pattern))
}

fn has_inline_citation(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("http://")
        || lower.contains("https://")
        || lower.contains("doi:")
        || lower.contains("[")
        || (lower.contains('(')
            && lower.contains(')')
            && lower.chars().any(|ch| ch.is_ascii_digit()))
}

fn extract_sentences(text: &str) -> Vec<String> {
    text.split_inclusive(['.', '!', '?'])
        .map(str::trim)
        .filter(|sentence| !sentence.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn matching_sources<'a>(sentence: &str, sources: &'a [AcademicSource]) -> Vec<&'a AcademicSource> {
    let claim_terms = meaningful_terms(sentence);
    sources
        .iter()
        .filter(|source| {
            let source_terms = meaningful_terms(&source.excerpt);
            claim_terms
                .iter()
                .filter(|term| source_terms.contains(term))
                .count()
                >= 3
        })
        .collect()
}

fn meaningful_terms(text: &str) -> Vec<String> {
    let mut terms = BTreeSet::new();
    for term in text
        .split(|ch: char| !ch.is_alphanumeric())
        .map(str::trim)
        .filter(|term| term.len() >= 5)
    {
        terms.insert(term.to_ascii_lowercase());
    }
    terms.into_iter().collect()
}

fn normalized_contains(haystack: &str, needle: &str) -> bool {
    normalize_for_overlap(haystack).contains(&normalize_for_overlap(needle))
}

fn normalize_for_overlap(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn suggested_source_query(sentence: &str) -> String {
    meaningful_terms(sentence)
        .into_iter()
        .take(8)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn academic_review_flags_unsourced_claims() {
        let planner = FlowProofingPlanner::academic_default();
        let report = planner.review_academic(AcademicReviewRequest {
            text: "Studies show Flow always improves writing by 90%.".to_string(),
            sources: Vec::new(),
            strict: true,
        });

        assert_eq!(report.source_count, 0);
        assert_eq!(report.citation_needs.len(), 1);
        assert!(
            report
                .claim_reviews
                .iter()
                .any(|review| review.status == AcademicClaimStatus::NeedsCitation)
        );
    }

    #[test]
    fn academic_review_links_matching_sources() {
        let planner = FlowProofingPlanner::academic_default();
        let report = planner.review_academic(AcademicReviewRequest {
            text: "Research proves local dictation improves writing speed.".to_string(),
            sources: vec![AcademicSource {
                id: "source-1".to_string(),
                title: "Local Dictation Study".to_string(),
                url: None,
                excerpt: "A research study reports local dictation improves writing speed for daily drafting.".to_string(),
            }],
            strict: true,
        });

        assert!(report.citation_needs.is_empty());
        assert_eq!(
            report
                .claim_reviews
                .first()
                .map(|review| review.status.clone()),
            Some(AcademicClaimStatus::Supported)
        );
        assert_eq!(
            report.claim_reviews[0].evidence_source_ids,
            vec!["source-1"]
        );
    }

    #[test]
    fn academic_review_flags_source_overlap() {
        let planner = FlowProofingPlanner::academic_default();
        let copied = "Local dictation improves writing speed for daily drafting.";
        let report = planner.review_academic(AcademicReviewRequest {
            text: copied.to_string(),
            sources: vec![AcademicSource {
                id: "source-1".to_string(),
                title: "Local Dictation Study".to_string(),
                url: None,
                excerpt: copied.to_string(),
            }],
            strict: true,
        });

        assert!(
            report
                .claim_reviews
                .iter()
                .any(|review| review.status == AcademicClaimStatus::SourceOverlap)
        );
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.goal == ProofingGoal::PlagiarismScreen)
        );
    }
}
