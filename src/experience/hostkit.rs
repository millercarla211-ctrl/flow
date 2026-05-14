use super::{
    FlowExperienceHub, FlowHostSnapshot,
    accessibility::FlowAccessibilityRuntime,
    bundle::FlowHostBundle,
    capture::{CpalCaptureWorker, FlowCaptureWorker},
    consent::{FlowConsentPlan, FlowConsentPlanner},
    contracts::{
        FlowAutomationBridge, FlowModuleInstaller, FlowStateStore, MemoryPermissionGate,
        RecordingModuleInstaller,
    },
    control::ControlCapability,
    dictation::FlowDictationEngine,
    engine::{FlowCommandExecution, FlowTextExecution, FlowTierRefreshReport},
    executors::NativeControlExecutor,
    health::FlowHealthReport,
    hostdictation::{
        FlowHostDictationExecution, FlowHostDictationReadiness, FlowHostDictationRequest,
    },
    microphone::{FlowMicrophoneService, ManagedMicrophoneService},
    persistence::FlowPersistentState,
    presenters::{ManagedAudioRuntime, NativeOverlayPresenter},
    recovery::{FlowRecoveryPlan, RecoveryEvent},
    runtime_policy::DeviceBenchmarkSnapshot,
    selection::NativeSelectionBridge,
    session::FlowSessionContext,
    snooze::{FlowHostPauseController, FlowHostPauseSnapshot},
    stores::FlowFileStateStore,
    types::{DictationAssistRequest, TypingAssistRequest},
    wake::{FlowWakeRuntime, ManagedWakeRuntime, WakeRuntimeState},
    wakedetect::{FlowWakeInferenceWorker, OpenWakeInferenceWorker},
};

#[derive(Debug, Clone, PartialEq)]
pub struct FlowDefaultHostKit {
    pub snapshot: FlowHostSnapshot,
    pub bundle: FlowHostBundle,
    pub installer: RecordingModuleInstaller,
    pub store: FlowFileStateStore,
    pub permissions: MemoryPermissionGate,
    pub executor: NativeControlExecutor,
    pub overlay: NativeOverlayPresenter,
    pub audio: ManagedAudioRuntime,
    pub capture: CpalCaptureWorker,
    pub microphone: ManagedMicrophoneService,
    pub accessibility: FlowAccessibilityRuntime,
    pub automation: NativeSelectionBridge,
    pub wake: ManagedWakeRuntime,
    pub wake_worker: OpenWakeInferenceWorker,
    pub pause: FlowHostPauseController,
}

impl FlowDefaultHostKit {
    pub fn new(
        snapshot: FlowHostSnapshot,
        hub: FlowExperienceHub,
        state_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        let bundle = FlowHostBundle::for_host(&snapshot, hub);
        let accessibility = FlowAccessibilityRuntime::dry_run(snapshot.os.clone());
        Self {
            snapshot: snapshot.clone(),
            bundle,
            installer: RecordingModuleInstaller::default(),
            store: FlowFileStateStore::new(state_path),
            permissions: MemoryPermissionGate::default(),
            executor: NativeControlExecutor::new(snapshot.os.clone()),
            overlay: NativeOverlayPresenter::new(snapshot.os.clone()),
            audio: ManagedAudioRuntime::default(),
            capture: CpalCaptureWorker::new(),
            microphone: ManagedMicrophoneService::default(),
            accessibility: accessibility.clone(),
            automation: NativeSelectionBridge::with_accessibility(accessibility, true),
            wake: ManagedWakeRuntime::default(),
            wake_worker: OpenWakeInferenceWorker::default(),
            pause: FlowHostPauseController::default(),
        }
    }

    pub fn live(
        snapshot: FlowHostSnapshot,
        hub: FlowExperienceHub,
        state_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        let bundle = FlowHostBundle::for_host(&snapshot, hub);
        let accessibility = FlowAccessibilityRuntime::live(snapshot.os.clone());
        Self {
            snapshot: snapshot.clone(),
            bundle,
            installer: RecordingModuleInstaller::default(),
            store: FlowFileStateStore::new(state_path),
            permissions: MemoryPermissionGate::default(),
            executor: NativeControlExecutor::live(snapshot.os.clone()),
            overlay: NativeOverlayPresenter::live(snapshot.os.clone()),
            audio: ManagedAudioRuntime::default(),
            capture: CpalCaptureWorker::live(),
            microphone: ManagedMicrophoneService::default(),
            accessibility: accessibility.clone(),
            automation: NativeSelectionBridge::with_accessibility(accessibility, false),
            wake: ManagedWakeRuntime::default(),
            wake_worker: OpenWakeInferenceWorker::default(),
            pause: FlowHostPauseController::default(),
        }
    }

