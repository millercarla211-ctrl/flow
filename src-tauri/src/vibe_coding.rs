use std::sync::OnceLock;

use regex::{Captures, Regex};

use crate::{
    accessibility_context,
    settings::{Personality, UserSettings},
};

const CODE_EXTENSIONS: &[&str] = &[
    "c", "cpp", "css", "env", "go", "h", "hpp", "html", "java", "js", "json", "jsx", "kt", "lock",
    "md", "py", "rs", "scss", "swift", "toml", "ts", "tsx", "yaml", "yml",
];

pub(crate) fn postprocess_coding_transcript(
    text: &str,
    settings: &UserSettings,
    mode: Option<&Personality>,
) -> String {
    if !should_postprocess(settings, mode) {
        return text.to_string();
    }

    let mut result = text.to_string();
    result = normalize_spoken_extensions(&result);
    result = normalize_path_separators(&result);
    result = normalize_identifier_separators(&result);
    result = normalize_spoken_code_symbols(&result);

    if settings.vibe_coding_file_tagging {
        result = tag_spoken_files(&result);
        result = tag_recent_files(&result, &settings.vibe_coding_recent_files);
    }

    if settings.vibe_coding_variable_recognition {
        result = normalize_explicit_backticks(&result);
    }

    result
}

fn should_postprocess(settings: &UserSettings, mode: Option<&Personality>) -> bool {
    settings.vibe_coding_enabled
        && mode.as_ref().is_some_and(|mode| {
            mode.id.eq_ignore_ascii_case("coding") || mode.name.eq_ignore_ascii_case("coding")
        })
}

pub(crate) fn refresh_recent_files_from_active_context(
    settings: &mut UserSettings,
    mode: Option<&Personality>,
) -> bool {
    if !settings.vibe_coding_enabled
        || !settings.vibe_coding_file_tagging
        || !settings.vibe_coding_include_window_context
        || !mode.as_ref().is_some_and(|mode| {
            mode.id.eq_ignore_ascii_case("coding") || mode.name.eq_ignore_ascii_case("coding")
        })
    {
        return false;
    }

    let Some(context) = accessibility_context::get_active_context() else {
        return false;
    };

    if !looks_like_supported_ide(&context.app_name)
        && !looks_like_supported_ide(&context.window_title)
    {
        return false;
    }

    let mut candidates = extract_file_names(&context.window_title);
    if let Some(url) = context.url.as_ref() {
        candidates.extend(extract_file_names(url));
    }

    merge_recent_files(&mut settings.vibe_coding_recent_files, candidates)
}

fn looks_like_supported_ide(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        "cursor",
        "windsurf",
        "visual studio code",
        "code",
        "vscode",
        "webstorm",
        "intellij",
        "terminal",
        "powershell",
    ]
    .iter()
    .any(|needle| value.contains(needle))
}

fn extension_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\s+(?:dot|period)\s+(h\s*t\s*m\s*l|j\s*s\s*o\s*n|j\s*s\s*x|s\s*c\s*s\s*s|t\s*o\s*m\s*l|t\s*s\s*x|y\s*a\s*m\s*l|c\s*p\s*p|c\s*s\s*s|h\s*p\s*p|j\s*s|t\s*s|y\s*m\s*l|env|go|java|k\s*t|lock|m\s*d|p\s*y|r\s*s|swift|c|h)\b",
        )
        .expect("valid extension regex")
    })
}

fn normalize_extension(raw: &str) -> String {
    raw.chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn normalize_spoken_extensions(text: &str) -> String {
    extension_regex()
        .replace_all(text, |caps: &Captures| {
            format!(".{}", normalize_extension(&caps[1]))
        })
        .to_string()
}

fn forward_slash_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b([@A-Za-z0-9_./\\-]+)\s+(?:forward\s+slash|slash)\s+([@A-Za-z0-9_./\\-]+)\b",
        )
        .expect("valid slash regex")
    })
}

fn backslash_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b([@A-Za-z0-9_./\\-]+)\s+(?:back\s+slash|backslash)\s+([@A-Za-z0-9_./\\-]+)\b",
        )
        .expect("valid backslash regex")
    })
}

fn replace_repeated(text: &str, regex: &Regex, separator: &str) -> String {
    let mut current = text.to_string();
    loop {
        let next = regex
            .replace_all(&current, |caps: &Captures| {
                format!("{}{}{}", &caps[1], separator, &caps[2])
            })
            .to_string();
        if next == current {
            return current;
        }
        current = next;
    }
}

fn normalize_path_separators(text: &str) -> String {
    let with_slashes = replace_repeated(text, forward_slash_regex(), "/");
    replace_repeated(&with_slashes, backslash_regex(), "\\")
}

fn identifier_separator_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b([A-Za-z][A-Za-z0-9]*)\s+(underscore|dash|hyphen)\s+([A-Za-z0-9][A-Za-z0-9]*)\b")
            .expect("valid identifier separator regex")
    })
}

