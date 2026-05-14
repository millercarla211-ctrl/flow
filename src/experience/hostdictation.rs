use super::{
    accessibility::FlowAccessibilityRuntime,
    capture::CaptureWorkerStatus,
    health::{FlowHealthReport, FlowHealthSeverity},
    lifecycle::FlowRuntimeState,
    microphone::{MicrophoneMode, MicrophoneSnapshot},
    overlay::OverlayMode,
    session::FlowSessionContext,
    snooze::FlowHostPauseSnapshot,
    types::{AppContext, DictationAssistResult},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowHostDictationBlocker {
    EmptyTranscript,
    NonDesktopHost,
    HostPaused,
    AccessibilityUnavailable,
    MicrophoneNotConfigured,
    MicrophoneNotStreaming,
    CaptureNotRunning,
    OverlayNotInDictationMode,
    HealthCritical(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowHostDictationReadiness {
    pub ready: bool,
    pub blockers: Vec<FlowHostDictationBlocker>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowHostDictationRequest {
    pub transcript: String,
    pub app_context: AppContext,
    pub replace_focused_input: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowHostDictationExecution {
    pub raw_transcript: String,
    pub cleaned_text: String,
    pub inserted: bool,
    pub readiness: FlowHostDictationReadiness,
    pub dictation: Option<DictationAssistResult>,
    pub notes: Vec<String>,
}

impl FlowHostDictationRequest {
    pub fn new(transcript: impl Into<String>) -> Self {
        Self {
            transcript: transcript.into(),
            app_context: AppContext::default(),
            replace_focused_input: true,
        }
    }

    pub fn with_app_context(mut self, app_context: AppContext) -> Self {
        self.app_context = app_context;
        self
    }

    pub fn preview_only(mut self) -> Self {
        self.replace_focused_input = false;
        self
    }
}

impl FlowHostDictationReadiness {
    pub fn evaluate(
        context: &FlowSessionContext,
        health: &FlowHealthReport,
        accessibility: &FlowAccessibilityRuntime,
        microphone: &MicrophoneSnapshot,
        capture: &CaptureWorkerStatus,
        pause: &FlowHostPauseSnapshot,
        transcript: &str,
        replace_focused_input: bool,
    ) -> Self {
        let mut blockers = Vec::new();
        let mut notes = Vec::new();

        if transcript.trim().is_empty() {
            blockers.push(FlowHostDictationBlocker::EmptyTranscript);
        }

        if !context.os.is_desktop() {
            blockers.push(FlowHostDictationBlocker::NonDesktopHost);
        }

        if pause.active
            || matches!(
                context.lifecycle.state,
                FlowRuntimeState::Paused | FlowRuntimeState::Sleeping
            )
        {
            blockers.push(FlowHostDictationBlocker::HostPaused);
        }

        if replace_focused_input
            && (!accessibility.available || !accessibility.can_replace_selection)
        {
            blockers.push(FlowHostDictationBlocker::AccessibilityUnavailable);
        }

        if !microphone.configured {
            blockers.push(FlowHostDictationBlocker::MicrophoneNotConfigured);
        }

        if !matches!(microphone.mode, MicrophoneMode::Streaming) {
            blockers.push(FlowHostDictationBlocker::MicrophoneNotStreaming);
        }

        if !capture.running {
            blockers.push(FlowHostDictationBlocker::CaptureNotRunning);
        }

        if !matches!(context.overlay.mode, OverlayMode::Dictation) {
            blockers.push(FlowHostDictationBlocker::OverlayNotInDictationMode);
        }

        for issue in health
            .issues
            .iter()
            .filter(|issue| issue.severity == FlowHealthSeverity::Critical)
        {
            blockers.push(FlowHostDictationBlocker::HealthCritical(
                issue.title.clone(),
            ));
        }

        notes.push(format!(
            "trigger={}",
            context
                .lifecycle
                .last_trigger
                .as_deref()
                .unwrap_or("manual")
        ));
        notes.push(format!("stt={}", context.audio.dictation.stt_model_hint));
        notes.push(format!("vad={}", context.audio.dictation.vad_hint));
        notes.push(format!(
            "focused_input={}",
            if replace_focused_input {
                "replace"
            } else {
                "preview"
            }
        ));

        Self {
            ready: blockers.is_empty(),
            blockers,
            notes,
        }
    }
}
