use std::sync::OnceLock;

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PressEnterCommand {
    pub(crate) text: String,
    pub(crate) press_enter: bool,
}

pub(crate) fn extract_press_enter_command(text: &str) -> PressEnterCommand {
    static PRESS_ENTER_RE: OnceLock<Regex> = OnceLock::new();
    let regex = PRESS_ENTER_RE.get_or_init(|| {
        Regex::new(r"(?i)(?:^|[\s,.!?;:])press\s+enter[.!?]*\s*$")
            .expect("press enter command regex")
    });

    let Some(command) = regex.find(text) else {
        return PressEnterCommand {
            text: text.to_string(),
            press_enter: false,
        };
    };

    let stripped = text[..command.start()]
        .trim_end_matches(char::is_whitespace)
        .to_string();

    PressEnterCommand {
        text: stripped,
        press_enter: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_press_enter_at_the_end() {
        assert_eq!(
            extract_press_enter_command("Hello world. Press enter."),
            PressEnterCommand {
                text: "Hello world.".to_string(),
                press_enter: true,
            }
        );
    }

    #[test]
    fn supports_enter_only_command() {
        assert_eq!(
            extract_press_enter_command("press enter"),
            PressEnterCommand {
                text: String::new(),
                press_enter: true,
            }
        );
    }

    #[test]
    fn ignores_press_enter_inside_text() {
        assert_eq!(
            extract_press_enter_command("Tell me how to press enter safely"),
            PressEnterCommand {
                text: "Tell me how to press enter safely".to_string(),
                press_enter: false,
            }
        );
    }
}
