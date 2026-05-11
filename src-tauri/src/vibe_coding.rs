use std::sync::OnceLock;

use regex::{Captures, Regex};

use crate::settings::{Personality, UserSettings};

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

    if settings.vibe_coding_file_tagging {
        result = tag_spoken_files(&result);
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

fn tag_spoken_files(text: &str) -> String {
    file_tag_regex()
        .replace_all(text, |caps: &Captures| {
            let file = caps[1].trim_start_matches('@');
            format!("@{file}")
        })
        .to_string()
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
}