    pub fn bootstrap(&mut self) -> FlowSessionContext {
        let report = self
            .bundle
            .bootstrap(&self.snapshot, &mut self.installer, &mut self.store);
        let context = report.context;
        self.bundle
            .sync_presenters(&context, &mut self.overlay, &mut self.audio);
        self.capture.configure(&context.audio);
        self.capture.start();
        self.microphone.configure(&context.audio);
        self.microphone.arm();
        self.wake.configure(&context.activation, &context.audio);
        self.wake_worker.configure(&context.activation);
        self.wake_worker.arm();
        self.wake.sync_lifecycle(&context.lifecycle.state);
        if let Some(state) = self.store.load_state() {
            let _ = self.restore_permissions(&state);
        }
        context
    }

    pub fn process_text(
        &mut self,
        context: &mut FlowSessionContext,
        request: TypingAssistRequest,
    ) -> FlowTextExecution {
        let execution = self.bundle.engine.process_text(
            context,
            request,
            &mut self.permissions,
            &mut self.executor,
        );
        self.sync(context);
        execution
    }

    pub fn process_command(
        &mut self,
        context: &mut FlowSessionContext,
        transcript: impl Into<String>,
    ) -> FlowCommandExecution {
        let execution = self.bundle.engine.process_command(
            context,
            transcript,
            &mut self.permissions,
            &mut self.executor,
        );
        self.sync(context);
        execution
    }

    pub fn rewrite_selection(
        &mut self,
        context: &mut FlowSessionContext,
    ) -> Option<super::automation::FlowSelectionExecution> {
        let execution =
            self.bundle
                .rewrite_selection(context, &mut self.permissions, &mut self.automation);
        self.sync(context);
        execution
    }

    pub fn dictate_to_focused_input(
        &mut self,
        context: &mut FlowSessionContext,
        transcript: impl Into<String>,
    ) -> FlowHostDictationExecution {
        self.dictate_request_to_focused_input(context, FlowHostDictationRequest::new(transcript))
    }

    pub fn dictate_request_to_focused_input(
        &mut self,
        context: &mut FlowSessionContext,
        request: FlowHostDictationRequest,
    ) -> FlowHostDictationExecution {
        let raw_transcript = request.transcript.clone();
        let mut notes = Vec::new();
        let host_was_paused = self.pause.is_active()
            || matches!(
                context.lifecycle.state,
                super::lifecycle::FlowRuntimeState::Paused
                    | super::lifecycle::FlowRuntimeState::Sleeping
            );
        let started_here = !host_was_paused
            && !matches!(
                context.lifecycle.state,
                super::lifecycle::FlowRuntimeState::Dictating
            );

        if started_here {
            let _ = self.advance(
                context,
                super::lifecycle::FlowRuntimeEvent::HoldToDictateStarted,
            );
        }

        let health = self.health_report(context);
        let readiness = FlowHostDictationReadiness::evaluate(
            context,
            &health,
            &self.accessibility,
            &self.microphone.snapshot(),
            &self.capture.status(),
            &self.pause.snapshot(),
            &request.transcript,
            request.replace_focused_input,
        );

        let mut cleaned_text = String::new();
        let mut dictation = None;
        let mut inserted = false;

        if readiness.ready {
            match FlowDictationEngine::new().process(DictationAssistRequest {
                transcript: request.transcript.clone(),
                app_context: request.app_context.clone(),
                dictionary: self.bundle.engine.session.hub.dictionary_for_context(),
                snippets: self.bundle.engine.session.hub.snippets_for_context(),
                styles: self
                    .bundle
                    .engine
                    .session
                    .hub
                    .styles_for_context(&request.app_context),
                remove_fillers: true,
                auto_punctuate: context.audio.dictation.punctuation,
                format_lists: true,
                tag_workspace_files: true,
            }) {
                Ok(result) => {
                    cleaned_text = result.cleaned_text.clone();
                    if request.replace_focused_input {
                        inserted = if cleaned_text.trim().is_empty() {
                            notes.push(
                                "Dictation cleanup produced empty text; focused input was not changed."
                                    .to_string(),
                            );
                            false
                        } else {
                            self.automation.replace_selection(&cleaned_text)
                        };
                        context.audit.record(
                            ControlCapability::ReplaceSelection,
                            format!("{:?}", context.control.surface),
                            "Dictate cleaned speech into focused input.",
                            inserted,
                        );
                    }
                    dictation = Some(result);
                }
                Err(error) => {
                    notes.push(format!("Dictation cleanup failed: {error}"));
                    if request.replace_focused_input {
                        context.audit.record(
                            ControlCapability::ReplaceSelection,
                            format!("{:?}", context.control.surface),
                            "Dictation cleanup failed before focused-input replacement.",
                            false,
                        );
                    }
                }
            }
        } else if request.replace_focused_input {
            context.audit.record(
                ControlCapability::ReplaceSelection,
                format!("{:?}", context.control.surface),
                "Dictation host readiness blocked focused-input replacement.",
                false,
            );
        }

        if started_here {
            let _ = self.advance(
                context,
                super::lifecycle::FlowRuntimeEvent::HoldToDictateReleased,
            );
        } else {
            self.sync(context);
        }

        let mut execution_notes = readiness.notes.clone();
        execution_notes.extend(notes);

        FlowHostDictationExecution {
            raw_transcript,
            cleaned_text,
            inserted,
            readiness,
            dictation,
            notes: execution_notes,
        }
    }

