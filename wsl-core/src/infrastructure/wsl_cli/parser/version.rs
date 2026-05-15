use super::error::parse_failed;
use super::locale::canonical_version_label as lookup_canonical_version_label;
use super::normalize_ascii_whitespace;
use crate::{WslCommandContext, WslError, WslVersion};

pub(crate) fn parse_version_output(output: &str) -> Result<WslVersion, WslError> {
    #[derive(Default)]
    struct Builder {
        wsl: Option<String>,
        kernel: Option<String>,
        wslg: Option<String>,
        msrdc: Option<String>,
        direct3d: Option<String>,
        dxcore: Option<String>,
        windows: Option<String>,
    }

    let mut builder = Builder::default();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (label, value) = split_label_and_value(trimmed).ok_or_else(|| {
            parse_failed(
                WslCommandContext::Version,
                "encountered version line without label separator",
                output,
            )
        })?;

        let value = value.trim();
        if value.is_empty() {
            return Err(parse_failed(
                WslCommandContext::Version,
                "encountered version line without value",
                output,
            ));
        }

        let Some(canonical) = canonical_version_label(label) else {
            continue;
        };

        let slot = match canonical {
            "wsl" => &mut builder.wsl,
            "kernel" => &mut builder.kernel,
            "wslg" => &mut builder.wslg,
            "msrdc" => &mut builder.msrdc,
            "direct3d" => &mut builder.direct3d,
            "dxcore" => &mut builder.dxcore,
            "windows" => &mut builder.windows,
            _ => unreachable!("unexpected canonical version label"),
        };

        if slot.is_some() {
            return Err(parse_failed(
                WslCommandContext::Version,
                "encountered duplicate version label",
                output,
            ));
        }

        *slot = Some(value.to_string());
    }

    let wsl = builder.wsl.ok_or_else(|| {
        parse_failed(
            WslCommandContext::Version,
            "missing required WSL version field",
            output,
        )
    })?;
    let windows = builder.windows.ok_or_else(|| {
        parse_failed(
            WslCommandContext::Version,
            "missing required Windows version field",
            output,
        )
    })?;

    Ok(WslVersion {
        wsl,
        kernel: builder.kernel,
        wslg: builder.wslg,
        msrdc: builder.msrdc,
        direct3d: builder.direct3d,
        dxcore: builder.dxcore,
        windows,
    })
}

fn split_label_and_value(line: &str) -> Option<(&str, &str)> {
    let separator_index = line
        .char_indices()
        .find_map(|(index, ch)| matches!(ch, ':' | '：').then_some(index))?;
    let separator_width = line[separator_index..].chars().next()?.len_utf8();

    Some((
        line[..separator_index].trim(),
        line[(separator_index + separator_width)..].trim(),
    ))
}

fn canonical_version_label(label: &str) -> Option<&'static str> {
    let normalized = normalize_ascii_whitespace(&label.trim().to_ascii_lowercase());
    lookup_canonical_version_label(&normalized)
}
