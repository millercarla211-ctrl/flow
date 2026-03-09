use anyhow::{anyhow, Result};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::AppRuntime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HotkeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone)]
pub(crate) struct HotkeyEvent {
    pub state: HotkeyState,
}

pub(crate) trait HotkeyProvider {
    fn unregister_all(&self) -> Result<()>;

    fn on_shortcut<F>(&self, shortcut: &str, handler: F) -> Result<()>
    where
        F: Fn(&AppHandle<AppRuntime>, HotkeyEvent) + Send + Sync + 'static;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModifierToken {
    Control,
    Shift,
    Alt,
    Command,
}

fn platform_command_or_control() -> ModifierToken {
    #[cfg(target_os = "macos")]
    {
        ModifierToken::Command
    }
    #[cfg(not(target_os = "macos"))]
    {
        ModifierToken::Control
    }
}

fn parse_modifier_token(token: &str) -> Option<ModifierToken> {
    match token.to_ascii_lowercase().as_str() {
        "control" | "ctrl" | "leftcontrol" | "rightcontrol" => Some(ModifierToken::Control),
        "shift" | "leftshift" | "rightshift" => Some(ModifierToken::Shift),
        "alt" | "option" | "leftalt" | "rightalt" | "leftoption" | "rightoption" => {
            Some(ModifierToken::Alt)
        }
        "command" | "cmd" | "meta" | "super" | "leftcommand" | "rightcommand" => {
            Some(ModifierToken::Command)
        }
        "commandorcontrol" | "commandorctrl" | "cmdorctrl" | "cmdorcontrol" => {
            Some(platform_command_or_control())
        }
        _ => None,
    }
}

fn normalize_registered_key(token: &str) -> Option<String> {
    let key = token.trim();
    if key.is_empty() {
        return None;
    }
    if parse_modifier_token(key).is_some() {
        return None;
    }

    if key.len() == 1 {
        let ch = key.chars().next()?;
        if ch.is_ascii_alphabetic() || ch.is_ascii_digit() {
            return Some(ch.to_ascii_lowercase().to_string());
        }
        if "`-=[]\\;',./".contains(ch) {
            return Some(ch.to_string());
        }
    }

    let lower = key.to_ascii_lowercase();
    if let Some(number) = lower.strip_prefix('f') {
        if let Ok(value) = number.parse::<u8>() {
            if (1..=24).contains(&value) {
                return Some(format!("f{value}"));
            }
        }
    }

    match lower.as_str() {
        "space" | "spacebar" => Some("space".to_string()),
        "enter" => Some("enter".to_string()),
        "tab" => Some("tab".to_string()),
        "backspace" => Some("backspace".to_string()),
        "escape" | "esc" => Some("escape".to_string()),
        "delete" | "del" => Some("delete".to_string()),
        "up" | "arrowup" => Some("arrowup".to_string()),
        "down" | "arrowdown" => Some("arrowdown".to_string()),
        "left" | "arrowleft" => Some("arrowleft".to_string()),
        "right" | "arrowright" => Some("arrowright".to_string()),
        _ => None,
    }
}

pub(crate) fn normalize_shortcut(shortcut: &str) -> Result<String> {
    let mut saw_token = false;
    let mut control = false;
    let mut shift = false;
    let mut alt = false;
    let mut command = false;
    let mut key: Option<String> = None;

    for token in shortcut
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        saw_token = true;

        if let Some(modifier) = parse_modifier_token(token) {
            match modifier {
                ModifierToken::Control => control = true,
                ModifierToken::Shift => shift = true,
                ModifierToken::Alt => alt = true,
                ModifierToken::Command => command = true,
            }
            continue;
        }

        let normalized_key = normalize_registered_key(token)
            .ok_or_else(|| anyhow!("Shortcut `{shortcut}` has an invalid base key `{token}`"))?;

        if key.replace(normalized_key).is_some() {
            return Err(anyhow!(
                "Shortcut `{shortcut}` has multiple base keys; expected one"
            ));
        }
    }

    if !saw_token {
        return Err(anyhow!("Shortcut `{shortcut}` is empty"));
    }

    let key = key.ok_or_else(|| {
        anyhow!("Shortcut `{shortcut}` must include a non-modifier key (e.g. Control+Space)")
    })?;

    let mut parts = Vec::with_capacity(5);
    if command {
        parts.push("Command".to_string());
    }
    if alt {
        parts.push("Alt".to_string());
    }
    if control {
        parts.push("Control".to_string());
    }
    if shift {
        parts.push("Shift".to_string());
    }
    parts.push(key);

    Ok(parts.join("+"))
}

struct GlobalShortcutProvider<'a> {
    app: &'a AppHandle<AppRuntime>,
}

impl<'a> GlobalShortcutProvider<'a> {
    fn new(app: &'a AppHandle<AppRuntime>) -> Self {
        Self { app }
    }
}

impl HotkeyProvider for GlobalShortcutProvider<'_> {
    fn unregister_all(&self) -> Result<()> {
        self.app
            .global_shortcut()
            .unregister_all()
            .map_err(|err| anyhow!("Failed to unregister shortcuts: {err}"))?;

        Ok(())
    }

    fn on_shortcut<F>(&self, shortcut: &str, handler: F) -> Result<()>
    where
        F: Fn(&AppHandle<AppRuntime>, HotkeyEvent) + Send + Sync + 'static,
    {
        let normalized_shortcut = normalize_shortcut(shortcut)?;
        self.app
            .global_shortcut()
            .on_shortcut(normalized_shortcut.as_str(), move |app, _pressed_shortcut, event| {
                let state = match event.state {
                    ShortcutState::Pressed => HotkeyState::Pressed,
                    ShortcutState::Released => HotkeyState::Released,
                };
                handler(app, HotkeyEvent { state });
            })
            .map_err(|err| {
                anyhow!(
                    "Failed to register shortcut `{shortcut}` (normalized `{normalized_shortcut}`): {err}"
                )
            })?;

        Ok(())
    }
}

pub(crate) fn provider(app: &AppHandle<AppRuntime>) -> impl HotkeyProvider + '_ {
    GlobalShortcutProvider::new(app)
}
