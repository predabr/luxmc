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
	for (const extension of accepted) {
		const asset = assets.find((candidate) => candidate.name.toLowerCase().endsWith(extension));
		if (asset) return asset.browser_download_url;
	}
	return "";
}
