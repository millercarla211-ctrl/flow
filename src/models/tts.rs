use anyhow::Result;
use ndarray::{Array1, Array2, ArrayBase, IxDyn, OwnedRepr};
use ndarray_npy::NpzReader;
use ort::session::Session;
use ort::value::Value;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// Kokoro TTS using local ONNX models (pure Rust, no Python!)
pub struct KokoroTTS {
    session: Session,
    vocab: HashMap<String, i64>,
    voices: HashMap<String, Vec<f32>>,
    default_voice: String,
}

impl KokoroTTS {
    pub fn is_available() -> bool {
        required_tts_files()
            .iter()
            .all(|path| Path::new(path).exists())
    }

    pub async fn new_async() -> Result<Self> {
        println!("[TTS] Initializing Kokoro TTS engine...");

        // Load ONNX model
        let session = Session::builder()?.commit_from_file("models/tts/kokoro-v1.0.int8.onnx")?;

        // Load vocabulary from config.json
        let config_str = std::fs::read_to_string("models/tts/config.json")?;
        let config: serde_json::Value = serde_json::from_str(&config_str)?;

        let mut vocab = HashMap::new();
        if let Some(vocab_obj) = config["vocab"].as_object() {
            for (k, v) in vocab_obj {
                if let Some(id) = v.as_i64() {
                    vocab.insert(k.clone(), id);
                }
            }
        }

        // Load voice embeddings from NPZ file
        let voices = Self::load_voices("models/tts/voices-v1.0.bin")?;

        // Use af_sky as default voice
        let default_voice = "af_sky".to_string();
        if !voices.contains_key(&default_voice) {
            return Err(anyhow::anyhow!(
                "Default voice 'af_sky' not found in voices file"
            ));
        }

        println!("[TTS] Ready ({} voices available)", voices.len());

        Ok(Self {
            session,
            vocab,
            voices,
            default_voice,
        })
    }

    fn load_voices(path: &str) -> Result<HashMap<String, Vec<f32>>> {
        let file = File::open(path)?;
        let mut npz = NpzReader::new(file)?;
        let mut voices = HashMap::new();

        for name in npz.names()? {
            // Read the voice array
            let arr: ArrayBase<OwnedRepr<f32>, IxDyn> = npz.by_name(&name)?;

            // Voice arrays are shaped (512, 1, 256) - 512 length variants, 1 batch, 256 features
            // We need to extract the first variant (index 0) which gives us (1, 256)
            let shape = arr.shape();
            if shape.len() == 3 && shape[1] == 1 && shape[2] == 256 {
                // Extract the first length variant (index 0)
                let start = 0;
                let end = 256;
                let data = arr
                    .as_slice()
                    .ok_or_else(|| anyhow::anyhow!("Failed to get slice for voice {}", name))?
                    [start..end]
                    .to_vec();

                // Remove .npy extension from name
                let voice_name = name.trim_end_matches(".npy").to_string();
                voices.insert(voice_name, data);
            }
        }

        Ok(voices)
    }

    pub fn new() -> Result<Self> {
        Err(anyhow::anyhow!(
            "Use new_async() instead - TTS requires async initialization"
        ))
    }

    pub fn synthesize(&mut self, text: &str) -> Result<Vec<f32>> {
        // Simple text-to-phoneme conversion for Kokoro
        // Kokoro accepts IPA-like phonemes, but can also work with cleaned text
        let cleaned_text = self.clean_text_for_synthesis(text);

        // Convert text to token IDs
        let mut token_ids = vec![0i64]; // Start with pad token

        for c in cleaned_text.chars() {
            let char_str = c.to_string();
            if let Some(&id) = self.vocab.get(&char_str) {
                token_ids.push(id);
            }
        }

        token_ids.push(0); // End with pad token

        // Get the voice embedding for the default voice
        let style = self
            .voices
            .get(&self.default_voice)
            .ok_or_else(|| anyhow::anyhow!("Voice '{}' not found", self.default_voice))?
            .clone();

        // Create style tensor (1, 256)
        let style_array = Array2::from_shape_vec((1, 256), style)?;

        // Create input tensors
        let tokens_len = token_ids.len();
        let tokens_array = Array2::from_shape_vec((1, tokens_len), token_ids)?;
        let speed_array = Array1::from_vec(vec![1.0f32]);

        // Run inference
        let outputs = self.session.run(ort::inputs![
            "tokens" => Value::from_array(tokens_array)?,
            "style" => Value::from_array(style_array)?,
            "speed" => Value::from_array(speed_array)?
        ])?;

        // Extract audio
        let audio_tensor = &outputs[0];
        let (_shape, audio_data) = audio_tensor.try_extract_tensor::<f32>()?;
        let audio: Vec<f32> = audio_data.to_vec();

        Ok(audio)
    }

    /// Clean and normalize text for synthesis
    fn clean_text_for_synthesis(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphabetic() || c.is_whitespace() || ".,!?'-".contains(*c))
            .collect()
    }

    pub fn speak(&mut self, text: &str) -> Result<()> {
        println!("[TTS] Synthesizing: \"{}\"", text);

        let audio = self.synthesize(text)?;

        let duration = audio.len() as f64 / 24000.0;
        println!("[TTS] Generated {:.2}s of audio", duration);

        // Save to output.wav in root FIRST
        self.save_wav(&audio, "output.wav")?;
        println!("[TTS] Saved to output.wav");

        // Then play
        crate::audio::AudioPlayer::play(&audio, 24000)?;

        Ok(())
    }

    pub fn save_wav(&self, audio: &[f32], path: &str) -> Result<()> {
        use hound::{WavSpec, WavWriter};

        let spec = WavSpec {
            channels: 1,
            sample_rate: 24000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path, spec)?;

        for &sample in audio {
            let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
            writer.write_sample(sample_i16)?;
        }

        writer.finalize()?;

        Ok(())
    }
}

fn required_tts_files() -> [&'static str; 3] {
    [
        "models/tts/kokoro-v1.0.int8.onnx",
        "models/tts/voices-v1.0.bin",
        "models/tts/config.json",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_availability_reflects_required_local_files() {
        let expected = required_tts_files()
            .iter()
            .all(|path| Path::new(path).exists());

        assert_eq!(KokoroTTS::is_available(), expected);
    }
}
