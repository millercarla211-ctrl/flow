use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use crossbeam_channel::{bounded, Sender};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    dictionary, model_manager, pill::PillStatus, settings::UserSettings, wake_speaker, AppRuntime,
    AppState,
};

const EVENT_WAKE_STATUS: &str = "wake:status";
const WAKE_LEVEL_BUFFER: usize = 8;
const WAKE_MIN_TRIGGER: f32 = 0.0022;
const WAKE_TRIGGER_MULTIPLIER: f32 = 4.5;
const WAKE_TRIGGER_FRAMES: u8 = 4;
const WAKE_COOLDOWN: Duration = Duration::from_millis(2500);
const WAKE_PREROLL: Duration = Duration::from_millis(320);
const WAKE_CANDIDATE_MIN: Duration = Duration::from_millis(220);
const WAKE_CANDIDATE_SILENCE: Duration = Duration::from_millis(560);
const WAKE_CANDIDATE_MAX: Duration = Duration::from_millis(2600);
const WAKE_AUTOSTOP_MIN: Duration = Duration::from_millis(2500);
const WAKE_AUTOSTOP_SILENCE: Duration = Duration::from_millis(1800);
const WAKE_AUTOSTOP_MAX: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
struct WakeConfig {
    device_id: Option<String>,
    phrases: Vec<String>,
}

#[derive(Debug)]
struct WakeAudioFrame {
    level: f32,
    samples: Vec<i16>,
    sample_rate: u32,
}

#[derive(Debug)]
struct WakeCandidate {
    samples: Vec<i16>,
    sample_rate: u32,
    started_at: Instant,
    quiet_for: Duration,
}

#[derive(Debug, Clone)]
struct WakeMatch {
    command: String,
    transcript: String,
    confidence: f64,
    speaker_score: Option<f32>,
}

#[derive(Default)]
pub(crate) struct WakeCoordinator {
    session: Mutex<Option<WakeSession>>,
}

impl WakeCoordinator {
    pub(crate) fn sync(&self, app: &AppHandle<AppRuntime>, settings: &UserSettings) -> Result<()> {
        if !settings.wake_listening_enabled {
            self.stop();
            emit_wake_status(app, false, "Wake listener off");
            return Ok(());
        }

        let config = WakeConfig {
            device_id: settings.microphone_device.clone(),
            phrases: settings.wake_phrases.clone(),
        };

        if self
            .session
            .lock()
            .as_ref()
            .is_some_and(|session| session.config == config)
        {
            return Ok(());
        }

        self.stop();
        let session = WakeSession::start(app.clone(), config)?;
        *self.session.lock() = Some(session);
        emit_wake_status(app, true, "Wake listener armed");
        Ok(())
    }

    pub(crate) fn stop(&self) {
        self.session.lock().take();
    }
}

struct WakeSession {
    config: WakeConfig,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl WakeSession {
    fn start(app: AppHandle<AppRuntime>, config: WakeConfig) -> Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_signal = Arc::clone(&stop);
        let thread_config = config.clone();
        let handle = thread::Builder::new()
            .name("flow-wake-listener".to_string())
            .spawn(move || {
                if let Err(err) = run_wake_listener(app, thread_config, stop_signal) {
                    eprintln!("Wake listener stopped: {err}");
                }
            })
            .map_err(|err| anyhow!("Failed to spawn wake listener: {err}"))?;

        Ok(Self {
            config,
            stop,
            handle: Some(handle),
        })
    }
}

