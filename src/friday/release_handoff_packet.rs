use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    FridayDashboardPanelStatus, FridayReleaseEvidenceAttachmentReview,
    FridayReleaseEvidenceAttachmentState, read_friday_release_evidence_attachment_review,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayReleaseHandoffPacketSectionKind {
    OperatorSummary,
    AttachableFile,
    InlineNote,
    UnresolvedBlocker,
    ManifestChecksum,
}

impl FridayReleaseHandoffPacketSectionKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::OperatorSummary => "operator-summary",
            Self::AttachableFile => "attachable-file",
            Self::InlineNote => "inline-note",
            Self::UnresolvedBlocker => "unresolved-blocker",
            Self::ManifestChecksum => "manifest-checksum",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffPacketSection {
    pub id: String,
    pub kind: FridayReleaseHandoffPacketSectionKind,
    pub title: String,
    pub body: String,
    pub path: String,
    pub source_id: String,
    pub required: bool,
    pub included: bool,
    pub checksum: Option<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayReleaseHandoffPacket {
    pub packet_id: String,
    pub packet_json: String,
    pub generated_at_unix_ms: u128,
    pub product_name: String,
    pub local_only: bool,
    pub status: FridayDashboardPanelStatus,
    pub ready_to_send: bool,
    pub attachment_review_id: String,
    pub attachment_review_json: String,
    pub manifest_sha256: String,
    pub section_count: usize,
    pub included_count: usize,
    pub attachable_file_count: usize,
    pub inline_note_count: usize,
    pub unresolved_blocker_count: usize,
    pub checksum_count: usize,
    pub missing_count: usize,
    pub first_blocker: Option<String>,
    pub sections: Vec<FridayReleaseHandoffPacketSection>,
    pub file_checklist_copy: String,
    pub handoff_packet_copy: String,
    pub summary: String,
    pub commands: Vec<String>,
}

impl FridayReleaseHandoffPacket {
    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn friday_release_handoff_packet_report(
    packet_path: impl AsRef<Path>,
    attachment_review_path: impl AsRef<Path>,
) -> FridayReleaseHandoffPacket {
    let packet_path = packet_path.as_ref();
    let attachment_review_path = attachment_review_path.as_ref();
    let generated_at_unix_ms = unix_ms();
    let review = read_friday_release_evidence_attachment_review(attachment_review_path).ok();
    let fallback = fallback_attachment_review(attachment_review_path);
    let review = review.as_ref().unwrap_or(&fallback);

    let mut sections = packet_sections(review);
    sections.sort_by(|left, right| {
        section_rank(left.kind)
            .cmp(&section_rank(right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });

    let section_count = sections.len();
    let included_count = sections.iter().filter(|section| section.included).count();
    let attachable_file_count = kind_count(
        &sections,
        FridayReleaseHandoffPacketSectionKind::AttachableFile,
    );
    let inline_note_count =
        kind_count(&sections, FridayReleaseHandoffPacketSectionKind::InlineNote);
    let unresolved_blocker_count = kind_count(
        &sections,
        FridayReleaseHandoffPacketSectionKind::UnresolvedBlocker,
    );
    let checksum_count = kind_count(
        &sections,
        FridayReleaseHandoffPacketSectionKind::ManifestChecksum,
    );
    let missing_count = review.missing_count + review.checksum_missing_count;
    let ready_to_send = review.ready_for_handoff
        && unresolved_blocker_count == 0
        && missing_count == 0
        && attachable_file_count > 0;
    let status = if unresolved_blocker_count > 0 || missing_count > 0 {
        FridayDashboardPanelStatus::Blocked
    } else if inline_note_count > 0 {
        FridayDashboardPanelStatus::Warning
    } else {
        FridayDashboardPanelStatus::Ready
    };
    let packet_json = path_string(packet_path);

    FridayReleaseHandoffPacket {
        packet_id: format!("friday-release-handoff-packet-{generated_at_unix_ms}"),
        packet_json: packet_json.clone(),
        generated_at_unix_ms,
        product_name: "Friday".to_string(),
        local_only: true,
        status,
        ready_to_send,
        attachment_review_id: review.review_id.clone(),
        attachment_review_json: path_string(attachment_review_path),
        manifest_sha256: review.manifest_sha256.clone(),
        section_count,
        included_count,
        attachable_file_count,
        inline_note_count,
        unresolved_blocker_count,
        checksum_count,
        missing_count,
        first_blocker: sections
            .iter()
            .find(|section| {
                section.kind == FridayReleaseHandoffPacketSectionKind::UnresolvedBlocker
            })
            .map(|section| section.next_action.clone())
            .or_else(|| review.first_blocker.clone()),
        file_checklist_copy: file_checklist_copy(&sections),
        handoff_packet_copy: handoff_packet_copy(
            &packet_json,
            ready_to_send,
            &review.manifest_sha256,
            &sections,
        ),
        summary: format!(
            "Friday release handoff packet has {} section(s), {} attachable file(s), {} inline note(s), {} unresolved blocker(s), and {} checksum section(s).",
            section_count,
            attachable_file_count,
            inline_note_count,
            unresolved_blocker_count,
            checksum_count
        ),
        commands: vec![
            format!(
                "flow --friday-release-handoff-packet --output {} --attachment-review {}",
                packet_json,
                path_string(attachment_review_path)
            ),
            format!(
                "flow --friday-release-handoff-packet-json --output {} --attachment-review {}",
                packet_json,
                path_string(attachment_review_path)
            ),
        ],
        sections,
    }
}

pub fn write_friday_release_handoff_packet(
    packet_path: impl AsRef<Path>,
    packet: &FridayReleaseHandoffPacket,
) -> Result<()> {
    let packet_path = packet_path.as_ref();
    if let Some(parent) = packet_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create Friday release handoff packet directory {}",
                parent.display()
            )
        })?;
    }
    fs::write(packet_path, packet.to_pretty_json()?).with_context(|| {
        format!(
            "Could not write Friday release handoff packet {}",
            packet_path.display()
        )
    })
}

