use std::f32::consts::PI;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use crossbeam_channel::{bounded, Sender};
use rustfft::{num_complex::Complex, FftPlanner};
use serde::{Deserialize, Serialize};
use tauri::{async_runtime, AppHandle, Emitter, Manager};

use crate::{settings::UserSettings, AppRuntime, AppState, EVENT_SETTINGS_CHANGED};

const ENROLL_SAMPLE_SECONDS: f32 = 3.4;
const ENROLL_MIN_SECONDS: f32 = 1.5;
const ENROLL_MAX_SECONDS: f32 = 6.0;
const FEATURE_BANDS: usize = 10;
const BASE_FEATURES: usize = 6 + FEATURE_BANDS;
const DEFAULT_WAKE_SPEAKER_THRESHOLD: f32 = 0.82;
const MIN_ENROLL_RMS: f32 = 0.0012;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeSpeakerProfile {
    pub phrase: String,
    pub enrolled_at: String,
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub sample_count: usize,
    pub threshold: f32,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WakeSpeakerCheck {
    pub verified: bool,
    pub score: f32,
    pub threshold: f32,
}

#[derive(Debug)]
struct CaptureFrame {
    samples: Vec<i16>,
    sample_rate: u32,
}

#[tauri::command]
pub async fn enroll_wake_speaker_profile(
    app: AppHandle<AppRuntime>,
    phrase: Option<String>,
    seconds: Option<f32>,
) -> Result<UserSettings, String> {
    async_runtime::spawn_blocking(move || {
        let phrase = normalize_phrase(phrase.unwrap_or_else(|| "hello".to_string()));
        let seconds = seconds
            .unwrap_or(ENROLL_SAMPLE_SECONDS)
            .clamp(ENROLL_MIN_SECONDS, ENROLL_MAX_SECONDS);
        let state = app.state::<AppState>();
        let settings = state.current_settings();
        state.wake().stop();
        let capture_result = record_voice_sample(settings.microphone_device.as_deref(), seconds);
        let (samples, sample_rate) = match capture_result {
            Ok(captured) => captured,
            Err(err) => {
                let _ = state.wake().sync(&app, &settings);
                return Err(err.to_string());
            }
        };
        let profile = match build_profile(&phrase, &samples, sample_rate) {
            Ok(profile) => profile,
            Err(err) => {
                let _ = state.wake().sync(&app, &settings);
                return Err(err.to_string());
            }
        };

        let mut next = state.current_settings();
        next.wake_listening_enabled = true;
        next.wake_speaker_verification_enabled = true;
        next.wake_speaker_profile = Some(profile);
        if !next
            .wake_phrases
            .iter()
            .any(|candidate| normalize_phrase(candidate) == phrase)
        {
            next.wake_phrases = vec![phrase];
        }

        let saved = match state.persist_settings(next) {
            Ok(saved) => saved,
            Err(err) => {
                let _ = state.wake().sync(&app, &settings);
                return Err(err.to_string());
            }
        };
        state
            .wake()
            .sync(&app, &saved)
            .map_err(|err| err.to_string())?;
        let _ = app.emit(EVENT_SETTINGS_CHANGED, &saved);
        Ok(saved)
    })
    .await
    .map_err(|err| format!("Wake speaker enrollment task failed: {err}"))?
}

#[tauri::command]
pub fn clear_wake_speaker_profile(app: AppHandle<AppRuntime>) -> Result<UserSettings, String> {
    let state = app.state::<AppState>();
    let mut next = state.current_settings();
    next.wake_speaker_verification_enabled = false;
    next.wake_speaker_profile = None;
    let saved = state
        .persist_settings(next)
        .map_err(|err| err.to_string())?;
    state
        .wake()
        .sync(&app, &saved)
        .map_err(|err| err.to_string())?;
    let _ = app.emit(EVENT_SETTINGS_CHANGED, &saved);
    Ok(saved)
}

pub(crate) fn verify_samples(
    samples: &[i16],
    sample_rate: u32,
    profile: &WakeSpeakerProfile,
) -> Option<WakeSpeakerCheck> {
    let embedding = build_voiceprint(samples, sample_rate).ok()?;
    let score = cosine_similarity(&embedding, &profile.embedding);
    let threshold = profile.threshold.clamp(0.65, 0.94);
    Some(WakeSpeakerCheck {
        verified: score >= threshold,
        score,
        threshold,
    })
}

fn build_profile(phrase: &str, samples: &[i16], sample_rate: u32) -> Result<WakeSpeakerProfile> {
    let embedding = build_voiceprint(samples, sample_rate)?;
    Ok(WakeSpeakerProfile {
        phrase: phrase.to_string(),
        enrolled_at: chrono::Utc::now().to_rfc3339(),
        duration_ms: ((samples.len() as f64 / sample_rate.max(1) as f64) * 1000.0).round() as u64,
        sample_rate,
        sample_count: samples.len(),
        threshold: DEFAULT_WAKE_SPEAKER_THRESHOLD,
        embedding,
    })
}

fn build_voiceprint(samples: &[i16], sample_rate: u32) -> Result<Vec<f32>> {
    if samples.is_empty() || sample_rate == 0 {
        return Err(anyhow!("No wake voice audio was captured"));
    }

    let audio = trim_edges(samples);
    let overall_rms = rms_i16(&audio);
    if overall_rms < MIN_ENROLL_RMS {
        return Err(anyhow!(
            "Wake voice sample is too quiet. Move closer and say hello clearly."
        ));
    }

    let frame_len = ((sample_rate as f32 * 0.03).round() as usize).clamp(320, 2048);
    let hop = (frame_len / 2).max(1);
    let fft_len = frame_len.next_power_of_two();
    let nyquist = sample_rate as f32 / 2.0;
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_len);
    let mut buffer = vec![Complex::new(0.0, 0.0); fft_len];
    let mut frame_features: Vec<[f32; BASE_FEATURES]> = Vec::new();
    let max_frame_rms = audio
        .windows(frame_len)
        .step_by(hop)
        .map(rms_i16)
        .fold(0.0f32, f32::max);
    let voice_floor = (max_frame_rms * 0.18).max(MIN_ENROLL_RMS);

    let mut start = 0usize;
    while start + frame_len <= audio.len() {
        let frame = &audio[start..start + frame_len];
        let frame_rms = rms_i16(frame);
        if frame_rms >= voice_floor {
            frame_features.push(extract_frame_features(
                frame,
                frame_rms,
                sample_rate,
                nyquist,
                &mut buffer,
                &fft,
            ));
        }
        start += hop;
    }

    if frame_features.len() < 6 {
        return Err(anyhow!(
            "Wake voice sample was too short. Say hello two or three times."
        ));
    }

    let mut embedding = Vec::with_capacity(BASE_FEATURES * 2);
    for idx in 0..BASE_FEATURES {
        let mean = frame_features
            .iter()
            .map(|features| features[idx])
            .sum::<f32>()
            / frame_features.len() as f32;
        let variance = frame_features
            .iter()
            .map(|features| {
                let delta = features[idx] - mean;
                delta * delta
            })
            .sum::<f32>()
            / frame_features.len() as f32;
        embedding.push(mean);
        embedding.push(variance.sqrt());
    }

    normalize_vector(&mut embedding);
    Ok(embedding)
}

