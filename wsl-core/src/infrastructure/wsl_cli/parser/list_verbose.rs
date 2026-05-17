use super::error::parse_failed;
use super::{column_split_regex, normalize_ascii_whitespace};
use crate::domain::model::distro::InstalledDistroSnapshot;
use crate::{DistroState, WslCommandContext, WslError};

pub(crate) fn parse_list_verbose_output(
    output: &str,
) -> Result<Vec<InstalledDistroSnapshot>, WslError> {
    let mut lines = output.lines().filter(|line| !line.trim().is_empty());
    let header = lines.next().ok_or_else(|| {
        parse_failed(
            WslCommandContext::ListVerbose,
            "missing list --verbose header",
            output,
        )
    })?;

    if !is_list_verbose_header(header) {
        return Err(parse_failed(
            WslCommandContext::ListVerbose,
            "missing list --verbose header",
            output,
        ));
    }

    let mut entries = Vec::new();
    for line in lines {
        entries.push(parse_list_verbose_line(line, output)?);
    }

    Ok(entries)
}

fn parse_list_verbose_line(
    line: &str,
    raw_output: &str,
) -> Result<InstalledDistroSnapshot, WslError> {
    let mut trimmed = line.trim_start();
    let is_default = trimmed.starts_with('*');
    if is_default {
        trimmed = trimmed[1..].trim_start();
    }

    let segments = split_columns_with_fallback(trimmed, 3).ok_or_else(|| {
        parse_failed(
            WslCommandContext::ListVerbose,
            "failed to split list --verbose row into 3 columns",
            raw_output,
        )
    })?;

    let name = segments[0].trim();
    if name.is_empty() {
        return Err(parse_failed(
            WslCommandContext::ListVerbose,
            "encountered empty distro name",
            raw_output,
        ));
    }
    if name.chars().any(char::is_whitespace) {
        return Err(parse_failed(
            WslCommandContext::ListVerbose,
            "distro names with spaces are not allowed by project constraints",
            raw_output,
        ));
    }

    let version = segments[2].trim().parse::<u8>().map_err(|_| {
        parse_failed(
            WslCommandContext::ListVerbose,
            "failed to parse distro version as u8",
            raw_output,
        )
    })?;

    Ok(InstalledDistroSnapshot {
        name: name.to_string(),
        state: parse_distro_state(segments[1].trim()),
        version,
        is_default,
    })
}

fn parse_distro_state(raw_state: &str) -> DistroState {
    match raw_state {
        "Running" => DistroState::Running,
        "Stopped" => DistroState::Stopped,
        "Installing" => DistroState::Installing,
        other => DistroState::Unknown(other.to_string()),
    }
}

fn split_columns_with_fallback(line: &str, expected: usize) -> Option<Vec<&str>> {
    let segments = column_split_regex()
        .split(line.trim())
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.len() == expected {
        return Some(segments);
    }

    let fallback = line.split_whitespace().collect::<Vec<_>>();
    if fallback.len() == expected {
        return Some(fallback);
    }

    None
}

fn is_list_verbose_header(line: &str) -> bool {
    normalize_ascii_whitespace(&line.trim().to_ascii_lowercase()) == "name state version"
}

#[cfg(test)]
mod tests {
    use super::parse_list_verbose_output;
    use crate::DistroState;

    #[test]
    fn unknown_state_strings_fall_back_to_unknown_variant() {
        let raw = "  NAME              STATE           VERSION\r\n\
                   * Ubuntu            Paused          2\r\n\
                     Debian            Suspended       2\r\n\
                     Fedora            Hibernating     2\r\n";

        let entries = parse_list_verbose_output(raw).expect("parse with unknown states");
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].state, DistroState::Unknown("Paused".to_string()));
        assert_eq!(
            entries[1].state,
            DistroState::Unknown("Suspended".to_string())
        );
        assert_eq!(
            entries[2].state,
            DistroState::Unknown("Hibernating".to_string())
        );
    }
}
