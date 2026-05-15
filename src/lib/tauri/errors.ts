const WSL_COMMAND_TIMED_OUT_MESSAGE =
  "The WSL command timed out before a stable result was available.";
const HOST_COMMAND_TIMED_OUT_MESSAGE =
  "The host command timed out before a stable result was available.";

export type RecoverableCommandCode =
  | "wsl-command-timed-out"
  | "host-command-timed-out";

export class RecoverableCommandError extends Error {
  readonly code: RecoverableCommandCode;

  constructor(code: RecoverableCommandCode, message: string) {
    super(message);
    this.name = "RecoverableCommandError";
    this.code = code;
  }
}

export function normalizeTauriCommandError(error: unknown): Error | unknown {
  const message =
    typeof error === "string"
      ? error
      : error instanceof Error
        ? error.message
        : null;

  if (message === WSL_COMMAND_TIMED_OUT_MESSAGE) {
    return new RecoverableCommandError("wsl-command-timed-out", message);
  }

  if (message === HOST_COMMAND_TIMED_OUT_MESSAGE) {
    return new RecoverableCommandError("host-command-timed-out", message);
  }

  return error;
}

export function isRecoverableCommandError(
  error: unknown,
): error is RecoverableCommandError {
  return error instanceof RecoverableCommandError;
}
