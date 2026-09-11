import { describe, it, expect, beforeEach, vi } from "vitest";
import { themeStore, THEMES, ACCENTS } from "../theme.svelte";

describe("themeStore", () => {
  beforeEach(() => {
    localStorage.clear();
    themeStore.setTheme("dark");
    themeStore.setAccent("gold");
  });

  it("has default theme 'dark'", () => {
    expect(themeStore.theme).toBe("dark");
  });

  it("has default accent 'gold'", () => {
    expect(themeStore.accent).toBe("gold");
  });

  it("can set valid theme", () => {
    themeStore.setTheme("light");
    expect(themeStore.theme).toBe("light");
  });

  it("ignores invalid theme", () => {
    themeStore.setTheme("invalid");
    expect(themeStore.theme).toBe("dark");
  });

  it("can set valid accent", () => {
    themeStore.setAccent("cyan");
    expect(themeStore.accent).toBe("cyan");
  });

  it("ignores invalid accent", () => {
    themeStore.setAccent("invalid");
    expect(themeStore.accent).toBe("gold");
  });

  it("returns current accent data", () => {
    const data = themeStore.currentAccentData;
    expect(data).toHaveProperty("hex");
    expect(data).toHaveProperty("rgb");
    expect(data.id).toBe("gold");
  });

  it("THEMES has dark and light", () => {
    expect(THEMES).toHaveProperty("dark");
    expect(THEMES).toHaveProperty("light");
  });

  it("ACCENTS has all 7 accents", () => {
    expect(Object.keys(ACCENTS)).toHaveLength(7);
    expect(ACCENTS).toHaveProperty("gold");
    expect(ACCENTS).toHaveProperty("cyan");
    expect(ACCENTS).toHaveProperty("emerald");
    expect(ACCENTS).toHaveProperty("rose");
    expect(ACCENTS).toHaveProperty("violet");
    expect(ACCENTS).toHaveProperty("orange");
    expect(ACCENTS).toHaveProperty("blue");
  });
});
