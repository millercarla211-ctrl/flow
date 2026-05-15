use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::BrowserHostFlavor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserExtensionSmokeStatus {
    Passed,
    Warning,
    Failed,
    Skipped,
}

impl BrowserExtensionSmokeStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    fn score(self) -> f32 {
        match self {
            Self::Passed | Self::Skipped => 1.0,
            Self::Warning => 0.5,
            Self::Failed => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserExtensionInstallProbe {
    pub target_id: String,
    pub platform_supported: bool,
    pub detected_executable: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserExtensionSmokeTarget {
    pub id: String,
    pub browser_name: String,
    pub host_flavor: BrowserHostFlavor,
    pub extension_target: String,
    pub platform_supported: bool,
    pub detected_executable: Option<String>,
    pub executable_candidates: Vec<String>,
    pub dist_dir: String,
    pub package_zip: String,
    pub package_sha256: String,
    pub status: BrowserExtensionSmokeStatus,
    pub evidence: Vec<String>,
    pub launch_command_hint: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserExtensionSmokeReport {
    pub generated_at_unix_ms: u128,
    pub summary: String,
    pub score_out_of_100: u8,
    pub local_only: bool,
    pub touches_network: bool,
    pub targets: Vec<BrowserExtensionSmokeTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserExtensionLaunchTarget {
    pub id: String,
    pub browser_name: String,
    pub extension_target: String,
    pub status: BrowserExtensionSmokeStatus,
    pub executable: Option<String>,
    pub profile_dir: Option<String>,
    pub command_preview: String,
    pub executed: bool,
    pub timed_out: bool,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    pub stdout_preview: String,
    pub stderr_preview: String,
    pub evidence: Vec<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserExtensionLaunchReport {
    pub generated_at_unix_ms: u128,
    pub summary: String,
    pub score_out_of_100: u8,
    pub execute: bool,
    pub timeout_ms: u64,
    pub local_only: bool,
    pub touches_network: bool,
    pub targets: Vec<BrowserExtensionLaunchTarget>,
}

impl BrowserExtensionSmokeReport {
    pub fn passed_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == BrowserExtensionSmokeStatus::Passed)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == BrowserExtensionSmokeStatus::Warning)
            .count()
    }

    pub fn blocking_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == BrowserExtensionSmokeStatus::Failed)
            .count()
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

impl BrowserExtensionLaunchReport {
    pub fn passed_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == BrowserExtensionSmokeStatus::Passed)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == BrowserExtensionSmokeStatus::Warning)
            .count()
    }

    pub fn blocking_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| target.status == BrowserExtensionSmokeStatus::Failed)
            .count()
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub fn browser_extension_smoke_report() -> BrowserExtensionSmokeReport {
    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    browser_extension_smoke_report_for_root(root, &default_install_probes())
}

pub fn browser_extension_launch_smoke_report(
    execute: bool,
    timeout_ms: u64,
) -> BrowserExtensionLaunchReport {
    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    browser_extension_launch_smoke_report_for_root(
        root,
        &default_install_probes(),
        execute,
        timeout_ms,
    )
}

pub fn browser_extension_smoke_report_for_root(
    root: impl AsRef<Path>,
    probes: &[BrowserExtensionInstallProbe],
) -> BrowserExtensionSmokeReport {
    let root = root.as_ref();
    let specs = target_specs();
    let targets = specs
        .iter()
        .map(|spec| smoke_target(root, spec, probes))
        .collect::<Vec<_>>();
    let score_out_of_100 = score_targets(&targets);
    let blocking = targets
        .iter()
        .filter(|target| target.status == BrowserExtensionSmokeStatus::Failed)
        .count();
    let warnings = targets
        .iter()
        .filter(|target| target.status == BrowserExtensionSmokeStatus::Warning)
        .count();

    BrowserExtensionSmokeReport {
        generated_at_unix_ms: unix_ms(),
        summary: format!(
            "{}/{} packaged browser smoke targets passed; {warnings} warning(s), {blocking} blocking issue(s).",
            targets
                .iter()
                .filter(|target| target.status == BrowserExtensionSmokeStatus::Passed)
                .count(),
            targets.len()
        ),
        score_out_of_100,
        local_only: true,
        touches_network: false,
        targets,
    }
}

