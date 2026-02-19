use anyhow::Result;
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
    pub shortcut: String,
}

pub(crate) trait HotkeyProvider {
    fn unregister_all(&self) -> Result<()>;

    fn on_shortcut<F>(&self, shortcut: &str, handler: F) -> Result<()>
    where
        F: Fn(&AppHandle<AppRuntime>, HotkeyEvent) + Send + Sync + 'static;
}

struct TauriGlobalHotkeyProvider<'a> {
    app: &'a AppHandle<AppRuntime>,
}

impl<'a> TauriGlobalHotkeyProvider<'a> {
    fn new(app: &'a AppHandle<AppRuntime>) -> Self {
        Self { app }
    }
}

impl HotkeyProvider for TauriGlobalHotkeyProvider<'_> {
    fn unregister_all(&self) -> Result<()> {
        self.app.global_shortcut().unregister_all()?;
        Ok(())
    }

    fn on_shortcut<F>(&self, shortcut: &str, handler: F) -> Result<()>
    where
        F: Fn(&AppHandle<AppRuntime>, HotkeyEvent) + Send + Sync + 'static,
    {
        self.app
            .global_shortcut()
            .on_shortcut(shortcut, move |app, pressed_shortcut, event| {
                let state = match event.state {
                    ShortcutState::Pressed => HotkeyState::Pressed,
                    ShortcutState::Released => HotkeyState::Released,
                };
                handler(
                    app,
                    HotkeyEvent {
                        state,
                        shortcut: pressed_shortcut.to_string(),
                    },
                );
            })?;
        Ok(())
    }
}

pub(crate) fn provider(app: &AppHandle<AppRuntime>) -> impl HotkeyProvider + '_ {
    TauriGlobalHotkeyProvider::new(app)
}
