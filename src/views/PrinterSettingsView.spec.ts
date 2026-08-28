import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import PrinterSettingsView from "./PrinterSettingsView.vue";
import PrinterForm from "../components/printer/PrinterForm.vue";
import { usePrinterStore } from "../stores/printer";
import type { Printer, PrinterConfig, SequenceInfo } from "../types";

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

function configFixture(): PrinterConfig {
  return {
    name: "Nueva Zebra",
    model: "ZD421",
    dpi: 203,
    label_width_mm: 50,
    label_height_mm: 50,
    columns: 2,
    connection_type: "tcp",
    ip_address: "192.168.1.50",
    port: 9100,
  };
}

async function mountView(): Promise<ReturnType<typeof mount>> {
  const wrapper = mount(PrinterSettingsView);
  await flushPromises();
  return wrapper;
}

describe("PrinterSettingsView", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
  });

  it("loads an existing printer into the form", async () => {
    invokeMock.mockResolvedValueOnce(printerFixture());
    invokeMock.mockResolvedValueOnce(sequenceFixture());

    const wrapper = await mountView();
    const form = wrapper.findComponent(PrinterForm);

    expect(form.props("initial")?.name).toBe("Zebra ZT410");
    expect(form.props("initial")?.ip_address).toBe("192.168.1.100");
  });

  it("saves a new config and shows a success message", async () => {
    invokeMock.mockResolvedValueOnce(null);
    invokeMock.mockResolvedValueOnce(sequenceFixture());
    invokeMock.mockResolvedValueOnce(printerFixture());
    invokeMock.mockResolvedValueOnce(printerFixture());
    invokeMock.mockResolvedValueOnce(sequenceFixture());

    const wrapper = await mountView();
    wrapper.findComponent(PrinterForm).vm.$emit("submit", configFixture());
    await flushPromises();

    expect(invokeMock).toHaveBeenCalledWith("save_printer_config", {
      config: configFixture(),
    });
    expect(wrapper.find("[role='status']").text()).toContain("Configuración guardada.");
    const store = usePrinterStore();
    expect(store.printer?.name).toBe("Zebra ZT410");
  });

  it("shows an error message when saving fails", async () => {
    invokeMock.mockResolvedValueOnce(null);
    invokeMock.mockResolvedValueOnce(sequenceFixture());
    invokeMock.mockRejectedValueOnce({
      code: "DATABASE_ERROR",
      message: "No se pudo guardar la configuración",
    });

    const wrapper = await mountView();
    wrapper.findComponent(PrinterForm).vm.$emit("submit", configFixture());
    await flushPromises();

    const message = wrapper.find(".settings__message--error");
    expect(message.text()).toContain("No se pudo guardar la configuración");
  });

  it("persists a brand new printer before testing the connection", async () => {
    invokeMock.mockResolvedValueOnce(null);
    invokeMock.mockResolvedValueOnce(sequenceFixture());
    invokeMock.mockResolvedValueOnce(printerFixture());
    invokeMock.mockResolvedValueOnce(true);

    const wrapper = await mountView();
    wrapper.findComponent(PrinterForm).vm.$emit("test", configFixture());
    await flushPromises();

    expect(invokeMock).toHaveBeenCalledWith("save_printer_config", {
      config: configFixture(),
    });
    expect(invokeMock).toHaveBeenCalledWith("test_printer_connection", {
      printerId: "printer-1",
    });
    expect(wrapper.find("[role='status']").text()).toContain(
      "Conexión exitosa con la impresora.",
    );
    const store = usePrinterStore();
    expect(store.connected).toBe(true);
  });

  it("tests an already-persisted printer without saving again", async () => {
    invokeMock.mockResolvedValueOnce(printerFixture());
    invokeMock.mockResolvedValueOnce(sequenceFixture());
    invokeMock.mockResolvedValueOnce(true);

    const wrapper = await mountView();
    wrapper.findComponent(PrinterForm).vm.$emit("test", configFixture());
    await flushPromises();

    expect(invokeMock).not.toHaveBeenCalledWith("save_printer_config", {
      config: configFixture(),
    });
    expect(invokeMock).toHaveBeenCalledWith("test_printer_connection", {
      printerId: "printer-1",
    });
  });
});