fn normalize_identifier_separators(text: &str) -> String {
    let mut current = text.to_string();
    loop {
        let next = identifier_separator_regex()
            .replace_all(&current, |caps: &Captures| {
                let separator = match caps[2].to_ascii_lowercase().as_str() {
                    "underscore" => "_",
                    _ => "-",
                };
                format!("{}{}{}", &caps[1], separator, &caps[3])
            })
            .to_string();
        if next == current {
            return current;
        }
        current = next;
    }
}

fn file_tag_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        let extensions = CODE_EXTENSIONS.join("|");
        Regex::new(&format!(
            r"(?i)\b(?:tag|tagged|at|@)\s+([A-Za-z0-9_./\\-]+\.({extensions}))\b"
        ))
        .expect("valid file tag regex")
    })
}

fn file_candidate_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        let extensions = CODE_EXTENSIONS.join("|");
        Regex::new(&format!(r"(?i)([A-Za-z0-9_./\\:-]+\.({extensions}))\b"))
            .expect("valid file candidate regex")
    })
}

fn normalize_recent_file(raw: &str) -> Option<String> {
    let raw = raw.trim().trim_matches(|ch: char| {
        matches!(ch, '"' | '\'' | '`' | ',' | ';' | ':' | ')' | ']' | '}')
    });
    if raw.is_empty() {
        return None;
    }

    let basename = raw
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(raw)
        .trim_start_matches('@');
    if basename.is_empty() || !basename.contains('.') {
        return None;
    }

    let extension = basename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if !CODE_EXTENSIONS.contains(&extension.as_str()) {
        return None;
    }

    Some(basename.chars().take(120).collect())
}

fn extract_file_names(text: &str) -> Vec<String> {
    file_candidate_regex()
        .captures_iter(text)
        .filter_map(|caps| normalize_recent_file(&caps[1]))
        .collect()
}

fn merge_recent_files(recent_files: &mut Vec<String>, candidates: Vec<String>) -> bool {
    if candidates.is_empty() {
        return false;
    }

    let mut changed = false;
    for candidate in candidates {
        if let Some(position) = recent_files
            .iter()
            .position(|file| file.eq_ignore_ascii_case(&candidate))
        {
            if position != 0 {
                let existing = recent_files.remove(position);
                recent_files.insert(0, existing);
                changed = true;
            }
            continue;
        }

        recent_files.insert(0, candidate);
        changed = true;
    }

    let mut deduped = Vec::with_capacity(recent_files.len());
    for file in recent_files.drain(..) {
        if !deduped
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&file))
        {
            deduped.push(file);
        }
    }

    if deduped.len() > 100 {
        deduped.truncate(100);
        changed = true;
    }

    *recent_files = deduped;
    changed
}

fn tag_spoken_files(text: &str) -> String {
    file_tag_regex()
        .replace_all(text, |caps: &Captures| {
            let file = caps[1].trim_start_matches('@');
            format!("@{file}")
        })
        .to_string()
}

fn tag_recent_files(text: &str, recent_files: &[String]) -> String {
    let mut result = text.to_string();
    let mut files = recent_files
        .iter()
        .filter_map(|file| normalize_recent_file(file))
        .collect::<Vec<_>>();
    files.sort_by_key(|file| std::cmp::Reverse(file.len()));

    for file in files {
        let pattern = format!(
            r"(?i)(^|[^@\w./\\-])({})($|[^\w./\\-]|\.(?:\s|$))",
            regex::escape(&file)
        );
        if let Ok(regex) = Regex::new(&pattern) {
            result = regex
                .replace_all(&result, |caps: &Captures| {
                    format!("{}@{}{}", &caps[1], &caps[2], &caps[3])
                })
                .to_string();
        }
    }

    result
}

fn explicit_backtick_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\bbacktick\s+([A-Za-z_][A-Za-z0-9_./:-]{0,80})\s+backtick\b")
            .expect("valid explicit backtick regex")
    })
}

fn normalize_explicit_backticks(text: &str) -> String {
    explicit_backtick_regex()
        .replace_all(text, |caps: &Captures| format!("`{}`", &caps[1]))
        .to_string()
}

fn code_symbol_patterns() -> &'static [(Regex, &'static str)] {
    static PATTERNS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        [
            (r"(?i)\bopen\s+(?:paren|parenthesis)\b", "("),
            (r"(?i)\bclose\s+(?:paren|parenthesis)\b", ")"),
            (r"(?i)\bopen\s+(?:bracket|square bracket)\b", "["),
            (r"(?i)\bclose\s+(?:bracket|square bracket)\b", "]"),
            (r"(?i)\bopen\s+(?:brace|curly brace)\b", "{"),
            (r"(?i)\bclose\s+(?:brace|curly brace)\b", "}"),
            (r"(?i)\btriple\s+equals\b", " === "),
            (r"(?i)\bdouble\s+equals\b", " == "),
            (r"(?i)\bnot\s+equals\b", " != "),
            (r"(?i)\bfat\s+arrow\b", " => "),
            (r"(?i)\barrow\b", " -> "),
            (r"(?i)\bequals\b", " = "),
            (r"(?i)\bsemicolon\b", ";"),
            (r"(?i)\bcolon\b", ":"),
            (r"(?i)\bcomma\b", ","),
            (r"(?i)\bpipe\b", " | "),
        ]
        .into_iter()
        .map(|(pattern, replacement)| {
            (
                Regex::new(pattern).expect("valid code symbol regex"),
                replacement,
            )
        })
        .collect()
    })
}

