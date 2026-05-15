use super::locale::error_code_label_regexes;
use crate::{WslCommandContext, WslError};
use regex::Regex;
use std::sync::OnceLock;

pub(crate) fn extract_wsl_error_code(output: &str) -> Option<String> {
    for regex in error_code_label_regexes() {
        if let Some(captures) = regex.captures(output) {
            return captures.get(1).map(|matched| matched.as_str().to_string());
        }
    }

    if let Some(captures) = generic_error_code_regex().captures(output) {
        return captures.get(1).map(|matched| matched.as_str().to_string());
    }

    None
}

pub(crate) fn looks_like_invalid_argument_output(output: &str) -> bool {
    if output.contains("Wsl/E_INVALIDARG") {
        return true;
    }

    let lower = output.to_ascii_lowercase();
    lower.contains("wsl.exe --help")
        || lower.contains("supported argument")
        || lower.contains("invalid command")
        || lower.contains("unrecognized option")
}

pub(super) fn parse_failed(context: WslCommandContext, detail: &str, raw_output: &str) -> WslError {
    WslError::OutputParseFailed {
        context,
        detail: detail.to_string(),
        raw_output: raw_output.trim().to_string(),
    }
}

fn generic_error_code_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"\b(Wsl/[A-Za-z0-9_./-]+)\b").expect("valid generic error regex")
    })
}

#[cfg(test)]
mod tests {
    use super::{extract_wsl_error_code, looks_like_invalid_argument_output};

    #[test]
    fn extracts_error_code_when_colon_is_omitted() {
        assert_eq!(
            extract_wsl_error_code("错误代码 Wsl/E_INVALIDARG"),
            Some("Wsl/E_INVALIDARG".to_string())
        );
        assert_eq!(
            extract_wsl_error_code("Error code Wsl/Service/E_FAIL"),
            Some("Wsl/Service/E_FAIL".to_string())
        );
    }

    #[test]
    fn extracts_error_code_via_generic_fallback() {
        assert_eq!(
            extract_wsl_error_code("something went wrong: Wsl/E_INVALIDARG end"),
            Some("Wsl/E_INVALIDARG".to_string())
        );
    }

    #[test]
    fn detects_invalid_argument_via_explicit_code() {
        assert!(looks_like_invalid_argument_output(
            "no localized hints here, only Wsl/E_INVALIDARG"
        ));
    }

    #[test]
    fn detects_invalid_argument_via_english_hint_in_localized_help() {
        assert!(looks_like_invalid_argument_output(
            "请使用“wsl.exe --help”获取受支持的参数列表。"
        ));
    }

    #[test]
    fn ignores_unrelated_output() {
        assert!(!looks_like_invalid_argument_output(
            "completely unrelated diagnostic text"
        ));
    }
}
