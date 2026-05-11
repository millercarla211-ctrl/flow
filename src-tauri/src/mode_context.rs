use crate::settings::{Personality, UserSettings};
use crate::{accessibility_context, permissions};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ModeContextMode {
    pub name: String,
    pub instructions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveStyleMatch {
    pub id: String,
    pub name: String,
    pub instruction_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveStylePreview {
    pub permission_granted: bool,
    pub context_available: bool,
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub url: Option<String>,
    pub matches: Vec<ActiveStyleMatch>,
}

fn extract_host(candidate: &str) -> Option<String> {
    let mut value = candidate.trim().to_lowercase();
    if value.is_empty() {
        return None;
    }

    if let Some(index) = value.find("://") {
        value = value[(index + 3)..].to_string();
    }

    let end_index = value
        .find(|ch| matches!(ch, '/' | '?' | '#'))
        .unwrap_or(value.len());
    let host_port = &value[..end_index];
    let host_port = host_port.split('@').next_back().unwrap_or(host_port);
    let host = if let Some(rest) = host_port.strip_prefix('[') {
        rest.find(']')
            .map(|end| &rest[..end])
            .unwrap_or_else(|| host_port.split(':').next().unwrap_or(host_port))
    } else {
        host_port.split(':').next().unwrap_or(host_port)
    };
    let host = host.trim_start_matches("www.");

    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

fn site_matches(url: &str, site: &str) -> bool {
    let Some(host) = extract_host(url) else {
        return false;
    };
    let Some(site_host) = extract_host(site) else {
        return false;
    };

    host == site_host || host.ends_with(&format!(".{site_host}"))
}

fn app_tokens(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_end_matches(".exe")
        .to_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn tokens_contain_all(haystack: &[String], needles: &[String]) -> bool {
    !needles.is_empty() && needles.iter().all(|needle| haystack.contains(needle))
}

fn app_matches(active_app_name: &str, configured_app_name: &str) -> bool {
    let active_tokens = app_tokens(active_app_name);
    let configured_tokens = app_tokens(configured_app_name);
    if active_tokens.is_empty() || configured_tokens.is_empty() {
        return false;
    }

    active_tokens == configured_tokens
        || tokens_contain_all(&configured_tokens, &active_tokens)
        || tokens_contain_all(&active_tokens, &configured_tokens)
}

fn mode_matches(mode: &Personality, context: &accessibility_context::ActiveContext) -> bool {
    let app_match = mode
        .apps
        .iter()
        .any(|app| app_matches(&context.app_name, app));
    let site_match = if let Some(url) = context.url.as_ref() {
        mode.websites.iter().any(|site| site_matches(url, site))
    } else {
        mode.websites.iter().any(|site| {
            site_matches(&context.window_title, site) || site_matches(&context.app_name, site)
        })
    };

    app_match || site_match
}

fn resolve_mode_context(settings: &UserSettings) -> Option<Vec<ModeContextMode>> {
    if !settings.context_awareness_enabled {
        return None;
    }

    if !permissions::check_accessibility_permission() {
        return None;
    }

    let context = accessibility_context::get_active_context()?;
    let modes: Vec<ModeContextMode> = settings
        .personalities
        .iter()
        .filter(|mode| mode.enabled)
        .filter(|mode| mode_matches(mode, &context))
        .map(|mode| ModeContextMode {
            name: mode.name.clone(),
            instructions: mode.instructions.clone(),
        })
        .collect();

    if modes.is_empty() {
        return None;
    }

    Some(modes)
}

pub fn format_mode_context(modes: &[ModeContextMode]) -> String {
    let mut lines = Vec::new();
    for mode in modes {
        let normalized_instructions: Vec<String> = mode
            .instructions
            .iter()
            .filter_map(|instruction| {
                let normalized = instruction.trim().trim_start_matches('-').trim();
                (!normalized.is_empty()).then(|| format!("- {}", normalized))
            })
            .collect();

        if normalized_instructions.is_empty() {
            continue;
        }

        lines.push(format!("Mode: {}", mode.name));
        lines.extend(normalized_instructions);
    }

    lines.join("\n")
}

pub fn format_active_cleanup_style_guidance(settings: &UserSettings) -> Option<String> {
    let modes = resolve_mode_context(settings)?;
    let instructions = format_mode_context(&modes);
    if instructions.is_empty() {
        None
    } else {
        Some(instructions)
    }
}

pub fn format_cleanup_style_guidance_for_personality(personality: &Personality) -> Option<String> {
    let mode = ModeContextMode {
        name: personality.name.clone(),
        instructions: personality.instructions.clone(),
    };
    let instructions = format_mode_context(&[mode]);
    if instructions.is_empty() {
        None
    } else {
        Some(instructions)
    }
}

pub fn resolve_active_personality(settings: &UserSettings) -> Option<Personality> {
    if !settings.context_awareness_enabled {
        return None;
    }

    if !permissions::check_accessibility_permission() {
        return None;
    }

    let context = accessibility_context::get_active_context()?;

    settings
        .personalities
        .iter()
        .filter(|mode| mode.enabled)
        .find(|mode| mode_matches(mode, &context))
        .cloned()
}

pub fn inspect_active_style_preview(settings: &UserSettings) -> ActiveStylePreview {
    if !settings.context_awareness_enabled {
        return ActiveStylePreview {
            permission_granted: true,
            context_available: false,
            app_name: None,
            window_title: None,
            url: None,
            matches: Vec::new(),
        };
    }

    if !permissions::check_accessibility_permission() {
        return ActiveStylePreview {
            permission_granted: false,
            context_available: false,
            app_name: None,
            window_title: None,
            url: None,
            matches: Vec::new(),
        };
    }

    let Some(context) = accessibility_context::get_active_context() else {
        return ActiveStylePreview {
            permission_granted: true,
            context_available: false,
            app_name: None,
            window_title: None,
            url: None,
            matches: Vec::new(),
        };
    };

    let matches = settings
        .personalities
        .iter()
        .filter(|mode| mode.enabled)
        .filter(|mode| mode_matches(mode, &context))
        .map(|mode| ActiveStyleMatch {
            id: mode.id.clone(),
            name: mode.name.clone(),
            instruction_count: mode
                .instructions
                .iter()
                .filter(|instruction| !instruction.trim().is_empty())
                .count(),
        })
        .collect();

    ActiveStylePreview {
        permission_granted: true,
        context_available: true,
        app_name: Some(context.app_name),
        window_title: (!context.window_title.trim().is_empty()).then_some(context.window_title),
        url: context.url,
        matches,
    }
}