    pub fn refresh_runtime(
        &mut self,
        context: &mut FlowSessionContext,
        benchmark: DeviceBenchmarkSnapshot,
    ) -> Option<FlowTierRefreshReport> {
        let report = self.bundle.engine.refresh_runtime(
            context,
            benchmark,
            &mut self.installer,
            &mut self.store,
        );
        self.sync(context);
        report
    }

    pub fn advance(
        &mut self,
        context: &mut FlowSessionContext,
        event: super::lifecycle::FlowRuntimeEvent,
    ) -> super::lifecycle::FlowLifecycleSnapshot {
        let snapshot = self.bundle.advance(context, event);
        self.sync(context);
        snapshot
    }

    pub fn pause_host(
        &mut self,
        context: &mut FlowSessionContext,
        reason: impl Into<String>,
    ) -> FlowHostPauseSnapshot {
        let snapshot = self.pause.pause(reason);
        let _ = self.advance(context, super::lifecycle::FlowRuntimeEvent::PauseRequested);
        snapshot
    }

    pub fn snooze_host(
        &mut self,
        context: &mut FlowSessionContext,
        duration: std::time::Duration,
        reason: impl Into<String>,
    ) -> FlowHostPauseSnapshot {
        let snapshot = self.pause.snooze(duration, reason);
        let _ = self.advance(context, super::lifecycle::FlowRuntimeEvent::PauseRequested);
        snapshot
    }

    pub fn resume_host(&mut self, context: &mut FlowSessionContext) -> FlowHostPauseSnapshot {
        let snapshot = self.pause.resume();
        let _ = self.advance(context, super::lifecycle::FlowRuntimeEvent::ResumeRequested);
        snapshot
    }

    pub fn refresh_pause(&mut self, context: &mut FlowSessionContext) -> FlowHostPauseSnapshot {
        if self.pause.refresh() {
            let _ = self.advance(context, super::lifecycle::FlowRuntimeEvent::ResumeRequested);
        } else if self.pause.is_active() {
            self.sync(context);
        }

        self.pause.snapshot()
    }

    pub fn pause_snapshot(&self) -> FlowHostPauseSnapshot {
        self.pause.snapshot()
    }

    pub fn recover(
        &mut self,
        context: &mut FlowSessionContext,
        event: RecoveryEvent,
    ) -> FlowRecoveryPlan {
        let plan = self.bundle.recovery_plan(event.clone());

        for action in &plan.actions {
            if action.persist_state {
                self.persist(context);
            }
            if action.reload_modules {
                let required: Vec<_> = context
                    .install_state
                    .installed_required_modules()
                    .into_iter()
                    .cloned()
                    .collect();
                let receipts = self.installer.install_modules(&required);
                context.install_state.apply_install_receipts(&receipts);
            }
            if action.restart_audio || action.reset_overlay {
                if action.restart_audio {
                    self.microphone.restart();
                }
                self.sync(context);
            }
        }

        match event {
            RecoveryEvent::Suspend
            | RecoveryEvent::ThermalPause
            | RecoveryEvent::BatteryFallback
            | RecoveryEvent::MicrophoneLost => {
                let _ = self.advance(context, super::lifecycle::FlowRuntimeEvent::PauseRequested);
            }
            RecoveryEvent::Resume | RecoveryEvent::MicrophoneRestored => {
                let _ = self.advance(context, super::lifecycle::FlowRuntimeEvent::ResumeRequested);
            }
            RecoveryEvent::RuntimeCrash => {
                let _ = self.advance(context, super::lifecycle::FlowRuntimeEvent::BootCompleted);
            }
        }

        plan
    }

