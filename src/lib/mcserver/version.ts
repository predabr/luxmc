export interface McVersionRef {
	major: number;
	minor: number;
	patch: number;
}

export function parseMcVersion(id: string): McVersionRef | null {
	const match = /^(\d+)\.(\d+)(?:\.(\d+))?/.exec(id);
	if (!match) return null;
	return {
		major: Number(match[1]),
		minor: Number(match[2]),
		patch: match[3] ? Number(match[3]) : 0
	};
}

export function compareMcVersion(a: string, b: string): number {
	const pa = parseMcVersion(a);
	const pb = parseMcVersion(b);
	if (!pa || !pb) return a.localeCompare(b);
	if (pa.major !== pb.major) return pa.major - pb.major;
	if (pa.minor !== pb.minor) return pa.minor - pb.minor;
	return pa.patch - pb.patch;
}

export function mcVersionAtLeast(id: string, min: string): boolean {
	return compareMcVersion(id, min) >= 0;
}

export function isSnapshot(id: string): boolean {
	return /\dw\d{2}[a-z]$/i.test(id) || /-(pre|rc)\d*$/i.test(id) || /^3D Shareware/i.test(id);
}

export function requiredJavaMajor(mcVersion: string): number {
	if (!mcVersionAtLeast(mcVersion, '1.17')) return 8;
	if (!mcVersionAtLeast(mcVersion, '1.18')) return 16;
	if (!mcVersionAtLeast(mcVersion, '1.20.5')) return 17;
	return 21;
}

export function javaVersionLabel(major: number): string {
	return `Java ${major}`;
}

export function formatGiB(mb: number): string {
	return `${(mb / 1024).toFixed(1)} GiB`;
}

export function formatBytes(bytes: number): string {
	if (!Number.isFinite(bytes) || bytes < 0) return '—';
	const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
	let value = bytes;
	let unit = 0;
	while (value >= 1024 && unit < units.length - 1) {
		value /= 1024;
		unit += 1;
	}
	return `${value >= 100 || unit === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unit]}`;
}

export function formatRate(bytesPerSecond: number): string {
	return `${formatBytes(bytesPerSecond)}/s`;
}

export function formatUptime(ms: number): string {
	if (!Number.isFinite(ms) || ms < 0) return '—';
	const totalSeconds = Math.floor(ms / 1000);
	const hours = Math.floor(totalSeconds / 3600);
	const minutes = Math.floor((totalSeconds % 3600) / 60);
	const seconds = totalSeconds % 60;
	if (hours > 0) return `${hours}h ${String(minutes).padStart(2, '0')}min`;
	if (minutes > 0) return `${minutes}min ${String(seconds).padStart(2, '0')}s`;
	return `${seconds}s`;
}

export function formatClock(timestamp: number): string {
	return new Date(timestamp).toLocaleTimeString('pt-BR', { hour12: false });
}

export function formatDateTime(iso: string): string {
	const date = new Date(iso);
	if (Number.isNaN(date.getTime())) return iso;
	return date.toLocaleString('pt-BR', { day: '2-digit', month: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit' });
}
