import { describe, expect, it } from "vitest";
import { isNewerVersion } from "./updateVersion";

describe("update version ordering", () => {
	it.each([
		["2.0.2", "v2.0.3", true],
		["2.0.3", "2.0.2", false],
		["2.0.2-beta.9", "2.0.2-beta.10", true],
		["2.0.2-beta.10", "2.0.2", true],
		["2.0.2", "2.0.2-beta.10", false],
		["2.0.2-beta", "2.0.2-beta.1", true],
		["2.0.2+build1", "2.0.2+build2", false],
		["2.0.2", "invalid", false],
	])("compares %s with %s", (current, latest, expected) => {
		expect(isNewerVersion(current, latest)).toBe(expected);
	});
});
