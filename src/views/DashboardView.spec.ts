import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import DashboardView from "./DashboardView.vue";
import PrintQuantityForm from "../components/printing/PrintQuantityForm.vue";
import type { LabelPreview, Printer, PrintResult, SequenceInfo } from "../types";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

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
  return { last_used_code: "Z0000000", next_code: "Z0000001" };
}

function previewFixture(): LabelPreview {
  return { code: "Z0000001", timestamp: "28/08/2026 08:00:00", zpl: "^XA^XZ" };
}

function printResultFixture(): PrintResult {
  return {
    job_id: "job-1",
    start_code: "Z0000001",
    end_code: "Z0000007",
    quantity: 7,
    status: "completed",
  };
}

describe("DashboardView", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  it("loads printer, sequence and preview on mount", async () => {
    invokeMock.mockResolvedValueOnce(printerFixture());
    invokeMock.mockResolvedValueOnce(sequenceFixture());
    invokeMock.mockResolvedValueOnce(previewFixture());

    const wrapper = mount(DashboardView, {
      global: {
        stubs: { RouterLink: { template: "<a><slot /></a>" } },
      },
    });
    await flushPromises();

    expect(wrapper.text()).toContain("Zebra ZT410");
    expect(wrapper.text()).toContain("Z0000001");
    expect(wrapper.text()).toContain(previewFixture().code);
    expect(invokeMock).toHaveBeenCalledWith("get_configured_printer", undefined);
    expect(invokeMock).toHaveBeenCalledWith("get_current_sequence", undefined);
    expect(invokeMock).toHaveBeenCalledWith(
      "preview_label",
      expect.objectContaining({ labelWidthMm: 50, columns: 2 }),
    );
  });

  it("shows empty state and disables printing without a printer", async () => {
    invokeMock.mockResolvedValueOnce(null);
    invokeMock.mockResolvedValueOnce(sequenceFixture());

    const wrapper = mount(DashboardView, {
      global: {
        stubs: { RouterLink: { template: "<a><slot /></a>" } },
      },
    });
    await flushPromises();

    expect(wrapper.text()).toContain("No hay impresora configurada.");
    expect(wrapper.findComponent(PrintQuantityForm).props("disabled")).toBe(true);
  });

  it("prints labels, refreshes state and shows the result", async () => {
    invokeMock.mockResolvedValueOnce(printerFixture());
    invokeMock.mockResolvedValueOnce(sequenceFixture());
    invokeMock.mockResolvedValueOnce(previewFixture());
    invokeMock.mockResolvedValueOnce(printResultFixture());
    invokeMock.mockResolvedValueOnce(printerFixture());
    invokeMock.mockResolvedValueOnce(sequenceFixture());
    invokeMock.mockResolvedValueOnce(previewFixture());

    const wrapper = mount(DashboardView, {
      global: {
        stubs: { RouterLink: { template: "<a><slot /></a>" } },
      },
    });
    await flushPromises();

    wrapper.findComponent(PrintQuantityForm).vm.$emit("print", 7);
    await flushPromises();

    expect(invokeMock).toHaveBeenCalledWith("print_labels", {
      request: { quantity: 7, printer_id: "printer-1" },
    });
    expect(wrapper.text()).toContain("Impresión enviada");
    expect(wrapper.text()).toContain("Z0000001");
    expect(wrapper.text()).toContain("Z0000007");
    expect(wrapper.text()).toContain("completed");
  });

  it("surfaces backend errors from printing", async () => {
    invokeMock.mockResolvedValueOnce(printerFixture());
    invokeMock.mockResolvedValueOnce(sequenceFixture());
    invokeMock.mockResolvedValueOnce(previewFixture());
    invokeMock.mockRejectedValueOnce({
      code: "PRINT_JOB_FAILED",
      message: "No se pudo contactar la impresora",
    });

    const wrapper = mount(DashboardView, {
      global: {
        stubs: { RouterLink: { template: "<a><slot /></a>" } },
      },
    });
    await flushPromises();

    wrapper.findComponent(PrintQuantityForm).vm.$emit("print", 7);
    await flushPromises();

    expect(wrapper.find("[role='alert']").text()).toContain(
      "No se pudo contactar la impresora",
    );
    expect(wrapper.text()).not.toContain("Impresión enviada");
  });
});