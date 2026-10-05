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
