use anyhow::{Context, Result};
#[cfg(feature = "sherpa-stt")]
use sherpa_onnx::{OfflineRecognizer, OfflineRecognizerConfig, OfflineTransducerModelConfig};
use std::path::{Path, PathBuf};
use transcribe_rs::onnx::Quantization;
use transcribe_rs::onnx::moonshine::{MoonshineModel, MoonshineParams, MoonshineVariant};

use crate::runtime::{BrokerRequest, Modality, RuntimeBroker};

const MOONSHINE_TINY_KEY: &str = "moonshine-tiny";
const PARAKEET_TDT_KEY: &str = "parakeet-tdt-0.6b-v3-int8";
const NEMOTRON_STREAMING_KEY: &str = "nemotron-speech-streaming-en-0.6b-int8";

/// Moonshine STT using pure ONNX Runtime (no whisper-cpp conflicts!)
///
/// Uses the unquantized fp32 models from onnx-community which are guaranteed
/// to have the correct graph structure with use_cache_branch routing.
pub struct MoonshineSTT {
    model: MoonshineModel,
}

impl MoonshineSTT {
    pub fn is_available() -> bool {
        Path::new("models/stt/encoder_model.onnx").exists()
            && Path::new("models/stt/decoder_model_merged.onnx").exists()
            && Path::new("models/stt/tokenizer.json").exists()
    }

    pub fn new() -> Result<Self> {
        println!("[STT] Initializing Moonshine STT engine...");

        let model_dir = PathBuf::from("models/stt");

        // Load the Moonshine models via pure ONNX Runtime
        // Using MoonshineVariant::Tiny for the Moonshine Tiny model
        // Using Quantization::FP32 for the unquantized fp32 models
        let model = MoonshineModel::load(
            &model_dir,
            MoonshineVariant::Tiny,
            &Quantization::FP32
        ).context("Failed to load Moonshine models. Ensure encoder_model.onnx, decoder_model_merged.onnx, and tokenizer.json are in models/stt/")?;

        println!("[STT] Moonshine ready!");
        Ok(Self { model })
    }

    /// Transcribe audio from a WAV file
    pub fn transcribe(&mut self, audio_path: &str) -> Result<String> {
        let (processed_samples, sample_rate, channels, bits_per_sample) =
            load_wav_mono_16k(audio_path)?;

        println!(
            "[STT] Audio: {}Hz, {} channel(s), {} bits",
            sample_rate, channels, bits_per_sample
        );
        println!(
            "[STT] Loaded {} samples ({:.2}s)",
            processed_samples.len(),
            processed_samples.len() as f32 / 16000.0
        );
        println!(
            "[STT] Processing {} samples for transcription...",
            processed_samples.len()
        );

        // Transcribe
        self.transcribe_samples(&processed_samples)
    }

    /// Transcribe a 16kHz mono f32 audio buffer
    pub fn transcribe_samples(&mut self, audio_samples: &[f32]) -> Result<String> {
        let params = MoonshineParams::default();

        let result = self
            .model
            .transcribe_with(audio_samples, &params)
            .context("Failed to transcribe audio")?;

        Ok(result.text)
    }
}

pub enum LocalSttEngine {
    Moonshine(MoonshineSTT),
    #[cfg(feature = "sherpa-stt")]
    SherpaTransducer(SherpaTransducerStt),
}

impl LocalSttEngine {
    pub fn model_files_ready(model_key: &str, local_path: Option<&str>) -> bool {
        match model_key {
            MOONSHINE_TINY_KEY => MoonshineSTT::is_available(),
            PARAKEET_TDT_KEY | NEMOTRON_STREAMING_KEY => {
                local_path.is_some_and(sherpa_transducer_files_ready)
            }
            _ => local_path.map(Path::new).map(Path::exists).unwrap_or(false),
        }
    }

