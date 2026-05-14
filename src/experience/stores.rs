use std::fs;
use std::path::PathBuf;

use super::{
    ApprovalScope,
    always_on::FlowDeviceTier,
    audit::CompactAuditRecord,
    contracts::FlowStateStore,
    installer::ModuleInstallStatus,
    modules::OperatingSystemFamily,
    persistence::{FlowPersistentState, PersistedApprovalRecord, PersistedModuleRecord},
    runtime_policy::DeviceBenchmarkSnapshot,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowFileStateStore {
    pub path: PathBuf,
}

impl FlowFileStateStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl FlowStateStore for FlowFileStateStore {
    fn load_state(&self) -> Option<FlowPersistentState> {
        let raw = fs::read_to_string(&self.path).ok()?;
        parse_state(&raw)
    }

    fn save_state(&mut self, state: FlowPersistentState) {
        let serialized = serialize_state(&state);
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&self.path, serialized);
    }
}

fn serialize_state(state: &FlowPersistentState) -> String {
    let mut lines = vec![
        "flow_state_v1".to_string(),
        format!("os={}", os_to_str(&state.os)),
        format!("tier={}", tier_to_str(&state.tier)),
    ];

    for module in &state.modules {
        lines.push(format!(
            "module={}|{}",
            escape(&module.id),
            install_status_to_str(&module.status)
        ));
    }

    for approval in &state.approvals {
        lines.push(format!(
            "approval={}|{}|{}",
            escape(&approval.capability),
            approval_scope_to_str(&approval.scope),
            approval.granted
        ));
    }

    for entry in &state.audit_entries {
        lines.push(format!(
            "audit={}|{}|{}|{}",
            escape(&entry.capability),
            escape(&entry.surface),
            entry.approved,
            escape(&entry.description)
        ));
    }

    for benchmark in &state.benchmark_history {
        lines.push(format!(
            "benchmark={}|{}|{}|{}|{}|{}|{}",
            benchmark.ram_gb,
            benchmark
                .vram_gb
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            benchmark.average_prompt_latency_ms,
            benchmark.average_decode_tokens_per_sec,
            benchmark
                .battery_percent
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            benchmark
                .thermal_celsius
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            benchmark.cpu_only
        ));
    }

    lines.join("\n")
}

fn parse_state(raw: &str) -> Option<FlowPersistentState> {
    let mut os = None;
    let mut tier = None;
    let mut modules = Vec::new();
    let mut approvals = Vec::new();
    let mut audit_entries = Vec::new();
    let mut benchmark_history = Vec::new();

    for line in raw.lines() {
        if line == "flow_state_v1" || line.trim().is_empty() {
            continue;
        }

        if let Some(value) = line.strip_prefix("os=") {
            os = Some(parse_os(value)?);
            continue;
        }

        if let Some(value) = line.strip_prefix("tier=") {
            tier = Some(parse_tier(value)?);
            continue;
        }

        if let Some(value) = line.strip_prefix("module=") {
            let parts = split_escaped(value);
            modules.push(PersistedModuleRecord {
                id: parts.first()?.clone(),
                status: parse_install_status(parts.get(1)?.as_str())?,
            });
            continue;
        }

        if let Some(value) = line.strip_prefix("approval=") {
            let parts = split_escaped(value);
            approvals.push(PersistedApprovalRecord {
                capability: parts.first()?.clone(),
                scope: parse_approval_scope(parts.get(1)?.as_str())?,
                granted: parts.get(2)?.parse().ok()?,
            });
            continue;
        }

        if let Some(value) = line.strip_prefix("audit=") {
            let parts = split_escaped(value);
            audit_entries.push(CompactAuditRecord {
                capability: parts.first()?.clone(),
                surface: parts.get(1)?.clone(),
                approved: parts.get(2)?.parse().ok()?,
                description: parts.get(3)?.clone(),
            });
            continue;
        }

        if let Some(value) = line.strip_prefix("benchmark=") {
            let parts = split_escaped(value);
            benchmark_history.push(DeviceBenchmarkSnapshot {
                ram_gb: parts.first()?.parse().ok()?,
                vram_gb: parse_optional_f32(parts.get(1)?.as_str()),
                average_prompt_latency_ms: parts.get(2)?.parse().ok()?,
                average_decode_tokens_per_sec: parts.get(3)?.parse().ok()?,
                battery_percent: parse_optional_u8(parts.get(4)?.as_str()),
                thermal_celsius: parse_optional_u8(parts.get(5)?.as_str()),
                cpu_only: parts.get(6)?.parse().ok()?,
            });
        }
    }

    Some(FlowPersistentState {
        os: os?,
        tier: tier?,
        modules,
        approvals,
        audit_entries,
        benchmark_history,
    })
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('|', "\\|")
}