pub fn browser_extension_launch_smoke_report_for_root(
    root: impl AsRef<Path>,
    probes: &[BrowserExtensionInstallProbe],
    execute: bool,
    timeout_ms: u64,
) -> BrowserExtensionLaunchReport {
    let smoke = browser_extension_smoke_report_for_root(root, probes);
    let targets = smoke
        .targets
        .iter()
        .map(|target| launch_target(target, execute, timeout_ms))
        .collect::<Vec<_>>();
    let score_out_of_100 = score_launch_targets(&targets);
    let blocking = targets
        .iter()
        .filter(|target| target.status == BrowserExtensionSmokeStatus::Failed)
        .count();
    let warnings = targets
        .iter()
        .filter(|target| target.status == BrowserExtensionSmokeStatus::Warning)
        .count();
    let passed = targets
        .iter()
        .filter(|target| target.status == BrowserExtensionSmokeStatus::Passed)
        .count();

    BrowserExtensionLaunchReport {
        generated_at_unix_ms: unix_ms(),
        summary: format!(
            "{passed}/{} browser launch smoke targets passed; {warnings} warning(s), {blocking} blocking issue(s).",
            targets.len()
        ),
        score_out_of_100,
        execute,
        timeout_ms,
        local_only: true,
        touches_network: false,
        targets,
    }
}

struct BrowserExtensionTargetSpec {
    id: &'static str,
    browser_name: &'static str,
    host_flavor: BrowserHostFlavor,
    extension_target: &'static str,
    platform_supported: bool,
    candidates: Vec<String>,
    launch_command_hint: &'static str,
    next_action: &'static str,
}

