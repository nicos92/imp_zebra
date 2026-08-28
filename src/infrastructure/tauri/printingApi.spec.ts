import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { listPrintJobs, previewLabel, printLabels } from "./printingApi";

describe("printingApi", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("printLabels invokes with the request payload", async () => {
    invokeMock.mockResolvedValueOnce({
      job_id: "job-1",
      start_code: "Z0000101",
      end_code: "Z0000200",
      quantity: 100,
      status: "pending",
    });

    const result = await printLabels({ quantity: 100, printer_id: "printer-1" });

    expect(invokeMock).toHaveBeenCalledWith("print_labels", {
      request: { quantity: 100, printer_id: "printer-1" },
    });
    expect(result.job_id).toBe("job-1");
  });

  it("previewLabel maps dimensions to command args", async () => {
    invokeMock.mockResolvedValueOnce({
      code: "Z0000001",
      timestamp: "2026-08-26T07:15:32Z",
      zpl: "^XA...",
    });

    const preview = await previewLabel(50, 50, 2, 203);

    expect(invokeMock).toHaveBeenCalledWith("preview_label", {
      labelWidthMm: 50,
      labelHeightMm: 50,
      columns: 2,
      dpi: 203,
    });
    expect(preview.code).toBe("Z0000001");
  });

  it("listPrintJobs passes limit", async () => {
    invokeMock.mockResolvedValueOnce([]);

    const jobs = await listPrintJobs(50);

    expect(invokeMock).toHaveBeenCalledWith("list_print_jobs", { limit: 50 });
    expect(jobs).toEqual([]);
  });

  it("propagates rejected errors from invoke", async () => {
    invokeMock.mockRejectedValueOnce({
      code: "PRINTER_UNAVAILABLE",
      message: "Impresora no disponible",
    });

    await expect(printLabels({ quantity: 1, printer_id: "printer-1" })).rejects.toMatchObject({
      code: "PRINTER_UNAVAILABLE",
      message: "Impresora no disponible",
    });
  });
});