pub fn read_friday_release_handoff_packet(
    packet_path: impl AsRef<Path>,
) -> Result<FridayReleaseHandoffPacket> {
    let packet_path = packet_path.as_ref();
    let bytes = fs::read(packet_path).with_context(|| {
        format!(
            "Could not read Friday release handoff packet {}",
            packet_path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "Could not parse Friday release handoff packet {}",
            packet_path.display()
        )
    })
}

fn packet_sections(
    review: &FridayReleaseEvidenceAttachmentReview,
) -> Vec<FridayReleaseHandoffPacketSection> {
    let mut sections = vec![
        FridayReleaseHandoffPacketSection {
            id: "operator-summary".to_string(),
            kind: FridayReleaseHandoffPacketSectionKind::OperatorSummary,
            title: "Operator summary".to_string(),
            body: review.summary.clone(),
            path: review.review_json.clone(),
            source_id: review.review_id.clone(),
            required: true,
            included: true,
            checksum: None,
            next_action: if review.ready_for_handoff {
                "Send the handoff packet with the listed attachments.".to_string()
            } else {
                "Resolve unresolved blockers before sending this handoff packet.".to_string()
            },
        },
        FridayReleaseHandoffPacketSection {
            id: "manifest-checksum".to_string(),
            kind: FridayReleaseHandoffPacketSectionKind::ManifestChecksum,
            title: "Manifest checksum".to_string(),
            body: review.manifest_sha256.clone(),
            path: review.vault_json.clone(),
            source_id: review.vault_id.clone(),
            required: true,
            included: !review.manifest_sha256.trim().is_empty(),
            checksum: Some(review.manifest_sha256.clone()),
            next_action: "Include this checksum in the operator handoff note.".to_string(),
        },
    ];

    for item in &review.items {
        match item.state {
            FridayReleaseEvidenceAttachmentState::Ready => {
                sections.push(FridayReleaseHandoffPacketSection {
                    id: format!("attachable-file-{}", item.id),
                    kind: FridayReleaseHandoffPacketSectionKind::AttachableFile,
                    title: item.label.clone(),
                    body: item.summary.clone(),
                    path: item.path.clone(),
                    source_id: item.source_id.clone(),
                    required: item.required,
                    included: item.attachable,
                    checksum: item.sha256.clone(),
                    next_action: item.next_action.clone(),
                });
            }
            FridayReleaseEvidenceAttachmentState::InlineOnly => {
                sections.push(FridayReleaseHandoffPacketSection {
                    id: format!("inline-note-{}", item.id),
                    kind: FridayReleaseHandoffPacketSectionKind::InlineNote,
                    title: item.label.clone(),
                    body: item.summary.clone(),
                    path: item.path.clone(),
                    source_id: item.source_id.clone(),
                    required: item.required,
                    included: item.present,
                    checksum: item.sha256.clone(),
                    next_action: item.next_action.clone(),
                });
            }
            FridayReleaseEvidenceAttachmentState::Missing
            | FridayReleaseEvidenceAttachmentState::ChecksumMissing
            | FridayReleaseEvidenceAttachmentState::Blocked => {
                sections.push(FridayReleaseHandoffPacketSection {
                    id: format!("unresolved-blocker-{}", item.id),
                    kind: FridayReleaseHandoffPacketSectionKind::UnresolvedBlocker,
                    title: item.label.clone(),
                    body: item.summary.clone(),
                    path: item.path.clone(),
                    source_id: item.source_id.clone(),
                    required: item.required,
                    included: false,
                    checksum: item.sha256.clone(),
                    next_action: item.next_action.clone(),
                });
            }
        }
    }

    sections
}

