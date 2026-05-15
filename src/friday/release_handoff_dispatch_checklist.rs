use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};

use super::{
    FridayDashboardPanelStatus, FridayReleaseHandoffGovernanceReview,
    FridayReleaseHandoffGovernanceState, read_friday_release_handoff_governance_review,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseHandoffDispatchChecklistState {
    Ready,
    Held,
    MissingRecipient,
    MissingAttachment,
    PrivacyReview,
    Blocked,
}

impl FridayReleaseHandoffDispatchChecklistState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Held => "held",
            Self::MissingRecipient => "missing-recipient",
            Self::MissingAttachment => "missing-attachment",
            Self::PrivacyReview => "privacy-review",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseHandoffDispatchChecklistSource {
    GovernanceReview,
    Recipient,
    Attachment,
    DispatchNote,
    PrivacyBoundary,
    NoSendSafeguard,
}

impl FridayReleaseHandoffDispatchChecklistSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::GovernanceReview => "governance-review",
            Self::Recipient => "recipient",
            Self::Attachment => "attachment",
            Self::DispatchNote => "dispatch-note",
            Self::PrivacyBoundary => "privacy-boundary",
            Self::NoSendSafeguard => "no-send-safeguard",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffDispatchChecklistRequest {
    pub recipients: Vec<String>,
    pub attachments: Vec<String>,
    pub dispatch_note: String,
    pub privacy_note: String,
}

impl Default for FridayReleaseHandoffDispatchChecklistRequest {
    fn default() -> Self {
        Self {
            recipients: Vec::new(),
            attachments: Vec::new(),
            dispatch_note: "No external send is performed by this checklist.".to_string(),
            privacy_note: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffDispatchChecklistItem {
    pub id: String,
    pub source: FridayReleaseHandoffDispatchChecklistSource,
    pub state: FridayReleaseHandoffDispatchChecklistState,
    pub required: bool,
    pub ready: bool,
    pub release_gate_blocking: bool,
    pub title: String,
    pub detail: String,
    pub evidence_path: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffDispatchChecklist {
    pub checklist_id: String,
    pub checklist_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub state: FridayReleaseHandoffDispatchChecklistState,
    pub ready_to_dispatch: bool,
    pub governance_review_id: String,
    pub governance_review_json: String,
    pub governance_state: FridayReleaseHandoffGovernanceState,
    pub approved_for_external_handoff: bool,
    pub latest_packet_id: Option<String>,
    pub active_packet_id: Option<String>,
    pub recipient_count: usize,
    pub attachment_count: usize,
    pub dispatch_note_count: usize,
    pub privacy_boundary_count: usize,
    pub no_send_safeguard_count: usize,
    pub item_count: usize,
    pub ready_count: usize,
    pub missing_recipient_count: usize,
    pub missing_attachment_count: usize,
    pub privacy_review_count: usize,
    pub held_count: usize,
    pub blocked_count: usize,
    pub release_gate_blocking_count: usize,
    pub items: Vec<FridayReleaseHandoffDispatchChecklistItem>,
    pub dispatch_checklist_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseHandoffDispatchChecklist {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_handoff_dispatch_checklist_report(
    checklist_path: impl AsRef<Path>,
    governance_review_path: impl AsRef<Path>,
    request: FridayReleaseHandoffDispatchChecklistRequest,
) -> FridayReleaseHandoffDispatchChecklist {
    let checklist_path = checklist_path.as_ref();
    let governance_review_path = governance_review_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    let review = read_friday_release_handoff_governance_review(governance_review_path).ok();
    let fallback = fallback_governance_review(governance_review_path);
    let review = review.as_ref().unwrap_or(&fallback);

    let mut items = dispatch_items(review, &request);
    items.sort_by(|left, right| {
        state_rank(left.state)
            .cmp(&state_rank(right.state))
            .then_with(|| left.id.cmp(&right.id))
    });

    let item_count = items.len();
    let ready_count = items.iter().filter(|item| item.ready).count();
    let recipient_count = request
        .recipients
        .iter()
        .filter(|recipient| !recipient.trim().is_empty())
        .count();
    let attachment_count = request
        .attachments
        .iter()
        .filter(|attachment| !attachment.trim().is_empty())
        .count();
    let dispatch_note_count = usize::from(!request.dispatch_note.trim().is_empty());
    let privacy_boundary_count = usize::from(!request.privacy_note.trim().is_empty());
    let no_send_safeguard_count = items
        .iter()
        .filter(|item| item.source == FridayReleaseHandoffDispatchChecklistSource::NoSendSafeguard)
        .count();
    let missing_recipient_count = state_count(
        &items,
        FridayReleaseHandoffDispatchChecklistState::MissingRecipient,
    );
    let missing_attachment_count = state_count(
        &items,
        FridayReleaseHandoffDispatchChecklistState::MissingAttachment,
    );
    let privacy_review_count = state_count(
        &items,
        FridayReleaseHandoffDispatchChecklistState::PrivacyReview,
    );
    let held_count = state_count(&items, FridayReleaseHandoffDispatchChecklistState::Held);
    let blocked_count = state_count(&items, FridayReleaseHandoffDispatchChecklistState::Blocked);
    let release_gate_blocking_count = items
        .iter()
        .filter(|item| item.release_gate_blocking)
        .count();
    let state = if blocked_count > 0 {
        FridayReleaseHandoffDispatchChecklistState::Blocked
    } else if missing_recipient_count > 0 {
        FridayReleaseHandoffDispatchChecklistState::MissingRecipient
    } else if missing_attachment_count > 0 {
        FridayReleaseHandoffDispatchChecklistState::MissingAttachment
    } else if privacy_review_count > 0 {
        FridayReleaseHandoffDispatchChecklistState::PrivacyReview
    } else if held_count > 0 {
        FridayReleaseHandoffDispatchChecklistState::Held
    } else {
        FridayReleaseHandoffDispatchChecklistState::Ready
    };
    let ready_to_dispatch = state == FridayReleaseHandoffDispatchChecklistState::Ready
        && release_gate_blocking_count == 0;
    let status = if release_gate_blocking_count > 0 || blocked_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if ready_to_dispatch {
        FridayDashboardPanelStatus::Ready
    } else {
        FridayDashboardPanelStatus::Warning
    };
    let checklist_json = path_string(checklist_path);
    let governance_review_json = path_string(governance_review_path);

    FridayReleaseHandoffDispatchChecklist {
        checklist_id: format!("friday-release-handoff-dispatch-checklist-{generated_at_unix_ms}"),
        checklist_json: checklist_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        state,
        ready_to_dispatch,
        governance_review_id: review.review_id.clone(),
        governance_review_json: governance_review_json.clone(),
        governance_state: review.state,
        approved_for_external_handoff: review.approved_for_external_handoff,
        latest_packet_id: review.latest_packet_id.clone(),
        active_packet_id: review.active_packet_id.clone(),
        recipient_count,
        attachment_count,
        dispatch_note_count,
        privacy_boundary_count,
        no_send_safeguard_count,
        item_count,
        ready_count,
        missing_recipient_count,
        missing_attachment_count,
        privacy_review_count,
        held_count,
        blocked_count,
        release_gate_blocking_count,
        dispatch_checklist_copy: dispatch_checklist_copy(
            &items,
            ready_to_dispatch,
            &request.recipients,
            &request.attachments,
        ),
        summary: format!(
            "Friday release handoff dispatch checklist is {} with {} item(s), {} recipient(s), {} attachment(s), {} privacy note(s), and {} blocking issue(s).",
            state.label(),
            item_count,
            recipient_count,
            attachment_count,
            privacy_boundary_count,
            release_gate_blocking_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-handoff-dispatch-checklist --output {} --governance-review {} --recipient <recipient> --attachment <file>",
                checklist_json, governance_review_json
            ),
            format!(
                "flow --friday-release-handoff-dispatch-checklist-json --output {} --governance-review {} --recipient <recipient> --attachment <file>",
                checklist_json, governance_review_json
            ),
        ],
        items,
    }
}

pub fn write_friday_release_handoff_dispatch_checklist(
    checklist_path: impl AsRef<Path>,
    checklist: &FridayReleaseHandoffDispatchChecklist,
) -> Result<()> {
    let checklist_path = checklist_path.as_ref();
    if let Some(parent) = checklist_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday handoff dispatch checklist directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(checklist_path, checklist.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday handoff dispatch checklist {}",
            checklist_path.display()
        )
    })
}

pub fn read_friday_release_handoff_dispatch_checklist(
    checklist_path: impl AsRef<Path>,
) -> Result<FridayReleaseHandoffDispatchChecklist> {
    let checklist_path = checklist_path.as_ref();
    let bytes = fs::read(checklist_path).with_context(|| {
        format!(
            "Could not read Friday handoff dispatch checklist {}",
            checklist_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday handoff dispatch checklist {}",
            checklist_path.display()
        )
    })
}

fn dispatch_items(
    review: &FridayReleaseHandoffGovernanceReview,
    request: &FridayReleaseHandoffDispatchChecklistRequest,
) -> Vec<FridayReleaseHandoffDispatchChecklistItem> {
    let mut items = Vec::new();

    items.push(item(
        "governance-review",
        FridayReleaseHandoffDispatchChecklistSource::GovernanceReview,
        if review.approved_for_external_handoff {
            FridayReleaseHandoffDispatchChecklistState::Ready
        } else {
            FridayReleaseHandoffDispatchChecklistState::Blocked
        },
        true,
        "Governance review approval",
        &review.summary,
        &review.review_json,
        if review.approved_for_external_handoff {
            "Keep the governance review attached to the dispatch checklist."
        } else {
            "Resolve governance findings before preparing external dispatch."
        },
    ));

    let recipients = request
        .recipients
        .iter()
        .map(|recipient| recipient.trim())
        .filter(|recipient| !recipient.is_empty())
        .collect::<Vec<_>>();
    if recipients.is_empty() {
        items.push(item(
            "missing-recipient",
            FridayReleaseHandoffDispatchChecklistSource::Recipient,
            FridayReleaseHandoffDispatchChecklistState::MissingRecipient,
            true,
            "Recipient is missing",
            "No recipient was supplied for this handoff dispatch checklist.",
            "",
            "Add at least one intended recipient before preparing the handoff.",
        ));
    } else {
        for (index, recipient) in recipients.iter().enumerate() {
            items.push(item(
                &format!("recipient-{}", index + 1),
                FridayReleaseHandoffDispatchChecklistSource::Recipient,
                FridayReleaseHandoffDispatchChecklistState::Ready,
                true,
                "Dispatch recipient",
                recipient,
                "",
                "Confirm this recipient is allowed to receive the handoff.",
            ));
        }
    }

    let attachments = request
        .attachments
        .iter()
        .map(|attachment| attachment.trim())
        .filter(|attachment| !attachment.is_empty())
        .collect::<Vec<_>>();
    if attachments.is_empty() {
        items.push(item(
            "missing-attachment",
            FridayReleaseHandoffDispatchChecklistSource::Attachment,
            FridayReleaseHandoffDispatchChecklistState::MissingAttachment,
            true,
            "Attachment list is missing",
            "No attachment path was supplied for the handoff packet.",
            "",
            "Attach the handoff packet and supporting evidence files before dispatch.",
        ));
    } else {
        for (index, attachment) in attachments.iter().enumerate() {
            let present = fs::metadata(attachment)
                .map(|metadata| metadata.is_file())
                .unwrap_or(false);
            items.push(item(
                &format!("attachment-{}", index + 1),
                FridayReleaseHandoffDispatchChecklistSource::Attachment,
                if present {
                    FridayReleaseHandoffDispatchChecklistState::Ready
                } else {
                    FridayReleaseHandoffDispatchChecklistState::MissingAttachment
                },
                true,
                "Dispatch attachment",
                if present {
                    "Attachment exists on disk."
                } else {
                    "Attachment path is missing or unreadable."
                },
                attachment,
                if present {
                    "Keep this attachment with the dispatch checklist."
                } else {
                    "Create or correct this attachment path before dispatch."
                },
            ));
        }
    }

    items.push(item(
        "dispatch-note",
        FridayReleaseHandoffDispatchChecklistSource::DispatchNote,
        if request.dispatch_note.trim().is_empty() {
            FridayReleaseHandoffDispatchChecklistState::Held
        } else {
            FridayReleaseHandoffDispatchChecklistState::Ready
        },
        true,
        "Dispatch note",
        if request.dispatch_note.trim().is_empty() {
            "No dispatch note was supplied."
        } else {
            request.dispatch_note.trim()
        },
        "",
        "Keep the dispatch note in the operator handoff record.",
    ));

    items.push(item(
        "privacy-boundary",
        FridayReleaseHandoffDispatchChecklistSource::PrivacyBoundary,
        if request.privacy_note.trim().is_empty() {
            FridayReleaseHandoffDispatchChecklistState::PrivacyReview
        } else {
            FridayReleaseHandoffDispatchChecklistState::Ready
        },
        true,
        "Privacy boundary",
        if request.privacy_note.trim().is_empty() {
            "No privacy boundary note was supplied."
        } else {
            request.privacy_note.trim()
        },
        "",
        "Record what may and may not leave the local machine before external handoff.",
    ));

    items.push(item(
        "no-send-safeguard",
        FridayReleaseHandoffDispatchChecklistSource::NoSendSafeguard,
        FridayReleaseHandoffDispatchChecklistState::Ready,
        true,
        "No-send safeguard",
        "This checklist command only writes local JSON and never sends, uploads, deploys, or mutates external systems.",
        "",
        "Use a separate explicit operator action if external sending is ever approved.",
    ));

    items
}

fn item(
    id: &str,
    source: FridayReleaseHandoffDispatchChecklistSource,
    state: FridayReleaseHandoffDispatchChecklistState,
    required: bool,
    title: &str,
    detail: &str,
    evidence_path: &str,
    next_action: &str,
) -> FridayReleaseHandoffDispatchChecklistItem {
    let ready = state == FridayReleaseHandoffDispatchChecklistState::Ready;
    let release_gate_blocking = required
        && matches!(
            state,
            FridayReleaseHandoffDispatchChecklistState::MissingRecipient
                | FridayReleaseHandoffDispatchChecklistState::MissingAttachment
                | FridayReleaseHandoffDispatchChecklistState::PrivacyReview
                | FridayReleaseHandoffDispatchChecklistState::Blocked
        );

    FridayReleaseHandoffDispatchChecklistItem {
        id: id.to_string(),
        source,
        state,
        required,
        ready,
        release_gate_blocking,
        title: title.to_string(),
        detail: detail.to_string(),
        evidence_path: evidence_path.to_string(),
        next_action: next_action.to_string(),
    }
}

fn state_count(
    items: &[FridayReleaseHandoffDispatchChecklistItem],
    state: FridayReleaseHandoffDispatchChecklistState,
) -> usize {
    items.iter().filter(|item| item.state == state).count()
}

fn state_rank(state: FridayReleaseHandoffDispatchChecklistState) -> u8 {
    match state {
        FridayReleaseHandoffDispatchChecklistState::Blocked => 0,
        FridayReleaseHandoffDispatchChecklistState::MissingRecipient => 1,
        FridayReleaseHandoffDispatchChecklistState::MissingAttachment => 2,
        FridayReleaseHandoffDispatchChecklistState::PrivacyReview => 3,
        FridayReleaseHandoffDispatchChecklistState::Held => 4,
        FridayReleaseHandoffDispatchChecklistState::Ready => 5,
    }
}

fn dispatch_checklist_copy(
    items: &[FridayReleaseHandoffDispatchChecklistItem],
    ready_to_dispatch: bool,
    recipients: &[String],
    attachments: &[String],
) -> String {
    let mut lines = vec![
        "Friday release handoff dispatch checklist".to_string(),
        format!(
            "Status: {}",
            if ready_to_dispatch {
                "ready for explicit operator dispatch"
            } else {
                "hold dispatch"
            }
        ),
    ];

    lines.push("Recipients:".to_string());
    for recipient in recipients
        .iter()
        .map(|recipient| recipient.trim())
        .filter(|recipient| !recipient.is_empty())
    {
        lines.push(format!("- {recipient}"));
    }
    if recipients
        .iter()
        .all(|recipient| recipient.trim().is_empty())
    {
        lines.push("- missing".to_string());
    }

    lines.push("Attachments:".to_string());
    for attachment in attachments
        .iter()
        .map(|attachment| attachment.trim())
        .filter(|attachment| !attachment.is_empty())
    {
        lines.push(format!("- {attachment}"));
    }
    if attachments
        .iter()
        .all(|attachment| attachment.trim().is_empty())
    {
        lines.push("- missing".to_string());
    }

    lines.push("Checklist:".to_string());
    for item in items.iter().take(10) {
        lines.push(format!(
            "- [{}] {} -> {}",
            item.state.label(),
            item.title,
            item.next_action
        ));
    }
    lines.join("\n")
}

fn fallback_governance_review(path: &Path) -> FridayReleaseHandoffGovernanceReview {
    FridayReleaseHandoffGovernanceReview {
        review_id: "missing-release-handoff-governance-review".to_string(),
        review_json: path_string(path),
        generated_at_unix_ms: unix_ms(),
        product_name: "Friday".to_string(),
        local_only: true,
        status: FridayDashboardPanelStatus::Blocked,
        score_out_of_100: 0,
        state: FridayReleaseHandoffGovernanceState::Held,
        approved_for_external_handoff: false,
        trail_id: "missing".to_string(),
        trail_json: String::new(),
        latest_audit_id: None,
        latest_packet_id: None,
        active_audit_id: None,
        active_packet_id: None,
        latest_state: None,
        record_count: 0,
        finding_count: 1,
        acknowledgement_gap_count: 0,
        stale_active_packet_count: 0,
        blocked_carryover_count: 0,
        held_count: 1,
        release_gate_blocking_count: 1,
        unresolved_blocker_count: 0,
        findings: Vec::new(),
        governance_notes_copy: String::new(),
        summary: "Handoff governance review could not be loaded.".to_string(),
        commands: Vec::new(),
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
