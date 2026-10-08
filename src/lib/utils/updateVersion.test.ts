import { describe, expect, it } from "vitest";
import { isNewerVersion, isReleaseUpdate, releaseIdentity } from "./updateVersion";

describe("update version ordering", () => {
	it.each([
		['3.5.0', 0, '3.5.0', 2, true],
		['3.5.0', 2, '3.5.0', 2, false],
		['3.5.0', 2, '3.5.0', 1, false],
		['3.5.0', 2, '3.5.1', 0, true],
		['3.5.0', 2, '3.4.0', 9, false],
		['3.5.0', 2, '3.5.0', NaN, false],
		['invalid', 0, 'invalid', 2, false],
	] as const)('orders revision %s/%s against %s/%s', (current, revision, latest, remoteRevision, expected) => {
		expect(isReleaseUpdate(current, revision, latest, remoteRevision)).toBe(expected);
	});
	it('offers the compatibility marker to the old client', () => {
		expect(isNewerVersion('3.5.0', '3.5.1')).toBe(true);
		expect(isNewerVersion('3.5.0', 'v3.5.1-revision.2')).toBe(true);
		const identity=releaseIdentity('v3.5.1-revision.2');
		expect(identity).toEqual({version:'3.5.0',revision:2});
		expect(isReleaseUpdate('3.5.0',2,identity.version,identity.revision)).toBe(false);
	});
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
