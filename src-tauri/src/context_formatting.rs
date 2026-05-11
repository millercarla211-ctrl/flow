use crate::settings::Personality;

pub(crate) fn apply_context_formatting(text: &str, mode: Option<&Personality>) -> String {
    let Some(mode) = mode else {
        return text.to_string();
    };

    if is_mode(mode, "coding") {
        return text.to_string();
    }

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return text.to_string();
    }

    if is_mode(mode, "messaging") {
        return format_for_messaging(trimmed);
    }

    if is_mode(mode, "email") || is_mode(mode, "notes") {
        return capitalize_first_alpha(trimmed);
    }

    text.to_string()
}

fn is_mode(mode: &Personality, needle: &str) -> bool {
    mode.id.eq_ignore_ascii_case(needle) || mode.name.eq_ignore_ascii_case(needle)
}

fn format_for_messaging(text: &str) -> String {
    let without_period = strip_plain_trailing_period(text);
    lowercase_chat_start(&without_period)
}

fn strip_plain_trailing_period(text: &str) -> String {
    let trimmed = text.trim_end();
    if !trimmed.ends_with('.') || trimmed.ends_with("...") {
        return text.to_string();
    }

    let without_period = trimmed.trim_end_matches('.');
    let last_word = without_period
        .split_whitespace()
        .last()
        .unwrap_or_default()
        .trim_matches(|ch: char| !ch.is_alphanumeric());

    let preserve = last_word.len() <= 2
        || last_word.chars().all(|ch| ch.is_ascii_uppercase())
        || last_word.contains('.');

    if preserve {
        text.to_string()
    } else {
        without_period.to_string()
    }
}

fn lowercase_chat_start(text: &str) -> String {
    let mut chars = text.char_indices();
    let Some((index, first)) = chars.find(|(_, ch)| ch.is_alphabetic()) else {
        return text.to_string();
    };

    if index > 0 || !first.is_uppercase() {
        return text.to_string();
    }

    let first_word = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|ch: char| !ch.is_alphanumeric());

    let preserve = first_word == "I"
        || first_word.len() <= 2
        || first_word.chars().all(|ch| ch.is_ascii_uppercase())
        || matches!(
            first_word,
            "OK" | "Okay" | "Hey" | "Hi" | "Hello" | "Thanks" | "Thank"
        );

    if preserve {
        return text.to_string();
    }

    let lower = first.to_lowercase().collect::<String>();
    format!("{lower}{}", &text[index + first.len_utf8()..])
}

fn capitalize_first_alpha(text: &str) -> String {
    let mut chars = text.char_indices();
    let Some((index, first)) = chars.find(|(_, ch)| ch.is_alphabetic()) else {
        return text.to_string();
    };

    if first.is_uppercase() {
        return text.to_string();
    }

    let upper = first.to_uppercase().collect::<String>();
    format!(
        "{}{}{}",
        &text[..index],
        upper,
        &text[index + first.len_utf8()..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mode(id: &str, name: &str) -> Personality {
        Personality {
            id: id.to_string(),
            name: name.to_string(),
            enabled: true,
            apps: Vec::new(),
            websites: Vec::new(),
            instructions: Vec::new(),
        }
    }

    #[test]
    fn messaging_strips_plain_period_and_softens_start() {
        let output = apply_context_formatting(
            "Sounds good, let's sync tomorrow morning.",
            Some(&mode("messaging", "Messaging")),
        );

        assert_eq!(output, "sounds good, let's sync tomorrow morning");
    }

    #[test]
    fn messaging_preserves_direct_greeting_and_question_mark() {
        let output = apply_context_formatting(
            "Hey Sarah, does Friday work?",
            Some(&mode("messaging", "Messaging")),
        );

        assert_eq!(output, "Hey Sarah, does Friday work?");
    }

    #[test]
    fn email_capitalizes_sentence_start() {
        let output = apply_context_formatting(
            "thanks for sending this over.",
            Some(&mode("email", "Email")),
        );

        assert_eq!(output, "Thanks for sending this over.");
    }

    #[test]
    fn coding_mode_is_left_unchanged() {
        let output = apply_context_formatting(
            "Return createUser(user_id);",
            Some(&mode("coding", "Coding")),
        );

        assert_eq!(output, "Return createUser(user_id);");
    }
}
