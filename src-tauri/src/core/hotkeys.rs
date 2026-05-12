use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{anyhow, Result};
use handy_keys::{Hotkey, KeyboardListener, Modifiers};
use handy_keys::{HotkeyManager, HotkeyState as HandyHotkeyState};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::{pill, AppRuntime};

const REGISTRATION_POLL_INTERVAL: Duration = Duration::from_millis(10);
const CAPTURE_POLL_INTERVAL: Duration = Duration::from_millis(20);
pub(crate) const SHORTCUT_CAPTURE_EVENT: &str = "shortcut:capture";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HotkeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShortcutAction {
    Smart,
    Hold,
    Toggle,
    Command,
    PasteLastTranscript,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MouseShortcut {
    Middle,
    Back,
    Forward,
}

impl MouseShortcut {
    fn as_str(self) -> &'static str {
        match self {
            Self::Middle => "MouseMiddle",
            Self::Back => "MouseBack",
            Self::Forward => "MouseForward",
        }
    }
}

impl std::fmt::Display for MouseShortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShortcutBinding {
    Keyboard(Hotkey),
    Mouse(MouseShortcut),
}

impl std::fmt::Display for ShortcutBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keyboard(hotkey) => write!(f, "{hotkey}"),
            Self::Mouse(mouse) => write!(f, "{mouse}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RegisteredShortcut {
    pub binding: ShortcutBinding,
    pub action: ShortcutAction,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ShortcutCapturePayload {
    Preview { shortcut: String },
    Captured { shortcut: String },
    Error { message: String },
}

#[derive(Default)]
pub(crate) struct HotkeyCoordinator {
    registration: Mutex<Option<WorkerSession>>,
    capture: Mutex<Option<WorkerSession>>,
}

impl HotkeyCoordinator {
    pub(crate) fn replace_registrations(
        &self,
        app: &AppHandle<AppRuntime>,
        bindings: Vec<RegisteredShortcut>,
    ) -> Result<()> {
        self.stop_registration();

        if bindings.is_empty() {
            return Ok(());
        }

        let app_handle = app.clone();
        let session = WorkerSession::spawn("shortcut-registration", move |stop_rx| {
            let keyboard_bindings: Vec<_> = bindings
                .iter()
                .filter_map(|binding| match binding.binding {
                    ShortcutBinding::Keyboard(hotkey) => Some((hotkey, binding.action)),
                    ShortcutBinding::Mouse(_) => None,
                })
                .collect();
            let mouse_bindings: Vec<_> = bindings
                .iter()
                .filter_map(|binding| match binding.binding {
                    ShortcutBinding::Mouse(mouse) => Some((mouse, binding.action)),
                    ShortcutBinding::Keyboard(_) => None,
                })
                .collect();

            let manager = if keyboard_bindings.is_empty() {
                None
            } else {
                Some(HotkeyManager::new_with_blocking()?)
            };

            let mut actions = Vec::with_capacity(keyboard_bindings.len());
            if let Some(manager) = manager.as_ref() {
                for (hotkey, action) in keyboard_bindings {
                    let id = manager
                        .register(hotkey)
                        .map_err(|err| anyhow!("Failed to register shortcut `{hotkey}`: {err}"))?;
                    actions.push((id, action));
                }
            }

            let mut mouse_states = vec![false; mouse_bindings.len()];

            loop {
                if should_stop(&stop_rx) {
                    break;
                }

                if let Some(manager) = manager.as_ref() {
                    if let Some(event) = manager.try_recv() {
                        if let Some((_, action)) = actions
                            .iter()
                            .find(|(registered_id, _)| *registered_id == event.id)
                        {
                            let state = match event.state {
                                HandyHotkeyState::Pressed => HotkeyState::Pressed,
                                HandyHotkeyState::Released => HotkeyState::Released,
                            };
                            pill::handle_registered_hotkey_event(&app_handle, *action, state);
                        }
                        continue;
                    }
                }

                poll_mouse_bindings(&app_handle, &mouse_bindings, &mut mouse_states);
                thread::sleep(REGISTRATION_POLL_INTERVAL);
            }

            Ok(())
        })?;

        *self.registration.lock() = Some(session);
        Ok(())
    }

    pub(crate) fn stop_registration(&self) {
        self.registration.lock().take();
    }

    pub(crate) fn start_capture(&self, app: &AppHandle<AppRuntime>) -> Result<()> {
        self.stop_capture();

        let app_handle = app.clone();
        let session = WorkerSession::spawn("shortcut-capture", move |stop_rx| {
            let listener = KeyboardListener::new()?;
            let mut captured_hotkey: Option<Hotkey> = None;

            loop {
                if should_stop(&stop_rx) {
                    break;
                }

                match listener.recv_timeout(CAPTURE_POLL_INTERVAL) {
                    Ok(event) => {
                        if !event.is_key_down {
                            if captured_hotkey
                                .as_ref()
                                .is_some_and(|hotkey| hotkey.key.is_some())
                            {
                                let hotkey = captured_hotkey.take().expect("checked above");
                                emit_capture_event(
                                    &app_handle,
                                    ShortcutCapturePayload::Captured {
                                        shortcut: hotkey.to_string(),
                                    },
                                );
                                break;
                            }
                            continue;
                        }

                        if let Ok(hotkey) = event.as_hotkey() {
                            let captured = merge_capture_hotkey(captured_hotkey, hotkey);
                            captured_hotkey = Some(captured);
                            emit_capture_event(
                                &app_handle,
                                ShortcutCapturePayload::Preview {
                                    shortcut: captured.to_string(),
                                },
                            );
                        }
                    }
                    Err(handy_keys::Error::Timeout) => {}
                    Err(err) => {
                        emit_capture_event(
                            &app_handle,
                            ShortcutCapturePayload::Error {
                                message: format!("Shortcut capture failed: {err}"),
                            },
                        );
                        break;
                    }
                }
            }

            Ok(())
        })?;

        *self.capture.lock() = Some(session);
        Ok(())
    }

    pub(crate) fn stop_capture(&self) {
        self.capture.lock().take();
    }
}

fn merge_capture_hotkey(previous: Option<Hotkey>, current: Hotkey) -> Hotkey {
    if let Some(previous) = previous {
        Hotkey {
            modifiers: previous.modifiers | current.modifiers,
            key: current.key.or(previous.key),
        }
    } else {
        current
    }
}

fn emit_capture_event(app: &AppHandle<AppRuntime>, payload: ShortcutCapturePayload) {
    if let Err(err) = app.emit(SHORTCUT_CAPTURE_EVENT, payload) {
        eprintln!("Failed to emit shortcut capture event: {err}");
    }
}

struct WorkerSession {
    stop_tx: Sender<()>,
    join_handle: Option<JoinHandle<()>>,
}

impl WorkerSession {
    fn spawn<F>(thread_name: &str, task: F) -> Result<Self>
    where
        F: FnOnce(Receiver<()>) -> Result<()> + Send + 'static,
    {
        let (stop_tx, stop_rx) = mpsc::channel();
        let join_handle = thread::Builder::new()
            .name(thread_name.to_string())
            .spawn(move || {
                if let Err(err) = task(stop_rx) {
                    eprintln!("Hotkey worker exited with error: {err}");
                }
            })
            .map_err(|err| anyhow!("Failed to spawn hotkey worker: {err}"))?;

        Ok(Self {
            stop_tx,
            join_handle: Some(join_handle),
        })
    }
}

impl Drop for WorkerSession {
    fn drop(&mut self) {
        let _ = self.stop_tx.send(());
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

fn should_stop(stop_rx: &Receiver<()>) -> bool {
    matches!(stop_rx.try_recv(), Ok(()) | Err(TryRecvError::Disconnected))
}

pub(crate) fn parse_shortcut(shortcut: &str) -> Result<Hotkey> {
    let normalized = normalize_legacy_shortcut_input(shortcut);
    normalized
        .parse::<Hotkey>()
        .map_err(|err| anyhow!("Shortcut `{shortcut}` is invalid: {err}"))
}

pub(crate) fn parse_shortcut_binding(shortcut: &str) -> Result<ShortcutBinding> {
    if let Some(mouse) = parse_mouse_shortcut(shortcut) {
        return Ok(ShortcutBinding::Mouse(mouse));
    }

    parse_shortcut(shortcut).map(ShortcutBinding::Keyboard)
}

pub(crate) fn normalize_recording_shortcut(shortcut: &str) -> Result<String> {
    Ok(parse_shortcut_binding(shortcut)?.to_string())
}

fn normalize_legacy_shortcut_input(shortcut: &str) -> String {
    shortcut
        .split('+')
        .map(|token| match token.trim().to_ascii_lowercase().as_str() {
            "commandorcontrol" | "commandorctrl" | "cmdorctrl" | "cmdorcontrol" => {
                if cfg!(target_os = "macos") {
                    "Cmd".to_string()
                } else {
                    "Ctrl".to_string()
                }
            }
            "command" | "cmd" | "meta" | "win" | "windows" => "Cmd".to_string(),
            "control" | "ctrl" => "Ctrl".to_string(),
            "alt" | "option" | "opt" => "Opt".to_string(),
            "shift" => "Shift".to_string(),
            "leftcommand" => "CmdLeft".to_string(),
            "rightcommand" => "CmdRight".to_string(),
            "leftcontrol" => "CtrlLeft".to_string(),
            "rightcontrol" => "CtrlRight".to_string(),
            "leftalt" | "leftoption" => "OptLeft".to_string(),
            "rightalt" | "rightoption" => "OptRight".to_string(),
            "leftshift" => "ShiftLeft".to_string(),
            "rightshift" => "ShiftRight".to_string(),
            // Older builds stored `Delete` for the forward-delete key.
            "delete" => "ForwardDelete".to_string(),
            "arrowleft" => "Left".to_string(),
            "arrowright" => "Right".to_string(),
            "arrowup" => "Up".to_string(),
            "arrowdown" => "Down".to_string(),
            "spacebar" => "Space".to_string(),
            _ => token.trim().to_string(),
        })
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>()
        .join("+")
}

pub(crate) fn shortcuts_conflict(left: &Hotkey, right: &Hotkey) -> bool {
    left == right || is_modifier_only_prefix(left, right) || is_modifier_only_prefix(right, left)
}

pub(crate) fn shortcut_bindings_conflict(left: &ShortcutBinding, right: &ShortcutBinding) -> bool {
    match (left, right) {
        (ShortcutBinding::Keyboard(left), ShortcutBinding::Keyboard(right)) => {
            shortcuts_conflict(left, right)
        }
        (ShortcutBinding::Mouse(left), ShortcutBinding::Mouse(right)) => left == right,
        (ShortcutBinding::Keyboard(_), ShortcutBinding::Mouse(_))
        | (ShortcutBinding::Mouse(_), ShortcutBinding::Keyboard(_)) => false,
    }
}

fn parse_mouse_shortcut(shortcut: &str) -> Option<MouseShortcut> {
    let normalized = shortcut
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-', '_'], "");
    match normalized.as_str() {
        "mousemiddle" | "middlemouse" | "middleclick" | "mouse3" | "mbutton" => {
            Some(MouseShortcut::Middle)
        }
        "mouseback" | "backmouse" | "mouse4" | "xbutton1" | "browserback" => {
            Some(MouseShortcut::Back)
        }
        "mouseforward" | "forwardmouse" | "mouse5" | "xbutton2" | "browserforward" => {
            Some(MouseShortcut::Forward)
        }
        _ => None,
    }
}

fn poll_mouse_bindings(
    app: &AppHandle<AppRuntime>,
    bindings: &[(MouseShortcut, ShortcutAction)],
    states: &mut [bool],
) {
    for (idx, (mouse, action)) in bindings.iter().enumerate() {
        let pressed = is_mouse_shortcut_pressed(*mouse);
        if states.get(idx).copied().unwrap_or(false) == pressed {
            continue;
        }

        if let Some(state) = states.get_mut(idx) {
            *state = pressed;
        }

        let state = if pressed {
            HotkeyState::Pressed
        } else {
            HotkeyState::Released
        };
        pill::handle_registered_hotkey_event(app, *action, state);
    }
}

#[cfg(target_os = "windows")]
fn is_mouse_shortcut_pressed(mouse: MouseShortcut) -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_MBUTTON, VK_XBUTTON1, VK_XBUTTON2,
    };

    let key = match mouse {
        MouseShortcut::Middle => VK_MBUTTON,
        MouseShortcut::Back => VK_XBUTTON1,
        MouseShortcut::Forward => VK_XBUTTON2,
    };

    unsafe { (GetAsyncKeyState(key.0 as i32) as u16 & 0x8000) != 0 }
}

#[cfg(not(target_os = "windows"))]
fn is_mouse_shortcut_pressed(_mouse: MouseShortcut) -> bool {
    false
}

fn is_modifier_only_prefix(prefix: &Hotkey, full: &Hotkey) -> bool {
    prefix.key.is_none()
        && !prefix.modifiers.is_empty()
        && modifier_group_subset(
            prefix.modifiers,
            full.modifiers,
            Modifiers::CMD_LEFT,
            Modifiers::CMD_RIGHT,
        )
        && modifier_group_subset(
            prefix.modifiers,
            full.modifiers,
            Modifiers::CTRL_LEFT,
            Modifiers::CTRL_RIGHT,
        )
        && modifier_group_subset(
            prefix.modifiers,
            full.modifiers,
            Modifiers::OPT_LEFT,
            Modifiers::OPT_RIGHT,
        )
        && modifier_group_subset(
            prefix.modifiers,
            full.modifiers,
            Modifiers::SHIFT_LEFT,
            Modifiers::SHIFT_RIGHT,
        )
        && (!prefix.modifiers.contains(Modifiers::FN) || full.modifiers.contains(Modifiers::FN))
        && (full.key.is_some() || prefix.modifiers != full.modifiers)
}

fn modifier_group_subset(
    prefix: Modifiers,
    full: Modifiers,
    left: Modifiers,
    right: Modifiers,
) -> bool {
    let prefix_has_left = prefix.contains(left);
    let prefix_has_right = prefix.contains(right);

    if !prefix_has_left && !prefix_has_right {
        return true;
    }

    let full_has_left = full.contains(left);
    let full_has_right = full.contains(right);

    if prefix_has_left && prefix_has_right {
        full_has_left || full_has_right
    } else if prefix_has_left {
        full_has_left
    } else {
        full_has_right
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mouse_shortcut_aliases() {
        assert_eq!(
            parse_shortcut_binding("MouseBack").unwrap(),
            ShortcutBinding::Mouse(MouseShortcut::Back)
        );
        assert_eq!(
            parse_shortcut_binding("mouse forward").unwrap(),
            ShortcutBinding::Mouse(MouseShortcut::Forward)
        );
        assert_eq!(
            parse_shortcut_binding("middle-click").unwrap(),
            ShortcutBinding::Mouse(MouseShortcut::Middle)
        );
    }

    #[test]
    fn mouse_shortcuts_conflict_only_with_same_button() {
        assert!(shortcut_bindings_conflict(
            &ShortcutBinding::Mouse(MouseShortcut::Back),
            &ShortcutBinding::Mouse(MouseShortcut::Back)
        ));
        assert!(!shortcut_bindings_conflict(
            &ShortcutBinding::Mouse(MouseShortcut::Back),
            &ShortcutBinding::Mouse(MouseShortcut::Forward)
        ));
    }
}
