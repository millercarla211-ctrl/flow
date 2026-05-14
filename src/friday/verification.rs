use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::default_friday_local_execution_checks;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FridayVerificationStatus {
    Passed,
    Warning,
    Failed,
}

impl FridayVerificationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayBrowserVerificationTarget {
    pub id: String,
    pub surface: String,
    pub command: String,
    pub status: FridayVerificationStatus,
    pub evidence: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayDeployGate {
    pub deployment_allowed: bool,
    pub major_user_visible_feature: String,
    pub required_verification_command: String,
    pub local_checks_passed: bool,
    pub browser_targets_passed: bool,
    pub deploy_rule: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FridayBrowserVerificationReport {
    pub generated_at_unix_ms: u128,
    pub summary: String,
    pub targets: Vec<FridayBrowserVerificationTarget>,
    pub deploy_gate: FridayDeployGate,
}

impl FridayBrowserVerificationReport {
    pub fn passed_target_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == FridayVerificationStatus::Passed)
            .count()
    }

    pub fn blocking_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == FridayVerificationStatus::Failed)
            .count()
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn default_friday_browser_verification_report() -> FridayBrowserVerificationReport {
    let targets = vec![
        extension_source_target(),
        extension_chromium_target(),
        extension_firefox_target(),
        extension_safari_target(),
        extension_artifact_target("chromium"),
        extension_artifact_target("firefox"),
        extension_artifact_target("safari"),
    ];
    let local_report = default_friday_local_execution_checks();
    let local_checks_passed = local_report.blocking_count() == 0;
    let browser_targets_passed = targets
        .iter()
        .all(|target| target.status == FridayVerificationStatus::Passed);
    let major_user_visible_feature =
        "Friday UI contracts, local execution checks, and production route states".to_string();
    let deployment_allowed = local_checks_passed && browser_targets_passed;

    FridayBrowserVerificationReport {
        generated_at_unix_ms: unix_ms(),
        summary: format!(
            "{}/{} browser targets passed; local checks {}; deploy gate {}.",
            targets
                .iter()
                .filter(|target| target.status == FridayVerificationStatus::Passed)
                .count(),
            targets.len(),
            if local_checks_passed { "passed" } else { "blocked" },
            if deployment_allowed { "open" } else { "closed" }
        ),
        targets,
        deploy_gate: FridayDeployGate {
            deployment_allowed,
            major_user_visible_feature,
            required_verification_command:
                "npm run typecheck (extensions/flow-webext) + flow --friday-local-checks"
                    .to_string(),
            local_checks_passed,
            browser_targets_passed,
            deploy_rule:
                "Deploy only after a major user-visible feature ships and both browser and local execution checks pass."
                    .to_string(),
            notes: vec![
                "This repository's tracked browser surface is the Flow WebExtension, not an untracked Next/Tauri app."
                    .to_string(),
                "Vercel deployment remains gated until a tracked web app surface exists or the release target is explicitly browser-extension packaging."
                    .to_string(),
            ],
        },
    }
}

fn extension_source_target() -> FridayBrowserVerificationTarget {
    let required = [
        "extensions/flow-webext/package.json",
        "extensions/flow-webext/tsconfig.json",
        "extensions/flow-webext/src/ui/app.ts",
        "extensions/flow-webext/src/runtime/flow-engine.ts",
        "extensions/flow-webext/static/flow.css",
    ];
    file_target(
        "flow-webext-source",
        "Browser extension source",
        "npm run typecheck",
        &required,
        "Run `npm run typecheck` from `extensions/flow-webext` after source edits.",
    )
}

fn extension_chromium_target() -> FridayBrowserVerificationTarget {
    extension_dist_target("chromium")
}

fn extension_firefox_target() -> FridayBrowserVerificationTarget {
    extension_dist_target("firefox")
}

fn extension_safari_target() -> FridayBrowserVerificationTarget {
    extension_dist_target("safari")
}

fn extension_dist_target(target: &str) -> FridayBrowserVerificationTarget {
    let required = [
        format!("extensions/flow-webext/dist/{target}/manifest.json"),
        format!("extensions/flow-webext/dist/{target}/popup.html"),
        format!("extensions/flow-webext/dist/{target}/sidepanel.html"),
        format!("extensions/flow-webext/dist/{target}/sidebar.html"),
        format!("extensions/flow-webext/dist/{target}/options.html"),
        format!("extensions/flow-webext/dist/{target}/flow.css"),
    ];
    let refs = required.iter().map(String::as_str).collect::<Vec<_>>();
    file_target(
        &format!("flow-webext-{target}-dist"),
        &format!("{target} extension dist"),
        &format!("npm run build:{target}"),
        &refs,
        &format!("Run `npm run build:{target}` from `extensions/flow-webext`."),
    )
}

fn extension_artifact_target(target: &str) -> FridayBrowserVerificationTarget {
    let required = [
        format!("extensions/flow-webext/artifacts/flow-webext-{target}-v0.1.0.zip"),
        format!("extensions/flow-webext/artifacts/flow-webext-{target}-v0.1.0.zip.sha256"),
    ];
    let refs = required.iter().map(String::as_str).collect::<Vec<_>>();
    file_target(
        &format!("flow-webext-{target}-artifact"),
        &format!("{target} packaged artifact"),
        &format!("npm run package:{target}"),
        &refs,
        &format!("Run `npm run package:{target}` from `extensions/flow-webext`."),
    )
}

fn file_target(
    id: &str,
    surface: &str,
    command: &str,
    required_files: &[&str],
    next_action: &str,
) -> FridayBrowserVerificationTarget {
    let evidence = required_files
        .iter()
        .map(|path| {
            let present = file_ready(path);
            format!("{path}={}", if present { "present" } else { "missing" })
        })
        .collect::<Vec<_>>();
    let passed = required_files.iter().all(|path| file_ready(path));

    FridayBrowserVerificationTarget {
        id: id.to_string(),
        surface: surface.to_string(),
        command: command.to_string(),
        status: if passed {
            FridayVerificationStatus::Passed
        } else {
            FridayVerificationStatus::Failed
        },
        evidence,
        next_action: next_action.to_string(),
    }
}

fn file_ready(path: &str) -> bool {
    let path = Path::new(path);
    path.exists()
        && fs::metadata(path)
            .map(|metadata| metadata.is_dir() || metadata.len() > 0)
            .unwrap_or(false)
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_verification_report_checks_extension_surfaces() {
        let report = default_friday_browser_verification_report();
        let ids = report
            .targets
            .iter()
            .map(|target| target.id.as_str())
            .collect::<std::collections::HashSet<_>>();

        assert!(ids.contains("flow-webext-source"));
        assert!(ids.contains("flow-webext-chromium-dist"));
        assert!(ids.contains("flow-webext-firefox-dist"));
        assert!(ids.contains("flow-webext-safari-dist"));
        assert!(ids.contains("flow-webext-chromium-artifact"));
        assert!(ids.contains("flow-webext-firefox-artifact"));
        assert!(ids.contains("flow-webext-safari-artifact"));
    }

    #[test]
    fn deploy_gate_requires_local_and_browser_checks() {
        let report = default_friday_browser_verification_report();

        assert_eq!(
            report.deploy_gate.browser_targets_passed,
            report.blocking_count() == 0
        );
        assert!(
            report
                .deploy_gate
                .required_verification_command
                .contains("flow --friday-local-checks")
        );
        assert!(
            report
                .deploy_gate
                .deploy_rule
                .contains("major user-visible feature")
        );
    }
}
