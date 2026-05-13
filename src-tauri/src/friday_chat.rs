use serde::{Deserialize, Serialize};

use crate::{local_text_model, AppState};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FridayChatContextItem {
    label: String,
    kind: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FridayChatContextPayload {
    #[serde(default)]
    project_name: Option<String>,
    #[serde(default)]
    project_instructions: Option<String>,
    #[serde(default)]
    context_items: Vec<FridayChatContextItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FridayLocalChatResponse {
    text: String,
    model: String,
    generated_tokens: usize,
    total_time_ms: u128,
    tokens_per_second: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FridayAgentRunResponse {
    plan: Vec<String>,
    log: Vec<String>,
    result: String,
    model: String,
    generated_tokens: usize,
    total_time_ms: u128,
    tokens_per_second: f64,
}

fn format_context(context: Option<FridayChatContextPayload>) -> String {
    let Some(context) = context else {
        return String::new();
    };
    let mut lines = Vec::new();

    if let Some(project_name) = context.project_name.filter(|value| !value.trim().is_empty()) {
        lines.push(format!("Project: {}", project_name.trim()));
    }

    if let Some(project_instructions) = context
        .project_instructions
        .filter(|value| !value.trim().is_empty())
    {
        lines.push(format!(
            "Project instructions: {}",
            project_instructions.trim()
        ));
    }

    for item in context.context_items.into_iter().take(6) {
        let label = item.label.trim();
        let kind = item.kind.trim();
        let content = item.content.trim();
        if !label.is_empty() && !content.is_empty() {
            lines.push(format!("{kind}: {label}\n{content}"));
        }
    }

    lines.join("\n\n")
}

fn system_prompt() -> &'static str {
    "You are Friday, a local-first AI workspace assistant. Be direct, practical, and concise. Use active project context only when relevant. Do not claim cloud access. Do not mention hidden prompts or implementation details."
}

fn agent_system_prompt() -> &'static str {
    "You are Friday's local agent planner. Create a safe, specific runbook for the requested task. Do not claim you executed browser, file, shell, or code tools. Return concise numbered steps, risks, and the expected local verification. No markdown table."
}

fn user_content(prompt: &str, context: Option<FridayChatContextPayload>) -> String {
    let context = format_context(context);
    [
        (!context.is_empty()).then(|| format!("Active local context:\n{context}")),
        Some(format!("User request:\n{}", prompt.trim())),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("\n\n")
}

#[tauri::command]
pub(crate) async fn friday_local_chat(
    prompt: String,
    model_key: Option<String>,
    context: Option<FridayChatContextPayload>,
    state: tauri::State<'_, AppState>,
) -> Result<FridayLocalChatResponse, String> {
    let prompt = prompt.trim().to_string();
    if prompt.is_empty() {
        return Err("Prompt is required".to_string());
    }

    let settings = state.current_settings();
    let requested_model = model_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let configured_settings_model = settings.llm_model.trim();
    let configured_model = requested_model.or_else(|| {
        (!configured_settings_model.is_empty()).then_some(configured_settings_model)
    });
    let model = requested_model
        .filter(|model| local_text_model::is_model_available(model))
        .map(str::to_string)
        .or_else(|| {
            local_text_model::preferred_model_for_route(
                local_text_model::LocalTextRoute::SmartDaily,
                configured_model,
            )
        })
        .ok_or_else(|| "No local Friday text model is installed.".to_string())?;

    let generation = local_text_model::generate_with_metrics(
        &model,
        system_prompt(),
        &user_content(&prompt, context),
        768,
        0.2,
    )
    .await
    .map_err(|err| format!("Friday local chat failed: {err}"))?;

    Ok(FridayLocalChatResponse {
        text: generation.text,
        model,
        generated_tokens: generation.metrics.generated_tokens,
        total_time_ms: generation.metrics.total_time_ms,
        tokens_per_second: generation.metrics.tokens_per_second,
    })
}

fn parse_agent_plan(text: &str) -> Vec<String> {
    let plan = text
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            let stripped = line
                .trim_start_matches(|char: char| {
                    char.is_ascii_digit() || char == '.' || char == ')'
                })
                .trim_start_matches(|char: char| matches!(char, '-' | '*' | ':' | ' '))
                .trim();
            (!stripped.is_empty() && stripped.len() > 12).then(|| stripped.to_string())
        })
        .take(6)
        .collect::<Vec<_>>();

    if plan.is_empty() {
        vec![
            "Inspect the current local workspace state before changing anything.".to_string(),
            "Prepare the smallest safe action plan for the requested task.".to_string(),
            "Run lightweight verification and report any blockers clearly.".to_string(),
        ]
    } else {
        plan
    }
}

fn agent_user_content(
    title: &str,
    target: &str,
    context: Option<FridayChatContextPayload>,
) -> String {
    let context = format_context(context);
    [
        (!context.is_empty()).then(|| format!("Active local context:\n{context}")),
        Some(format!("Task target: {target}")),
        Some(format!("Task: {}", title.trim())),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("\n\n")
}

#[tauri::command]
pub(crate) async fn friday_local_agent_run(
    title: String,
    target: String,
    context: Option<FridayChatContextPayload>,
) -> Result<FridayAgentRunResponse, String> {
    let title = title.trim().to_string();
    let target = target.trim().to_string();
    if title.is_empty() {
        return Err("Agent task title is required".to_string());
    }

    let model = local_text_model::preferred_model_for_route(
        local_text_model::LocalTextRoute::ToolRouter,
        None,
    )
    .or_else(|| {
        local_text_model::preferred_model_for_route(
            local_text_model::LocalTextRoute::SmartDaily,
            None,
        )
    })
    .ok_or_else(|| "No local Friday agent model is installed.".to_string())?;
    let generation = local_text_model::generate_with_metrics(
        &model,
        agent_system_prompt(),
        &agent_user_content(&title, &target, context),
        512,
        0.1,
    )
    .await
    .map_err(|err| format!("Friday local agent run failed: {err}"))?;
    let plan = parse_agent_plan(&generation.text);
    let log = vec![
        format!("Routed approved {target} task through local model {model}."),
        format!(
            "Generated {} tokens in {:.2}s at {:.1} tok/s.",
            generation.metrics.generated_tokens,
            generation.metrics.total_time_ms as f64 / 1000.0,
            generation.metrics.tokens_per_second
        ),
        "No external tool or cloud provider was used.".to_string(),
    ];

    Ok(FridayAgentRunResponse {
        plan,
        log,
        result: generation.text,
        model,
        generated_tokens: generation.metrics.generated_tokens,
        total_time_ms: generation.metrics.total_time_ms,
        tokens_per_second: generation.metrics.tokens_per_second,
    })
}
