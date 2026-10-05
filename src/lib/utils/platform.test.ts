import { describe, expect, it } from "vitest";
import { detectPlatform, recommendMemory } from "./platform";
import { quickProfiles } from "./quickProfiles";
import type { Profile } from "$lib/stores/profiles.svelte";

describe("platform and memory", () => {
    it("detects native OS names and browser user agents", () => {
        expect(detectPlatform("Windows 11 Pro")).toBe("windows");
        expect(detectPlatform("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")).toBe("windows");
        expect(detectPlatform("Arch Linux")).toBe("linux");
        expect(detectPlatform("Darwin")).toBe("macos");
        expect(detectPlatform("Unknown")).toBe("unknown");
    });
    it("reserves RAM for the operating system and caps the game allocation", () => {
        expect(recommendMemory(4096)).toEqual({ minRamMb: 1024, maxRamMb: 2048 });
        expect(recommendMemory(8192).maxRamMb).toBe(4096);
        expect(recommendMemory(32768).maxRamMb).toBe(8192);
        expect(() => recommendMemory(NaN)).toThrow();
    });
});

describe("quick instance actions", () => {
    const create = (id: string, name: string, favorite: boolean, lastPlayed: number) => ({ id, name, favorite, lastPlayed, mcVersion: "1.21.1", loader: "fabric" }) as Profile;
    it("ranks favorites before recent instances without mutating the source", () => {
        const list = [create("a", "Antiga", false, 1), create("b", "Recente", false, 8), create("c", "Favorita", true, 2)];
        expect(quickProfiles(list, "").map(profile => profile.id)).toEqual(["c", "b", "a"]);
        expect(list.map(profile => profile.id)).toEqual(["a", "b", "c"]);
    });
    it("searches normalized names, loaders and Minecraft versions", () => {
        const list = [create("a", "Exploração", false, 1)];
        expect(quickProfiles(list, "  exploracao ")).toHaveLength(1);
        expect(quickProfiles(list, "1.21")).toHaveLength(1);
        expect(quickProfiles(list, "forge")).toHaveLength(0);
    });
});
