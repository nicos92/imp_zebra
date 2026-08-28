import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import PrintQuantityForm from "./PrintQuantityForm.vue";

describe("PrintQuantityForm", () => {
  it("mounts", () => {
    const wrapper = mount(PrintQuantityForm);
    expect(wrapper.exists()).toBe(true);
    expect(wrapper.text()).toContain("Imprimir");
  });

  it("emits print with quantity on submit", async () => {
    const wrapper = mount(PrintQuantityForm);
    await wrapper.find("form").trigger("submit");
    expect(wrapper.emitted("print")).toHaveLength(1);
    expect(wrapper.emitted("print")![0]).toEqual([100]);
  });
});