    pub fn sync(&mut self, context: &FlowSessionContext) {
        self.bundle
            .sync_presenters(context, &mut self.overlay, &mut self.audio);
        self.capture.configure(&context.audio);
        self.microphone.configure(&context.audio);
        match context.lifecycle.state {
            super::lifecycle::FlowRuntimeState::Listening
            | super::lifecycle::FlowRuntimeState::Overlay
            | super::lifecycle::FlowRuntimeState::CommandMode => {
                self.microphone.arm();
                self.capture.start();
                self.wake_worker.arm();
            }
            super::lifecycle::FlowRuntimeState::Dictating => {
                self.microphone.stream();
                self.capture.start();
                self.wake_worker.arm();
            }
            super::lifecycle::FlowRuntimeState::Sleeping
            | super::lifecycle::FlowRuntimeState::Paused
            | super::lifecycle::FlowRuntimeState::ColdBoot => {
                self.microphone.pause();
                self.capture.stop();
                self.wake_worker.disarm();
            }
        }
        self.wake.configure(&context.activation, &context.audio);
        self.wake_worker.configure(&context.activation);
        self.wake.sync_lifecycle(&context.lifecycle.state);
        self.persist(context);
    }

    pub fn note_wake_detection(
        &mut self,
        context: &mut FlowSessionContext,
        phrase: impl Into<String>,
    ) -> WakeRuntimeState {
        let phrase = phrase.into();
        let detected = self.wake_worker.evaluate_phrase(&phrase).unwrap_or(phrase);
        self.wake.note_detection(detected.clone());
        let _ = self.advance(
            context,
            super::lifecycle::FlowRuntimeEvent::WakeWordDetected(detected),
        );
        self.wake.snapshot()
    }

    pub fn feed_audio_frame(
        &mut self,
        context: &mut FlowSessionContext,
        samples: &[f32],
    ) -> Option<WakeRuntimeState> {
        let capture_report = self.capture.feed_samples(samples);
        let detected = self
            .wake_worker
            .evaluate_audio_frame(samples, capture_report.speech_detected)?;
        self.wake.note_detection(detected.clone());
        let _ = self.advance(
            context,
            super::lifecycle::FlowRuntimeEvent::WakeWordDetected(detected),
        );
        Some(self.wake.snapshot())
    }

    pub fn health_report(&self, context: &FlowSessionContext) -> FlowHealthReport {
        FlowHealthReport::evaluate(self, context)
    }

    pub fn live_consent_plan(&self, context: &FlowSessionContext) -> FlowConsentPlan {
        FlowConsentPlanner::for_live_host(self, context)
    }

    fn persist(&mut self, context: &FlowSessionContext) {
        let state = FlowPersistentState::from_runtime(
            &context.install_state,
            &context.audit,
            self.bundle.engine.benchmark_history.clone(),
        );
        self.store.save_state(state);
    }

    fn restore_permissions(&mut self, state: &FlowPersistentState) -> usize {
        let mut restored = 0;
        for approval in &state.approvals {
            if approval.granted {
                if let Some(capability) = parse_capability(&approval.capability) {
                    self.permissions.grant(capability);
                    restored += 1;
                }
            }
        }
        restored
    }
}

