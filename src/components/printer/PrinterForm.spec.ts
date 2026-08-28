import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import PrinterForm from "./PrinterForm.vue";

describe("PrinterForm", () => {
  it("mounts with defaults", () => {
    const wrapper = mount(PrinterForm);
    expect(wrapper.exists()).toBe(true);
    expect(wrapper.text()).toContain("Guardar");
  });

  it("mounts with initial config", () => {
    const wrapper = mount(PrinterForm, {
      props: {
        initial: {
          name: "Zebra ZT410",
          model: "ZT410",
          dpi: 203,
          label_width_mm: 50,
          label_height_mm: 50,
          columns: 2,
          connection_type: "tcp",
          ip_address: "192.168.1.100",
          port: 9100,
        },
      },
    });
    expect(wrapper.exists()).toBe(true);
  });
});
