import { describe, expect, it } from "vitest";
import { YugenDbError, isYugenDbError } from "./errors";

describe("YugenDbError", () => {
  it("keeps the stable error code and retryable flag", () => {
    const error = new YugenDbError({
      code: "TIMEOUT",
      message: "The operation timed out.",
      retryable: true,
    });

    expect(error.code).toBe("TIMEOUT");
    expect(error.retryable).toBe(true);
    expect(error.name).toBe("YugenDbError");
  });

  it("can be identified with isYugenDbError", () => {
    expect(isYugenDbError(new YugenDbError({ code: "NOT_FOUND", message: "Missing." }))).toBe(true);
    expect(isYugenDbError(new Error("plain"))).toBe(false);
  });
});
