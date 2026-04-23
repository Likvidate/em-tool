import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import ColorSwatches from "./ColorSwatches.vue";

describe("ColorSwatches", () => {
  it("renders 5 swatches in canonical order", () => {
    const w = mount(ColorSwatches, { props: { modelValue: null } });
    const buttons = w.findAll("button.sw");
    expect(buttons).toHaveLength(5);
    expect(buttons[0].classes()).toContain("red");
    expect(buttons[1].classes()).toContain("yellow");
    expect(buttons[2].classes()).toContain("grey");
    expect(buttons[3].classes()).toContain("green");
    expect(buttons[4].classes()).toContain("blue");
  });

  it("marks the current selection active", () => {
    const w = mount(ColorSwatches, { props: { modelValue: "yellow" } });
    const yellow = w.findAll("button.sw")[1];
    expect(yellow.classes()).toContain("active");
  });

  it("emits update:modelValue on click", async () => {
    const w = mount(ColorSwatches, { props: { modelValue: null } });
    await w.findAll("button.sw")[3].trigger("click");   // green
    expect(w.emitted("update:modelValue")?.[0]).toEqual(["green"]);
  });

  it("clicking the active color clears the selection", async () => {
    const w = mount(ColorSwatches, { props: { modelValue: "red" } });
    await w.findAll("button.sw")[0].trigger("click");
    expect(w.emitted("update:modelValue")?.[0]).toEqual([null]);
  });
});
