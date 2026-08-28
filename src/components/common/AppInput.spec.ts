import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import AppInput from "./AppInput.vue";

describe("AppInput", () => {
  it("mounts", () => {
    const wrapper = mount(AppInput);
    expect(wrapper.exists()).toBe(true);
  });

  it("renders label and input", () => {
    const wrapper = mount(AppInput, { props: { label: "Cantidad" } });
    expect(wrapper.text()).toContain("Cantidad");
    expect(wrapper.find("input").exists()).toBe(true);
  });
});
