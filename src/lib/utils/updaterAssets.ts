import type { UpdateEnvironment } from "$lib/api/updater";

export interface UpdateAsset {
	name: string;
	browser_download_url: string;
	size: number;
}

const extensions: Partial<Record<UpdateEnvironment["mode"], string[]>> = {
	appimage: [".appimage"],
	pacman: [".pkg.tar.zst"],
	debian: [".deb"],
	rpm: [".rpm"],
	windows: [".exe", ".msi"],
	macos: [".dmg"]
};

export function resolveUpdateAssetUrl(
	assets: UpdateAsset[],
	installation: UpdateEnvironment["mode"]
): string {
	const accepted = extensions[installation];
	if (!accepted) return "";
	if (installation === "windows") {
		const installers = assets.filter((candidate) => {
			const name = candidate.name.toLowerCase();
			return name.endsWith(".msi") || (name.endsWith(".exe") &&
				(/setup|install(?:er|ador)?/.test(name) || /^lux\s*mc\s+launcher\.exe$/.test(name)));
		});
		installers.sort((a, b) => Number(/\.exe$/i.test(b.name)) - Number(/\.exe$/i.test(a.name)) ||
			Number(/x64|x86_64|amd64/i.test(b.name)) - Number(/x64|x86_64|amd64/i.test(a.name)));
		return installers[0]?.browser_download_url || "";
	}
	for (const extension of accepted) {
		const asset = assets.find((candidate) => candidate.name.toLowerCase().endsWith(extension));
		if (asset) return asset.browser_download_url;
	}
	return "";
}
