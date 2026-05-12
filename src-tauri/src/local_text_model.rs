use anyhow::{anyhow, Context, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::async_runtime;

const QWEN3_06B_ID: &str = "qwen3-0.6b";
const QWEN3_06B_FILE: &str = "Qwen3-0.6B-Q4_K_M.gguf";
const XLAM2_3B_ID: &str = "xlam2-3b-fc-r-q4km";
const XLAM2_3B_FILE: &str = "xLAM-2-3B-fc-r-Q4_K_M.gguf";
const QWEN35_4B_REVISED_ID: &str = "qwen35-4b-revised-q4km";
const QWEN35_4B_REVISED_FILE: &str = "Qwen3.5-4B-q4_k_m.gguf";
const QWEN35_9B_ID: &str = "qwen35-9b-q4km";
const QWEN35_9B_FILE: &str = "Qwen3.5-9B-Q4_K_M.gguf";
const TINYLM_ID: &str = "smollm2-135m-instruct-q4km";
const TINYLM_FILE: &str = "SmolLM2-135M-Instruct-Q4_K_M.gguf";
const DEFAULT_CONTEXT_TOKENS: u32 = 1_024;
const DEFAULT_PROMPT_BATCH: usize = 256;

pub fn default_model_id() -> &'static str {
    QWEN3_06B_ID
}

#[derive(Clone, Copy)]
struct LocalTextModelDefinition {
    id: &'static str,
    file_name: &'static str,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum LocalTextRoute {
    InstantHelper,
    SmartDaily,
    ToolRouter,
    SlowBackup,
}

const LOCAL_TEXT_MODELS: &[LocalTextModelDefinition] = &[
    LocalTextModelDefinition {
        id: QWEN3_06B_ID,
        file_name: QWEN3_06B_FILE,
    },
    LocalTextModelDefinition {
        id: XLAM2_3B_ID,
        file_name: XLAM2_3B_FILE,
    },
    LocalTextModelDefinition {
        id: QWEN35_4B_REVISED_ID,
        file_name: QWEN35_4B_REVISED_FILE,
    },
    LocalTextModelDefinition {
        id: QWEN35_9B_ID,
        file_name: QWEN35_9B_FILE,
    },
    LocalTextModelDefinition {
        id: TINYLM_ID,
        file_name: TINYLM_FILE,
    },
];

struct LocalTextModelInner {
    backend: LlamaBackend,
    model: LlamaModel,
}

#[derive(Clone)]
struct LocalTextModel {
    inner: Arc<Mutex<Option<LocalTextModelInner>>>,
    model_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LocalGenerationMetrics {
    pub generated_tokens: usize,
    pub total_time_ms: u128,
    pub tokens_per_second: f64,
}

#[derive(Debug, Clone)]
pub struct LocalGeneration {
    pub text: String,
    pub metrics: LocalGenerationMetrics,
}

static MODEL_CACHE: OnceLock<Mutex<HashMap<String, LocalTextModel>>> = OnceLock::new();

fn model_cache() -> &'static Mutex<HashMap<String, LocalTextModel>> {
    MODEL_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn data_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("FLOW_DATA_DIR") {
        return PathBuf::from(path);
    }

    #[cfg(target_os = "windows")]
    {
        let g_drive = PathBuf::from(r"G:\Flow\data");
        if g_drive.exists() {
            return g_drive;
        }
    }

    PathBuf::from("data")
}

fn model_dir() -> PathBuf {
    data_dir().join("models").join("llm")
}

fn model_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![model_dir()];

    #[cfg(target_os = "windows")]
    {
        dirs.push(PathBuf::from(r"G:\Flow\data\models\llm"));
        dirs.push(PathBuf::from(r"G:\Workspaces\flow\models\llm"));
    }

    let mut seen = std::collections::HashSet::new();
    dirs.into_iter()
        .filter(|dir| seen.insert(dir.clone()))
        .collect()
}

fn definition(model_id: &str) -> Option<LocalTextModelDefinition> {
    LOCAL_TEXT_MODELS
        .iter()
        .copied()
        .find(|model| model.id.eq_ignore_ascii_case(model_id))
}

pub fn model_path(model_id: &str) -> Option<PathBuf> {
    let definition = definition(model_id)?;
    let candidates: Vec<PathBuf> = model_dirs()
        .into_iter()
        .map(|dir| dir.join(definition.file_name))
        .collect();
    candidates
        .iter()
        .find(|path| is_ready_file(path))
        .cloned()
        .or_else(|| candidates.into_iter().next())
}

pub fn is_model_available(model_id: &str) -> bool {
    model_path(model_id)
        .map(|path| is_ready_file(&path))
        .unwrap_or(false)
}

