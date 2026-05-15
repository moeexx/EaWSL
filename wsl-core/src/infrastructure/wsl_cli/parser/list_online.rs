use super::column_split_regex;
use super::error::parse_failed;
use crate::{OnlineDistro, WslCommandContext, WslError};
use regex::Regex;
use std::sync::OnceLock;

pub(crate) fn parse_list_online_output(output: &str) -> Result<Vec<OnlineDistro>, WslError> {
    let mut header_found = false;
    let mut entries = Vec::new();

    for line in output.lines() {
        if !header_found {
            if is_list_online_header(line) {
                header_found = true;
            }
            continue;
        }

        if line.trim().is_empty() {
            continue;
        }

        entries.push(parse_list_online_line(line, output)?);
    }

    if !header_found {
        return Err(parse_failed(
            WslCommandContext::ListOnline,
            "missing list --online header",
            output,
        ));
    }

    Ok(entries)
}

fn parse_list_online_line(line: &str, raw_output: &str) -> Result<OnlineDistro, WslError> {
    let segments = column_split_regex()
        .split(line.trim())
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if segments.len() != 2 {
        return Err(parse_failed(
            WslCommandContext::ListOnline,
            "failed to split list --online row into 2 columns",
            raw_output,
        ));
    }

    let name = segments[0].trim();
    let friendly_name = segments[1].trim();

    if name.is_empty() || friendly_name.is_empty() {
        return Err(parse_failed(
            WslCommandContext::ListOnline,
            "encountered empty list --online field",
            raw_output,
        ));
    }
    if name.chars().any(char::is_whitespace) {
        return Err(parse_failed(
            WslCommandContext::ListOnline,
            "online distro identifier cannot contain spaces",
            raw_output,
        ));
    }

    Ok(OnlineDistro {
        name: name.to_string(),
        friendly_name: friendly_name.to_string(),
    })
}

fn is_list_online_header(line: &str) -> bool {
    online_header_regex().is_match(line.trim())
}

fn online_header_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?i)^\s*NAME\s{2,}FRIENDLY NAME\s*$")
            .expect("valid list --online header regex")
    })
}
