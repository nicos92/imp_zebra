import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import AppButton from "./AppButton.vue";

describe("AppButton", () => {
  it("mounts", () => {
    const wrapper = mount(AppButton);
    expect(wrapper.exists()).toBe(true);
  });

  it("renders slot content", () => {
    const wrapper = mount(AppButton, { slots: { default: "Imprimir" } });
    expect(wrapper.text()).toContain("Imprimir");
  });
});
