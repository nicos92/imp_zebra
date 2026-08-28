import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import PrintResult from "./PrintResult.vue";

describe("PrintResult", () => {
  it("renders nothing when result is null", () => {
    const wrapper = mount(PrintResult, { props: { result: null } });
    expect(wrapper.find(".print-result").exists()).toBe(false);
  });

  it("renders result data", () => {
    const result = {
      job_id: "job-1",
      start_code: "Z0000101",
      end_code: "Z0000200",
      quantity: 100,
      status: "completed",
    };
    const wrapper = mount(PrintResult, { props: { result } });
    expect(wrapper.exists()).toBe(true);
  });
});
