import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import HistoryView from "./HistoryView.vue";
import type { PrintJob } from "../types";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

function jobFixture(overrides: Partial<PrintJob> = {}): PrintJob {
  return {
    id: "job-1",
    printer_id: "printer-1",
    start_code: "Z0000001",
    end_code: "Z0000004",
    quantity: 4,
    status: "completed",
    created_at: "2026-08-28T08:00:00Z",
    completed_at: "2026-08-28T08:00:05Z",
    ...overrides,
  };
}

describe("HistoryView", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads and renders the job table", async () => {
    invokeMock.mockResolvedValueOnce([
      jobFixture(),
      jobFixture({ id: "job-2", status: "failed", start_code: "Z0000005", end_code: "Z0000008" }),
    ]);

    const wrapper = mount(HistoryView);
    await flushPromises();

    expect(invokeMock).toHaveBeenCalledWith("list_print_jobs", { limit: 50 });
    expect(wrapper.find(".history__table").exists()).toBe(true);
    expect(wrapper.findAll("tbody tr")).toHaveLength(2);
    expect(wrapper.text()).toContain("Z0000001");
    expect(wrapper.text()).toContain("Z0000004");
    expect(wrapper.text()).toContain("Completado");
    expect(wrapper.text()).toContain("Fallido");
  });

  it("shows the empty state when there are no jobs", async () => {
    invokeMock.mockResolvedValueOnce([]);

    const wrapper = mount(HistoryView);
    await flushPromises();

    expect(wrapper.text()).toContain("No hay trabajos de impresión registrados.");
    expect(wrapper.find(".history__table").exists()).toBe(false);
  });

  it("shows an error message when loading fails", async () => {
    invokeMock.mockRejectedValueOnce({
      code: "DATABASE_ERROR",
      message: "No se pudo leer el historial",
    });

    const wrapper = mount(HistoryView);
    await flushPromises();

    expect(wrapper.find("[role='alert']").text()).toContain(
      "No se pudo leer el historial",
    );
  });

  it("reloads the list when clicking Actualizar", async () => {
    invokeMock.mockResolvedValueOnce([]);
    invokeMock.mockResolvedValueOnce([jobFixture()]);

    const wrapper = mount(HistoryView);
    await flushPromises();
    expect(wrapper.text()).toContain("No hay trabajos de impresión registrados.");

    await wrapper.find("button").trigger("click");
    await flushPromises();

    expect(invokeMock).toHaveBeenCalledTimes(2);
    expect(wrapper.text()).toContain("Z0000001");
  });

  it("opens the detail panel and fetches job detail on Detalle", async () => {
    invokeMock.mockResolvedValueOnce([jobFixture()]);
    invokeMock.mockResolvedValueOnce(jobFixture({ printer_id: "printer-7", quantity: 9 }));

    const wrapper = mount(HistoryView);
    await flushPromises();

    const detailButtons = wrapper.findAll("button");
    const detailButton = detailButtons.find((b) => b.text() === "Detalle");
    expect(detailButton).toBeDefined();

    await detailButton!.trigger("click");
    await flushPromises();

    expect(invokeMock).toHaveBeenCalledWith("get_print_job", { jobId: "job-1" });
    expect(wrapper.find(".history__detail").exists()).toBe(true);
    expect(wrapper.text()).toContain("printer-7");
    expect(wrapper.text()).toContain("Código inicial");
  });

  it("shows an error when fetching the detail fails", async () => {
    invokeMock.mockResolvedValueOnce([jobFixture()]);
    invokeMock.mockRejectedValueOnce({
      code: "DATABASE_ERROR",
      message: "No se pudo cargar el detalle",
    });

    const wrapper = mount(HistoryView);
    await flushPromises();

    const detailButton = wrapper.findAll("button").find((b) => b.text() === "Detalle");
    await detailButton!.trigger("click");
    await flushPromises();

    expect(wrapper.find("[role='alert']").text()).toContain(
      "No se pudo cargar el detalle",
    );
  });
});