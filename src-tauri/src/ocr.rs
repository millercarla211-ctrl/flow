use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use anyhow::{anyhow, Context, Result};
use base64::Engine;
use chrono::Local;
use serde::Serialize;
use tauri::{async_runtime, AppHandle, Runtime};

use crate::{app_paths, AppRuntime};

const OCR_MODEL_ROOT: &str = "models/ocr/glm-ocr-gguf";
const OCR_MODEL_FILE: &str = "GLM-OCR.Q4_K_M.gguf";
const OCR_MMPROJ_FILE: &str = "GLM-OCR.mmproj-Q8_0.gguf";
const OCR_RUNNER_RELATIVE: &str = "runtime/llama.cpp/build/bin/Release/llama-mtmd-cli.exe";

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone)]
struct OcrPaths {
    model: PathBuf,
    mmproj: PathBuf,
    runner: PathBuf,
}

#[derive(Debug, Serialize, Clone)]
pub struct OcrStatus {
    pub installed: bool,
    pub label: String,
    pub engine: String,
    pub model_path: String,
    pub mmproj_path: String,
    pub runner_path: String,
    pub bytes_on_disk: u64,
    pub missing_files: Vec<String>,
    pub benchmark_word_accuracy_percent: f32,
    pub benchmark_char_accuracy_percent: f32,
    pub benchmark_mean_elapsed_seconds: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct OcrResult {
    pub text: String,
    pub model: String,
    pub image_path: String,
    pub image_data_url: Option<String>,
    pub elapsed_ms: u128,
    pub runner_path: String,
}

#[tauri::command]
pub fn get_ocr_status<R: Runtime>(app: AppHandle<R>) -> Result<OcrStatus, String> {
    status(&app).map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn run_ocr_image(
    app: AppHandle<AppRuntime>,
    image_path: String,
    prompt: Option<String>,
) -> Result<OcrResult, String> {
    let paths = resolve_paths(&app).map_err(|err| err.to_string())?;
    let status = status_from_paths(&paths);
    if !status.installed {
        return Err(format!(
            "GLM OCR is not ready. Missing: {}",
            status.missing_files.join(", ")
        ));
    }

    let source = PathBuf::from(image_path);
    validate_image_path(&source).map_err(|err| err.to_string())?;
    let stored_image = copy_input_image(&app, &source).map_err(|err| err.to_string())?;
    let clean_prompt = prompt
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Text Recognition:")
        .to_string();

    async_runtime::spawn_blocking(move || run_ocr(paths, stored_image, clean_prompt))
        .await
        .map_err(|err| format!("OCR task failed: {err}"))?
        .map_err(|err| err.to_string())
}

fn status<R: Runtime>(app: &AppHandle<R>) -> Result<OcrStatus> {
    let paths = resolve_paths(app)?;
    Ok(status_from_paths(&paths))
}

fn status_from_paths(paths: &OcrPaths) -> OcrStatus {
    let mut missing_files = Vec::new();
    if !is_ready_file(&paths.model) {
        missing_files.push(paths.model.display().to_string());
    }
    if !is_ready_file(&paths.mmproj) {
        missing_files.push(paths.mmproj.display().to_string());
    }
    if !is_ready_file(&paths.runner) {
        missing_files.push(paths.runner.display().to_string());
    }

    let bytes_on_disk = [&paths.model, &paths.mmproj, &paths.runner]
        .iter()
        .filter_map(|path| path.metadata().ok().map(|metadata| metadata.len()))
        .sum();

    OcrStatus {
        installed: missing_files.is_empty(),
        label: "GLM OCR Q4_K_M".to_string(),
        engine: "llama.cpp multimodal".to_string(),
        model_path: paths.model.display().to_string(),
        mmproj_path: paths.mmproj.display().to_string(),
        runner_path: paths.runner.display().to_string(),
        bytes_on_disk,
        missing_files,
        benchmark_word_accuracy_percent: 94.3,
        benchmark_char_accuracy_percent: 95.6,
        benchmark_mean_elapsed_seconds: 66.0,
    }
}

fn is_ready_file(path: &Path) -> bool {
    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false)
}

fn resolve_paths<R: Runtime>(app: &AppHandle<R>) -> Result<OcrPaths> {
    let data_dir = app_paths::app_data_dir(app)?;
    let model_root = data_dir.join(OCR_MODEL_ROOT);
    let flow_root = data_dir.parent().map(Path::to_path_buf);

    Ok(OcrPaths {
        model: first_existing_file(vec![
            model_root.join(OCR_MODEL_FILE),
            PathBuf::from("G:/Flow/data")
                .join(OCR_MODEL_ROOT)
                .join(OCR_MODEL_FILE),
        ])
        .unwrap_or_else(|| model_root.join(OCR_MODEL_FILE)),
        mmproj: first_existing_file(vec![
            model_root.join(OCR_MMPROJ_FILE),
            PathBuf::from("G:/Flow/data")
                .join(OCR_MODEL_ROOT)
                .join(OCR_MMPROJ_FILE),
        ])
        .unwrap_or_else(|| model_root.join(OCR_MMPROJ_FILE)),
        runner: resolve_runner(flow_root),
    })
}

fn first_existing_file(candidates: Vec<PathBuf>) -> Option<PathBuf> {
    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|path| seen.insert(path.clone()))
        .find(|path| is_ready_file(path))
}

