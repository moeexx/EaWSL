use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// `wsl.exe --export` formats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportFormat {
    Tar,
    TarGz,
    TarXz,
    Vhd,
}

/// WSL disk-size string value object, such as `20GB` or `1.25TB`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskSize(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskSizeParseError;

impl DiskSize {
    pub fn parse(value: &str) -> Result<Self, DiskSizeParseError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(DiskSizeParseError);
        }

        let normalized = trimmed.to_ascii_uppercase();

        if let Some(raw_number) = normalized.strip_suffix("GB") {
            return parse_gb(raw_number.trim()).map(Self);
        }

        if let Some(raw_number) = normalized.strip_suffix("TB") {
            return parse_tb(raw_number.trim()).map(Self);
        }

        Err(DiskSizeParseError)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DiskSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for DiskSizeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid WSL disk size")
    }
}

impl std::error::Error for DiskSizeParseError {}

impl FromStr for DiskSize {
    type Err = DiskSizeParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for DiskSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DiskSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(serde::de::Error::custom)
    }
}

fn parse_gb(raw_number: &str) -> Result<String, DiskSizeParseError> {
    if raw_number.is_empty()
        || raw_number.starts_with('0')
        || !raw_number.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(DiskSizeParseError);
    }

    Ok(format!("{raw_number}GB"))
}

fn parse_tb(raw_number: &str) -> Result<String, DiskSizeParseError> {
    let centi_tb = parse_tb_centi(raw_number)?;
    if centi_tb == 0 {
        return Err(DiskSizeParseError);
    }

    Ok(format_tb(centi_tb))
}

fn parse_tb_centi(raw_number: &str) -> Result<u64, DiskSizeParseError> {
    let Some((whole, fraction)) = raw_number.split_once('.') else {
        return parse_unsigned(raw_number)
            .and_then(|value| value.checked_mul(100).ok_or(DiskSizeParseError));
    };

    if whole.is_empty() || fraction.is_empty() || fraction.contains('.') {
        return Err(DiskSizeParseError);
    }

    let whole = parse_unsigned(whole)?;
    let mut fraction_digits = fraction.chars();
    let first = fraction_digits
        .next()
        .and_then(|ch| ch.to_digit(10))
        .ok_or(DiskSizeParseError)?;
    let second = fraction_digits
        .next()
        .map(|ch| ch.to_digit(10).ok_or(DiskSizeParseError))
        .transpose()?
        .unwrap_or(0);
    let third = fraction_digits
        .next()
        .map(|ch| ch.to_digit(10).ok_or(DiskSizeParseError))
        .transpose()?
        .unwrap_or(0);

    if !fraction_digits.all(|ch| ch.is_ascii_digit()) {
        return Err(DiskSizeParseError);
    }

    let base = whole
        .checked_mul(100)
        .and_then(|value| value.checked_add(u64::from(first * 10 + second)))
        .ok_or(DiskSizeParseError)?;

    if third >= 5 {
        base.checked_add(1).ok_or(DiskSizeParseError)
    } else {
        Ok(base)
    }
}

fn parse_unsigned(value: &str) -> Result<u64, DiskSizeParseError> {
    if value.is_empty() || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(DiskSizeParseError);
    }

    value.parse::<u64>().map_err(|_| DiskSizeParseError)
}

fn format_tb(centi_tb: u64) -> String {
    let whole = centi_tb / 100;
    let fraction = centi_tb % 100;

    if fraction == 0 {
        format!("{whole}TB")
    } else if fraction.is_multiple_of(10) {
        format!("{}.{}TB", whole, fraction / 10)
    } else {
        format!("{whole}.{fraction:02}TB")
    }
}

#[cfg(test)]
mod tests {
    use super::DiskSize;

    #[test]
    fn disk_size_accepts_gb_integer_and_tb_decimal_values() {
        let cases = [
            ("20gb", "20GB"),
            ("20 GB", "20GB"),
            ("1 tb", "1TB"),
            ("1.5TB", "1.5TB"),
            ("0.75TB", "0.75TB"),
        ];

        for (input, expected) in cases {
            assert_eq!(DiskSize::parse(input).expect(input).as_str(), expected);
        }
    }

    #[test]
    fn disk_size_rounds_tb_to_two_decimal_places() {
        let cases = [
            ("1.234TB", "1.23TB"),
            ("1.235TB", "1.24TB"),
            ("1.999TB", "2TB"),
            ("0.005TB", "0.01TB"),
        ];

        for (input, expected) in cases {
            assert_eq!(DiskSize::parse(input).expect(input).as_str(), expected);
        }
    }

    #[test]
    fn disk_size_rejects_invalid_or_unrealistic_units() {
        for input in [
            "", "0GB", "01GB", "0TB", "0.00TB", "0.004TB", "-1GB", "20G", "20MB", "20GiB", "1TiB",
        ] {
            assert!(DiskSize::parse(input).is_err(), "{input} should fail");
        }
    }
}