fn extract_frame_features(
    frame: &[i16],
    frame_rms: f32,
    sample_rate: u32,
    nyquist: f32,
    buffer: &mut [Complex<f32>],
    fft: &std::sync::Arc<dyn rustfft::Fft<f32>>,
) -> [f32; BASE_FEATURES] {
    for value in buffer.iter_mut() {
        *value = Complex::new(0.0, 0.0);
    }

    for (idx, sample) in frame.iter().enumerate() {
        let window = 0.5 - 0.5 * ((2.0 * PI * idx as f32) / (frame.len() - 1) as f32).cos();
        buffer[idx] = Complex::new((*sample as f32 / i16::MAX as f32) * window, 0.0);
    }
    fft.process(buffer);

    let half = buffer.len() / 2;
    let magnitudes: Vec<f32> = buffer[..half].iter().map(|bin| bin.norm()).collect();
    let total = magnitudes.iter().sum::<f32>().max(1e-8);

    let centroid = magnitudes
        .iter()
        .enumerate()
        .map(|(idx, mag)| (idx as f32 / half.max(1) as f32) * *mag)
        .sum::<f32>()
        / total;

    let mut cumulative = 0.0f32;
    let mut rolloff = 0.0f32;
    for (idx, mag) in magnitudes.iter().enumerate() {
        cumulative += *mag;
        if cumulative >= total * 0.85 {
            rolloff = idx as f32 / half.max(1) as f32;
            break;
        }
    }

    let flatness = {
        let geometric = magnitudes.iter().map(|mag| (mag + 1e-8).ln()).sum::<f32>()
            / magnitudes.len().max(1) as f32;
        let arithmetic = total / magnitudes.len().max(1) as f32;
        (geometric.exp() / arithmetic.max(1e-8)).clamp(0.0, 1.0)
    };

    let (pitch_norm, pitch_confidence) = estimate_pitch(frame, sample_rate);
    let mut features = [0.0; BASE_FEATURES];
    features[0] = normalize_log_unit(frame_rms);
    features[1] = zero_crossing_rate(frame);
    features[2] = centroid.clamp(0.0, 1.0);
    features[3] = rolloff.clamp(0.0, 1.0);
    features[4] = flatness;
    features[5] = (pitch_norm * pitch_confidence).clamp(0.0, 1.0);

    for band in 0..FEATURE_BANDS {
        let (low, high) = band_bounds_hz(band, nyquist);
        let low_bin = ((low / nyquist.max(1.0)) * half as f32)
            .floor()
            .clamp(0.0, (half.saturating_sub(1)) as f32) as usize;
        let high_bin = ((high / nyquist.max(1.0)) * half as f32).ceil().clamp(
            (low_bin + 1).min(half.saturating_sub(1)) as f32,
            half as f32,
        ) as usize;
        let band_energy = magnitudes[low_bin..high_bin.min(magnitudes.len())]
            .iter()
            .sum::<f32>();
        features[6 + band] = normalize_band_ratio(band_energy / total);
    }

    features
}

