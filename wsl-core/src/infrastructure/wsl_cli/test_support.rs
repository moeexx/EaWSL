use std::{collections::VecDeque, future::Future, io, pin::Pin, sync::Mutex};

use tokio::sync::mpsc;

use super::runner::{CommandOutput, WslCommandRunner};
use crate::ProgressEvent;

pub(crate) struct FakeCall {
    expected_args: Option<Vec<String>>,
    invoke_started: bool,
    stdout_chunks: Vec<Vec<u8>>,
    stderr_chunks: Vec<Vec<u8>>,
    result: io::Result<CommandOutput>,
}

pub(crate) struct FakeRunner {
    calls: Mutex<VecDeque<FakeCall>>,
}

impl FakeRunner {
    pub(crate) fn from_result(result: io::Result<CommandOutput>) -> Self {
        let mut calls = VecDeque::new();
        calls.push_back(FakeCall {
            expected_args: None,
            invoke_started: false,
            stdout_chunks: Vec::new(),
            stderr_chunks: Vec::new(),
            result,
        });
        Self {
            calls: Mutex::new(calls),
        }
    }

    pub(crate) fn ok_utf16(stdout: &str) -> Self {
        Self::from_result(Ok(CommandOutput {
            status_code: Some(0),
            stdout: encode_utf16le(stdout),
            stderr: Vec::new(),
        }))
    }

    pub(crate) fn ok_utf16_with_args(expected_args: &[&str], stdout: &str) -> Self {
        Self::from_call(
            expected_args,
            Ok(CommandOutput {
                status_code: Some(0),
                stdout: encode_utf16le(stdout),
                stderr: Vec::new(),
            }),
        )
    }

    pub(crate) fn ok_empty_with_args(expected_args: &[&str]) -> Self {
        Self::from_call(
            expected_args,
            Ok(CommandOutput {
                status_code: Some(0),
                stdout: Vec::new(),
                stderr: Vec::new(),
            }),
        )
    }

    pub(crate) fn from_call(expected_args: &[&str], result: io::Result<CommandOutput>) -> Self {
        let mut calls = VecDeque::new();
        calls.push_back(FakeCall {
            expected_args: Some(
                expected_args
                    .iter()
                    .map(|value| value.to_string())
                    .collect(),
            ),
            invoke_started: false,
            stdout_chunks: Vec::new(),
            stderr_chunks: Vec::new(),
            result,
        });
        Self {
            calls: Mutex::new(calls),
        }
    }

    pub(crate) fn streaming_utf16(
        expected_args: &[&str],
        stdout_chunks: &[&str],
        status_code: i32,
    ) -> Self {
        let encoded_chunks = stdout_chunks
            .iter()
            .map(|chunk| encode_utf16le(chunk))
            .collect::<Vec<_>>();
        Self::streaming(expected_args, encoded_chunks, Vec::new(), status_code)
    }

    pub(crate) fn streaming_with_stderr(
        expected_args: &[&str],
        stdout_chunks: Vec<Vec<u8>>,
        stderr_chunks: Vec<Vec<u8>>,
        status_code: i32,
    ) -> Self {
        Self::streaming(expected_args, stdout_chunks, stderr_chunks, status_code)
    }

    pub(crate) fn streaming(
        expected_args: &[&str],
        stdout_chunks: Vec<Vec<u8>>,
        stderr_chunks: Vec<Vec<u8>>,
        status_code: i32,
    ) -> Self {
        let stdout = stdout_chunks.concat();
        let stderr = stderr_chunks.concat();
        let mut calls = VecDeque::new();
        calls.push_back(FakeCall {
            expected_args: Some(
                expected_args
                    .iter()
                    .map(|value| value.to_string())
                    .collect(),
            ),
            invoke_started: true,
            stdout_chunks,
            stderr_chunks,
            result: Ok(CommandOutput {
                status_code: Some(status_code),
                stdout,
                stderr,
            }),
        });
        Self {
            calls: Mutex::new(calls),
        }
    }
}

impl WslCommandRunner for FakeRunner {
    fn run<'a, FStarted, FStdout, FStderr>(
        &'a self,
        args: &'a [&'a str],
        _cancel_token: Option<tokio_util::sync::CancellationToken>,
        mut on_started: FStarted,
        mut on_stdout: FStdout,
        mut on_stderr: FStderr,
    ) -> Pin<Box<dyn Future<Output = io::Result<CommandOutput>> + Send + 'a>>
    where
        FStarted: FnMut() + Send + 'a,
        FStdout: FnMut(&[u8]) + Send + 'a,
        FStderr: FnMut(&[u8]) + Send + 'a,
    {
        let call = self
            .calls
            .lock()
            .expect("lock fake runner calls")
            .pop_front()
            .expect("fake runner call");

        if let Some(expected_args) = &call.expected_args {
            let actual_args = args
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>();
            assert_eq!(&actual_args, expected_args);
        }

        Box::pin(async move {
            if call.invoke_started {
                on_started();
            }

            for chunk in &call.stdout_chunks {
                on_stdout(chunk);
            }

            for chunk in &call.stderr_chunks {
                on_stderr(chunk);
            }

            call.result
        })
    }
}

pub(crate) fn encode_utf16le(value: &str) -> Vec<u8> {
    value
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>()
}

pub(crate) async fn collect_events(mut rx: mpsc::Receiver<ProgressEvent>) -> Vec<ProgressEvent> {
    let mut events = Vec::new();
    while let Some(event) = rx.recv().await {
        events.push(event);
    }
    events
}