impl Drop for WakeSession {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_wake_listener(
    app: AppHandle<AppRuntime>,
    config: WakeConfig,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    let (frame_tx, frame_rx) = bounded::<WakeAudioFrame>(WAKE_LEVEL_BUFFER);
    let stream = build_wake_stream(config.device_id.as_deref(), frame_tx)?;
    stream
        .play()
        .context("Failed to start wake microphone stream")?;

    let mut noise_floor = 0.0008f32;
    let mut voiced_frames = 0u8;
    let mut last_trigger = Instant::now() - WAKE_COOLDOWN;
    let mut pre_roll = Vec::<i16>::new();
    let mut candidate: Option<WakeCandidate> = None;

    while !stop.load(Ordering::Relaxed) {
        let Ok(frame) = frame_rx.recv_timeout(Duration::from_millis(100)) else {
            continue;
        };

        let level = frame.level;
        let trigger = (noise_floor * WAKE_TRIGGER_MULTIPLIER).max(WAKE_MIN_TRIGGER);
        let frame_duration = duration_for_samples(frame.samples.len(), frame.sample_rate);

        if let Some(active) = candidate.as_mut() {
            active.samples.extend_from_slice(&frame.samples);
            if level >= trigger * 0.62 {
                active.quiet_for = Duration::ZERO;
            } else {
                active.quiet_for = active.quiet_for.saturating_add(frame_duration);
            }

            let elapsed = active.started_at.elapsed();
            let should_finish = (elapsed >= WAKE_CANDIDATE_MIN
                && active.quiet_for >= WAKE_CANDIDATE_SILENCE)
                || elapsed >= WAKE_CANDIDATE_MAX;

            if should_finish {
                if let Some(done) = candidate.take() {
                    if last_trigger.elapsed() >= WAKE_COOLDOWN {
                        if let Some(wake_match) =
                            transcribe_and_match_wake_candidate(&app, &config, &done)
                        {
                            last_trigger = Instant::now();
                            voiced_frames = 0;
                            pre_roll.clear();
                            trigger_recording_from_wake(&app, &wake_match);
                        }
                    }
                }
            }
            continue;
        }

        if level < trigger {
            noise_floor = (noise_floor * 0.985 + level * 0.015).clamp(0.00005, 0.04);
            voiced_frames = 0;
            push_preroll(&mut pre_roll, &frame.samples, frame.sample_rate);
            continue;
        }

        voiced_frames = voiced_frames.saturating_add(1);
        if voiced_frames < WAKE_TRIGGER_FRAMES || last_trigger.elapsed() < WAKE_COOLDOWN {
            push_preroll(&mut pre_roll, &frame.samples, frame.sample_rate);
            continue;
        }

        voiced_frames = 0;
        if app.state::<AppState>().pill().status() == PillStatus::Idle {
            let mut samples = pre_roll.clone();
            samples.extend_from_slice(&frame.samples);
            candidate = Some(WakeCandidate {
                samples,
                sample_rate: frame.sample_rate,
                started_at: Instant::now(),
                quiet_for: Duration::ZERO,
            });
        }
        pre_roll.clear();
    }

    drop(stream);
    Ok(())
}

fn trigger_recording_from_wake(app: &AppHandle<AppRuntime>, wake_match: &WakeMatch) {
    let state = app.state::<AppState>();
    if state.pill().status() != PillStatus::Idle {
        return;
    }

    if let Err(err) = app.emit(
        EVENT_WAKE_STATUS,
        WakeStatusPayload {
            active: true,
            message: format!("Wake heard: {}", wake_match.command),
            command: Some(wake_match.command.clone()),
            transcript: Some(wake_match.transcript.clone()),
            confidence: Some(wake_match.confidence),
            speaker_score: wake_match.speaker_score,
        },
    ) {
        eprintln!("Failed to emit wake status: {err}");
    }

    crate::pill::show_overlay(app);
    state.pill().toggle_from_overlay(app);
    spawn_auto_stop(app.clone());
}

fn spawn_auto_stop(app: AppHandle<AppRuntime>) {
    thread::spawn(move || {
        let started = Instant::now();
        let mut offset = 0usize;
        let mut quiet_for = Duration::ZERO;
        let mut last_tick = Instant::now();

        loop {
            thread::sleep(Duration::from_millis(100));
            let now = Instant::now();
            let elapsed_tick = now.saturating_duration_since(last_tick);
            last_tick = now;

            let state = app.state::<AppState>();
            if state.pill().status() != PillStatus::Listening {
                break;
            }

            let Some((samples, _sample_rate, next_offset)) =
                state.pill().recorder().read_live_samples(offset)
            else {
                continue;
            };
            offset = next_offset;

            let rms = rms_from_f32(&samples);
            if rms > WAKE_MIN_TRIGGER * 1.5 {
                quiet_for = Duration::ZERO;
            } else {
                quiet_for += elapsed_tick;
            }

            if started.elapsed() > WAKE_AUTOSTOP_MIN && quiet_for >= WAKE_AUTOSTOP_SILENCE {
                state.pill().toggle_from_overlay(&app);
                break;
            }

            if started.elapsed() >= WAKE_AUTOSTOP_MAX {
                state.pill().toggle_from_overlay(&app);
                break;
            }
        }
    });
}

fn build_wake_stream(device_id: Option<&str>, frame_tx: Sender<WakeAudioFrame>) -> Result<Stream> {
    let host = cpal::default_host();
    let device = if let Some(selected) = device_id {
        selected
            .parse::<cpal::DeviceId>()
            .ok()
            .and_then(|parsed| host.device_by_id(&parsed))
            .or_else(|| {
                host.input_devices().ok()?.find(|device| {
                    device
                        .id()
                        .map(|id| id.to_string() == selected)
                        .unwrap_or(false)
                        || device
                            .description()
                            .map(|desc| desc.name() == selected)
                            .unwrap_or(false)
                })
            })
            .or_else(|| host.default_input_device())
            .context("Selected wake device not found and no default microphone available")?
    } else {
        host.default_input_device()
            .context("No default microphone available for wake listener")?
    };

    let config = device
        .default_input_config()
        .context("No supported wake input configuration found")?;
    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();
    let channels = stream_config.channels as usize;
    let sample_rate = stream_config.sample_rate;
    let err_fn = |err| eprintln!("Wake microphone stream error: {err}");

    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| publish_f32_frame(data, channels, sample_rate, &frame_tx),
            err_fn,
            None,
        )?,
        SampleFormat::I16 => device.build_input_stream(
            &stream_config,
            move |data: &[i16], _| publish_i16_frame(data, channels, sample_rate, &frame_tx),
            err_fn,
            None,
        )?,
        SampleFormat::U16 => device.build_input_stream(
            &stream_config,
            move |data: &[u16], _| publish_u16_frame(data, channels, sample_rate, &frame_tx),
            err_fn,
            None,
        )?,
        _ => return Err(anyhow!("Unsupported wake input sample format")),
    };

    Ok(stream)
}