fn kind_count(
    sections: &[FridayReleaseHandoffPacketSection],
    kind: FridayReleaseHandoffPacketSectionKind,
) -> usize {
    sections
        .iter()
        .filter(|section| section.kind == kind)
        .count()
}

fn section_rank(kind: FridayReleaseHandoffPacketSectionKind) -> u8 {
    match kind {
        FridayReleaseHandoffPacketSectionKind::OperatorSummary => 0,
        FridayReleaseHandoffPacketSectionKind::ManifestChecksum => 1,
        FridayReleaseHandoffPacketSectionKind::UnresolvedBlocker => 2,
        FridayReleaseHandoffPacketSectionKind::AttachableFile => 3,
        FridayReleaseHandoffPacketSectionKind::InlineNote => 4,
    }
}

fn file_checklist_copy(sections: &[FridayReleaseHandoffPacketSection]) -> String {
    let mut lines = vec!["Friday release handoff file checklist".to_string()];
    for section in sections
        .iter()
        .filter(|section| section.kind == FridayReleaseHandoffPacketSectionKind::AttachableFile)
    {
        lines.push(format!(
            "- [ ] {} -> {} ({})",
            section.title,
            section.path,
            section.checksum.as_deref().unwrap_or("checksum missing")
        ));
    }
    if lines.len() == 1 {
        lines.push("- [ ] No attachable files are ready.".to_string());
    }
    lines.join("\n")
}

fn handoff_packet_copy(
    packet_json: &str,
    ready_to_send: bool,
    manifest_sha256: &str,
    sections: &[FridayReleaseHandoffPacketSection],
) -> String {
    let mut lines = vec![
        format!("Friday release handoff packet: {packet_json}"),
        format!(
            "Status: {}",
            if ready_to_send {
                "ready to send"
            } else {
                "blocked before send"
            }
        ),
        format!("Manifest checksum: {manifest_sha256}"),
    ];

    lines.push("Operator summary:".to_string());
    for section in sections
        .iter()
        .filter(|section| section.kind == FridayReleaseHandoffPacketSectionKind::OperatorSummary)
    {
        lines.push(format!("- {}", section.body));
    }

    lines.push("Attachments:".to_string());
    for section in sections.iter().filter(|section| {
        section.kind == FridayReleaseHandoffPacketSectionKind::AttachableFile && section.included
    }) {
        lines.push(format!("- {} -> {}", section.title, section.path));
    }

    lines.push("Inline notes:".to_string());
    for section in sections
        .iter()
        .filter(|section| section.kind == FridayReleaseHandoffPacketSectionKind::InlineNote)
    {
        lines.push(format!("- {} -> {}", section.title, section.body));
    }

    lines.push("Unresolved blockers:".to_string());
    for section in sections
        .iter()
        .filter(|section| section.kind == FridayReleaseHandoffPacketSectionKind::UnresolvedBlocker)
    {
        lines.push(format!("- {} -> {}", section.title, section.next_action));
    }
    lines.join("\n")
}

fn fallback_attachment_review(review_path: &Path) -> FridayReleaseEvidenceAttachmentReview {
    FridayReleaseEvidenceAttachmentReview {
        review_id: "missing-release-evidence-attachment-review".to_string(),
        review_json: path_string(review_path),
        generated_at_unix_ms: unix_ms(),
        product_name: "Friday".to_string(),
        local_only: true,
        status: FridayDashboardPanelStatus::Blocked,
        ready_for_handoff: false,
        vault_id: "missing".to_string(),
        vault_json: String::new(),
        manifest_sha256: "missing".to_string(),
        item_count: 0,
        attachable_count: 0,
        missing_count: 1,
        inline_only_count: 0,
        checksum_missing_count: 0,
        blocked_count: 1,
        release_gate_blocking_count: 1,
        first_blocker: Some("Create a release evidence attachment review first.".to_string()),
        items: Vec::new(),
        handoff_notes_copy: String::new(),
        summary: "Release evidence attachment review could not be loaded.".to_string(),
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