    pub fn from_broker(broker: &RuntimeBroker) -> Result<Self> {
        let mut request = BrokerRequest::new(Modality::SpeechToText);
        request.allow_conversion = false;
        request.allow_publish = false;
        let plan = broker.build_plan(request);
        let model_key = plan
            .selected_model
            .as_deref()
            .context("No STT model selected by the runtime broker")?;
        let manifest = broker
            .catalog()
            .iter()
            .find(|candidate| candidate.key == model_key)
            .context("Selected STT model is missing from the broker catalog")?;

        if !Self::model_files_ready(model_key, manifest.local_path.as_deref()) {
            return Err(anyhow::anyhow!(
                "Selected STT model '{}' is not available locally at {}",
                model_key,
                manifest.local_path.as_deref().unwrap_or("<missing>")
            ));
        }

        Self::from_selection(model_key, manifest.local_path.as_deref())
    }

    pub fn from_selection(model_key: &str, local_path: Option<&str>) -> Result<Self> {
        #[cfg(not(feature = "sherpa-stt"))]
        let _ = local_path;

        match model_key {
            MOONSHINE_TINY_KEY => Ok(Self::Moonshine(MoonshineSTT::new()?)),
            PARAKEET_TDT_KEY | NEMOTRON_STREAMING_KEY => {
                #[cfg(feature = "sherpa-stt")]
                {
                    let root = local_path
                        .and_then(sherpa_model_root_from_local_path)
                        .context("Sherpa STT selection did not include a model directory")?;
                    Ok(Self::SherpaTransducer(SherpaTransducerStt::new(
                        model_key, &root,
                    )?))
                }
                #[cfg(not(feature = "sherpa-stt"))]
                {
                    Err(anyhow::anyhow!(
                        "STT model '{}' requires building Flow with the 'sherpa-stt' feature",
                        model_key
                    ))
                }
            }
            other => Err(anyhow::anyhow!(
                "Unsupported local STT engine '{}'. Supported: {}, {}, {}",
                other,
                MOONSHINE_TINY_KEY,
                PARAKEET_TDT_KEY,
                NEMOTRON_STREAMING_KEY
            )),
        }
    }

    pub fn transcribe(&mut self, audio_path: &str) -> Result<String> {
        match self {
            Self::Moonshine(engine) => engine.transcribe(audio_path),
            #[cfg(feature = "sherpa-stt")]
            Self::SherpaTransducer(engine) => engine.transcribe(audio_path),
        }
    }

    pub fn transcribe_samples(&mut self, audio_samples: &[f32]) -> Result<String> {
        match self {
            Self::Moonshine(engine) => engine.transcribe_samples(audio_samples),
            #[cfg(feature = "sherpa-stt")]
            Self::SherpaTransducer(engine) => engine.transcribe_samples(audio_samples),
        }
    }
}

#[cfg(feature = "sherpa-stt")]
pub struct SherpaTransducerStt {
    model_key: String,
    recognizer: OfflineRecognizer,
}

#[cfg(feature = "sherpa-stt")]
impl SherpaTransducerStt {
    pub fn new(model_key: &str, root: &Path) -> Result<Self> {
        let paths = SherpaTransducerPaths::from_root(root)
            .with_context(|| format!("Missing sherpa-onnx STT files in {}", root.display()))?;

        println!("[STT] Initializing sherpa-onnx STT engine ({model_key})...");
        let mut config = OfflineRecognizerConfig::default();
        config.model_config.transducer = OfflineTransducerModelConfig {
            encoder: Some(paths.encoder.to_string_lossy().into_owned()),
            decoder: Some(paths.decoder.to_string_lossy().into_owned()),
            joiner: Some(paths.joiner.to_string_lossy().into_owned()),
        };
        config.model_config.tokens = Some(paths.tokens.to_string_lossy().into_owned());
        config.model_config.model_type = Some("nemo_transducer".to_string());

        let recognizer = OfflineRecognizer::create(&config).ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to create sherpa-onnx recognizer for '{}' from {}",
                model_key,
                root.display()
            )
        })?;

        println!("[STT] sherpa-onnx ready!");
        Ok(Self {
            model_key: model_key.to_string(),
            recognizer,
        })
    }

    pub fn transcribe(&mut self, audio_path: &str) -> Result<String> {
        let (samples, sample_rate, channels, bits_per_sample) = load_wav_mono_16k(audio_path)?;
        println!(
            "[STT] Audio: {}Hz, {} channel(s), {} bits",
            sample_rate, channels, bits_per_sample
        );
        self.transcribe_samples(&samples)
    }

    pub fn transcribe_samples(&mut self, audio_samples: &[f32]) -> Result<String> {
        let stream = self.recognizer.create_stream();
        stream.accept_waveform(16_000, audio_samples);
        self.recognizer.decode(&stream);
        let result = stream.get_result().ok_or_else(|| {
            anyhow::anyhow!(
                "sherpa-onnx returned no transcription result for '{}'",
                self.model_key
            )
        })?;
        Ok(result.text)
    }
}

