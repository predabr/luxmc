import { describe, it, expect, beforeEach, vi } from "vitest";
import { themeStore, THEMES, ACCENTS } from "../theme.svelte";
import { settings } from "../settings.svelte";

describe("themeStore", () => {
  beforeEach(() => {
    localStorage.clear();
    themeStore.setTheme("dark");
    themeStore.setAccent("gold");
  });

  it("has default theme 'dark'", () => {
    expect(themeStore.theme).toBe("dark");
  });

  it("persists a video wallpaper independently of the media server port", () => {
    const path = "C:\\Luxmc\\wallpapers\\scene.mp4";
    themeStore.setCustomWallpaper(`http://127.0.0.1:58732/media?path=${encodeURIComponent(path)}`, "video", "Scene");
    expect(settings.value.customBackground).toBe("custom");
    expect(settings.value.customWallpaperUrl).toBe(path);
    expect(settings.value.customWallpaperType).toBe("video");
    expect(settings.value.wallpaperLibrary?.some(entry => entry.url === path && entry.type === "video")).toBe(true);
    themeStore.init();
    expect(themeStore.background).toBe("custom");
    expect(themeStore.customWallpaperUrl).toBe(path);
    expect(themeStore.customWallpaperType).toBe("video");
    themeStore.clearCustomWallpaper();
    expect(settings.value.customWallpaperUrl).toBe("");
    expect(settings.value.customBackground).toBe("obsidian");
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

  it("ACCENTS preserves the legacy accents and adds purple", () => {
    expect(Object.keys(ACCENTS)).toHaveLength(8);
    expect(ACCENTS).toHaveProperty("gold");
    expect(ACCENTS).toHaveProperty("cyan");
    expect(ACCENTS).toHaveProperty("emerald");
    expect(ACCENTS).toHaveProperty("rose");
    expect(ACCENTS).toHaveProperty("violet");
    expect(ACCENTS).toHaveProperty("orange");
    expect(ACCENTS).toHaveProperty("blue");
  });
});
