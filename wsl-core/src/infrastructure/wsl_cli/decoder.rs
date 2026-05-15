use std::mem;

use super::runner::CommandOutput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecodedCommandOutput {
    pub status_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub merged_output: String,
}

pub(crate) fn decode_command_output(output: CommandOutput) -> DecodedCommandOutput {
    let stdout = decode_stdout(&output.stdout);
    let stderr = decode_stderr(&output.stderr);
    let merged_output = merge_output(&stdout, &stderr);

    DecodedCommandOutput {
        status_code: output.status_code,
        stdout,
        stderr,
        merged_output,
    }
}

fn decode_stdout(bytes: &[u8]) -> String {
    decode_stream(bytes, true)
}

pub(crate) fn decode_stderr(bytes: &[u8]) -> String {
    decode_stream(bytes, false)
}

fn decode_stream(bytes: &[u8], prefer_utf16: bool) -> String {
    if should_decode_utf16le(bytes, prefer_utf16) {
        if let Some(decoded) = decode_utf16le(bytes) {
            return decoded;
        }
    }

    String::from_utf8_lossy(bytes).into_owned()
}

fn should_decode_utf16le(bytes: &[u8], prefer_utf16: bool) -> bool {
    if bytes.len() < 2 || !bytes.len().is_multiple_of(2) {
        return false;
    }

    looks_like_utf16le(bytes, prefer_utf16)
}

fn looks_like_utf16le(bytes: &[u8], prefer_utf16: bool) -> bool {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return true;
    }

    let nul_count = bytes.iter().filter(|&&byte| byte == 0).count();
    if prefer_utf16 {
        nul_count > 0
    } else {
        nul_count * 5 >= bytes.len()
    }
}

fn decode_utf16le(bytes: &[u8]) -> Option<String> {
    if !bytes.len().is_multiple_of(2) {
        return None;
    }

    let mut units = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();

    if matches!(units.first(), Some(0xFEFF)) {
        units.remove(0);
    }

    Some(String::from_utf16_lossy(&units))
}

fn merge_output(stdout: &str, stderr: &str) -> String {
    match (stdout.trim(), stderr.trim()) {
        ("", "") => String::new(),
        ("", _) => stderr.trim().to_string(),
        (_, "") => stdout.trim().to_string(),
        _ => format!("{}\n{}", stdout.trim(), stderr.trim()),
    }
}

pub(crate) struct StreamingTextRecords {
    decoder: Utf16StreamDecoder,
    pending_record: String,
}

impl StreamingTextRecords {
    pub(crate) fn new() -> Self {
        Self {
            decoder: Utf16StreamDecoder::new(),
            pending_record: String::new(),
        }
    }

    pub(crate) fn push_bytes(&mut self, chunk: &[u8]) -> Vec<String> {
        let decoded = self.decoder.push(chunk);
        self.push_text(&decoded)
    }

    pub(crate) fn finish(&mut self) -> Vec<String> {
        let decoded = self.decoder.finish();
        let mut records = self.push_text(&decoded);
        if !self.pending_record.trim().is_empty() {
            records.push(mem::take(&mut self.pending_record));
        }
        self.pending_record.clear();
        records
    }

    fn push_text(&mut self, text: &str) -> Vec<String> {
        let mut records = Vec::new();

        for ch in text.chars() {
            match ch {
                '\r' | '\n' => {
                    if !self.pending_record.trim().is_empty() {
                        records.push(mem::take(&mut self.pending_record));
                    } else {
                        self.pending_record.clear();
                    }
                }
                _ => self.pending_record.push(ch),
            }
        }

        records
    }
}

struct Utf16StreamDecoder {
    pending_bytes: Vec<u8>,
}

impl Utf16StreamDecoder {
    fn new() -> Self {
        Self {
            pending_bytes: Vec::new(),
        }
    }

    fn push(&mut self, chunk: &[u8]) -> String {
        self.pending_bytes.extend_from_slice(chunk);
        self.take_text(false)
    }

    fn finish(&mut self) -> String {
        self.take_text(true)
    }

    fn take_text(&mut self, flush_all: bool) -> String {
        if self.pending_bytes.is_empty() {
            return String::new();
        }

        if self.pending_bytes.len() < 2 && !flush_all {
            return String::new();
        }

        if looks_like_utf16le(&self.pending_bytes, true) {
            let even_len = self.pending_bytes.len() - (self.pending_bytes.len() % 2);
            if even_len == 0 {
                if flush_all {
                    return String::from_utf8_lossy(&mem::take(&mut self.pending_bytes))
                        .into_owned();
                }
                return String::new();
            }

            let decoded = decode_utf16le(&self.pending_bytes[..even_len]).unwrap_or_else(|| {
                String::from_utf8_lossy(&self.pending_bytes[..even_len]).into_owned()
            });
            let remaining = self.pending_bytes.split_off(even_len);
            self.pending_bytes = remaining;

            if flush_all && !self.pending_bytes.is_empty() {
                let remainder =
                    String::from_utf8_lossy(&mem::take(&mut self.pending_bytes)).into_owned();
                return format!("{decoded}{remainder}");
            }

            return decoded;
        }

        String::from_utf8_lossy(&mem::take(&mut self.pending_bytes)).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::StreamingTextRecords;
    use crate::infrastructure::wsl_cli::test_support::encode_utf16le;

    #[test]
    fn streaming_records_decode_utf16le_across_odd_chunks() {
        let bytes = encode_utf16le("alpha\r\nbeta\r\n");
        let mut records = StreamingTextRecords::new();

        let mut decoded = records.push_bytes(&bytes[..5]);
        decoded.extend(records.push_bytes(&bytes[5..]));
        decoded.extend(records.finish());

        assert_eq!(decoded, vec!["alpha".to_string(), "beta".to_string()]);
    }

    #[test]
    fn streaming_records_flush_odd_utf8_byte() {
        let mut records = StreamingTextRecords::new();

        assert!(records.push_bytes(b"o").is_empty());
        assert_eq!(records.finish(), vec!["o".to_string()]);
    }

    #[test]
    fn streaming_records_fall_back_to_utf8() {
        let mut records = StreamingTextRecords::new();

        assert_eq!(
            records.push_bytes(b"plain utf8\r\n"),
            vec!["plain utf8".to_string()]
        );
        assert!(records.finish().is_empty());
    }
}
