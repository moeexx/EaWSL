//! Locale-coupled lookup tables for `wsl.exe` parsing.
//!
//! Keep this table small and evidence-based. Prefer structural cues first;
//! add locale labels only when `wsl.exe` actually varies by locale.

use regex::Regex;
use std::sync::OnceLock;

/// Normalized `wsl --version` labels mapped to canonical keys.
const VERSION_LABEL_ALIASES: &[(&str, &str)] = &[
    ("wsl 版本", "wsl"),
    ("wsl version", "wsl"),
    ("内核版本", "kernel"),
    ("kernel version", "kernel"),
    ("wslg 版本", "wslg"),
    ("wslg version", "wslg"),
    ("msrdc 版本", "msrdc"),
    ("msrdc version", "msrdc"),
    ("direct3d 版本", "direct3d"),
    ("direct3d version", "direct3d"),
    ("dxcore 版本", "dxcore"),
    ("dxcore version", "dxcore"),
    ("windows", "windows"),
    ("windows version", "windows"),
];

/// Localized "error code" labels used to match `Wsl/<code>` diagnostics.
//TODO: Add more labels only after capturing real localized output.
const ERROR_CODE_LABELS: &[&str] = &["错误代码", "Error code"];

/// Look up the canonical version key for a normalized label.
pub(super) fn canonical_version_label(normalized: &str) -> Option<&'static str> {
    VERSION_LABEL_ALIASES
        .iter()
        .find(|(alias, _)| *alias == normalized)
        .map(|(_, canonical)| *canonical)
}

/// Cached regexes generated from `ERROR_CODE_LABELS`.
pub(super) fn error_code_label_regexes() -> &'static [Regex] {
    static REGEXES: OnceLock<Vec<Regex>> = OnceLock::new();
    REGEXES.get_or_init(|| {
        ERROR_CODE_LABELS
            .iter()
            .map(|label| {
                let pattern = format!(r"(?i){}[:：]?\s*(Wsl/[^\s]+)", regex::escape(label));
                Regex::new(&pattern).expect("valid label error regex")
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::error_code_label_regexes;

    #[test]
    fn error_code_label_regexes_match_known_locales() {
        let regexes = error_code_label_regexes();
        assert!(!regexes.is_empty());

        let zh = "错误代码: Wsl/E_INVALIDARG";
        let en = "Error code: Wsl/Service/E_FAIL";
        assert!(regexes.iter().any(|r| r.is_match(zh)));
        assert!(regexes.iter().any(|r| r.is_match(en)));
    }
}
