import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import PrintProgress from "./PrintProgress.vue";

describe("PrintProgress", () => {
  it("renders nothing for idle", () => {
    const wrapper = mount(PrintProgress, { props: { stage: "idle" } });
    expect(wrapper.find(".print-progress").exists()).toBe(false);
  });

  it("renders status text for preparing", () => {
    const wrapper = mount(PrintProgress, { props: { stage: "preparing" } });
    expect(wrapper.text()).toContain("Preparando impresión...");
  });

  it("renders status text for sending", () => {
    const wrapper = mount(PrintProgress, { props: { stage: "sending" } });
    expect(wrapper.text()).toContain("Enviando datos...");
  });
});