fn launch_target(
    target: &BrowserExtensionSmokeTarget,
    execute: bool,
    timeout_ms: u64,
) -> BrowserExtensionLaunchTarget {
    let profile_dir = temp_profile_dir(&target.id);
    let args = launch_args(target, &profile_dir);
    let command_preview = target
        .detected_executable
        .as_ref()
        .map(|executable| format!("{} {}", executable, args.join(" ")))
        .unwrap_or_else(|| target.launch_command_hint.clone());

    if target.status == BrowserExtensionSmokeStatus::Failed {
        return launch_target_status(
            target,
            BrowserExtensionSmokeStatus::Failed,
            Some(profile_dir),
            command_preview,
            false,
            false,
            None,
            0,
            "",
            "",
            vec!["package_or_dist_ready=no".to_string()],
            "Fix packaged dist/artifact readiness before attempting a browser launch.",
        );
    }

    if target.status == BrowserExtensionSmokeStatus::Skipped {
        return launch_target_status(
            target,
            BrowserExtensionSmokeStatus::Skipped,
            None,
            command_preview,
            false,
            false,
            None,
            0,
            "",
            "",
            vec!["platform_supported=no".to_string()],
            &target.next_action,
        );
    }

    let Some(executable) = target.detected_executable.as_deref() else {
        return launch_target_status(
            target,
            BrowserExtensionSmokeStatus::Warning,
            Some(profile_dir),
            command_preview,
            false,
            false,
            None,
            0,
            "",
            "",
            vec!["browser_detected=no".to_string()],
            &target.next_action,
        );
    };

    if target.extension_target != "chromium" {
        return launch_target_status(
            target,
            BrowserExtensionSmokeStatus::Warning,
            Some(profile_dir),
            command_preview,
            false,
            false,
            None,
            0,
            "",
            "",
            vec!["bounded_runner=chromium-only".to_string()],
            "Add a dedicated bounded runner for this browser family before marking it launch-smoked.",
        );
    }

    if !execute {
        return launch_target_status(
            target,
            BrowserExtensionSmokeStatus::Warning,
            Some(profile_dir),
            command_preview,
            false,
            false,
            None,
            0,
            "",
            "",
            vec![
                "execute=no".to_string(),
                "launch_args=ready".to_string(),
                "profile_scope=temp".to_string(),
            ],
            "Run `flow --browser-extension-launch-smoke --execute` for the bounded live smoke.",
        );
    }

    let execution = run_with_timeout(executable, &args, timeout_ms);
    let status = if execution.timed_out {
        BrowserExtensionSmokeStatus::Failed
    } else if execution.exit_code == Some(0) {
        BrowserExtensionSmokeStatus::Passed
    } else {
        BrowserExtensionSmokeStatus::Warning
    };
    let mut evidence = vec![
        "execute=yes".to_string(),
        "profile_scope=temp".to_string(),
        format!("exit_code={:?}", execution.exit_code),
        format!("timed_out={}", yes_no(execution.timed_out)),
    ];
    if let Err(error) = remove_profile_dir(&profile_dir) {
        evidence.push(format!("profile_cleanup={error}"));
    } else {
        evidence.push("profile_cleanup=ok".to_string());
    }

    launch_target_status(
        target,
        status,
        Some(profile_dir),
        command_preview,
        true,
        execution.timed_out,
        execution.exit_code,
        execution.duration_ms,
        &execution.stdout_preview,
        &execution.stderr_preview,
        evidence,
        if status == BrowserExtensionSmokeStatus::Passed {
            "Keep this target in the release smoke matrix."
        } else {
            "Inspect browser stderr and retry with the packaged dist after extension build changes."
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn launch_target_status(
    target: &BrowserExtensionSmokeTarget,
    status: BrowserExtensionSmokeStatus,
    profile_dir: Option<PathBuf>,
    command_preview: String,
    executed: bool,
    timed_out: bool,
    exit_code: Option<i32>,
    duration_ms: u128,
    stdout_preview: &str,
    stderr_preview: &str,
    evidence: Vec<String>,
    next_action: &str,
) -> BrowserExtensionLaunchTarget {
    BrowserExtensionLaunchTarget {
        id: target.id.clone(),
        browser_name: target.browser_name.clone(),
        extension_target: target.extension_target.clone(),
        status,
        executable: target.detected_executable.clone(),
        profile_dir: profile_dir.map(|path| path.to_string_lossy().into_owned()),
        command_preview,
        executed,
        timed_out,
        exit_code,
        duration_ms,
        stdout_preview: stdout_preview.to_string(),
        stderr_preview: stderr_preview.to_string(),
        evidence,
        next_action: next_action.to_string(),
    }
}

fn target_specs() -> Vec<BrowserExtensionTargetSpec> {
    vec![
        BrowserExtensionTargetSpec {
            id: "chrome",
            browser_name: "Google Chrome",
            host_flavor: BrowserHostFlavor::ChromiumExtension,
            extension_target: "chromium",
            platform_supported: cfg!(target_os = "windows")
                || cfg!(target_os = "macos")
                || cfg!(target_os = "linux"),
            candidates: chrome_candidates(),
            launch_command_hint: "chrome --user-data-dir <tmp-profile> --disable-extensions-except <dist/chromium> --load-extension <dist/chromium>",
            next_action: "Install Chrome or point the smoke runner at chrome.exe, then run the packaged extension launch check.",
        },
        BrowserExtensionTargetSpec {
            id: "edge",
            browser_name: "Microsoft Edge",
            host_flavor: BrowserHostFlavor::ChromiumExtension,
            extension_target: "chromium",
            platform_supported: cfg!(target_os = "windows")
                || cfg!(target_os = "macos")
                || cfg!(target_os = "linux"),
            candidates: edge_candidates(),
            launch_command_hint: "msedge --user-data-dir <tmp-profile> --disable-extensions-except <dist/chromium> --load-extension <dist/chromium>",
            next_action: "Install Edge or point the smoke runner at msedge.exe, then run the packaged extension launch check.",
        },
        BrowserExtensionTargetSpec {
            id: "firefox",
            browser_name: "Firefox",
            host_flavor: BrowserHostFlavor::FirefoxExtension,
            extension_target: "firefox",
            platform_supported: cfg!(target_os = "windows")
                || cfg!(target_os = "macos")
                || cfg!(target_os = "linux"),
            candidates: firefox_candidates(),
            launch_command_hint: "npx web-ext run --source-dir dist/firefox --firefox <firefox>",
            next_action: "Install Firefox or configure a Firefox path, then run the signed/temporary extension smoke.",
        },
        BrowserExtensionTargetSpec {
            id: "safari",
            browser_name: "Safari",
            host_flavor: BrowserHostFlavor::SafariWebExtension,
            extension_target: "safari",
            platform_supported: cfg!(target_os = "macos"),
            candidates: safari_candidates(),
            launch_command_hint: "xcrun safari-web-extension-converter dist/safari --project-location <tmp>",
            next_action: "Run the Safari smoke on macOS with Safari developer tools enabled.",
        },
    ]
}

fn launch_args(target: &BrowserExtensionSmokeTarget, profile_dir: &Path) -> Vec<String> {
    if target.extension_target == "chromium" {
        vec![
            "--headless=new".to_string(),
            "--disable-gpu".to_string(),
            "--no-first-run".to_string(),
            "--no-default-browser-check".to_string(),
            "--disable-background-networking".to_string(),
            "--disable-sync".to_string(),
            format!("--user-data-dir={}", profile_dir.display()),
            format!("--disable-extensions-except={}", target.dist_dir),
            format!("--load-extension={}", target.dist_dir),
            "about:blank".to_string(),
            "--dump-dom".to_string(),
        ]
    } else {
        Vec::new()
    }
}

#[derive(Debug)]
struct CommandExecution {
    timed_out: bool,
    exit_code: Option<i32>,
    duration_ms: u128,
    stdout_preview: String,
    stderr_preview: String,
}

fn run_with_timeout(executable: &str, args: &[String], timeout_ms: u64) -> CommandExecution {
    let started = std::time::Instant::now();
    let mut child = match Command::new(executable)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return CommandExecution {
                timed_out: false,
                exit_code: None,
                duration_ms: started.elapsed().as_millis(),
                stdout_preview: String::new(),
                stderr_preview: format!("spawn failed: {error}"),
            };
        }
    };

    let timeout = Duration::from_millis(timeout_ms.max(1));
    let mut timed_out = false;
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) if started.elapsed() >= timeout => {
                timed_out = true;
                let _ = child.kill();
                break;
            }
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(_error) => break,
        }
    }

    let output = child.wait_with_output();
    match output {
        Ok(output) => CommandExecution {
            timed_out,
            exit_code: output.status.code(),
            duration_ms: started.elapsed().as_millis(),
            stdout_preview: preview_bytes(&output.stdout),
            stderr_preview: preview_bytes(&output.stderr),
        },
        Err(error) => CommandExecution {
            timed_out,
            exit_code: None,
            duration_ms: started.elapsed().as_millis(),
            stdout_preview: String::new(),
            stderr_preview: format!("output failed: {error}"),
        },
    }
}