fn publish_frame(samples: Vec<i16>, sample_rate: u32, frame_tx: &Sender<WakeAudioFrame>) {
    let level = rms_from_i16(&samples);
    let _ = frame_tx.try_send(WakeAudioFrame {
        level,
        samples,
        sample_rate,
    });
}

fn publish_f32_frame(
    data: &[f32],
    channels: usize,
    sample_rate: u32,
    frame_tx: &Sender<WakeAudioFrame>,
) {
    let samples: Vec<i16> = if channels <= 1 {
        data.iter().map(|sample| f32_to_i16(*sample)).collect()
    } else {
        data.chunks(channels)
            .map(|frame| f32_to_i16(frame.iter().copied().sum::<f32>() / channels as f32))
            .collect()
    };
    publish_frame(samples, sample_rate, frame_tx);
}

fn publish_i16_frame(
    data: &[i16],
    channels: usize,
    sample_rate: u32,
    frame_tx: &Sender<WakeAudioFrame>,
) {
    let samples: Vec<i16> = if channels <= 1 {
        data.to_vec()
    } else {
        data.chunks(channels)
            .map(|frame| {
                (frame.iter().map(|sample| *sample as i32).sum::<i32>() / channels as i32) as i16
            })
            .collect()
    };
    publish_frame(samples, sample_rate, frame_tx);
}

fn publish_u16_frame(
    data: &[u16],
    channels: usize,
    sample_rate: u32,
    frame_tx: &Sender<WakeAudioFrame>,
) {
    let samples: Vec<i16> = if channels <= 1 {
        data.iter().map(|sample| u16_to_i16(*sample)).collect()
    } else {
        data.chunks(channels)
            .map(|frame| {
                let sum = frame
                    .iter()
                    .map(|sample| u16_to_i16(*sample) as i32)
                    .sum::<i32>();
                (sum / channels as i32) as i16
            })
            .collect()
    };
    publish_frame(samples, sample_rate, frame_tx);
}

fn f32_to_i16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16
}

fn u16_to_i16(sample: u16) -> i16 {
    (sample as i32 - 32768).clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

fn rms_from_f32(data: &[f32]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }

    let sum = data.iter().map(|sample| sample * sample).sum::<f32>();
    (sum / data.len() as f32).sqrt()
}

fn rms_from_i16(data: &[i16]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let sum = data
        .iter()
        .map(|sample| {
            let normalized = *sample as f32 / i16::MAX as f32;
            normalized * normalized
        })
        .sum::<f32>();
    (sum / data.len() as f32).sqrt()
}

fn duration_for_samples(sample_count: usize, sample_rate: u32) -> Duration {
    if sample_rate == 0 {
        return Duration::ZERO;
    }
    Duration::from_secs_f32(sample_count as f32 / sample_rate as f32)
}

fn push_preroll(pre_roll: &mut Vec<i16>, frame: &[i16], sample_rate: u32) {
    pre_roll.extend_from_slice(frame);
    let max_samples = (sample_rate as f32 * WAKE_PREROLL.as_secs_f32()).round() as usize;
    if max_samples > 0 && pre_roll.len() > max_samples {
        let excess = pre_roll.len() - max_samples;
        pre_roll.drain(0..excess);
    }
}

