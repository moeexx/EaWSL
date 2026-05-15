use regex::Regex;
use std::sync::OnceLock;

pub(crate) mod error;
pub(crate) mod list_online;
pub(crate) mod list_verbose;
pub(crate) mod locale;
pub(crate) mod progress;
pub(crate) mod version;

#[cfg(test)]
mod fixture_tests;

pub(super) fn normalize_ascii_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn column_split_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\s{2,}").expect("valid column split regex"))
}
