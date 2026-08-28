import { beforeEach, describe, expect, it, vi } from "vitest";
import { setActivePinia, createPinia } from "pinia";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { usePrinterStore } from "./printer";
import type { Printer, SequenceInfo } from "../types";

function printerFixture(): Printer {
  return {
    id: "printer-1",
    name: "Zebra ZT410",
    model: "ZT410",
    dpi: 203,
    label_width_mm: 50,
    label_height_mm: 50,
    columns: 2,
    connection_type: "tcp",
    ip_address: "192.168.1.100",
    port: 9100,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  };
}

function sequenceFixture(): SequenceInfo {
  return { last_used_code: "Z0000100", next_code: "Z0000101" };
}

describe("printer store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  it("loads printer and sequence", async () => {
    invokeMock.mockResolvedValueOnce(printerFixture());
    invokeMock.mockResolvedValueOnce(sequenceFixture());

    const store = usePrinterStore();
    await store.load();

    expect(store.printer?.name).toBe("Zebra ZT410");
    expect(store.nextCode).toBe("Z0000101");
    expect(store.hasPrinter).toBe(true);
    expect(store.loading).toBe(false);
    expect(store.error).toBeNull();
    expect(invokeMock).toHaveBeenCalledWith("get_configured_printer", undefined);
    expect(invokeMock).toHaveBeenCalledWith("get_current_sequence", undefined);
  });

  it("handles load error", async () => {
    invokeMock.mockRejectedValueOnce({
      code: "DATABASE_ERROR",
      message: "boom",
    });

    const store = usePrinterStore();
    await store.load();

    expect(store.error).toBe("boom");
    expect(store.loading).toBe(false);
  });

  it("sets printer manually", () => {
    const store = usePrinterStore();
    store.setPrinter(printerFixture());
    expect(store.hasPrinter).toBe(true);
  });

  it("tracks connection status", () => {
    const store = usePrinterStore();
    expect(store.status).toBe("unknown");
    store.setStatus("connected");
    expect(store.connected).toBe(true);
  });
});
