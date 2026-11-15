import { describe, expect, it } from "vitest";
import { contentSourceLabel } from "./contentSource";

describe("contentSourceLabel", () => {
	it("names every catalog provider in full", () => {
		expect(contentSourceLabel("all")).toBe("all");
		expect(contentSourceLabel("modrinth")).toBe("Modrinth");
		expect(contentSourceLabel("curseforge")).toBe("CurseForge");
		expect(contentSourceLabel("ftb")).toBe("Feed The Beast");
		expect(contentSourceLabel("FTB")).toBe("Feed The Beast");
	});

	it("keeps unknown providers readable instead of guessing", () => {
		expect(contentSourceLabel("technic")).toBe("technic");
		expect(contentSourceLabel(null)).toBe("");
		expect(contentSourceLabel(undefined)).toBe("");
	});
});
