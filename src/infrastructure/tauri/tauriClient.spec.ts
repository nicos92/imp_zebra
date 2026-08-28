import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { invokeCommand, isTauriError } from "./tauriClient";

describe("tauriClient", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("invokeCommand delegates to invoke with command and args", async () => {
    invokeMock.mockResolvedValueOnce({ ok: true });

    const result = await invokeCommand<{ ok: boolean }>("get_configured_printer", {
      id: "printer-1",
    });

    expect(invokeMock).toHaveBeenCalledWith("get_configured_printer", {
      id: "printer-1",
    });
    expect(result).toEqual({ ok: true });
  });

  it("invokeCommand works without args", async () => {
    invokeMock.mockResolvedValueOnce(null);

    await invokeCommand("get_current_sequence");

    expect(invokeMock).toHaveBeenCalledWith("get_current_sequence", undefined);
  });

  it("invokeCommand propagates rejections", async () => {
    invokeMock.mockRejectedValueOnce({
      code: "DATABASE_ERROR",
      message: "fail",
    });

    await expect(invokeCommand("print_labels")).rejects.toMatchObject({
      code: "DATABASE_ERROR",
      message: "fail",
    });
  });

  it("isTauriError accepts objects with code and message", () => {
    expect(isTauriError({ code: "X", message: "y" })).toBe(true);
  });

  it("isTauriError rejects null, primitives and partial objects", () => {
    expect(isTauriError(null)).toBe(false);
    expect(isTauriError("string")).toBe(false);
    expect(isTauriError({ code: "X" })).toBe(false);
    expect(isTauriError({ message: "y" })).toBe(false);
    expect(isTauriError({})).toBe(false);
  });
});
