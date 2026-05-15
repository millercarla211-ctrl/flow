use std::env;

use super::modules::OperatingSystemFamily;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessibilityBackend {
    UiAutomation,
    AppleScript,
    Xdotool,
    BrowserLimited,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessibilityMode {
    Full,
    ClipboardFallback,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowAccessibilityRuntime {
    pub os: OperatingSystemFamily,
    pub backend: AccessibilityBackend,
    pub mode: AccessibilityMode,
    pub can_read_selection: bool,
    pub can_replace_selection: bool,
    pub can_send_shortcuts: bool,
    pub availability_checked: bool,
    pub available: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessibilityDiagnosticSeverity {
    Ready,
    Degraded,
    Blocked,
}

impl AccessibilityDiagnosticSeverity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowAccessibilityDiagnostic {
    pub os: OperatingSystemFamily,
    pub backend: AccessibilityBackend,
    pub mode: AccessibilityMode,
    pub severity: AccessibilityDiagnosticSeverity,
    pub ready_for_full_automation: bool,
    pub ready_for_selection_rewrite: bool,
    pub ready_for_shortcuts: bool,
    pub summary: String,
    pub actions: Vec<String>,
    pub notes: Vec<String>,
}

impl FlowAccessibilityRuntime {
    pub fn dry_run(os: OperatingSystemFamily) -> Self {
        Self::for_host(os, false)
    }

    pub fn live(os: OperatingSystemFamily) -> Self {
        Self::for_host(os, true)
    }

    fn for_host(os: OperatingSystemFamily, live: bool) -> Self {
        match os {
            OperatingSystemFamily::Windows => windows_runtime(live),
            OperatingSystemFamily::Macos => macos_runtime(live),
            OperatingSystemFamily::Linux => linux_runtime(live),
            OperatingSystemFamily::BrowserWasm => Self {
                os,
                backend: AccessibilityBackend::BrowserLimited,
                mode: AccessibilityMode::Disabled,
                can_read_selection: false,
                can_replace_selection: false,
                can_send_shortcuts: false,
                availability_checked: true,
                available: false,
                notes: vec![
                    "Browser/WASM hosts require explicit DOM integration hooks.".to_string(),
                ],
            },
            _ => Self {
                os,
                backend: AccessibilityBackend::Unsupported,
                mode: AccessibilityMode::Disabled,
                can_read_selection: false,
                can_replace_selection: false,
                can_send_shortcuts: false,
                availability_checked: true,
                available: false,
                notes: vec![
                    "No accessibility automation backend is defined for this host family."
                        .to_string(),
                ],
            },
        }
    }

    pub fn is_full(&self) -> bool {
        matches!(self.mode, AccessibilityMode::Full)
    }

    pub fn can_automate_selection(&self) -> bool {
        self.available && self.can_read_selection && self.can_replace_selection
    }

    pub fn diagnostic(&self) -> FlowAccessibilityDiagnostic {
        let ready_for_selection_rewrite = self.can_automate_selection();
        let ready_for_shortcuts = self.available && self.can_send_shortcuts;
        let ready_for_full_automation = self.is_full()
            && ready_for_selection_rewrite
            && ready_for_shortcuts
            && self.os.is_desktop();

        let severity = if ready_for_full_automation {
            AccessibilityDiagnosticSeverity::Ready
        } else if self.available || ready_for_selection_rewrite || ready_for_shortcuts {
            AccessibilityDiagnosticSeverity::Degraded
        } else {
            AccessibilityDiagnosticSeverity::Blocked
        };

        let mut actions = Vec::new();
        if !self.os.is_desktop() {
            actions.push(
                "Use a desktop host adapter for native accessibility automation.".to_string(),
            );
        }
        if !self.available {
            actions.push(
                "Install or expose the native automation tools required for this host.".to_string(),
            );
        }
        if self.os.is_desktop() && !ready_for_selection_rewrite {
            actions.push(
                "Fix selection read/write support before enabling rewrite-selection automation."
                    .to_string(),
            );
        }
        if self.os.is_desktop() && !ready_for_shortcuts {
            actions.push(
                "Fix shortcut dispatch support before enabling global command shortcuts."
                    .to_string(),
            );
        }
        if self.os.is_desktop() && self.available && !self.is_full() {
            actions.push(
                "Run a live host adapter after first-run permission review to leave clipboard fallback mode."
                    .to_string(),
            );
        }
        if actions.is_empty() {
            actions.push("No accessibility action required for the current host mode.".to_string());
        }

        FlowAccessibilityDiagnostic {
            os: self.os.clone(),
            backend: self.backend.clone(),
            mode: self.mode.clone(),
            severity,
            ready_for_full_automation,
            ready_for_selection_rewrite,
            ready_for_shortcuts,
            summary: accessibility_summary(self),
            actions,
            notes: self.notes.clone(),
        }
    }
}

fn accessibility_summary(runtime: &FlowAccessibilityRuntime) -> String {
    if runtime.is_full() && runtime.can_automate_selection() && runtime.can_send_shortcuts {
        return "Full native accessibility automation is ready.".to_string();
    }

    if runtime.available && runtime.can_automate_selection() {
        return "Selection automation is available, but the host is not in full native mode."
            .to_string();
    }

    if runtime.available {
        return "Some native automation support is available, but Flow cannot safely rewrite selections yet."
            .to_string();
    }

    "No usable native accessibility automation path is available for this host.".to_string()
}

fn windows_runtime(live: bool) -> FlowAccessibilityRuntime {
    let mut notes = Vec::new();
    let powershell = command_available("powershell") || command_available("pwsh");
    if !powershell {
        notes.push(
            "PowerShell was not found in PATH, so desktop automation falls back.".to_string(),
        );
    }

    FlowAccessibilityRuntime {
        os: OperatingSystemFamily::Windows,
        backend: AccessibilityBackend::UiAutomation,
        mode: if live && powershell {
            AccessibilityMode::Full
        } else if powershell {
            AccessibilityMode::ClipboardFallback
        } else {
            AccessibilityMode::Disabled
        },
        can_read_selection: powershell,
        can_replace_selection: powershell,
        can_send_shortcuts: powershell,
        availability_checked: true,
        available: powershell,
        notes,
    }
}

fn macos_runtime(live: bool) -> FlowAccessibilityRuntime {
    let mut notes = Vec::new();
    let has_osascript = command_available("osascript");
    let has_pbcopy = command_available("pbcopy");
    let has_pbpaste = command_available("pbpaste");
    let clipboard_ready = has_pbcopy && has_pbpaste;

    if !has_osascript {
        notes
            .push("osascript is unavailable, so full shortcut automation is disabled.".to_string());
    }
    if !clipboard_ready {
        notes.push(
            "pbcopy/pbpaste are unavailable, so selection automation is limited.".to_string(),
        );
    }

    let available = has_osascript || clipboard_ready;
    let full = live && has_osascript && clipboard_ready;

    FlowAccessibilityRuntime {
        os: OperatingSystemFamily::Macos,
        backend: AccessibilityBackend::AppleScript,
        mode: if full {
            AccessibilityMode::Full
        } else if available {
            AccessibilityMode::ClipboardFallback
        } else {
            AccessibilityMode::Disabled
        },
        can_read_selection: clipboard_ready,
        can_replace_selection: clipboard_ready,
        can_send_shortcuts: has_osascript,
        availability_checked: true,
        available,
        notes,
    }
}

fn linux_runtime(live: bool) -> FlowAccessibilityRuntime {
    let mut notes = Vec::new();
    let has_xdotool = command_available("xdotool");
    let has_wl_copy = command_available("wl-copy");
    let has_wl_paste = command_available("wl-paste");
    let has_xclip = command_available("xclip");
    let clipboard_ready = (has_wl_copy && has_wl_paste) || has_xclip;

    if !has_xdotool {
        notes.push(
            "xdotool is unavailable, so shortcut dispatch will use clipboard fallback only."
                .to_string(),
        );
    }
    if !clipboard_ready {
        notes.push(
            "No supported Linux clipboard utility was found (wl-copy/wl-paste or xclip)."
                .to_string(),
        );
    }

    let available = has_xdotool || clipboard_ready;
    let full = live && has_xdotool && clipboard_ready;

    FlowAccessibilityRuntime {
        os: OperatingSystemFamily::Linux,
        backend: AccessibilityBackend::Xdotool,
        mode: if full {
            AccessibilityMode::Full
        } else if available {
            AccessibilityMode::ClipboardFallback
        } else {
            AccessibilityMode::Disabled
        },
        can_read_selection: clipboard_ready,
        can_replace_selection: clipboard_ready,
        can_send_shortcuts: has_xdotool,
        availability_checked: true,
        available,
        notes,
    }
}

fn command_available(command: &str) -> bool {
    let path = match env::var_os("PATH") {
        Some(path) => path,
        None => return false,
    };

    let path_dirs = env::split_paths(&path);
    #[cfg(windows)]
    let extensions: Vec<String> = env::var("PATHEXT")
        .ok()
        .map(|value| {
            value
                .split(';')
                .map(|item| item.to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_else(|| vec![".exe".to_string(), ".cmd".to_string(), ".bat".to_string()]);

    for dir in path_dirs {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return true;
        }

        #[cfg(windows)]
        {
            for extension in &extensions {
                let with_extension = dir.join(format!("{command}{extension}"));
                if with_extension.is_file() {
                    return true;
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        if std::path::Path::new(command).is_file() {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_hosts_are_disabled() {
        let runtime = FlowAccessibilityRuntime::dry_run(OperatingSystemFamily::BrowserWasm);
        assert!(matches!(runtime.mode, AccessibilityMode::Disabled));
        assert!(!runtime.available);
    }

    #[test]
    fn browser_diagnostic_is_blocked_with_action() {
        let runtime = FlowAccessibilityRuntime::dry_run(OperatingSystemFamily::BrowserWasm);
        let diagnostic = runtime.diagnostic();

        assert!(matches!(
            diagnostic.severity,
            AccessibilityDiagnosticSeverity::Blocked
        ));
        assert!(!diagnostic.ready_for_selection_rewrite);
        assert!(
            diagnostic
                .actions
                .iter()
                .any(|action| action.contains("desktop host adapter"))
        );
    }
}
