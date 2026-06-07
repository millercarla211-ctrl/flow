use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result, bail};

#[allow(dead_code)]
mod audio {
    use anyhow::{Result, bail};

    pub struct AudioPlayer;

    impl AudioPlayer {
        pub fn play(_samples: &[f32], _sample_rate: u32) -> Result<()> {
            bail!("flow-tts writes WAV output only and does not play audio");
        }
    }
}

#[allow(dead_code)]
#[path = "../models/tts.rs"]
mod tts;

use tts::KokoroTTS;

#[tokio::main]
async fn main() -> Result<()> {
    let request = match TtsRequest::parse(env::args())? {
        Some(request) => request,
        None => return Ok(()),
    };

    let flow_root = resolve_flow_root()?;
    env::set_current_dir(&flow_root)
        .with_context(|| format!("Failed to enter Flow root {}", flow_root.display()))?;

    if let Some(parent) = request.output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let output = request.output_path.to_str().with_context(|| {
        format!(
            "Output path is not valid UTF-8: {}",
            request.output_path.display()
        )
    })?;
    let mut tts = KokoroTTS::new_async().await?;
    let samples = tts.synthesize(&request.text)?;
    tts.save_wav(&samples, output)?;
    validate_wav_output(&request.output_path)?;

    println!("[tts] saved {}", request.output_path.display());
    Ok(())
}

#[derive(Debug)]
struct TtsRequest {
    text: String,
    output_path: PathBuf,
}

impl TtsRequest {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Option<Self>> {
        let mut text = None;
        let mut output_path = None;
        let mut args = args.into_iter().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--text" => {
                    text = Some(required_value(&mut args, "--text")?);
                }
                "--output" => {
                    output_path = Some(PathBuf::from(required_value(&mut args, "--output")?));
                }
                "--help" | "-h" => {
                    print_usage();
                    return Ok(None);
                }
                _ => bail!("Unsupported argument '{arg}'. Run flow-tts --help."),
            }
        }

        let text = text
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .context("Missing --text <text>")?;
        let output_path = output_path.context("Missing --output <wav-path>")?;

        Ok(Some(Self { text, output_path }))
    }
}

fn required_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    args.next()
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("Missing value for {flag}"))
}

fn print_usage() {
    println!("Usage: flow-tts --text <text> --output <wav-path>");
}

fn resolve_flow_root() -> Result<PathBuf> {
    let current_dir = env::current_dir().context("Failed to read current directory")?;
    if flow_root_ready(&current_dir) {
        return Ok(current_dir);
    }

    let current_exe = env::current_exe().context("Failed to read executable path")?;
    for ancestor in current_exe.ancestors() {
        if flow_root_ready(ancestor) {
            return Ok(ancestor.to_path_buf());
        }
    }

    bail!(
        "Could not find Flow root from {} or {}",
        current_dir.display(),
        current_exe.display()
    )
}

fn flow_root_ready(path: &Path) -> bool {
    [
        "models/tts/kokoro-v1.0.int8.onnx",
        "models/tts/voices-v1.0.bin",
        "models/tts/config.json",
    ]
    .iter()
    .all(|relative| file_is_nonempty(&path.join(relative)))
}

fn file_is_nonempty(path: &Path) -> bool {
    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false)
}

fn validate_wav_output(path: &Path) -> Result<()> {
    let size = path
        .metadata()
        .with_context(|| format!("TTS output was not written: {}", path.display()))?
        .len();
    if size <= 44 {
        bail!("TTS output WAV is empty: {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::TtsRequest;

    #[test]
    fn parses_text_and_output_path() {
        let request = TtsRequest::parse([
            "flow-tts".to_string(),
            "--text".to_string(),
            "hello".to_string(),
            "--output".to_string(),
            "tmp/speech.wav".to_string(),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(request.text, "hello");
        assert_eq!(
            request.output_path,
            std::path::PathBuf::from("tmp/speech.wav")
        );
    }

    #[test]
    fn rejects_missing_output_path() {
        let error = TtsRequest::parse([
            "flow-tts".to_string(),
            "--text".to_string(),
            "hello".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("Missing --output"));
    }
}