fn parse_capability(value: &str) -> Option<super::control::ControlCapability> {
    Some(match value {
        "ReadClipboard" => super::control::ControlCapability::ReadClipboard,
        "WriteClipboard" => super::control::ControlCapability::WriteClipboard,
        "ReadSelection" => super::control::ControlCapability::ReadSelection,
        "ReplaceSelection" => super::control::ControlCapability::ReplaceSelection,
        "SimulateShortcut" => super::control::ControlCapability::SimulateShortcut,
        "OpenUrl" => super::control::ControlCapability::OpenUrl,
        "OpenApplication" => super::control::ControlCapability::OpenApplication,
        "OpenFile" => super::control::ControlCapability::OpenFile,
        "RevealFile" => super::control::ControlCapability::RevealFile,
        "CreateDraftFile" => super::control::ControlCapability::CreateDraftFile,
        "FocusWindow" => super::control::ControlCapability::FocusWindow,
        "MediaPlayback" => super::control::ControlCapability::MediaPlayback,
        "VolumeControl" => super::control::ControlCapability::VolumeControl,
        "BrightnessControl" => super::control::ControlCapability::BrightnessControl,
        "SystemSearch" => super::control::ControlCapability::SystemSearch,
        "Notification" => super::control::ControlCapability::Notification,
        "ShellCommand" => super::control::ControlCapability::ShellCommand,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experience::{
        AccessibilityBackend, AccessibilityMode, FlowAccessibilityRuntime,
        FlowHostDictationBlocker, FlowRuntimeState, MicrophoneMode, OperatingSystemFamily,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_state_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "{name}_{}.txt",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ))
    }

    fn ready_windows_accessibility() -> FlowAccessibilityRuntime {
        FlowAccessibilityRuntime {
            os: OperatingSystemFamily::Windows,
            backend: AccessibilityBackend::UiAutomation,
            mode: AccessibilityMode::Full,
            can_read_selection: true,
            can_replace_selection: true,
            can_send_shortcuts: true,
            availability_checked: true,
            available: true,
            notes: Vec::new(),
        }
    }

    #[test]
    fn host_pause_and_resume_updates_runtime_services() {
        let state_path = unique_state_path("flow_host_pause_test_state");
        let _ = std::fs::remove_file(&state_path);
        let snapshot =
            FlowHostSnapshot::new(OperatingSystemFamily::Windows, "test-host", 8.0, None, true);
        let hub = FlowExperienceHub::new("test");
        let mut host = FlowDefaultHostKit::new(snapshot, hub, &state_path);
        let mut context = host.bootstrap();

        let pause = host.pause_host(&mut context, "operator break");
        assert!(pause.active);
        assert!(matches!(
            context.lifecycle.state,
            FlowRuntimeState::Sleeping
        ));
        assert!(matches!(
            host.microphone.snapshot().mode,
            MicrophoneMode::Paused
        ));
        assert!(!host.capture.status().running);

        let resumed = host.resume_host(&mut context);
        assert!(!resumed.active);
        assert!(matches!(
            context.lifecycle.state,
            FlowRuntimeState::Listening
        ));
        assert!(matches!(
            host.microphone.snapshot().mode,
            MicrophoneMode::Armed
        ));
        assert!(host.capture.status().running);

        let _ = std::fs::remove_file(&state_path);
    }

    #[test]
    fn host_dictation_replaces_focused_input_when_ready() {
        let state_path = unique_state_path("flow_host_dictation_ready_state");
        let _ = std::fs::remove_file(&state_path);
        let snapshot =
            FlowHostSnapshot::new(OperatingSystemFamily::Windows, "test-host", 8.0, None, true);
        let hub = FlowExperienceHub::new("test");
        let mut host = FlowDefaultHostKit::new(snapshot, hub, &state_path);
        host.accessibility = ready_windows_accessibility();
        host.automation =
            NativeSelectionBridge::with_accessibility(host.accessibility.clone(), true);
        let mut context = host.bootstrap();

        let execution = host.dictate_to_focused_input(&mut context, "hello world");

        assert!(execution.readiness.ready);
        assert!(execution.inserted);
        assert_eq!(execution.cleaned_text, "Hello world.");
        assert!(matches!(
            context.lifecycle.state,
            FlowRuntimeState::Listening
        ));
        assert!(matches!(
            host.microphone.snapshot().mode,
            MicrophoneMode::Armed
        ));
        assert!(context.audit.entries().iter().any(|entry| {
            entry.capability == ControlCapability::ReplaceSelection
                && entry.description == "Dictate cleaned speech into focused input."
                && entry.approved
        }));

        let _ = std::fs::remove_file(&state_path);
    }

    #[test]
    fn host_dictation_blocks_while_paused() {
        let state_path = unique_state_path("flow_host_dictation_paused_state");
        let _ = std::fs::remove_file(&state_path);
        let snapshot =
            FlowHostSnapshot::new(OperatingSystemFamily::Windows, "test-host", 8.0, None, true);
        let hub = FlowExperienceHub::new("test");
        let mut host = FlowDefaultHostKit::new(snapshot, hub, &state_path);
        host.accessibility = ready_windows_accessibility();
        host.automation =
            NativeSelectionBridge::with_accessibility(host.accessibility.clone(), true);
        let mut context = host.bootstrap();

        host.pause_host(&mut context, "operator break");
        let execution = host.dictate_to_focused_input(&mut context, "hello world");

        assert!(!execution.readiness.ready);
        assert!(!execution.inserted);
        assert!(execution.dictation.is_none());
        assert!(
            execution
                .readiness
                .blockers
                .contains(&FlowHostDictationBlocker::HostPaused)
        );
        assert!(context.audit.entries().iter().any(|entry| {
            entry.capability == ControlCapability::ReplaceSelection
                && entry.description
                    == "Dictation host readiness blocked focused-input replacement."
                && !entry.approved
        }));

        let _ = std::fs::remove_file(&state_path);
    }
}