fn band_bounds_hz(band: usize, nyquist: f32) -> (f32, f32) {
    let min_hz = 80.0f32;
    let max_hz = nyquist.min(4200.0).max(min_hz * 2.0);
    let step = (max_hz / min_hz).powf(1.0 / FEATURE_BANDS as f32);
    let low = min_hz * step.powf(band as f32);
    let high = min_hz * step.powf((band + 1) as f32);
    (low, high.min(max_hz))
}

fn estimate_pitch(frame: &[i16], sample_rate: u32) -> (f32, f32) {
    let min_lag = (sample_rate / 340).max(1) as usize;
    let max_lag = (sample_rate / 70).max(min_lag as u32 + 1) as usize;
    if frame.len() <= max_lag + 2 {
        return (0.0, 0.0);
    }

    let values: Vec<f32> = frame
        .iter()
        .map(|sample| *sample as f32 / i16::MAX as f32)
        .collect();
    let energy = values
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        .max(1e-8);
    let mut best_lag = 0usize;
    let mut best_score = 0.0f32;
    for lag in min_lag..=max_lag.min(frame.len() / 2) {
        let score = values[..values.len() - lag]
            .iter()
            .zip(values[lag..].iter())
            .map(|(a, b)| a * b)
            .sum::<f32>()
            / energy;
        if score > best_score {
            best_score = score;
            best_lag = lag;
        }
    }

    if best_lag == 0 || best_score < 0.12 {
        return (0.0, 0.0);
    }
    let pitch_hz = sample_rate as f32 / best_lag as f32;
    (
        (pitch_hz / 340.0).clamp(0.0, 1.0),
        best_score.clamp(0.0, 1.0),
    )
}

fn zero_crossing_rate(frame: &[i16]) -> f32 {
    if frame.len() < 2 {
        return 0.0;
    }
    let crossings = frame
        .windows(2)
        .filter(|pair| (pair[0] >= 0 && pair[1] < 0) || (pair[0] < 0 && pair[1] >= 0))
        .count();
    crossings as f32 / (frame.len() - 1) as f32
}

fn normalize_log_unit(value: f32) -> f32 {
    ((value.max(1e-5).log10() + 5.0) / 5.0).clamp(0.0, 1.0)
}

fn normalize_band_ratio(value: f32) -> f32 {
    ((value.max(1e-5).log10() + 5.0) / 5.0).clamp(0.0, 1.0)
}

fn normalize_vector(values: &mut [f32]) {
    let norm = values
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt()
        .max(1e-8);
    for value in values {
        *value /= norm;
    }
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    left.iter()
        .zip(right.iter())
        .map(|(a, b)| a * b)
        .sum::<f32>()
        .clamp(-1.0, 1.0)
}

fn trim_edges(samples: &[i16]) -> Vec<i16> {
    let max_abs = samples
        .iter()
        .map(|sample| sample.unsigned_abs() as i32)
        .max()
        .unwrap_or(0);
    let threshold = (max_abs as f32 * 0.08).max((i16::MAX as f32) * MIN_ENROLL_RMS) as i32;
    let start = samples
        .iter()
        .position(|sample| sample.unsigned_abs() as i32 >= threshold)
        .unwrap_or(0);
    let end = samples
        .iter()
        .rposition(|sample| sample.unsigned_abs() as i32 >= threshold)
        .map(|idx| idx + 1)
        .unwrap_or(samples.len());
    samples[start.min(end)..end].to_vec()
}

fn rms_i16(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum = samples
        .iter()
        .map(|sample| {
            let value = *sample as f32 / i16::MAX as f32;
            value * value
        })
        .sum::<f32>();
    (sum / samples.len() as f32).sqrt()
}

