import { convertFileSrc } from "@tauri-apps/api/core";

const PRESET_ICONS: Record<string, string> = {
	grass_block: "/grass_block.png",
	"/grass_block": "/grass_block.png",
	grass: "/grass_block.png",
	"/grass": "/grass_block.png",
	modpack_fo: "/modpack_fo_icon.png",
	modpack_better_mc: "/modpack_bmc_icon.webp",
	modpack_cobblemon: "/modpack_cobblemon_icon.png",
	logo: "/logo.png",
	grass_head: "/grass_head.png",
	default: "/grass_block.png"
};

export function getIconSrc(iconStr?: string | null): string {
	if (!iconStr) return "/grass_block.png";
	const trimmed = iconStr.trim();
	if (!trimmed) return "/grass_block.png";

	if (PRESET_ICONS[trimmed]) {
		return PRESET_ICONS[trimmed];
	}

	if (
		trimmed.startsWith("http://") ||
		trimmed.startsWith("https://") ||
		trimmed.startsWith("data:") ||
		trimmed.startsWith("asset:")
	) {
		return trimmed;
	}

	const isLocalSystemPath =
		trimmed.startsWith("/home/") ||
		trimmed.startsWith("/root/") ||
		trimmed.startsWith("/usr/") ||
		trimmed.startsWith("/var/") ||
		trimmed.startsWith("/tmp/") ||
		trimmed.startsWith("/mnt/") ||
		trimmed.startsWith("/media/") ||
		trimmed.includes("/.local/") ||
		trimmed.includes("/.config/") ||
		/^[A-Za-z]:[\\/]/.test(trimmed);

	if (trimmed.startsWith("/") && !isLocalSystemPath && !trimmed.includes("\\")) {
		return trimmed;
	}

	try {
		return convertFileSrc(trimmed);
	} catch {
		return "/grass_block.png";
	}
}
