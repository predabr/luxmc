export const RELEASE_REVISION = 2;

export function releaseIdentity(tag: string): { version: string; revision: number } {
	const value = tag.trim().replace(/^v/, '');
	const bridge = /^(\d+)\.(\d+)\.(\d+)-revision\.(\d+)$/.exec(value);
	if (bridge && Number(bridge[3]) > 0) return {version: `${bridge[1]}.${bridge[2]}.${Number(bridge[3]) - 1}`, revision: Number(bridge[4])};
	return {version: value, revision: 0};
}

export function isReleaseUpdate(current: string, currentRevision: number, latest: string, latestRevision: number): boolean {
	if (isNewerVersion(current, latest)) return true;
	const normalize = (value: string) => value.trim().replace(/^v/, '').split('+')[0];
	const version = normalize(latest);
	return /^\d+\.\d+\.\d+(?:-[\da-zA-Z.-]+)?$/.test(version) && normalize(current) === version && Number.isSafeInteger(latestRevision) && latestRevision > currentRevision;
}

export function isNewerVersion(current: string, latest: string): boolean {
	const parse = (value: string) => /^v?(\d+)\.(\d+)\.(\d+)(?:-([\da-zA-Z.-]+))?(?:\+[\da-zA-Z.-]+)?$/.exec(value.trim());
	const a = parse(current);
	const b = parse(latest);
	if (!a || !b) return false;
	for (let i = 1; i <= 3; i++) {
		if (Number(a[i]) !== Number(b[i])) return Number(b[i]) > Number(a[i]);
	}
	if (!a[4] || !b[4]) return Boolean(a[4] && !b[4]);
	const left = a[4].split(".");
	const right = b[4].split(".");
	for (let i = 0; i < Math.max(left.length, right.length); i++) {
		if (left[i] === undefined) return true;
		if (right[i] === undefined) return false;
		if (left[i] === right[i]) continue;
		const aNumeric = /^\d+$/.test(left[i]);
		const bNumeric = /^\d+$/.test(right[i]);
		if (aNumeric && bNumeric) return Number(right[i]) > Number(left[i]);
		if (aNumeric !== bNumeric) return aNumeric;
		return right[i] > left[i];
	}
	return false;
}
