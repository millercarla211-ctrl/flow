use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::Path,
};

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FridayResearchCitationPayload {
    label: String,
    kind: String,
    excerpt: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FridayResearchDraftResponse {
    plan: Vec<String>,
    report: String,
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

fn research_system_prompt() -> &'static str {
    "You are Friday's local-first research writer. Write an answer-first Markdown brief from the provided local evidence only. Cite local sources with [1], [2], etc. Do not invent web facts. If evidence is missing, say what is unknown and what source would be needed."
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
    workspace_snapshot: Option<&str>,
) -> String {
    let context = format_context(context);
    [
        (!context.is_empty()).then(|| format!("Active local context:\n{context}")),
        workspace_snapshot
            .filter(|snapshot| !snapshot.trim().is_empty())
            .map(|snapshot| format!("Read-only workspace inspection:\n{snapshot}")),
        Some(format!("Task target: {target}")),
        Some(format!("Task: {}", title.trim())),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("\n\n")
}

fn should_skip_agent_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    matches!(
        name,
        ".git"
            | ".next"
            | ".turbo"
            | "dist"
            | "build"
            | "node_modules"
            | "target"
            | "models"
            | "tmp"
            | "venv"
            | ".venv"
    )
}

fn collect_agent_paths(
    root: &Path,
    dir: &Path,
    depth: usize,
    files: &mut Vec<String>,
) -> Result<(), String> {
    if files.len() >= 90 || depth > 3 {
        return Ok(());
    }

    let entries = fs::read_dir(dir)
        .map_err(|err| format!("Could not inspect {}: {err}", dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if should_skip_agent_path(&path) {
            continue;
        }

        if path.is_dir() {
            let _ = collect_agent_paths(root, &path, depth + 1, files);
            continue;
        }

        if files.len() >= 90 {
            break;
        }

        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        files.push(relative);
    }

    Ok(())
}

fn collect_workspace_snapshot(title: &str, target: &str) -> Option<String> {
    if target != "code" && target != "files" {
        return None;
    }

    let root = env::current_dir().ok()?;
    let mut files = Vec::new();
    if collect_agent_paths(&root, &root, 0, &mut files).is_err() || files.is_empty() {
        return Some(format!("Root: {}\nNo inspectable local files found.", root.display()));
    }

    files.sort();
    let title_tokens = title
        .to_lowercase()
        .split(|char: char| !char.is_ascii_alphanumeric())
        .filter(|token| token.len() > 2)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let matches = files
        .iter()
        .filter(|path| {
            let lower = path.to_lowercase();
            title_tokens.iter().any(|token| lower.contains(token))
        })
        .take(12)
        .cloned()
        .collect::<Vec<_>>();

    let sample = files.iter().take(24).cloned().collect::<Vec<_>>();
    let match_block = if matches.is_empty() {
        "No direct filename matches for the task title.".to_string()
    } else {
        format!("Likely relevant paths:\n- {}", matches.join("\n- "))
    };

    Some(format!(
        "Root: {}\nInspected {} paths, capped at 90.\n{}\nSample paths:\n- {}",
        root.display(),
        files.len(),
        match_block,
        sample.join("\n- ")
    ))
}

fn research_user_content(
    topic: &str,
    citations: Vec<FridayResearchCitationPayload>,
    context: Option<FridayChatContextPayload>,
) -> String {
    let context = format_context(context);
    let source_lines = citations
        .into_iter()
        .take(6)
        .enumerate()
        .filter_map(|(index, citation)| {
            let label = citation.label.trim();
            let kind = citation.kind.trim();
            let excerpt = citation.excerpt.trim();
            if label.is_empty() && excerpt.is_empty() {
                None
            } else {
                Some(format!(
                    "[{}] {} ({})\n{}",
                    index + 1,
                    if label.is_empty() { "Local source" } else { label },
                    if kind.is_empty() { "local" } else { kind },
                    excerpt
                ))
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    [
        (!context.is_empty()).then(|| format!("Active local context:\n{context}")),
        Some(format!("Research topic:\n{}", topic.trim())),
        Some(format!(
            "Local evidence:\n{}",
            if source_lines.is_empty() {
                "No matching local evidence was found."
            } else {
                &source_lines
            }
        )),
        Some(
            "Write with these sections: Working Answer, Evidence, Gaps, Next Actions."
                .to_string(),
        ),
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
    let workspace_snapshot = collect_workspace_snapshot(&title, &target);

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
        &agent_user_content(&title, &target, context, workspace_snapshot.as_deref()),
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
        workspace_snapshot
            .as_ref()
            .map(|_| "Inspected a bounded read-only workspace file snapshot.".to_string())
            .unwrap_or_else(|| {
                "No workspace file inspection was needed for this target.".to_string()
            }),
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

#[tauri::command]
pub(crate) async fn friday_local_research(
    topic: String,
    citations: Vec<FridayResearchCitationPayload>,
    context: Option<FridayChatContextPayload>,
    state: tauri::State<'_, AppState>,
) -> Result<FridayResearchDraftResponse, String> {
    let topic = topic.trim().to_string();
    if topic.is_empty() {
        return Err("Research topic is required".to_string());
    }

    let settings = state.current_settings();
    let configured_settings_model = settings.llm_model.trim();
    let configured_model =
        (!configured_settings_model.is_empty()).then_some(configured_settings_model);
    let model = local_text_model::preferred_model_for_route(
        local_text_model::LocalTextRoute::SmartDaily,
        configured_model,
    )
    .ok_or_else(|| "No local Friday research model is installed.".to_string())?;
    let generation = local_text_model::generate_with_metrics(
        &model,
        research_system_prompt(),
        &research_user_content(&topic, citations, context),
        900,
        0.2,
    )
    .await
    .map_err(|err| format!("Friday local research failed: {err}"))?;

    Ok(FridayResearchDraftResponse {
        plan: vec![
            format!("Answer {topic} using the local citations first."),
            "List evidence gaps before enabling web, academic, or premium sources.".to_string(),
            "Save the final brief as an artifact, memory, or follow-up when useful.".to_string(),
        ],
        report: generation.text,
        model,
        generated_tokens: generation.metrics.generated_tokens,
        total_time_ms: generation.metrics.total_time_ms,
        tokens_per_second: generation.metrics.tokens_per_second,
    })
}
