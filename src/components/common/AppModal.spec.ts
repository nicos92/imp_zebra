import { afterEach, describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import AppModal from "./AppModal.vue";

describe("AppModal", () => {
  afterEach(() => {
    document.body.innerHTML = "";
  });

  it("mounts closed without rendering dialog", () => {
    mount(AppModal, { props: { open: false } });
    expect(document.body.querySelector(".app-modal")).toBeNull();
  });

  it("renders dialog with title and slot when open", () => {
    mount(AppModal, {
      props: { open: true, title: "Vista previa" },
      slots: { default: "<p>contenido</p>" },
    });
    const dialog = document.body.querySelector(".app-modal__dialog");
    expect(dialog).not.toBeNull();
    expect(dialog?.textContent).toContain("Vista previa");
    expect(dialog?.textContent).toContain("contenido");
  });

  it("emits close on close button", async () => {
    const wrapper = mount(AppModal, { props: { open: true } });
    const close = document.body.querySelector<HTMLElement>(".app-modal__close");
    close?.click();
    await wrapper.vm.$nextTick();
    expect(wrapper.emitted("close")).toHaveLength(1);
  });

  it("emits close on overlay click when closable", async () => {
    const wrapper = mount(AppModal, { props: { open: true } });
    document.body.querySelector<HTMLElement>(".app-modal")?.click();
    await wrapper.vm.$nextTick();
    expect(wrapper.emitted("close")).toHaveLength(1);
  });

  it("does not close on overlay click when closable is false", async () => {
    const wrapper = mount(AppModal, { props: { open: true, closable: false } });
    document.body.querySelector<HTMLElement>(".app-modal")?.click();
    await wrapper.vm.$nextTick();
    expect(wrapper.emitted("close")).toBeUndefined();
  });
});
