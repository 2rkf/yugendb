/** Stable error codes returned by yugendb TypeScript packages. */
export type YugenDbErrorCode =
  | "NOT_FOUND"
  | "CONNECTION_ERROR"
  | "SERIALISATION_ERROR"
  | "DESERIALISATION_ERROR"
  | "TIMEOUT"
  | "CONFLICT"
  | "TRANSACTION_ABORTED"
  | "UNSUPPORTED_FEATURE"
  | "CONSTRAINT_VIOLATION"
  | "INVALID_KEY"
  | "INVALID_NAMESPACE"
  | "INVALID_COLLECTION"
  | "INVALID_VALUE"
  | "DRIVER_ERROR"
  | "INTERNAL_ERROR";

/** Input used to create a structured yugendb error. */
export interface YugenDbErrorOptions {
  /** Stable machine-readable error code. */
  readonly code: YugenDbErrorCode;
  /** Human-readable error message. */
  readonly message: string;
  /** Original error or value that caused this error. */
  readonly cause?: unknown;
  /** Whether retrying the operation may succeed. */
  readonly retryable?: boolean;
}

/** A structured yugendb error with a stable cross-language code. */
export class YugenDbError extends Error {
  /** Stable machine-readable error code. */
  readonly code: YugenDbErrorCode;
  /** Whether retrying the operation may succeed. */
  readonly retryable: boolean;

  constructor(options: YugenDbErrorOptions) {
    super(options.message);
    this.name = "YugenDbError";
    this.code = options.code;
    this.retryable = options.retryable ?? false;

    if ("cause" in options) {
      (this as Error & { cause?: unknown }).cause = options.cause;
    }
  }
}

/** Returns true when a value is a yugendb error. */
export function isYugenDbError(error: unknown): error is YugenDbError {
  return error instanceof YugenDbError;
}

/** Converts any thrown value into a structured yugendb error. */
export function toYugenDbError(
  error: unknown,
  fallback: { code: YugenDbErrorCode; message: string; retryable?: boolean },
): YugenDbError {
  if (isYugenDbError(error)) {
    return error;
  }

  return new YugenDbError({
    code: fallback.code,
    message: fallback.message,
    cause: error,
    ...(fallback.retryable !== undefined ? { retryable: fallback.retryable } : {}),
  });
}
