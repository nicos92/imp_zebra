import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import PrinterStatus from "./PrinterStatus.vue";
import type { Printer } from "../../types";

const printerFixture: Printer = {
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

describe("PrinterStatus", () => {
  it("mounts with a configured printer", () => {
    const wrapper = mount(PrinterStatus, {
      props: { printer: printerFixture, status: "connected", nextCode: "Z0000101" },
      global: {
        stubs: { RouterLink: { template: "<a><slot /></a>" } },
      },
    });
    expect(wrapper.exists()).toBe(true);
    expect(wrapper.text()).toContain("Zebra ZT410");
    expect(wrapper.text()).toContain("Z0000101");
  });

  it("renders data without router dependency when no printer", () => {
    const wrapper = mount(PrinterStatus, {
      props: { printer: null, status: "unknown", nextCode: "" },
      global: {
        stubs: { RouterLink: { template: "<a><slot /></a>" } },
      },
    });
    expect(wrapper.text()).toContain("No hay impresora configurada.");
  });
});