fn is_ready_file(path: &std::path::Path) -> bool {
    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false)
}

pub fn available_model_ids() -> Vec<String> {
    LOCAL_TEXT_MODELS
        .iter()
        .filter(|model| is_model_available(model.id))
        .map(|model| model.id.to_string())
        .collect()
}

pub fn preferred_model_for_route(
    route: LocalTextRoute,
    configured_model: Option<&str>,
) -> Option<String> {
    let mut ordered: Vec<&str> = match route {
        LocalTextRoute::InstantHelper => vec![QWEN3_06B_ID],
        LocalTextRoute::SmartDaily => vec![QWEN35_4B_REVISED_ID],
        LocalTextRoute::ToolRouter => vec![XLAM2_3B_ID],
        LocalTextRoute::SlowBackup => vec![QWEN35_9B_ID],
    };

    if let Some(model) = configured_model
        .map(str::trim)
        .filter(|model| !model.is_empty())
    {
        ordered.push(model);
    }

    ordered.extend([
        QWEN3_06B_ID,
        QWEN35_4B_REVISED_ID,
        XLAM2_3B_ID,
        QWEN35_9B_ID,
        TINYLM_ID,
    ]);

    let mut seen = std::collections::HashSet::new();
    ordered
        .into_iter()
        .filter(|model| seen.insert(model.to_ascii_lowercase()))
        .find(|model| is_model_available(model))
        .map(str::to_string)
}

fn cached_model(model_id: &str) -> Result<LocalTextModel> {
    let path =
        model_path(model_id).ok_or_else(|| anyhow!("Unknown local text model: {model_id}"))?;
    if !path.exists() {
        return Err(anyhow!("Local text model is missing: {}", path.display()));
    }

    let mut cache = model_cache().lock();
    Ok(cache
        .entry(model_id.to_ascii_lowercase())
        .or_insert_with(|| LocalTextModel {
            inner: Arc::new(Mutex::new(None)),
            model_path: path,
        })
        .clone())
}

pub fn prewarm_blocking(model_id: &str) -> Result<Duration> {
    let model = cached_model(model_id)?;
    let start = Instant::now();
    model.initialize()?;
    Ok(start.elapsed())
}

pub async fn generate(
    model_id: &str,
    system_prompt: &str,
    user_content: &str,
    max_tokens: usize,
    temperature: f32,
) -> Result<String> {
    let output = generate_with_metrics(
        model_id,
        system_prompt,
        user_content,
        max_tokens,
        temperature,
    )
    .await?;
    eprintln!(
        "[LocalTextModel] {model_id} generated {} tokens in {:.2}s @ {:.1} tok/s",
        output.metrics.generated_tokens,
        output.metrics.total_time_ms as f64 / 1000.0,
        output.metrics.tokens_per_second
    );
    Ok(output.text)
}

pub async fn generate_with_metrics(
    model_id: &str,
    system_prompt: &str,
    user_content: &str,
    max_tokens: usize,
    temperature: f32,
) -> Result<LocalGeneration> {
    let model = cached_model(model_id)?;
    let system_prompt = system_prompt.to_string();
    let user_content = user_content.to_string();

    async_runtime::spawn_blocking(move || {
        model.initialize()?;
        model.generate_blocking(&system_prompt, &user_content, max_tokens, temperature)
    })
    .await
    .map_err(|err| anyhow!("Local text generation task failed: {err}"))?
}

impl LocalTextModel {
    fn initialize(&self) -> Result<()> {
        let mut guard = self.inner.lock();
        if guard.is_some() {
            return Ok(());
        }

        let mut backend = LlamaBackend::init().context("Failed to initialize llama backend")?;
        backend.void_logs();

        let model_params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers_from_env());
        let model = LlamaModel::load_from_file(&backend, &self.model_path, &model_params)
            .with_context(|| format!("Failed to load {}", self.model_path.display()))?;

