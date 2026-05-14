use super::control::ControlCapability;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalScope {
    Once,
    Session,
    Application,
    Workspace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlApproval {
    pub capability: ControlCapability,
    pub scope: ApprovalScope,
    pub granted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionAuditEntry {
    pub capability: ControlCapability,
    pub surface: String,
    pub description: String,
    pub approved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactAuditRecord {
    pub capability: String,
    pub surface: String,
    pub description: String,
    pub approved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowAuditSummary {
    pub total_entries: usize,
    pub approved_entries: usize,
    pub denied_entries: usize,
    pub recent_entries: Vec<CompactAuditRecord>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowControlAuditLog {
    approvals: Vec<ControlApproval>,
    entries: Vec<ActionAuditEntry>,
}

impl FlowControlAuditLog {
    pub fn grant(&mut self, capability: ControlCapability, scope: ApprovalScope) {
        self.approvals.push(ControlApproval {
            capability,
            scope,
            granted: true,
        });
    }

    pub fn record(
        &mut self,
        capability: ControlCapability,
        surface: impl Into<String>,
        description: impl Into<String>,
        approved: bool,
    ) {
        self.entries.push(ActionAuditEntry {
            capability,
            surface: surface.into(),
            description: description.into(),
            approved,
        });
    }

    pub fn approvals(&self) -> &[ControlApproval] {
        &self.approvals
    }

    pub fn entries(&self) -> &[ActionAuditEntry] {
        &self.entries
    }

    pub fn compact_records(&self, limit: usize) -> Vec<CompactAuditRecord> {
        compact_recent(
            &self
                .entries
                .iter()
                .map(CompactAuditRecord::from)
                .collect::<Vec<_>>(),
            limit,
        )
    }

    pub fn summary(&self, limit: usize) -> FlowAuditSummary {
        FlowAuditSummary::from_records(
            &self
                .entries
                .iter()
                .map(CompactAuditRecord::from)
                .collect::<Vec<_>>(),
            limit,
        )
    }
}

impl FlowAuditSummary {
    pub fn from_records(records: &[CompactAuditRecord], limit: usize) -> Self {
        let approved_entries = records.iter().filter(|record| record.approved).count();
        let total_entries = records.len();
        Self {
            total_entries,
            approved_entries,
            denied_entries: total_entries.saturating_sub(approved_entries),
            recent_entries: compact_recent(records, limit),
        }
    }
}

impl From<&ActionAuditEntry> for CompactAuditRecord {
    fn from(entry: &ActionAuditEntry) -> Self {
        Self {
            capability: format!("{:?}", entry.capability),
            surface: entry.surface.clone(),
            description: entry.description.clone(),
            approved: entry.approved,
        }
    }
}

fn compact_recent(records: &[CompactAuditRecord], limit: usize) -> Vec<CompactAuditRecord> {
    if limit == 0 {
        return Vec::new();
    }

    let start = records.len().saturating_sub(limit);
    records[start..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_audit_summary_limits_recent_entries() {
        let mut audit = FlowControlAuditLog::default();
        audit.record(
            ControlCapability::ReadSelection,
            "Desktop",
            "Read selection through automation bridge.",
            true,
        );
        audit.record(
            ControlCapability::ReplaceSelection,
            "Desktop",
            "Replace selection through automation bridge.",
            false,
        );
        audit.record(
            ControlCapability::SimulateShortcut,
            "Overlay",
            "Dispatch overlay shortcut.",
            true,
        );

        let summary = audit.summary(2);

        assert_eq!(summary.total_entries, 3);
        assert_eq!(summary.approved_entries, 2);
        assert_eq!(summary.denied_entries, 1);
        assert_eq!(summary.recent_entries.len(), 2);
        assert_eq!(summary.recent_entries[0].capability, "ReplaceSelection");
        assert_eq!(summary.recent_entries[1].capability, "SimulateShortcut");
    }
}
