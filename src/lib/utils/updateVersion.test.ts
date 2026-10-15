import { describe, expect, it } from "vitest";
import { isNewerVersion, isReleaseUpdate, releaseIdentity } from "./updateVersion";

describe("update version ordering", () => {
    it('compares successive builds without changing the product version or repeating a completed update', () => {
        const installed = releaseIdentity('v3.9.5-build.38553346641.1');
        const retry = releaseIdentity('v3.9.5-build.38553346641.2');
        const next = releaseIdentity('v3.9.5-build.38553346642.1');
        expect(installed.version).toBe('3.9.5');
        expect(isReleaseUpdate(installed.version,installed.revision,retry.version,retry.revision)).toBe(true);
        expect(isReleaseUpdate(retry.version,retry.revision,next.version,next.revision)).toBe(true);
        expect(isReleaseUpdate(next.version,next.revision,next.version,next.revision)).toBe(false);
        expect(isReleaseUpdate(next.version,next.revision,installed.version,installed.revision)).toBe(false);
        expect(releaseIdentity('v3.9.5-build.9007199254740991.1').revision).toBe(0);
    });
	it.each([
		['3.6.0', 4, '3.7.0', 0, true],
		['3.7.0', 0, '3.6.0', 4, false],
		['3.7.0', 0, '3.7.0', 0, false],
		['3.6.0', 0, '3.6.0', 2, true],
		['3.6.0', 1, '3.6.0', 2, true],
		['3.6.0', 2, '3.6.0', 2, false],
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
