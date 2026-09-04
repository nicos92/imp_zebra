import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import {
  getConfiguredPrinter,
  getCurrentSequence,
  getPrinterConfig,
  savePrinterConfig,
} from "./printerApi";
import { commandErrorMessage } from "./tauriClient";

describe("printerApi", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("getConfiguredPrinter invokes without args", async () => {
    invokeMock.mockResolvedValueOnce(null);
    const printer = await getConfiguredPrinter();
    expect(invokeMock).toHaveBeenCalledWith("get_configured_printer", undefined);
    expect(printer).toBeNull();
  });

  it("getCurrentSequence invokes and returns sequence", async () => {
    invokeMock.mockResolvedValueOnce({
      last_used_code: "Z0000100",
      next_code: "Z0000101",
    });
    const seq = await getCurrentSequence();
    expect(invokeMock).toHaveBeenCalledWith("get_current_sequence", undefined);
    expect(seq.next_code).toBe("Z0000101");
  });

  it("getPrinterConfig passes the id", async () => {
    invokeMock.mockResolvedValueOnce(null);
    await getPrinterConfig("printer-1");
    expect(invokeMock).toHaveBeenCalledWith("get_printer_config", {
      id: "printer-1",
    });
  });

  it("savePrinterConfig passes the config", async () => {
    const config = {
      name: "Zebra",
      model: "ZT410",
      dpi: 203,
      label_width_mm: 50,
      label_height_mm: 50,
      columns: 2,
      connection_type: "tcp" as const,
      ip_address: "192.168.1.100",
      port: 9100,
    };
    invokeMock.mockResolvedValueOnce({ id: "printer-1", ...config });
    await savePrinterConfig(config);
    expect(invokeMock).toHaveBeenCalledWith("save_printer_config", { config });
  });
});

describe("commandErrorMessage", () => {
  it("extracts message from a tauri error envelope", () => {
    const msg = commandErrorMessage({
      code: "PRINTER_CONNECTION_FAILED",
      message: "No fue posible conectarse",
    });
    expect(msg).toBe("No fue posible conectarse");
  });

  it("falls back to Error message", () => {
    expect(commandErrorMessage(new Error("generic"))).toBe("generic");
  });

  it("falls back to string for unknown values", () => {
    expect(commandErrorMessage("raw")).toBe("raw");
  });
});