        *guard = Some(LocalTextModelInner { backend, model });
        Ok(())
    }

    fn generate_blocking(
        &self,
        system_prompt: &str,
        user_content: &str,
        max_tokens: usize,
        temperature: f32,
    ) -> Result<LocalGeneration> {
        let start = Instant::now();
        let mut guard = self.inner.lock();
        let inner = guard
            .as_mut()
            .ok_or_else(|| anyhow!("Local text model is not initialized"))?;

        let full_prompt = build_chatml_prompt(system_prompt, user_content);
        let context_tokens = context_tokens_from_env();
        let n_threads = optimal_thread_count();
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(context_tokens))
            .with_n_batch(prompt_batch_from_env() as u32)
            .with_n_threads(n_threads)
            .with_n_threads_batch(n_threads)
            .with_flash_attention_policy(1);

        let mut ctx = inner
            .model
            .new_context(&inner.backend, ctx_params)
            .or_else(|_| {
                let fallback = LlamaContextParams::default()
                    .with_n_ctx(NonZeroU32::new(context_tokens))
                    .with_n_batch(prompt_batch_from_env() as u32)
                    .with_n_threads(n_threads)
                    .with_n_threads_batch(n_threads)
                    .with_flash_attention_policy(0);
                inner.model.new_context(&inner.backend, fallback)
            })?;

        ctx.clear_kv_cache();

        let tokens = inner
            .model
            .str_to_token(&full_prompt, AddBos::Always)
            .context("Tokenization failed")?;
        let available = (context_tokens as usize).saturating_sub(tokens.len());
        let max_tokens = available.min(max_tokens).max(1);

        let mut pos: i32 = 0;
        for chunk in tokens.chunks(prompt_batch_from_env()) {
            let mut batch = LlamaBatch::new(chunk.len(), 1);
            for (index, token) in chunk.iter().copied().enumerate() {
                let logits = index + 1 == chunk.len();
                batch.add(token, pos, &[0], logits)?;
                pos += 1;
            }
            ctx.decode(&mut batch)?;
        }

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(128, 1.08, 0.0, 0.0),
            LlamaSampler::top_k(30),
            LlamaSampler::top_p(0.85, 1),
            LlamaSampler::min_p(0.05, 1),
            LlamaSampler::temp(temperature.max(0.0)),
            LlamaSampler::dist(sampler_seed()),
        ]);
        sampler.accept_many(tokens.iter().copied());

        let generation_start = Instant::now();
        let mut generated_text = String::new();
        let mut generated_tokens = 0usize;
        let mut batch = LlamaBatch::new(1, 1);
        let mut n_cur = tokens.len() as i32;

        for _ in 0..max_tokens {
            if n_cur >= context_tokens as i32 {
                break;
            }

            let token = sampler.sample(&ctx, -1);
            if inner.model.is_eog_token(token) {
                break;
            }

            #[allow(deprecated)]
            let bytes = inner
                .model
                .token_to_bytes(token, llama_cpp_2::model::Special::Tokenize)?;
            generated_text.push_str(&String::from_utf8_lossy(&bytes));
            generated_tokens += 1;

            if should_stop(&generated_text) {
                break;
            }

            batch.clear();
            batch.add(token, n_cur, &[0], true)?;
            n_cur += 1;
            ctx.decode(&mut batch)?;
        }

        let generation_time_ms = generation_start.elapsed().as_millis();
        let tokens_per_second = if generation_time_ms > 0 {
            (generated_tokens as f64 / generation_time_ms as f64) * 1000.0
        } else {
            0.0
        };

        Ok(LocalGeneration {
            text: clean_generated_text(&generated_text),
            metrics: LocalGenerationMetrics {
                generated_tokens,
                total_time_ms: start.elapsed().as_millis(),
                tokens_per_second,
            },
        })
    }
}

fn build_chatml_prompt(system_prompt: &str, user_content: &str) -> String {
    format!(
        "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n/no_think\n{}<|im_end|>\n<|im_start|>assistant\n",
        system_prompt.trim(),
        user_content.trim()
    )
}

fn clean_generated_text(text: &str) -> String {
    let mut result = text.to_string();
    loop {
        let lowered = result.to_ascii_lowercase();
        let Some(start) = lowered.find("<think>") else {
            break;
        };
        if let Some(end) = lowered[start..].find("</think>") {
            result.replace_range(start..start + end + 8, "");
        } else {
            result.replace_range(start.., "");
            break;
        }
    }

    for token in [
        "<|im_end|>",
        "<|endoftext|>",
        "<end_of_turn>",
        "</s>",
        "<|eot_id|>",
    ] {
        result = result.replace(token, "");
    }

    result.trim().to_string()
}

fn should_stop(text: &str) -> bool {
    [
        "<|im_end|>",
        "<|endoftext|>",
        "<end_of_turn>",
        "</s>",
        "<|eot_id|>",
    ]
    .iter()
    .any(|stop| text.contains(stop))
}

fn context_tokens_from_env() -> u32 {
    std::env::var("FLOW_TEXT_LLM_CTX")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_CONTEXT_TOKENS)
}

fn prompt_batch_from_env() -> usize {
    std::env::var("FLOW_TEXT_LLM_BATCH")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_PROMPT_BATCH)
}

fn gpu_layers_from_env() -> u32 {
    std::env::var("FLOW_TEXT_LLM_GPU_LAYERS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

fn optimal_thread_count() -> i32 {
    let logical = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(4);
    logical.saturating_sub(1).clamp(2, 8) as i32
}

fn sampler_seed() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u32)
        .unwrap_or(0xC0DEC0DE)
}
