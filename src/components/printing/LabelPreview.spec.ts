import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import LabelPreview from "./LabelPreview.vue";

describe("LabelPreview", () => {
  it("renders loading state", () => {
    const wrapper = mount(LabelPreview, {
      props: { preview: null, loading: true },
    });
    expect(wrapper.text()).toContain("Generando vista previa...");
  });

  it("renders empty state", () => {
    const wrapper = mount(LabelPreview, {
      props: { preview: null, loading: false },
    });
    expect(wrapper.text()).toContain("La vista previa se genera al configurar la impresora.");
  });

  it("renders preview data", () => {
    const preview = { code: "Z0000001", timestamp: "2026-08-26T07:15:32Z", zpl: "^XA" };
    const wrapper = mount(LabelPreview, {
      props: { preview, loading: false },
    });
    expect(wrapper.text()).toContain("Z0000001");
  });
});