fn resolve_runner(flow_root: Option<PathBuf>) -> PathBuf {
    if let Some(path) = std::env::var_os("FLOW_OCR_RUNNER").map(PathBuf::from) {
        if is_ready_file(&path) {
            return path;
        }
    }

    let mut candidates = Vec::new();
    if let Some(root) = flow_root {
        candidates.push(root.join(OCR_RUNNER_RELATIVE));
    }
    candidates.push(PathBuf::from("G:/Flow").join(OCR_RUNNER_RELATIVE));

    first_existing_file(candidates)
        .unwrap_or_else(|| PathBuf::from("G:/Flow").join(OCR_RUNNER_RELATIVE))
}

fn validate_image_path(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("Image path does not exist"));
    }
    if !path.is_file() {
        return Err(anyhow!("Choose an image file"));
    }

    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "bmp" => Ok(()),
        _ => Err(anyhow!("Supported OCR images: png, jpg, jpeg, webp, bmp")),
    }
}

fn copy_input_image<R: Runtime>(app: &AppHandle<R>, source: &Path) -> Result<PathBuf> {
    let mut dir = app_paths::app_data_dir(app)?;
    dir.push("ocr");
    dir.push("inputs");
    fs::create_dir_all(&dir)?;

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("png");
    let stamp = Local::now().format("%Y%m%d-%H%M%S%.3f");
    let destination = dir.join(format!("flow-ocr-{stamp}.{extension}"));
    fs::copy(source, &destination)
        .with_context(|| format!("Failed to copy {}", source.display()))?;
    Ok(destination)
}

fn run_ocr(paths: OcrPaths, image_path: PathBuf, prompt: String) -> Result<OcrResult> {
    let started = Instant::now();
    let log_path = image_path.with_extension("llama.log");
    let threads = std::thread::available_parallelism()
        .map(|count| count.get().saturating_sub(1).clamp(2, 6))
        .unwrap_or(4)
        .to_string();

    let mut command = Command::new(&paths.runner);
    command
        .arg("--log-file")
        .arg(&log_path)
        .arg("-m")
        .arg(&paths.model)
        .arg("--mmproj")
        .arg(&paths.mmproj)
        .arg("--image")
        .arg(&image_path)
        .arg("-p")
        .arg(prompt)
        .arg("-n")
        .arg("1024")
        .arg("-c")
        .arg("8192")
        .arg("-t")
        .arg(threads)
        .arg("--temp")
        .arg("0")
        .arg("--no-mmproj-offload")
        .arg("--no-warmup");

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let output = command.output().context("Failed to start GLM OCR runner")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let detail = first_non_empty(&stderr, &stdout);
        return Err(anyhow!(
            "GLM OCR failed: {}",
            truncate(&detail, 1_200).unwrap_or_else(|| "runner exited without details".to_string())
        ));
    }

    let text =
        extract_ocr_text(&stdout).or_else(|| extract_ocr_text(&format!("{stdout}\n{stderr}")));
    let text = text
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("GLM OCR produced no readable text"))?;

    Ok(OcrResult {
        text,
        model: "glm-ocr-q4-k-m".to_string(),
        image_path: image_path.display().to_string(),
        image_data_url: image_data_url(&image_path).ok(),
        elapsed_ms: started.elapsed().as_millis(),
        runner_path: paths.runner.display().to_string(),
    })
}

fn extract_ocr_text(raw: &str) -> Option<String> {
    let mut lines: Vec<&str> = raw.lines().collect();
    if let Some(index) = lines
        .iter()
        .position(|line| line.trim_start().starts_with("image decoded"))
    {
        lines = lines.into_iter().skip(index + 1).collect();
    }

    let mut kept = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("llama_perf_context_print")
            || trimmed.starts_with("elapsed_seconds=")
            || trimmed.starts_with("common_")
            || trimmed.starts_with("llama_")
            || trimmed.starts_with("clip_")
            || trimmed.starts_with("load_")
            || trimmed.starts_with("print_info:")
            || trimmed.starts_with("sched_")
            || trimmed.starts_with("mtmd_")
            || trimmed.starts_with("main:")
            || trimmed.starts_with("WARN:")
            || trimmed.starts_with("--- vision")
            || trimmed.starts_with("alloc_compute")
            || trimmed.starts_with("encoding image")
            || trimmed.starts_with("decoding image")
            || trimmed.starts_with("warmup:")
            || trimmed.starts_with("model has unused tensor")
            || trimmed.starts_with('.')
        {
            continue;
        }
        kept.push(line);
    }

    let text = kept.join("\n").trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn first_non_empty(primary: &str, fallback: &str) -> String {
    if primary.trim().is_empty() {
        fallback.trim().to_string()
    } else {
        primary.trim().to_string()
    }
}

fn truncate(value: &str, max_chars: usize) -> Option<String> {
    if value.trim().is_empty() {
        return None;
    }
    if value.chars().count() <= max_chars {
        return Some(value.to_string());
    }
    Some(format!(
        "{}...",
        value.chars().take(max_chars).collect::<String>().trim_end()
    ))
}

fn image_data_url(path: &Path) -> Result<String> {
    const MAX_PREVIEW_BYTES: u64 = 8 * 1024 * 1024;
    let metadata = path.metadata()?;
    if metadata.len() > MAX_PREVIEW_BYTES {
        return Err(anyhow!("Image is too large for preview"));
    }

    let mime = match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "image/png",
    };
    let bytes = fs::read(path)?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:{mime};base64,{encoded}"))
}
