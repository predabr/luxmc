import { describe, expect, it } from "vitest";
import { catalogCompatibility } from "./catalogCompatibility";

describe("catalog compatibility", () => {
    const mod = { versions: ["1.21.1"], categories: ["fabric", "optimization"] };
    it("rejects vanilla, another loader and another Minecraft version for a mod", () => {
        expect(catalogCompatibility(mod, { mcVersion: "1.21.1", loader: "vanilla" }, "Mod")).toBeTruthy();
        expect(catalogCompatibility(mod, { mcVersion: "1.21.1", loader: "forge" }, "Mod")).toBeTruthy();
        expect(catalogCompatibility(mod, { mcVersion: "1.20.1", loader: "fabric" }, "Mod")).toBeTruthy();
        expect(catalogCompatibility(mod, { mcVersion: "1.21.1", loader: "fabric" }, "Mod")).toBeNull();
    });
    it("does not impose mod loaders on resource packs or shaders", () => {
        expect(catalogCompatibility(mod, { mcVersion: "1.21.1", loader: "vanilla" }, "Resource Pack")).toBeNull();
        expect(catalogCompatibility(mod, { mcVersion: "1.21.1", loader: "forge" }, "Shader")).toBeNull();
    });
    it("allows selecting a modpack instance before validating actual version files", () => {
        expect(catalogCompatibility(mod, { mcVersion: "26.3", loader: "fabric" }, "Mod", false)).toBeNull();
        expect(catalogCompatibility(mod, { mcVersion: "26.3", loader: "vanilla" }, "Resource Pack", false)).toBeNull();
    });
    it("accepts missing catalog metadata while still rejecting vanilla mods", () => {
        const unknown = { versions: [], categories: [] };
        expect(catalogCompatibility(unknown, { mcVersion: "1.21.1", loader: "fabric" }, "Mod")).toBeNull();
        expect(catalogCompatibility(unknown, { mcVersion: "1.21.1", loader: "vanilla" }, "Mod")).toBeTruthy();
    });
});