fn split_escaped(value: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut escaping = false;

    for ch in value.chars() {
        if escaping {
            current.push(ch);
            escaping = false;
        } else if ch == '\\' {
            escaping = true;
        } else if ch == '|' {
            parts.push(current);
            current = String::new();
        } else {
            current.push(ch);
        }
    }

    if escaping {
        current.push('\\');
    }
    parts.push(current);
    parts
}

fn parse_optional_f32(value: &str) -> Option<f32> {
    if value == "-" {
        None
    } else {
        value.parse().ok()
    }
}

fn parse_optional_u8(value: &str) -> Option<u8> {
    if value == "-" {
        None
    } else {
        value.parse().ok()
    }
}

fn os_to_str(os: &OperatingSystemFamily) -> &'static str {
    match os {
        OperatingSystemFamily::Windows => "windows",
        OperatingSystemFamily::Macos => "macos",
        OperatingSystemFamily::Linux => "linux",
        OperatingSystemFamily::Android => "android",
        OperatingSystemFamily::Ios => "ios",
        OperatingSystemFamily::BrowserWasm => "browser",
        OperatingSystemFamily::Server => "server",
    }
}

fn parse_os(value: &str) -> Option<OperatingSystemFamily> {
    Some(match value {
        "windows" => OperatingSystemFamily::Windows,
        "macos" => OperatingSystemFamily::Macos,
        "linux" => OperatingSystemFamily::Linux,
        "android" => OperatingSystemFamily::Android,
        "ios" => OperatingSystemFamily::Ios,
        "browser" => OperatingSystemFamily::BrowserWasm,
        "server" => OperatingSystemFamily::Server,
        _ => return None,
    })
}

fn tier_to_str(tier: &FlowDeviceTier) -> &'static str {
    match tier {
        FlowDeviceTier::LowEnd => "low_end",
        FlowDeviceTier::Balanced => "balanced",
        FlowDeviceTier::Creator => "creator",
        FlowDeviceTier::Workstation => "workstation",
    }
}

fn parse_tier(value: &str) -> Option<FlowDeviceTier> {
    Some(match value {
        "low_end" => FlowDeviceTier::LowEnd,
        "balanced" => FlowDeviceTier::Balanced,
        "creator" => FlowDeviceTier::Creator,
        "workstation" => FlowDeviceTier::Workstation,
        _ => return None,
    })
}

fn install_status_to_str(status: &ModuleInstallStatus) -> &'static str {
    match status {
        ModuleInstallStatus::Pending => "pending",
        ModuleInstallStatus::Installed => "installed",
        ModuleInstallStatus::Deferred => "deferred",
        ModuleInstallStatus::Failed => "failed",
    }
}

fn parse_install_status(value: &str) -> Option<ModuleInstallStatus> {
    Some(match value {
        "pending" => ModuleInstallStatus::Pending,
        "installed" => ModuleInstallStatus::Installed,
        "deferred" => ModuleInstallStatus::Deferred,
        "failed" => ModuleInstallStatus::Failed,
        _ => return None,
    })
}

fn approval_scope_to_str(scope: &ApprovalScope) -> &'static str {
    match scope {
        ApprovalScope::Once => "once",
        ApprovalScope::Session => "session",
        ApprovalScope::Application => "application",
        ApprovalScope::Workspace => "workspace",
    }
}

fn parse_approval_scope(value: &str) -> Option<ApprovalScope> {
    Some(match value {
        "once" => ApprovalScope::Once,
        "session" => ApprovalScope::Session,
        "application" => ApprovalScope::Application,
        "workspace" => ApprovalScope::Workspace,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn file_state_store_round_trips_compact_audit_entries() {
        let path = std::env::temp_dir().join(format!(
            "flow_state_audit_test_{}.txt",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let mut store = FlowFileStateStore::new(path.clone());
        let state = FlowPersistentState {
            os: OperatingSystemFamily::Windows,
            tier: FlowDeviceTier::LowEnd,
            modules: Vec::new(),
            approvals: Vec::new(),
            audit_entries: vec![CompactAuditRecord {
                capability: "ReadSelection".to_string(),
                surface: "Desktop|Main".to_string(),
                description: "Read selected \\ text | from host.".to_string(),
                approved: true,
            }],
            benchmark_history: Vec::new(),
        };

        store.save_state(state.clone());
        let loaded = store.load_state().expect("state should round-trip");
        let summary = loaded.audit_summary(10);

        assert_eq!(loaded.audit_entries, state.audit_entries);
        assert_eq!(summary.total_entries, 1);
        assert_eq!(summary.approved_entries, 1);

        let _ = fs::remove_file(path);
    }
}