fn preview_bytes(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    text.chars().take(240).collect()
}

fn temp_profile_dir(target_id: &str) -> PathBuf {
    env::temp_dir().join(format!("flow-webext-{target_id}-{}", unix_ms()))
}

fn remove_profile_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_dir_all(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn smoke_target(
    root: &Path,
    spec: &BrowserExtensionTargetSpec,
    probes: &[BrowserExtensionInstallProbe],
) -> BrowserExtensionSmokeTarget {
    let dist_dir = root
        .join("extensions")
        .join("flow-webext")
        .join("dist")
        .join(spec.extension_target);
    let package_zip = root
        .join("extensions")
        .join("flow-webext")
        .join("artifacts")
        .join(format!("flow-webext-{}-v0.1.0.zip", spec.extension_target));
    let package_sha256 = package_zip.with_extension("zip.sha256");
    let dist_ready = required_dist_files(spec.extension_target)
        .iter()
        .all(|relative| file_ready(&dist_dir.join(relative)));
    let package_ready = file_ready(&package_zip) && file_ready(&package_sha256);
    let detected_probe = probes.iter().find(|probe| probe.target_id == spec.id);
    let platform_supported = detected_probe
        .map(|probe| probe.platform_supported)
        .unwrap_or(spec.platform_supported);
    let detected_executable = detected_probe
        .and_then(|probe| probe.detected_executable.clone())
        .or_else(|| detect_first_existing(&spec.candidates));
    let status = if !dist_ready || !package_ready {
        BrowserExtensionSmokeStatus::Failed
    } else if !platform_supported {
        BrowserExtensionSmokeStatus::Skipped
    } else if detected_executable.is_some() {
        BrowserExtensionSmokeStatus::Passed
    } else {
        BrowserExtensionSmokeStatus::Warning
    };

    let mut evidence = vec![
        format!("dist_ready={}", yes_no(dist_ready)),
        format!("package_ready={}", yes_no(package_ready)),
        format!("platform_supported={}", yes_no(platform_supported)),
        format!("browser_detected={}", yes_no(detected_executable.is_some())),
    ];
    evidence.extend(
        required_dist_files(spec.extension_target)
            .iter()
            .map(|relative| {
                let path = dist_dir.join(relative);
                format!("{}={}", path.display(), ready_label(file_ready(&path)))
            }),
    );
    evidence.push(format!(
        "{}={}",
        package_zip.display(),
        ready_label(file_ready(&package_zip))
    ));
    evidence.push(format!(
        "{}={}",
        package_sha256.display(),
        ready_label(file_ready(&package_sha256))
    ));

    BrowserExtensionSmokeTarget {
        id: spec.id.to_string(),
        browser_name: spec.browser_name.to_string(),
        host_flavor: spec.host_flavor,
        extension_target: spec.extension_target.to_string(),
        platform_supported,
        detected_executable,
        executable_candidates: spec.candidates.clone(),
        dist_dir: dist_dir.to_string_lossy().into_owned(),
        package_zip: package_zip.to_string_lossy().into_owned(),
        package_sha256: package_sha256.to_string_lossy().into_owned(),
        status,
        evidence,
        launch_command_hint: spec.launch_command_hint.to_string(),
        next_action: spec.next_action.to_string(),
    }
}

fn required_dist_files(target: &str) -> Vec<String> {
    let mut files = vec![
        "manifest.json",
        "popup.html",
        "sidepanel.html",
        "sidebar.html",
        "options.html",
        "flow.css",
        "background/index.js",
        "content/index.js",
        "ui/popup.js",
        "ui/options.js",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();

    if target == "chromium" {
        files.push("offscreen.html".to_string());
        files.push("ui/offscreen.js".to_string());
    }

    files
}

fn default_install_probes() -> Vec<BrowserExtensionInstallProbe> {
    target_specs()
        .into_iter()
        .map(|spec| BrowserExtensionInstallProbe {
            target_id: spec.id.to_string(),
            platform_supported: spec.platform_supported,
            detected_executable: detect_first_existing(&spec.candidates),
        })
        .collect()
}

fn detect_first_existing(candidates: &[String]) -> Option<String> {
    candidates
        .iter()
        .find(|candidate| Path::new(candidate.as_str()).exists())
        .cloned()
}

fn chrome_candidates() -> Vec<String> {
    let mut candidates = Vec::new();
    push_env_path(
        &mut candidates,
        "PROGRAMFILES",
        "Google/Chrome/Application/chrome.exe",
    );
    push_env_path(
        &mut candidates,
        "PROGRAMFILES(X86)",
        "Google/Chrome/Application/chrome.exe",
    );
    push_env_path(
        &mut candidates,
        "LOCALAPPDATA",
        "Google/Chrome/Application/chrome.exe",
    );
    candidates.push("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".to_string());
    candidates.push("google-chrome".to_string());
    candidates.push("chromium".to_string());
    candidates
}

fn edge_candidates() -> Vec<String> {
    let mut candidates = Vec::new();
    push_env_path(
        &mut candidates,
        "PROGRAMFILES",
        "Microsoft/Edge/Application/msedge.exe",
    );
    push_env_path(
        &mut candidates,
        "PROGRAMFILES(X86)",
        "Microsoft/Edge/Application/msedge.exe",
    );
    candidates.push("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge".to_string());
    candidates.push("microsoft-edge".to_string());
    candidates.push("msedge".to_string());
    candidates
}

fn firefox_candidates() -> Vec<String> {
    let mut candidates = Vec::new();
    push_env_path(
        &mut candidates,
        "PROGRAMFILES",
        "Mozilla Firefox/firefox.exe",
    );
    push_env_path(
        &mut candidates,
        "PROGRAMFILES(X86)",
        "Mozilla Firefox/firefox.exe",
    );
    candidates.push("/Applications/Firefox.app/Contents/MacOS/firefox".to_string());
    candidates.push("firefox".to_string());
    candidates
}

fn safari_candidates() -> Vec<String> {
    vec!["/Applications/Safari.app/Contents/MacOS/Safari".to_string()]
}

fn push_env_path(candidates: &mut Vec<String>, env_key: &str, relative: &str) {
    if let Some(base) = env::var_os(env_key) {
        candidates.push(
            PathBuf::from(base)
                .join(relative)
                .to_string_lossy()
                .into_owned(),
        );
    }
}

fn file_ready(path: &Path) -> bool {
    path.exists()
        && path
            .metadata()
            .map(|metadata| metadata.len() > 0)
            .unwrap_or(false)
}

fn score_targets(targets: &[BrowserExtensionSmokeTarget]) -> u8 {
    if targets.is_empty() {
        return 0;
    }

    let earned = targets
        .iter()
        .map(|target| target.status.score())
        .sum::<f32>();
    ((earned / targets.len() as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn score_launch_targets(targets: &[BrowserExtensionLaunchTarget]) -> u8 {
    if targets.is_empty() {
        return 0;
    }

    let earned = targets
        .iter()
        .map(|target| target.status.score())
        .sum::<f32>();
    ((earned / targets.len() as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn ready_label(ready: bool) -> &'static str {
    if ready { "present" } else { "missing" }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