#[cfg(feature = "sherpa-stt")]
#[derive(Debug, Clone, PartialEq, Eq)]
struct SherpaTransducerPaths {
    encoder: PathBuf,
    decoder: PathBuf,
    joiner: PathBuf,
    tokens: PathBuf,
}

#[cfg(feature = "sherpa-stt")]
impl SherpaTransducerPaths {
    fn from_root(root: &Path) -> Option<Self> {
        Some(Self {
            encoder: find_first_existing(root, &["encoder.int8.onnx", "encoder.onnx"])?,
            decoder: find_first_existing(root, &["decoder.int8.onnx", "decoder.onnx"])?,
            joiner: find_first_existing(root, &["joiner.int8.onnx", "joiner.onnx"])?,
            tokens: root
                .join("tokens.txt")
                .exists()
                .then(|| root.join("tokens.txt"))?,
        })
    }
}

#[cfg(feature = "sherpa-stt")]
fn find_first_existing(root: &Path, candidates: &[&str]) -> Option<PathBuf> {
    candidates
        .iter()
        .map(|candidate| root.join(candidate))
        .find(|path| path.exists())
}

fn sherpa_model_root_from_local_path(local_path: &str) -> Option<PathBuf> {
    let path = PathBuf::from(local_path);
    if path.is_dir() {
        Some(path)
    } else {
        path.parent().map(Path::to_path_buf)
    }
}

fn sherpa_transducer_files_ready(local_path: &str) -> bool {
    sherpa_model_root_from_local_path(local_path).is_some_and(|root| {
        root.join("encoder.int8.onnx").exists()
            && root.join("decoder.int8.onnx").exists()
            && root.join("joiner.int8.onnx").exists()
            && root.join("tokens.txt").exists()
    })
}

fn load_wav_mono_16k(audio_path: &str) -> Result<(Vec<f32>, u32, u16, u16)> {
    let mut reader = hound::WavReader::open(audio_path).context("Failed to open audio file")?;
    let spec = reader.spec();
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("Failed to read float samples")?,
        hound::SampleFormat::Int => {
            let scale = (1_i64 << (spec.bits_per_sample.saturating_sub(1) as u32)) as f32;
            reader
                .samples::<i32>()
                .collect::<std::result::Result<Vec<_>, _>>()
                .context("Failed to read int samples")?
                .into_iter()
                .map(|sample| (sample as f32 / scale).clamp(-1.0, 1.0))
                .collect()
        }
    };

    let mono = if spec.channels <= 1 {
        samples
    } else {
        let channels = spec.channels as usize;
        samples
            .chunks(channels)
            .map(|frame| frame.iter().copied().sum::<f32>() / frame.len() as f32)
            .collect()
    };

    let processed_samples = if spec.sample_rate != 16_000 {
        println!("[STT] Resampling from {}Hz to 16000Hz...", spec.sample_rate);
        crate::audio::resample::resample(&mono, spec.sample_rate, 16_000)?
    } else {
        mono
    };

    Ok((
        processed_samples,
        spec.sample_rate,
        spec.channels,
        spec.bits_per_sample,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_sherpa_model_bundle_is_not_ready() {
        assert!(!LocalSttEngine::model_files_ready(
            PARAKEET_TDT_KEY,
            Some("models/stt/does-not-exist/encoder.int8.onnx")
        ));
    }

    #[test]
    fn sherpa_root_can_be_resolved_from_manifest_encoder_path() {
        let root = sherpa_model_root_from_local_path(
            "models/stt/parakeet-tdt-0.6b-v3-int8/encoder.int8.onnx",
        )
        .unwrap();
        assert_eq!(root, PathBuf::from("models/stt/parakeet-tdt-0.6b-v3-int8"));
    }
}