fn whitespace_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[ \t]{2,}").expect("valid whitespace regex"))
}

fn space_before_closer_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\s+([,.;:)\]}])").expect("valid closer spacing regex"))
}

fn space_after_opener_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"([(\[{])\s+").expect("valid opener spacing regex"))
}

fn space_before_function_paren_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"([A-Za-z0-9_`])\s+\(").expect("valid function paren regex"))
}

fn cleanup_symbol_spacing(text: &str) -> String {
    let text = whitespace_regex().replace_all(text, " ").to_string();
    let text = space_before_closer_regex()
        .replace_all(&text, "$1")
        .to_string();
    let text = space_after_opener_regex()
        .replace_all(&text, "$1")
        .to_string();
    space_before_function_paren_regex()
        .replace_all(&text, "$1(")
        .to_string()
}

fn normalize_spoken_code_symbols(text: &str) -> String {
    let mut result = text.to_string();
    for (regex, replacement) in code_symbol_patterns() {
        result = regex.replace_all(&result, *replacement).to_string();
    }
    cleanup_symbol_spacing(&result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Personality;

    fn settings() -> UserSettings {
        UserSettings::default()
    }

    fn coding_mode() -> Personality {
        Personality {
            id: "coding".to_string(),
            name: "Coding".to_string(),
            enabled: true,
            apps: Vec::new(),
            websites: Vec::new(),
            instructions: Vec::new(),
        }
    }

    #[test]
    fn postprocesses_file_tags_extensions_and_paths_in_coding_mode() {
        let output = postprocess_coding_transcript(
            "Please check at index dot t s x and src slash tauri slash lib dot r s.",
            &settings(),
            Some(&coding_mode()),
        );

        assert_eq!(output, "Please check @index.tsx and src/tauri/lib.rs.");
    }

    #[test]
    fn postprocesses_identifiers_and_explicit_backticks() {
        let output = postprocess_coding_transcript(
            "Use backtick createUser backtick with user underscore id and auth dash token.",
            &settings(),
            Some(&coding_mode()),
        );

        assert_eq!(output, "Use `createUser` with user_id and auth-token.");
    }

    #[test]
    fn postprocesses_spoken_code_symbols() {
        let output = postprocess_coding_transcript(
            "Return createUser open paren user underscore id comma auth dash token close paren semicolon",
            &settings(),
            Some(&coding_mode()),
        );

        assert_eq!(output, "Return createUser(user_id, auth-token);");
    }

    #[test]
    fn leaves_non_coding_modes_unchanged() {
        let output = postprocess_coding_transcript("meet me at index dot t s x", &settings(), None);

        assert_eq!(output, "meet me at index dot t s x");
    }

    #[test]
    fn respects_file_tagging_toggle() {
        let mut settings = settings();
        settings.vibe_coding_file_tagging = false;

        let output = postprocess_coding_transcript(
            "Please check at index dot tsx.",
            &settings,
            Some(&coding_mode()),
        );

        assert_eq!(output, "Please check at index.tsx.");
    }

    #[test]
    fn extracts_file_names_from_ide_titles() {
        let files = extract_file_names("lib.rs - Flow - Visual Studio Code");

        assert_eq!(files, vec!["lib.rs"]);
    }

    #[test]
    fn recent_files_are_deduped_and_promoted() {
        let mut recent_files = vec!["main.rs".to_string(), "index.tsx".to_string()];

        let changed = merge_recent_files(
            &mut recent_files,
            vec!["INDEX.tsx".to_string(), "transcribe.rs".to_string()],
        );

        assert!(changed);
        assert_eq!(
            recent_files,
            vec![
                "transcribe.rs".to_string(),
                "index.tsx".to_string(),
                "main.rs".to_string()
            ]
        );
    }

    #[test]
    fn tags_recent_files_without_explicit_trigger() {
        let mut settings = settings();
        settings.vibe_coding_recent_files = vec!["index.tsx".to_string()];

        let output = postprocess_coding_transcript(
            "Please update index dot t s x.",
            &settings,
            Some(&coding_mode()),
        );

        assert_eq!(output, "Please update @index.tsx.");
    }

    #[test]
    fn does_not_double_tag_recent_files() {
        let mut settings = settings();
        settings.vibe_coding_recent_files = vec!["index.tsx".to_string()];

        let output = postprocess_coding_transcript(
            "Please update @index dot t s x.",
            &settings,
            Some(&coding_mode()),
        );

        assert_eq!(output, "Please update @index.tsx.");
    }
}
