import { getCopy } from "$lib/i18n";

const WSL_COMMAND_TIMED_OUT_MESSAGE =
  "The WSL command timed out before a stable result was available.";
const HOST_COMMAND_TIMED_OUT_MESSAGE =
  "The host command timed out before a stable result was available.";

type ErrorCopy = ReturnType<typeof getCopy>["common"]["errors"];
type WslErrorExtras = { wslCode?: string; distro?: string; details?: string };

const WSL_ERROR_LOCALIZERS = {
  invalidArgument: (errors: ErrorCopy) => errors.invalidWslArguments,
  fileNotFound: (errors: ErrorCopy) => errors.fileNotFound,
  distroNotFound: (errors: ErrorCopy) => errors.distroNotFound,
  diskResizeFailed: (errors: ErrorCopy) => errors.diskResizeFailed,
  operationNotPermitted: (errors: ErrorCopy, error: WslErrorExtras) =>
    errors.wslOperationNotPermitted(error.distro ?? ""),
  unknownWslError: (errors: ErrorCopy, error: WslErrorExtras) =>
    errors.wslCommandFailed(error.wslCode ?? "unknown", error.details),
  registryReadFailed: (errors: ErrorCopy) => errors.registryReadFailed,
  outputParseFailed: (errors: ErrorCopy) => errors.outputParseFailed,
  wslCommandTimedOut: (errors: ErrorCopy) => errors.wslCommandTimedOut,
  processFailed: (errors: ErrorCopy) => errors.processFailed,
  processKilled: (errors: ErrorCopy) => errors.processKilled,
  cancelled: (errors: ErrorCopy) => errors.cancelled,
} as const;

export type WslCommandErrorCode = keyof typeof WSL_ERROR_LOCALIZERS;
export type WslCommandErrorDto = WslErrorExtras & {
  kind: "wsl";
  code: WslCommandErrorCode;
};
export type MessageCommandErrorDto = { kind: "message"; message: string };
export type RecoverableCommandCode =
  "wsl-command-timed-out" | "host-command-timed-out";
export type PersistedCommandError =
  | WslCommandErrorDto
  | MessageCommandErrorDto
  | { kind: "recoverable"; code: RecoverableCommandCode }
  | { kind: "unknown" };

export class RecoverableCommandError extends Error {
  constructor(readonly code: RecoverableCommandCode, message: string) {
    super(message);
    this.name = "RecoverableCommandError";
  }
}

export function normalizeTauriCommandError(error: unknown): unknown {
  if (isWslCommandErrorDto(error)) {
    return error.code === "wslCommandTimedOut"
      ? new RecoverableCommandError(
          "wsl-command-timed-out",
          WSL_COMMAND_TIMED_OUT_MESSAGE,
        )
      : error;
  }

  if (isMessageCommandErrorDto(error)) {
    return error;
  }

  return getErrorMessage(error) === HOST_COMMAND_TIMED_OUT_MESSAGE
    ? new RecoverableCommandError(
        "host-command-timed-out",
        HOST_COMMAND_TIMED_OUT_MESSAGE,
      )
    : error;
}

export function getTauriCommandErrorMessage(error: unknown): string | null {
  if (error instanceof RecoverableCommandError) {
    return getRecoverableCommandErrorMessage(error.code);
  }

  if (isWslCommandErrorDto(error)) {
    return WSL_ERROR_LOCALIZERS[error.code](getCopy().common.errors, error);
  }

  return isMessageCommandErrorDto(error) ? error.message : null;
}

export function persistCommandError(error: unknown): PersistedCommandError {
  if (isWslCommandErrorDto(error)) {
    return {
      kind: "wsl",
      code: error.code,
      ...(typeof error.wslCode === "string" ? { wslCode: error.wslCode } : {}),
      ...(typeof error.distro === "string" ? { distro: error.distro } : {}),
      ...(typeof error.details === "string" ? { details: error.details } : {}),
    };
  }

  if (isMessageCommandErrorDto(error)) {
    return {
      kind: "message",
      message: error.message,
    };
  }

  if (error instanceof RecoverableCommandError) {
    return {
      kind: "recoverable",
      code: error.code,
    };
  }

  return {
    kind: "unknown",
  };
}

export function getPersistedCommandErrorMessage(
  error: PersistedCommandError,
): string {
  if (isWslCommandErrorDto(error)) {
    return WSL_ERROR_LOCALIZERS[error.code](getCopy().common.errors, error);
  }

  if (isMessageCommandErrorDto(error)) {
    return error.message;
  }

  if (isPersistedRecoverableCommandError(error)) {
    return (
      getRecoverableCommandErrorMessage(error.code) ??
      getCopy().common.errors.operationFailed
    );
  }

  return getCopy().common.errors.operationFailed;
}

export function isRecoverableCommandError(
  error: unknown,
): error is RecoverableCommandError {
  return error instanceof RecoverableCommandError;
}

export function isWslCommandErrorDto(error: unknown): error is WslCommandErrorDto {
  return (
    isObject(error) &&
    error.kind === "wsl" &&
    typeof error.code === "string" &&
    error.code in WSL_ERROR_LOCALIZERS
  );
}

export function isMessageCommandErrorDto(
  error: unknown,
): error is MessageCommandErrorDto {
  return isObject(error) && error.kind === "message" && typeof error.message === "string";
}

function isPersistedRecoverableCommandError(
  error: unknown,
): error is Extract<PersistedCommandError, { kind: "recoverable" }> {
  return (
    isObject(error) &&
    error.kind === "recoverable" &&
    (error.code === "wsl-command-timed-out" ||
      error.code === "host-command-timed-out")
  );
}

function getRecoverableCommandErrorMessage(
  code: RecoverableCommandCode,
): string | null {
  return code === "wsl-command-timed-out"
    ? getCopy().common.errors.wslCommandTimedOut
    : null;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function getErrorMessage(error: unknown): string | null {
  return typeof error === "string"
    ? error
    : error instanceof Error
      ? error.message
      : null;
}
