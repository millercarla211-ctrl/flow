//! Local LLM integration using llama.cpp

use anyhow::{Context, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use sysinfo::System;

use crate::runtime::{
    BrokerRequest, DeviceProfile, Modality, RuntimeBroker, default_activation_config,
    default_model_catalog,
};

const FALLBACK_MODEL_PATH: &str = "models/llm/Qwen3-0.6B-Q4_K_M.gguf";
const FALLBACK_MODEL_KEY: &str = "qwen3-0.6b";
pub const FLOW_CODING_MODEL_KEY: &str = "qwen35-4b-revised-q4km";
pub const FLOW_QUALITY_CHAT_MODEL_KEY: &str = "ministral3-3b-instruct-q4km";
pub const FLOW_TOOL_MODEL_KEY: &str = "xlam2-3b-fc-r-q4km";
pub const FLOW_HELPER_MODEL_KEY: &str = "qwen3-0.6b";

// Available models
pub const AVAILABLE_MODELS: &[(&str, &str)] = &[
    (
        "webgen-4b-preview-i1-q4km",
        "models/llm/WEBGEN-4B-Preview.i1-Q4_K_M.gguf",
    ),
    ("qwendean-4b-q4km", "models/llm/Qwendean-4B.Q4_K_M.gguf"),
    ("qwen35-9b-q4km", "models/llm/Qwen3.5-9B-Q4_K_M.gguf"),
    (
        "qwen35-4b-revised-q4km",
        "models/llm/Qwen3.5-4B-q4_k_m.gguf",
    ),
    (
        "xlam2-3b-fc-r-q4km",
        "models/llm/xLAM-2-3B-fc-r-Q4_K_M.gguf",
    ),
    (
        "ministral3-3b-instruct-q4km",
        "models/llm/Ministral-3-3B-Instruct-2512-Q4_K_M.gguf",
    ),
    (
        "granite4-h-micro-q4km",
        "models/llm/granite-4.0-h-micro-Q4_K_M.gguf",
    ),
    (
        "phi4-mini-instruct-q4km",
        "models/llm/Phi-4-mini-instruct-Q4_K_M.gguf",
    ),
    ("smollm3-3b-q4km", "models/llm/SmolLM3-Q4_K_M.gguf"),
    (
        "gemma4-e4b-frontend-text-q4km",
        "models/llm/gemma-4-E4B-it.Q4_K_M.gguf",
    ),
    (
        "uigen-fx-4b-preview-q4km",
        "models/llm/UIGEN-FX-4B-Preview.Q4_K_M.gguf",
    ),
    (
        "gemma4-2b-q8",
        r"F:\flow\models\llm\gemma-4-E2B-it-Q8_0.gguf",
    ),
    ("qwen3.5-2b", r"F:\flow\models\llm\Qwen3.5-2B-Q4_K_M.gguf"),
    (
        "qwen3.5-0.8b",
        r"F:\flow\models\llm\Qwen3.5-0.8B-Q4_K_M.gguf",
    ),
    ("qwen3-0.6b", "models/llm/Qwen3-0.6B-Q4_K_M.gguf"),
];

const GENERAL_SYSTEM_PROMPT: &str = "\
You are Flow's local AI runtime. Answer directly, stay concise, and avoid hidden reasoning.
Never print chain-of-thought, analysis, <think> blocks, or reasoning traces. Return only the final answer.
";

const CODING_SYSTEM_PROMPT: &str = "\
You are Flow's local coding model for Next.js, React, TypeScript, Tailwind CSS, shadcn/ui, Rust, and local DX automation.
Return practical code or concise implementation guidance. Do not reveal reasoning or planning.
For code requests, prefer complete usable snippets and avoid markdown fences unless the user asks for explanation.
";

const QUALITY_CHAT_SYSTEM_PROMPT: &str = "\
You are Flow's daily smart model for thoughtful everyday answers, synthesis, short reasoning, and calm explanations.
Answer directly and helpfully. Do not reveal hidden reasoning or chain-of-thought.
Use this mode for commercial-safe daily assistant work. Do not use it for strict JSON tool routing; Flow has a dedicated tool router for that.
";

const TOOL_AGENT_SYSTEM_PROMPT: &str = "\
You are Flow's local tool-calling agent brain.
Decide whether a user request needs tools. If tools are available and one or more are needed, return only a single JSON array of calls:
[{\"name\":\"tool_name\",\"arguments\":{\"key\":\"value\"}}]
If no tool is suitable, say that no tool is suitable in one short sentence. If required arguments are missing, ask one concise clarification question.
For normal questions that need no tool, answer directly in plain text. Do not reveal hidden reasoning.
";

const TOOL_AGENT_TASK_INSTRUCTION: &str = "\
You have access to a set of tools. When using tools, make calls in a single JSON array:
[{\"name\":\"tool_call_name\",\"arguments\":{\"arg1\":\"value1\",\"arg2\":\"value2\"}}]
If no tool is suitable, state that explicitly. If the user's input lacks required parameters, ask for clarification.
Do not interpret or respond until tool results are returned. Once tool results are available, process them or make additional calls if needed.
For tasks that do not require tools, respond directly in plain text.
The available tools are:
";

const HELPER_SYSTEM_PROMPT: &str = "\
You are Flow's fastest helper model. Rewrite, compress, expand, classify, clean, and convert text.
Keep outputs short, deterministic, and immediately usable. Do not explain unless asked.
";

const WISPRFLOW_SYSTEM_PROMPT: &str = "\
You are a speech-to-text cleanup engine. Transform raw transcriptions into clean text.

RULES:
1. Remove filler words: um, uh, like, you know, sort of, kind of, basically, actually, literally, maybe
2. Remove false starts: \"I want... no, I need\" → \"I need\"
3. Add proper punctuation and capitalization
4. Handle stuttering: \"I I think\" → \"I think\"
5. Preserve exact meaning - never add new content
6. Output ONLY the cleaned text - NO explanations, NO thinking, NO commentary

EXAMPLES:
Input: \"um so I think we should uh you know maybe consider the the new approach\"
Output: I think we should consider the new approach.

Input: \"first we need to uh set up the database second um configure the API\"
Output: First, we need to set up the database. Second, configure the API.

Input: \"I want... no actually I need the report by friday\"
Output: I need the report by Friday.

CRITICAL: Output ONLY the cleaned text. No <think> tags, no explanations, just the result.
";

const UIGEN_SYSTEM_PROMPT: &str = "\
You are Flow's local UI generation model running as a frontend engineer.
Return only the requested code. Do not explain, do not include reasoning, and do not wrap output in Markdown fences.
Return a complete HTML document with <style> and <body>. Keep CSS short, avoid external scripts, and finish with </html>.
Prefer semantic HTML, responsive CSS, an 8px spacing rhythm, accessible labels, and shadcn/ui-like visual restraint.
";

const INFERENCE_CONTEXT_TOKENS: u32 = 32_768;
const PROMPT_BATCH_SIZE: usize = 512;
const SAMPLER_TEMPERATURE: f32 = 0.7;
const SAMPLER_TOP_P: f32 = 0.92;
const SAMPLER_TOP_K: i32 = 40;
const SAMPLER_MIN_P: f32 = 0.05;
const SAMPLER_REPEAT_LAST_N: i32 = 256;
const SAMPLER_REPEAT_PENALTY: f32 = 1.10;
const QWEN_NO_THINK_PREFIX: &str = "/no_think";
const QWEN_NO_THINK_SUFFIX: &str =
    "No planning. No explanation. Do not output <think> tags. Answer directly.";
const QWEN_RETRY_SUFFIX: &str = "The previous output was empty after hidden reasoning was removed. Start the final answer immediately. Do not use <think> tags. If this is code, output complete code only.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LocalLlmPromptFormat {
    ChatMl,
    Gemma,
    Llama3,
    MistralV7Tekken,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalLlmConfig {
    pub system_prompt: String,
    pub context_tokens: u32,
    pub max_new_tokens: usize,
    pub prompt_batch_size: usize,
    pub stop_sequences: Vec<String>,
    pub n_gpu_layers: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub min_p: f32,
    pub repeat_last_n: i32,
    pub repeat_penalty: f32,
}

impl LocalLlmConfig {
    pub fn general() -> Self {
        Self {
            system_prompt: GENERAL_SYSTEM_PROMPT.to_string(),
            context_tokens: llm_context_tokens_from_env().unwrap_or(INFERENCE_CONTEXT_TOKENS),
            max_new_tokens: llm_max_tokens_from_env().unwrap_or(4096),
            prompt_batch_size: llm_prompt_batch_size_from_env().unwrap_or(PROMPT_BATCH_SIZE),
            stop_sequences: Vec::new(),
            n_gpu_layers: gpu_layers_from_env().unwrap_or(0),
            temperature: SAMPLER_TEMPERATURE,
            top_p: SAMPLER_TOP_P,
            top_k: SAMPLER_TOP_K,
            min_p: SAMPLER_MIN_P,
            repeat_last_n: SAMPLER_REPEAT_LAST_N,
            repeat_penalty: SAMPLER_REPEAT_PENALTY,
        }
    }

    pub fn speech_cleanup() -> Self {
        Self {
            system_prompt: WISPRFLOW_SYSTEM_PROMPT.to_string(),
            temperature: 0.25,
            top_p: 0.85,
            max_new_tokens: 1024,
            ..Self::general()
        }
    }

    pub fn coding() -> Self {
        Self {
            system_prompt: CODING_SYSTEM_PROMPT.to_string(),
            context_tokens: llm_context_tokens_from_env().unwrap_or(8_192),
            max_new_tokens: llm_max_tokens_from_env().unwrap_or(2_048),
            temperature: 0.35,
            top_p: 0.9,
            repeat_penalty: 1.08,
            ..Self::general()
        }
    }

    pub fn quality_chat() -> Self {
        Self {
            system_prompt: QUALITY_CHAT_SYSTEM_PROMPT.to_string(),
            context_tokens: llm_context_tokens_from_env().unwrap_or(8_192),
            max_new_tokens: llm_max_tokens_from_env().unwrap_or(1_536),
            temperature: 0.15,
            top_p: 0.9,
            repeat_penalty: 1.05,
            ..Self::general()
        }
    }

    pub fn tool_agent() -> Self {
        Self {
            system_prompt: TOOL_AGENT_SYSTEM_PROMPT.to_string(),
            context_tokens: llm_context_tokens_from_env().unwrap_or(8_192),
            max_new_tokens: llm_max_tokens_from_env().unwrap_or(192),
            prompt_batch_size: llm_prompt_batch_size_from_env().unwrap_or(256),
            stop_sequences: vec![
                "|||<|end_of_binding|>".to_string(),
                "<|end_of_binding|>".to_string(),
            ],
            temperature: 0.1,
            top_p: 0.85,
            top_k: 40,
            min_p: 0.02,
            repeat_penalty: 1.05,
            ..Self::general()
        }
    }

    pub fn helper() -> Self {
        Self {
            system_prompt: HELPER_SYSTEM_PROMPT.to_string(),
            context_tokens: llm_context_tokens_from_env().unwrap_or(2_048),
            max_new_tokens: llm_max_tokens_from_env().unwrap_or(512),
            prompt_batch_size: llm_prompt_batch_size_from_env().unwrap_or(256),
            temperature: 0.25,
            top_p: 0.85,
            repeat_penalty: 1.05,
            ..Self::general()
        }
    }

    pub fn uigen() -> Self {
        Self {
            system_prompt: UIGEN_SYSTEM_PROMPT.to_string(),
            context_tokens: 8_192,
            max_new_tokens: uigen_max_tokens_from_env().unwrap_or(2_500),
            prompt_batch_size: 256,
            stop_sequences: vec!["</html>".to_string()],
            temperature: 0.55,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.12,
            ..Self::general()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GenerationMetrics {
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub total_time_ms: u128,
    pub tokens_per_second: f64,
    pub prompt_eval_time_ms: u128,
    pub generation_time_ms: u128,
}

#[derive(Clone)]
struct Message {
    role: String,
    content: String,
}

struct LocalLlmInner {
    backend: LlamaBackend,
    model: LlamaModel,
    history: Vec<Message>,
}

#[derive(Clone)]
pub struct LocalLlm {
    inner: Arc<Mutex<Option<LocalLlmInner>>>,
    model_path: String,
    config: LocalLlmConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowModelRole {
    pub role: &'static str,
    pub model_key: &'static str,
    pub model_path: &'static str,
    pub purpose: &'static str,
}

pub const FLOW_MODEL_ROLES: &[FlowModelRole] = &[
    FlowModelRole {
        role: "coding",
        model_key: FLOW_CODING_MODEL_KEY,
        model_path: "models/llm/Qwen3.5-4B-q4_k_m.gguf",
        purpose: "default local coding brain for shadcn/ui, Next.js, Tailwind, Rust, and DX edits",
    },
    FlowModelRole {
        role: "quality-chat",
        model_key: FLOW_QUALITY_CHAT_MODEL_KEY,
        model_path: "models/llm/Ministral-3-3B-Instruct-2512-Q4_K_M.gguf",
        purpose: "commercial-safe daily smart assistant brain for normal chat, synthesis, and low-latency reasoning",
    },
    FlowModelRole {
        role: "tool-agent",
        model_key: FLOW_TOOL_MODEL_KEY,
        model_path: "models/llm/xLAM-2-3B-fc-r-Q4_K_M.gguf",
        purpose: "small local research model for strict JSON tool routing and function-call decisions",
    },
    FlowModelRole {
        role: "helper",
        model_key: FLOW_HELPER_MODEL_KEY,
        model_path: "models/llm/Qwen3-0.6B-Q4_K_M.gguf",
        purpose: "fastest helper for prompt enhancement, text cleanup, conversion, labels, and tiny rewrites",
    },
];

impl LocalLlm {
    pub fn new() -> Self {
        Self::for_device_profile(crate::utils::detect_device_profile())
    }

    pub fn with_model_path(model_path: String) -> Self {
        Self::with_config(model_path, LocalLlmConfig::general())
    }

    pub fn with_config(model_path: String, config: LocalLlmConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
            model_path,
            config,
        }
    }

    pub fn for_coding() -> Self {
        Self::with_config(
            Self::model_path_for_key(FLOW_CODING_MODEL_KEY)
                .unwrap_or_else(|| "models/llm/Qwen3.5-4B-q4_k_m.gguf".to_string()),
            LocalLlmConfig::coding(),
        )
    }

    pub fn for_quality_chat() -> Self {
        Self::with_config(
            Self::model_path_for_key(FLOW_QUALITY_CHAT_MODEL_KEY).unwrap_or_else(|| {
                "models/llm/Ministral-3-3B-Instruct-2512-Q4_K_M.gguf".to_string()
            }),
            LocalLlmConfig::quality_chat(),
        )
    }

    pub fn for_tool_agent() -> Self {
        Self::with_config(
            Self::model_path_for_key(FLOW_TOOL_MODEL_KEY)
                .unwrap_or_else(|| "models/llm/xLAM-2-3B-fc-r-Q4_K_M.gguf".to_string()),
            LocalLlmConfig::tool_agent(),
        )
    }

    pub fn for_helper() -> Self {
        Self::with_config(
            Self::model_path_for_key(FLOW_HELPER_MODEL_KEY)
                .unwrap_or_else(|| FALLBACK_MODEL_PATH.to_string()),
            LocalLlmConfig::helper(),
        )
    }

    pub fn model_path_for_key(model_key: &str) -> Option<String> {
        AVAILABLE_MODELS
            .iter()
            .find(|(key, _path)| *key == model_key)
            .map(|(_key, path)| (*path).to_string())
            .or_else(|| {
                FLOW_MODEL_ROLES
                    .iter()
                    .find(|role| role.model_key == model_key)
                    .map(|role| role.model_path.to_string())
            })
    }

    pub fn model_roles() -> &'static [FlowModelRole] {
        FLOW_MODEL_ROLES
    }

    pub fn for_device_profile(device_profile: DeviceProfile) -> Self {
        Self::with_model_path(Self::recommended_model_path_for_device(&device_profile))
    }

    pub fn recommended_model_key() -> String {
        Self::recommended_model_key_for_device(&crate::utils::detect_device_profile())
    }

    pub fn recommended_model_key_for_device(device_profile: &DeviceProfile) -> String {
        let broker = RuntimeBroker::from_parts(
            device_profile.clone(),
            default_model_catalog(),
            default_activation_config(),
        );
        let mut request = BrokerRequest::new(Modality::Chat);
        request.allow_conversion = false;
        request.allow_publish = false;

        broker
            .build_plan(request)
            .selected_model
            .unwrap_or_else(|| FALLBACK_MODEL_KEY.to_string())
    }

    pub fn recommended_model_path() -> String {
        Self::recommended_model_path_for_device(&crate::utils::detect_device_profile())
    }

    pub fn recommended_model_path_for_device(device_profile: &DeviceProfile) -> String {
        let selected_key = Self::recommended_model_key_for_device(device_profile);
        default_model_catalog()
            .into_iter()
            .find(|manifest| manifest.key == selected_key)
            .and_then(|manifest| manifest.local_path)
            .unwrap_or_else(|| FALLBACK_MODEL_PATH.to_string())
    }

    pub async fn initialize(&self) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

        if inner.is_some() {
            return Ok(());
        }

        let mut backend = LlamaBackend::init().context("Failed to initialize llama backend")?;
        backend.void_logs();

        let model_params = LlamaModelParams::default().with_n_gpu_layers(self.config.n_gpu_layers);
        let model = LlamaModel::load_from_file(&backend, &self.model_path, &model_params).context(
            format!("Failed to load model from path: {}", &self.model_path),
        )?;

        *inner = Some(LocalLlmInner {
            backend,
            model,
            history: Vec::new(),
        });

        Ok(())
    }

    pub async fn generate_with_metrics(&self, prompt: &str) -> Result<(String, GenerationMetrics)> {
        self.generate_with_task_config(prompt, self.config.clone(), true)
            .await
    }

    pub async fn generate_coding_with_metrics(
        &self,
        prompt: &str,
    ) -> Result<(String, GenerationMetrics)> {
        self.generate_with_task_config(prompt, LocalLlmConfig::coding(), true)
            .await
    }

    pub async fn generate_quality_chat_with_metrics(
        &self,
        prompt: &str,
    ) -> Result<(String, GenerationMetrics)> {
        self.generate_with_task_config(prompt, LocalLlmConfig::quality_chat(), true)
            .await
    }

    pub async fn generate_tool_agent_with_metrics(
        &self,
        prompt: &str,
    ) -> Result<(String, GenerationMetrics)> {
        let (response, metrics) = self
            .generate_with_task_config(prompt, LocalLlmConfig::tool_agent(), true)
            .await?;
        Ok((Self::strip_tool_agent_tail(&response), metrics))
    }

    pub async fn generate_tool_call_with_metrics(
        &self,
        available_tools_json: &str,
        user_request: &str,
    ) -> Result<(String, GenerationMetrics)> {
        let mut config = LocalLlmConfig::tool_agent();
        config.system_prompt = Self::tool_agent_system_prompt(available_tools_json);
        let (response, metrics) = self
            .generate_with_task_config(user_request, config, false)
            .await?;
        Ok((Self::strip_tool_agent_tail(&response), metrics))
    }

    pub async fn generate_helper_with_metrics(
        &self,
        prompt: &str,
    ) -> Result<(String, GenerationMetrics)> {
        self.generate_with_task_config(prompt, LocalLlmConfig::helper(), false)
            .await
    }

    async fn generate_once_with_metrics(
        &self,
        prompt: &str,
        config: LocalLlmConfig,
    ) -> Result<(String, GenerationMetrics)> {
        self.generate_with_task_config(prompt, config, false).await
    }

    async fn generate_with_task_config(
        &self,
        prompt: &str,
        config: LocalLlmConfig,
        remember_history: bool,
    ) -> Result<(String, GenerationMetrics)> {
        let config = self.adapt_config_for_model(config);
        let guarded_prompt = self.prepare_user_prompt(prompt, false);
        let first = self
            .generate_with_task_config_once(&guarded_prompt, config.clone(), remember_history)
            .await?;

        if self.uses_qwen_no_think_guard() && Self::needs_retry_after_sanitizing(&first.0) {
            if remember_history {
                self.pop_last_user_message(&guarded_prompt)?;
            }

            let retry_prompt = self.prepare_user_prompt(prompt, true);
            let mut retry_config = config;
            retry_config.temperature = retry_config.temperature.min(0.25);
            retry_config.top_p = retry_config.top_p.min(0.85);
            retry_config.repeat_penalty = 1.08;

            let retry = self
                .generate_with_task_config_once(&retry_prompt, retry_config, remember_history)
                .await?;

            if !Self::needs_retry_after_sanitizing(&retry.0) || first.0.trim().is_empty() {
                return Ok(retry);
            }
        }

        Ok(first)
    }

    async fn generate_with_task_config_once(
        &self,
        prompt: &str,
        config: LocalLlmConfig,
        remember_history: bool,
    ) -> Result<(String, GenerationMetrics)> {
        let start_time = Instant::now();

        let mut inner_guard = self
            .inner
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let inner = inner_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("LLM not initialized"))?;

        let full_prompt = if remember_history {
            inner.history.push(Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            });
            self.build_prompt(&inner.history, &config.system_prompt)
        } else {
            self.build_single_turn_prompt(prompt, &config.system_prompt)
        };

        let n_threads = Self::optimal_thread_count();
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(config.context_tokens))
            .with_n_batch(config.prompt_batch_size as u32)
            .with_n_threads(n_threads)
            .with_n_threads_batch(n_threads)
            .with_flash_attention_policy(1);

        let mut ctx = inner.model.new_context(&inner.backend, ctx_params.clone()).or_else(|_e| {
            #[cfg(debug_assertions)]
            eprintln!("Warning: Flash attention context creation failed, falling back to standard attention");
            let fallback_params = LlamaContextParams::default()
                .with_n_ctx(NonZeroU32::new(config.context_tokens))
                .with_n_batch(config.prompt_batch_size as u32)
                .with_n_threads(n_threads)
                .with_n_threads_batch(n_threads)
                .with_flash_attention_policy(0);
            inner.model.new_context(&inner.backend, fallback_params)
        }).context("Failed to create inference context")?;

        ctx.clear_kv_cache();

        let add_bos = if matches!(self.prompt_format(), LocalLlmPromptFormat::Llama3) {
            AddBos::Never
        } else {
            AddBos::Always
        };
        let tokens = inner
            .model
            .str_to_token(&full_prompt, add_bos)
            .context("Tokenization failed")?;

        let prompt_tokens = tokens.len();
        let available = (config.context_tokens as usize).saturating_sub(tokens.len());
        let max_tokens = available.min(config.max_new_tokens);

        // Batched prompt evaluation
        let prompt_eval_start = Instant::now();
        let mut pos: i32 = 0;
        let total = tokens.len();
        let mut offset = 0;

        while offset < total {
            let end = (offset + config.prompt_batch_size).min(total);
            let chunk = &tokens[offset..end];
            let is_last_chunk = end == total;

            let mut batch = LlamaBatch::new(chunk.len(), 1);
            for (i, &token) in chunk.iter().enumerate() {
                let logits = is_last_chunk && i == chunk.len() - 1;
                batch.add(token, pos, &[0], logits)?;
                pos += 1;
            }
            ctx.decode(&mut batch)?;
            offset = end;
        }
        let prompt_eval_time_ms = prompt_eval_start.elapsed().as_millis();

        // Sampler chain
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(config.repeat_last_n, config.repeat_penalty, 0.0, 0.0),
            LlamaSampler::top_k(config.top_k),
            LlamaSampler::top_p(config.top_p, 1),
            LlamaSampler::min_p(config.min_p, 1),
            LlamaSampler::temp(config.temperature),
            LlamaSampler::dist(Self::sampler_seed()),
        ]);
        sampler.accept_many(tokens.iter().copied());

        // Generation loop
        let generation_start = Instant::now();
        let mut n_cur = tokens.len() as i32;
        let mut generated_text = String::with_capacity(max_tokens * 4);
        let mut gen_batch = LlamaBatch::new(1, 1);
        let mut generated_tokens = 0;

        let mut hit_limit = false;
        let mut extra_tokens = 0;
        let max_loop = max_tokens + 50;

        for i in 0..max_loop {
            if i >= max_tokens {
                hit_limit = true;
            }
            if n_cur >= config.context_tokens as i32 {
                break;
            }

            let token = sampler.sample(&ctx, -1);

            if inner.model.is_eog_token(token) {
                break;
            }

            #[allow(deprecated)]
            let piece_bytes = inner
                .model
                .token_to_bytes(token, llama_cpp_2::model::Special::Tokenize)?;
            let piece = String::from_utf8_lossy(&piece_bytes);
            generated_text.push_str(&piece);
            generated_tokens += 1;

            if config
                .stop_sequences
                .iter()
                .any(|stop| generated_text.contains(stop))
            {
                break;
            }

            gen_batch.clear();
            gen_batch.add(token, n_cur, &[0], true)?;
            n_cur += 1;

            ctx.decode(&mut gen_batch)?;

            if hit_limit {
                let last_char = piece.chars().last().unwrap_or(' ');
                if last_char == '.' || last_char == '?' || last_char == '!' || piece.contains('\n')
                {
                    break;
                }
                extra_tokens += 1;
                if extra_tokens >= 50 {
                    generated_text.push_str("...");
                    break;
                }
            }
        }

        let generation_time_ms = generation_start.elapsed().as_millis();
        let total_time_ms = start_time.elapsed().as_millis();

        let tokens_per_second = if generation_time_ms > 0 {
            (generated_tokens as f64 / generation_time_ms as f64) * 1000.0
        } else {
            0.0
        };

        let answer = Self::strip_thinking_tags(&generated_text);
        if remember_history && !answer.is_empty() {
            inner.history.push(Message {
                role: "assistant".to_string(),
                content: answer.clone(),
            });
        }

        let metrics = GenerationMetrics {
            prompt_tokens,
            generated_tokens,
            total_time_ms,
            tokens_per_second,
            prompt_eval_time_ms,
            generation_time_ms,
        };

        Ok((answer, metrics))
    }

    #[allow(dead_code)]
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let (response, _) = self.generate_with_metrics(prompt).await?;
        Ok(response)
    }

    /// Clean up raw speech transcription WisprFlow-style
    /// Removes filler words, adds punctuation, fixes formatting
    pub async fn clean_speech(&self, raw_transcription: &str) -> Result<String> {
        let (cleaned, _) = self.clean_speech_with_metrics(raw_transcription).await?;
        Ok(cleaned)
    }

    /// Clean raw speech transcription and return generation metrics.
    pub async fn clean_speech_with_metrics(
        &self,
        raw_transcription: &str,
    ) -> Result<(String, GenerationMetrics)> {
        let prompt = format!("Clean this speech:\n\n{}", raw_transcription);
        let (cleaned, metrics) = self
            .generate_once_with_metrics(&prompt, LocalLlmConfig::speech_cleanup())
            .await?;

        // Strip <think> tags if present (Qwen models sometimes output reasoning)
        let cleaned = Self::strip_thinking_tags(&cleaned);

        Ok((cleaned, metrics))
    }

    pub async fn generate_ui_with_metrics(
        &self,
        prompt: &str,
    ) -> Result<(String, GenerationMetrics)> {
        let (generated, metrics) = self
            .generate_once_with_metrics(prompt, LocalLlmConfig::uigen())
            .await?;
        Ok((Self::strip_thinking_tags(&generated), metrics))
    }

    /// Strip <think>...</think> tags from model output
    pub fn strip_thinking_tags(text: &str) -> String {
        let mut result = text.to_string();

        // Remove <think>...</think> blocks. If generation stopped mid-block,
        // drop the unfinished reasoning tail instead of leaking it to callers.
        loop {
            let lower = result.to_ascii_lowercase();
            let Some(start) = lower.find("<think>") else {
                break;
            };

            if let Some(end) = lower[start..].find("</think>") {
                result.replace_range(start..start + end + 8, "");
            } else {
                result.replace_range(start.., "");
                break;
            }
        }

        loop {
            let lower = result.to_ascii_lowercase();
            let Some(end) = lower.find("</think>") else {
                break;
            };
            result.replace_range(..end + 8, "");
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

    fn strip_tool_agent_tail(text: &str) -> String {
        let mut result = text.to_string();

        for marker in ["|||<|end_of_binding|>", "<|end_of_binding|>"] {
            if let Some(index) = result.find(marker) {
                result.truncate(index);
            }
        }

        if let Some(json) = Self::extract_leading_json_array(result.trim()) {
            return json;
        }

        result.trim().to_string()
    }

    fn extract_leading_json_array(text: &str) -> Option<String> {
        if !text.starts_with('[') {
            return None;
        }

        let mut depth = 0_i32;
        let mut in_string = false;
        let mut escaped = false;

        for (index, ch) in text.char_indices() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '"' => in_string = true,
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(text[..=index].trim().to_string());
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Apply a voice command to text (e.g., "make this professional", "format as list")
    pub async fn apply_command(&self, text: &str, command: &str) -> Result<String> {
        let prompt = format!(
            "Text: {}\n\nCommand: {}\n\nApply the command to the text:",
            text, command
        );
        let (result, _) = self.generate_with_metrics(&prompt).await?;
        Ok(result)
    }

    pub async fn generate_stream<F>(&self, prompt: &str, callback: F) -> Result<()>
    where
        F: Fn(String) + Send + 'static,
    {
        let (response, _) = self.generate_with_metrics(prompt).await?;
        callback(response);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_initialized(&self) -> bool {
        self.inner
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    pub fn clear_history(&self) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

        if let Some(inner) = inner.as_mut() {
            inner.history.clear();
        }

        Ok(())
    }

    pub fn model_path(&self) -> &str {
        &self.model_path
    }

    #[allow(dead_code)]
    pub fn get_model_name(&self) -> String {
        let display = Path::new(&self.model_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(FALLBACK_MODEL_KEY);
        format!("Local:{}", display)
    }

    fn build_prompt(&self, history: &[Message], system_prompt: &str) -> String {
        match self.prompt_format() {
            LocalLlmPromptFormat::ChatMl => Self::build_chatml_prompt(history, system_prompt),
            LocalLlmPromptFormat::Gemma => Self::build_gemma_prompt(history, system_prompt),
            LocalLlmPromptFormat::Llama3 => Self::build_llama3_prompt(history, system_prompt),
            LocalLlmPromptFormat::MistralV7Tekken => {
                Self::build_mistral_v7_tekken_prompt(history, system_prompt)
            }
        }
    }

    fn build_chatml_prompt(history: &[Message], system_prompt: &str) -> String {
        let mut prompt = String::with_capacity(4096);
        prompt.push_str("<|im_start|>system\n");
        prompt.push_str(system_prompt);
        prompt.push_str("<|im_end|>\n");

        for msg in history {
            prompt.push_str("<|im_start|>");
            prompt.push_str(&msg.role);
            prompt.push('\n');
            prompt.push_str(&msg.content);
            prompt.push_str("<|im_end|>\n");
        }

        prompt.push_str("<|im_start|>assistant\n");
        prompt
    }

    fn build_gemma_prompt(history: &[Message], system_prompt: &str) -> String {
        let mut prompt = String::with_capacity(4096);
        let mut injected_system = false;

        for msg in history {
            if msg.role == "assistant" {
                prompt.push_str("<start_of_turn>model\n");
                prompt.push_str(&msg.content);
                prompt.push_str("<end_of_turn>\n");
                continue;
            }

            prompt.push_str("<start_of_turn>user\n");
            if !injected_system && !system_prompt.trim().is_empty() {
                prompt.push_str(system_prompt.trim());
                prompt.push_str("\n\n");
                injected_system = true;
            }
            prompt.push_str(&msg.content);
            prompt.push_str("<end_of_turn>\n");
        }

        prompt.push_str("<start_of_turn>model\n");
        prompt
    }

    fn build_llama3_prompt(history: &[Message], system_prompt: &str) -> String {
        let mut prompt = String::with_capacity(4096);
        prompt.push_str("<|begin_of_text|><|start_header_id|>system<|end_header_id|>\n\n");
        prompt.push_str(system_prompt.trim());
        prompt.push_str("<|eot_id|>");

        for msg in history {
            let role = if msg.role == "assistant" {
                "assistant"
            } else {
                "user"
            };
            prompt.push_str("<|start_header_id|>");
            prompt.push_str(role);
            prompt.push_str("<|end_header_id|>\n\n");
            prompt.push_str(&msg.content);
            prompt.push_str("<|eot_id|>");
        }

        prompt.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");
        prompt
    }

    fn build_mistral_v7_tekken_prompt(history: &[Message], system_prompt: &str) -> String {
        let mut prompt = String::with_capacity(4096);
        if !system_prompt.trim().is_empty() {
            prompt.push_str("[SYSTEM_PROMPT]");
            prompt.push_str(system_prompt.trim());
            prompt.push_str("[/SYSTEM_PROMPT]");
        }

        for msg in history {
            if msg.role == "assistant" {
                prompt.push_str(&msg.content);
                prompt.push_str("</s>");
                continue;
            }

            prompt.push_str("[INST]");
            prompt.push_str(msg.content.trim());
            prompt.push_str("[/INST]");
        }

        prompt
    }

    fn tool_agent_system_prompt(available_tools_json: &str) -> String {
        let tools = available_tools_json.trim();
        let tools = if tools.is_empty() { "[]" } else { tools };
        format!("{TOOL_AGENT_TASK_INSTRUCTION}\n{tools}")
    }

    fn build_single_turn_prompt(&self, user_prompt: &str, system_prompt: &str) -> String {
        self.build_prompt(
            &[Message {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            }],
            system_prompt,
        )
    }

    fn prompt_format(&self) -> LocalLlmPromptFormat {
        let path = self.model_path.to_ascii_lowercase();
        if path.contains("gemma") {
            LocalLlmPromptFormat::Gemma
        } else if path.contains("xlam") || path.contains("llama") {
            LocalLlmPromptFormat::Llama3
        } else if path.contains("ministral") || path.contains("mistral") {
            LocalLlmPromptFormat::MistralV7Tekken
        } else {
            LocalLlmPromptFormat::ChatMl
        }
    }

    fn uses_qwen_no_think_guard(&self) -> bool {
        let path = self.model_path.to_ascii_lowercase();
        path.contains("qwen3-0.6b") || path.contains("qwen3.5-4b") || path.contains("qwen35-4b")
    }

    fn uses_mistral_daily_driver_settings(&self) -> bool {
        let path = self.model_path.to_ascii_lowercase();
        path.contains("ministral") || path.contains("mistral")
    }

    fn prepare_user_prompt(&self, prompt: &str, retry: bool) -> String {
        if !self.uses_qwen_no_think_guard() || prompt.trim_start().starts_with(QWEN_NO_THINK_PREFIX)
        {
            return prompt.to_string();
        }

        let suffix = if retry {
            QWEN_RETRY_SUFFIX
        } else {
            QWEN_NO_THINK_SUFFIX
        };

        format!("{QWEN_NO_THINK_PREFIX}\n{}\n{suffix}", prompt.trim())
    }

    fn adapt_config_for_model(&self, mut config: LocalLlmConfig) -> LocalLlmConfig {
        for stop in self.model_stop_sequences() {
            if !config.stop_sequences.iter().any(|item| item == stop) {
                config.stop_sequences.push(stop.to_string());
            }
        }

        if self.uses_qwen_no_think_guard() {
            config.temperature = config.temperature.min(0.35);
            config.top_p = config.top_p.min(0.9);
            config.repeat_penalty = 1.08;
        }

        if self.uses_mistral_daily_driver_settings() {
            config.temperature = config.temperature.min(0.15);
            config.top_p = config.top_p.min(0.9);
            config.repeat_penalty = 1.05;
        }

        config
    }

    fn model_stop_sequences(&self) -> &'static [&'static str] {
        match self.prompt_format() {
            LocalLlmPromptFormat::ChatMl => &["<|im_end|>", "<|endoftext|>"],
            LocalLlmPromptFormat::Gemma => &["<end_of_turn>"],
            LocalLlmPromptFormat::Llama3 => &["<|eot_id|>", "<|end_of_text|>"],
            LocalLlmPromptFormat::MistralV7Tekken => &["</s>", "[INST]", "[/INST]"],
        }
    }

    fn needs_retry_after_sanitizing(answer: &str) -> bool {
        answer.trim().is_empty()
    }

    fn pop_last_user_message(&self, expected_content: &str) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

        if let Some(inner) = inner.as_mut()
            && inner.history.last().is_some_and(|message| {
                message.role == "user" && message.content == expected_content
            })
        {
            inner.history.pop();
        }

        Ok(())
    }

    fn optimal_thread_count() -> i32 {
        let sys = System::new_all();
        let physical = sys.physical_core_count().unwrap_or(1).max(1);
        if physical > 4 {
            (physical - 1) as i32
        } else {
            physical as i32
        }
    }

    fn sampler_seed() -> u32 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u32)
            .unwrap_or(0xDEAD_BEEF)
    }
}

fn gpu_layers_from_env() -> Option<u32> {
    std::env::var("FLOW_LLAMA_GPU_LAYERS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
}

fn uigen_max_tokens_from_env() -> Option<usize> {
    std::env::var("FLOW_UIGEN_MAX_TOKENS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
}

fn llm_context_tokens_from_env() -> Option<u32> {
    std::env::var("FLOW_LLM_CTX")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
}

fn llm_max_tokens_from_env() -> Option<usize> {
    std::env::var("FLOW_LLM_MAX_TOKENS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
}

fn llm_prompt_batch_size_from_env() -> Option<usize> {
    std::env::var("FLOW_LLM_BATCH")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
}

impl Default for LocalLlm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{ComputeBackend, DeviceTier, GraphicsDevice};

    fn test_profile(total_memory_bytes: u64) -> DeviceProfile {
        DeviceProfile {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            cpu_model: "Test CPU".to_string(),
            physical_cores: 4,
            logical_cores: 8,
            total_memory_bytes,
            available_memory_bytes: total_memory_bytes,
            battery_powered: None,
            thermal_class: None,
            graphics: vec![GraphicsDevice {
                name: "Integrated GPU".to_string(),
                vendor: Some("intel".to_string()),
                vram_bytes: None,
                integrated: true,
                backends: vec![ComputeBackend::Cpu],
            }],
            tier: if total_memory_bytes < 8 * 1024 * 1024 * 1024 {
                DeviceTier::Low
            } else {
                DeviceTier::Balanced
            },
        }
    }

    #[test]
    fn low_end_profiles_default_to_qwen3() {
        let key = LocalLlm::recommended_model_key_for_device(&test_profile(6 * 1024 * 1024 * 1024));
        assert_eq!(key, "qwen3-0.6b");
    }

    #[test]
    fn flow_model_roles_match_local_model_policy() {
        let roles = LocalLlm::model_roles();
        assert_eq!(roles.len(), 4);
        assert_eq!(roles[0].model_key, FLOW_CODING_MODEL_KEY);
        assert_eq!(roles[1].model_key, FLOW_QUALITY_CHAT_MODEL_KEY);
        assert_eq!(roles[2].model_key, FLOW_TOOL_MODEL_KEY);
        assert_eq!(roles[3].model_key, FLOW_HELPER_MODEL_KEY);
        assert_eq!(
            LocalLlm::model_path_for_key(FLOW_HELPER_MODEL_KEY).as_deref(),
            Some("models/llm/Qwen3-0.6B-Q4_K_M.gguf")
        );
    }

    #[test]
    fn qwen35_4b_prompts_are_forced_into_no_think_mode() {
        let llm = LocalLlm::with_model_path("models/llm/Qwen3.5-4B-q4_k_m.gguf".to_string());
        let prompt = llm.prepare_user_prompt("Write a UsageCard.", false);

        assert!(prompt.starts_with("/no_think\n"));
        assert!(prompt.contains("Do not output <think> tags"));
    }

    #[test]
    fn qwen3_06b_helper_prompts_are_forced_into_no_think_mode() {
        let llm = LocalLlm::with_model_path("models/llm/Qwen3-0.6B-Q4_K_M.gguf".to_string());
        let prompt = llm.prepare_user_prompt("Clean this text.", false);

        assert!(prompt.starts_with("/no_think\n"));
        assert!(prompt.contains("Do not output <think> tags"));
    }

    #[test]
    fn gemma_models_use_gemma_turn_format() {
        let llm = LocalLlm::with_model_path("models/llm/gemma-4-E4B-it.Q4_K_M.gguf".to_string());
        let prompt = llm.build_single_turn_prompt("Hello", "System rules");

        assert!(prompt.contains("<start_of_turn>user\n"));
        assert!(prompt.contains("System rules\n\nHello<end_of_turn>"));
        assert!(prompt.ends_with("<start_of_turn>model\n"));
        assert!(!prompt.contains("<|im_start|>"));
    }

    #[test]
    fn xlam_models_use_llama3_tool_turn_format() {
        let llm = LocalLlm::with_model_path("models/llm/xLAM-2-3B-fc-r-Q4_K_M.gguf".to_string());
        let prompt = llm.build_single_turn_prompt("Call the weather tool.", "System rules");

        assert!(prompt.starts_with("<|begin_of_text|><|start_header_id|>system"));
        assert!(prompt.contains("System rules<|eot_id|>"));
        assert!(
            prompt.contains("<|start_header_id|>user<|end_header_id|>\n\nCall the weather tool.")
        );
        assert!(prompt.ends_with("<|start_header_id|>assistant<|end_header_id|>\n\n"));
        assert!(!prompt.contains("<|im_start|>"));
    }

    #[test]
    fn ministral_models_use_mistral_v7_tekken_format() {
        let llm = LocalLlm::with_model_path(
            "models/llm/Ministral-3-3B-Instruct-2512-Q4_K_M.gguf".to_string(),
        );
        let prompt = llm.build_single_turn_prompt("Hello", "System rules");

        assert!(prompt.starts_with("[SYSTEM_PROMPT]System rules[/SYSTEM_PROMPT]"));
        assert!(prompt.contains("[INST]Hello[/INST]"));
        assert!(!prompt.contains("<|im_start|>"));
        assert!(!prompt.contains("<start_of_turn>"));
    }

    #[test]
    fn xlam_tool_agent_system_prompt_embeds_tools() {
        let prompt = LocalLlm::tool_agent_system_prompt(
            r#"[{"name":"get_weather","parameters":{"type":"object"}}]"#,
        );

        assert!(prompt.contains("make calls in a single JSON array"));
        assert!(prompt.contains(r#""name":"get_weather""#));
    }

    #[test]
    fn tool_agent_output_is_trimmed_to_first_json_call_array() {
        let output = r#"[{"name":"get_weather","arguments":{"city":"Dhaka","date":"tomorrow"}}]|||<|end_of_binding|> trailing prose"#;
        assert_eq!(
            LocalLlm::strip_tool_agent_tail(output),
            r#"[{"name":"get_weather","arguments":{"city":"Dhaka","date":"tomorrow"}}]"#
        );
    }

    #[test]
    fn sanitizer_removes_orphan_think_tail_and_model_tokens() {
        let cleaned =
            LocalLlm::strip_thinking_tags("Thinking Process\n</think>\nfinal answer<end_of_turn>");
        assert_eq!(cleaned, "final answer");
    }
}