fn normalize_phrase(value: impl AsRef<str>) -> String {
    let value = value.as_ref();
    let normalized = value
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
        .join(" ");
    if normalized.is_empty() {
        "hello".to_string()
    } else {
        normalized
    }
}

fn record_voice_sample(device_id: Option<&str>, seconds: f32) -> Result<(Vec<i16>, u32)> {
    let (frame_tx, frame_rx) = bounded::<CaptureFrame>(32);
    let stream = build_capture_stream(device_id, frame_tx)?;
    stream
        .play()
        .context("Failed to start microphone capture")?;

    let started = Instant::now();
    let duration = Duration::from_secs_f32(seconds);
    let mut samples = Vec::new();
    let mut sample_rate = 0u32;
    while started.elapsed() < duration {
        if let Ok(frame) = frame_rx.recv_timeout(Duration::from_millis(80)) {
            sample_rate = frame.sample_rate;
            samples.extend_from_slice(&frame.samples);
        }
    }

    drop(stream);
    if samples.is_empty() || sample_rate == 0 {
        return Err(anyhow!("No microphone audio was captured"));
    }
    Ok((samples, sample_rate))
}

fn build_capture_stream(device_id: Option<&str>, frame_tx: Sender<CaptureFrame>) -> Result<Stream> {
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
            .context("Selected microphone not found and no default microphone available")?
    } else {
        host.default_input_device()
            .context("No default microphone available")?
    };

    let config = device
        .default_input_config()
        .context("No supported microphone input configuration found")?;
    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();
    let channels = stream_config.channels as usize;
    let sample_rate = stream_config.sample_rate;
    let err_fn = |err| eprintln!("Wake speaker enrollment microphone error: {err}");

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
        _ => return Err(anyhow!("Unsupported microphone input sample format")),
    };

    Ok(stream)
}

fn publish_capture(samples: Vec<i16>, sample_rate: u32, frame_tx: &Sender<CaptureFrame>) {
    let _ = frame_tx.try_send(CaptureFrame {
        samples,
        sample_rate,
    });
}

fn publish_f32_frame(
    data: &[f32],
    channels: usize,
    sample_rate: u32,
    frame_tx: &Sender<CaptureFrame>,
) {
    let samples = if channels <= 1 {
        data.iter().map(|sample| f32_to_i16(*sample)).collect()
    } else {
        data.chunks(channels)
            .map(|frame| f32_to_i16(frame.iter().copied().sum::<f32>() / channels as f32))
            .collect()
    };
    publish_capture(samples, sample_rate, frame_tx);
}

fn publish_i16_frame(
    data: &[i16],
    channels: usize,
    sample_rate: u32,
    frame_tx: &Sender<CaptureFrame>,
) {
    let samples = if channels <= 1 {
        data.to_vec()
    } else {
        data.chunks(channels)
            .map(|frame| {
                (frame.iter().map(|sample| *sample as i32).sum::<i32>() / channels as i32) as i16
            })
            .collect()
    };
    publish_capture(samples, sample_rate, frame_tx);
}

fn publish_u16_frame(
    data: &[u16],
    channels: usize,
    sample_rate: u32,
    frame_tx: &Sender<CaptureFrame>,
) {
    let samples = if channels <= 1 {
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
    publish_capture(samples, sample_rate, frame_tx);
}

fn f32_to_i16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16
}

fn u16_to_i16(sample: u16) -> i16 {
    (sample as i32 - 32768).clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(freq: f32, sample_rate: u32, seconds: f32) -> Vec<i16> {
        let total = (sample_rate as f32 * seconds) as usize;
        (0..total)
            .map(|idx| {
                let value = (2.0 * PI * freq * idx as f32 / sample_rate as f32).sin() * 0.25;
                f32_to_i16(value)
            })
            .collect()
    }

    #[test]
    fn voiceprint_is_stable_for_same_audio() {
        let samples = sine(170.0, 16_000, 2.0);
        let a = build_voiceprint(&samples, 16_000).expect("first voiceprint");
        let b = build_voiceprint(&samples, 16_000).expect("second voiceprint");
        assert!(cosine_similarity(&a, &b) > 0.99);
    }

    #[test]
    fn voiceprint_score_changes_with_different_audio() {
        let a = build_voiceprint(&sine(150.0, 16_000, 2.0), 16_000).expect("voiceprint a");
        let b = build_voiceprint(&sine(260.0, 16_000, 2.0), 16_000).expect("voiceprint b");
        assert!(cosine_similarity(&a, &b) < 0.98);
    }
}