fn transcribe_and_match_wake_candidate(
    app: &AppHandle<AppRuntime>,
    config: &WakeConfig,
    candidate: &WakeCandidate,
) -> Option<WakeMatch> {
    if candidate.samples.len() < (candidate.sample_rate as usize / 10).max(1) {
        return None;
    }

    let state = app.state::<AppState>();
    if state.pill().status() != PillStatus::Idle {
        return None;
    }

    let settings = state.current_settings();
    if !settings.wake_listening_enabled {
        return None;
    }

    let ready_model = match model_manager::ensure_model_ready(app, &settings.local_model) {
        Ok(model) => model,
        Err(err) => {
            emit_wake_status(
                app,
                false,
                &format!("Wake listener needs a local STT model: {err}"),
            );
            return None;
        }
    };

    let mut dictionary_terms = dictionary::dictionary_entries_for_model(&ready_model, &settings);
    for phrase in &config.phrases {
        let phrase = normalize_wake_text(phrase);
        if !phrase.is_empty() && !dictionary_terms.iter().any(|term| term == &phrase) {
            dictionary_terms.push(phrase);
        }
    }

    let language = settings.language.clone();
    let transcriber = state.local_transcriber();
    let result = match transcriber.transcribe(
        &ready_model,
        &candidate.samples,
        candidate.sample_rate,
        &dictionary_terms,
        Some(&language),
    ) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Wake command transcription failed: {err}");
            return None;
        }
    };

    let mut wake_match = match_wake_phrase(&result.transcript, &config.phrases);
    if let Some(wake_match) = wake_match.as_mut() {
        if settings.wake_speaker_verification_enabled {
            let Some(profile) = settings.wake_speaker_profile.as_ref() else {
                eprintln!("[wake] ignored command=hello reason=no speaker profile");
                return None;
            };
            let Some(check) =
                wake_speaker::verify_samples(&candidate.samples, candidate.sample_rate, profile)
            else {
                eprintln!("[wake] ignored command=hello reason=speaker check unavailable");
                return None;
            };
            wake_match.speaker_score = Some(check.score);
            if !check.verified {
                eprintln!(
                    "[wake] ignored command=hello reason=speaker mismatch score={:.2} threshold={:.2}",
                    check.score, check.threshold
                );
                return None;
            }
        }
    }
    if let Some(wake_match) = wake_match.as_ref() {
        eprintln!(
            "[wake] matched command={} confidence={:.2} speaker_score={} transcript={:?}",
            wake_match.command,
            wake_match.confidence,
            wake_match
                .speaker_score
                .map(|score| format!("{score:.2}"))
                .unwrap_or_else(|| "off".to_string()),
            wake_match.transcript
        );
    } else if !result.transcript.trim().is_empty() {
        eprintln!("[wake] ignored transcript={:?}", result.transcript.trim());
    }
    wake_match
}

fn match_wake_phrase(transcript: &str, phrases: &[String]) -> Option<WakeMatch> {
    let normalized_transcript = normalize_wake_text(transcript);
    if normalized_transcript != "hello" {
        return None;
    }

    if !phrases
        .iter()
        .any(|phrase| normalize_wake_text(phrase) == "hello")
    {
        return None;
    }

    Some(WakeMatch {
        command: "hello".to_string(),
        transcript: transcript.trim().to_string(),
        confidence: 1.0,
        speaker_score: None,
    })
}

fn normalize_wake_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phrases() -> Vec<String> {
        ["hello"].into_iter().map(String::from).collect()
    }

    #[test]
    fn matches_enabled_hello_wake_command() {
        let wake_match = match_wake_phrase("hello", &phrases()).expect("hello should wake");
        assert_eq!(wake_match.command, "hello");
    }

    #[test]
    fn ignores_commented_out_wake_commands() {
        assert!(match_wake_phrase("D X", &phrases()).is_none());
        assert!(match_wake_phrase("Friday", &phrases()).is_none());
        assert!(match_wake_phrase("Flow", &phrases()).is_none());
        assert!(match_wake_phrase("Aladdin", &phrases()).is_none());
        assert!(match_wake_phrase("Arise", &phrases()).is_none());
    }

    #[test]
    fn ignores_unrelated_speech() {
        assert!(match_wake_phrase("the app is still idle", &phrases()).is_none());
        assert!(match_wake_phrase("hallo", &phrases()).is_none());
        assert!(match_wake_phrase("hello there", &phrases()).is_none());
        assert!(match_wake_phrase("well hello", &phrases()).is_none());
        assert!(match_wake_phrase("hello hello", &phrases()).is_none());
    }
}

#[derive(Debug, Clone, Serialize)]
struct WakeStatusPayload {
    active: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transcript: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    speaker_score: Option<f32>,
}

fn emit_wake_status(app: &AppHandle<AppRuntime>, active: bool, message: &str) {
    if let Err(err) = app.emit(
        EVENT_WAKE_STATUS,
        WakeStatusPayload {
            active,
            message: message.to_string(),
            command: None,
            transcript: None,
            confidence: None,
            speaker_score: None,
        },
    ) {
        eprintln!("Failed to emit wake status: {err}");
    }
}
