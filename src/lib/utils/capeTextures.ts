import type { CapeType } from "$lib/stores/skin.svelte";

export function drawCapeToCanvas(ctx: CanvasRenderingContext2D, type: CapeType) {
	ctx.imageSmoothingEnabled = false;
	ctx.clearRect(0, 0, 64, 32);

	if (type === "luxmc") {
		// Luxmc Exclusive: Obsidian and Gold with Cyan Diamond Gem
		ctx.fillStyle = "#0a0e17";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#060910";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#0e1422";
		ctx.fillRect(12, 1, 10, 16);

		// Gold borders
		ctx.fillStyle = "#a17c24";
		ctx.fillRect(12, 1, 10, 1);
		ctx.fillRect(12, 16, 10, 1);
		ctx.fillRect(12, 1, 1, 16);
		ctx.fillRect(21, 1, 1, 16);

		ctx.fillStyle = "#e5b546";
		ctx.fillRect(13, 2, 8, 1);
		ctx.fillRect(13, 15, 8, 1);
		ctx.fillRect(13, 2, 1, 14);
		ctx.fillRect(20, 2, 1, 14);

		// Gold Crown / Emblem
		ctx.fillStyle = "#fadb70";
		ctx.fillRect(15, 5, 4, 1);
		ctx.fillRect(14, 6, 6, 2);
		ctx.fillStyle = "#e5b546";
		ctx.fillRect(15, 8, 4, 3);
		ctx.fillRect(16, 11, 2, 2);

		// Cyan core jewel
		ctx.fillStyle = "#38bdf8";
		ctx.fillRect(16, 7, 2, 2);
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(16, 7, 1, 1);
	} else if (type === "migrator") {
		// Migrator: Ruby and Gold Emblem
		ctx.fillStyle = "#4a0612";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#33030b";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#5c0817";
		ctx.fillRect(12, 1, 10, 16);

		// Gold trim at bottom
		ctx.fillStyle = "#946f1d";
		ctx.fillRect(12, 15, 10, 2);
		ctx.fillStyle = "#deb038";
		ctx.fillRect(13, 15, 8, 1);
		ctx.fillStyle = "#fae58c";
		ctx.fillRect(14, 15, 6, 1);

		// Gold Quill / Compass Emblem
		ctx.fillStyle = "#946f1d";
		ctx.fillRect(15, 4, 4, 1);
		ctx.fillStyle = "#deb038";
		ctx.fillRect(14, 5, 6, 2);
		ctx.fillStyle = "#fae58c";
		ctx.fillRect(15, 5, 4, 1);
		ctx.fillStyle = "#deb038";
		ctx.fillRect(16, 7, 2, 5);
		ctx.fillStyle = "#fae58c";
		ctx.fillRect(16, 8, 1, 3);
		ctx.fillStyle = "#deb038";
		ctx.fillRect(15, 12, 4, 1);
	} else if (type === "optifine") {
		// OptiFine: Classic Red with White OF
		ctx.fillStyle = "#991414";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#730c0c";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#b81818";
		ctx.fillRect(12, 1, 10, 16);

		// Darker shadow edge
		ctx.fillStyle = "#590909";
		ctx.fillRect(12, 16, 10, 1);
		ctx.fillRect(21, 1, 1, 16);

		// "O"
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(14, 5, 3, 6);
		ctx.fillStyle = "#b81818";
		ctx.fillRect(15, 6, 1, 4);

		// "F"
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(18, 5, 3, 1);
		ctx.fillRect(18, 5, 1, 6);
		ctx.fillRect(18, 7, 2, 1);
	} else if (type === "minecon2011") {
		// Minecon 2011: Classic Red with Green Creeper Face
		ctx.fillStyle = "#6e1010";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#4a0a0a";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#8a1515";
		ctx.fillRect(12, 1, 10, 16);

		// Creeper face
		ctx.fillStyle = "#1e3d1b";
		ctx.fillRect(14, 5, 2, 2); // left eye
		ctx.fillRect(18, 5, 2, 2); // right eye
		ctx.fillRect(16, 7, 2, 3); // nose
		ctx.fillRect(15, 9, 4, 3); // mouth
		ctx.fillRect(14, 10, 1, 3); // left drop
		ctx.fillRect(19, 10, 1, 3); // right drop
		ctx.fillStyle = "#8a1515";
		ctx.fillRect(16, 11, 2, 1); // mouth cutout
	} else if (type === "minecon2012") {
		// Minecon 2012: Midnight Navy with Gold Pickaxe
		ctx.fillStyle = "#0e1529";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#070b17";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#15203d";
		ctx.fillRect(12, 1, 10, 16);

		// Pickaxe head
		ctx.fillStyle = "#9e7519";
		ctx.fillRect(13, 6, 1, 2);
		ctx.fillRect(20, 6, 1, 2);
		ctx.fillStyle = "#e5b533";
		ctx.fillRect(14, 4, 6, 2);
		ctx.fillStyle = "#fced8a";
		ctx.fillRect(15, 4, 4, 1);

		// Wooden handle
		ctx.fillStyle = "#54371b";
		ctx.fillRect(16, 6, 2, 8);
		ctx.fillStyle = "#704b26";
		ctx.fillRect(16, 6, 1, 7);
	} else if (type === "minecon2013") {
		// Minecon 2013: Dark Forest Green with Redstone Piston
		ctx.fillStyle = "#16381e";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#0c2112";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#1f4d2a";
		ctx.fillRect(12, 1, 10, 16);

		// Wood piston head
		ctx.fillStyle = "#784b23";
		ctx.fillRect(14, 4, 6, 2);
		ctx.fillStyle = "#996231";
		ctx.fillRect(14, 4, 6, 1);

		// Stone piston base
		ctx.fillStyle = "#5c5e63";
		ctx.fillRect(15, 6, 4, 8);
		ctx.fillStyle = "#3d3e42";
		ctx.fillRect(16, 7, 2, 5);
		ctx.fillStyle = "#b81d1d";
		ctx.fillRect(16, 12, 2, 1); // redstone dot
	} else if (type === "minecon2015") {
		// Minecon 2015: London Iron Golem Teal
		ctx.fillStyle = "#172f33";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#0c1a1c";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#1f3e42";
		ctx.fillRect(12, 1, 10, 16);

		// Golem head & brow
		ctx.fillStyle = "#d1d5db";
		ctx.fillRect(14, 4, 6, 3);
		ctx.fillStyle = "#9ca3af";
		ctx.fillRect(15, 7, 4, 4);

		// Red Poppy
		ctx.fillStyle = "#dc2626";
		ctx.fillRect(18, 10, 2, 2);
		ctx.fillStyle = "#15803d";
		ctx.fillRect(18, 12, 1, 2);
	} else if (type === "minecon2016") {
		// Minecon 2016: Void Violet with Enderman Eyes
		ctx.fillStyle = "#11071c";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#07020d";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#1a0b2b";
		ctx.fillRect(12, 1, 10, 16);

		// Enderman glowing eyes
		ctx.fillStyle = "#c084fc";
		ctx.fillRect(13, 7, 3, 1);
		ctx.fillRect(18, 7, 3, 1);
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(14, 7, 1, 1);
		ctx.fillRect(19, 7, 1, 1);
		ctx.fillStyle = "#7e22ce";
		ctx.fillRect(13, 8, 3, 1);
		ctx.fillRect(18, 8, 3, 1);
	} else if (type === "cherry") {
		// Cherry Blossom 1.20: Sakura Petals & Spring Bloom
		ctx.fillStyle = "#ec4899";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#be185d";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#f472b6";
		ctx.fillRect(12, 1, 10, 16);

		// Blossom flowers
		ctx.fillStyle = "#fdf2f8";
		ctx.fillRect(14, 5, 2, 2);
		ctx.fillRect(18, 6, 2, 2);
		ctx.fillRect(15, 11, 2, 2);
		ctx.fillStyle = "#db2777";
		ctx.fillRect(15, 6, 1, 1);
		ctx.fillRect(19, 7, 1, 1);
		ctx.fillRect(16, 12, 1, 1);
		ctx.fillStyle = "#15803d";
		ctx.fillRect(14, 13, 1, 2);
	} else if (type === "vanilla") {
		// Vanilla Cape: Half Indigo, Half Gold
		ctx.fillStyle = "#1e1b4b";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#0f0e26";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#2e2a72";
		ctx.fillRect(12, 1, 5, 16);
		ctx.fillStyle = "#d97706";
		ctx.fillRect(17, 1, 5, 16);

		// Center Mojang seal
		ctx.fillStyle = "#fbbf24";
		ctx.fillRect(15, 6, 4, 5);
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(16, 7, 2, 3);
	} else if (type === "tiktok") {
		// TikTok Cape: Cyan/Magenta Glitch Wave
		ctx.fillStyle = "#09090b";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#000000";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#18181b";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#06b6d4";
		ctx.fillRect(14, 5, 4, 7);
		ctx.fillStyle = "#f43f5e";
		ctx.fillRect(16, 6, 4, 7);
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(15, 5, 4, 7);
		ctx.fillStyle = "#18181b";
		ctx.fillRect(16, 7, 2, 3);
	} else if (type === "twitch") {
		// Twitch Cape: Glitch Chat Icon on Purple
		ctx.fillStyle = "#581c87";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#3b0764";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#7e22ce";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#ffffff";
		ctx.fillRect(14, 4, 6, 7);
		ctx.fillRect(14, 11, 2, 2);
		ctx.fillStyle = "#7e22ce";
		ctx.fillRect(15, 6, 1, 2);
		ctx.fillRect(18, 6, 1, 2);
	} else {
		// Default Mojang Red Cape
		ctx.fillStyle = "#8b0e17";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#63070e";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#a8121d";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#ffffff";
		ctx.fillRect(15, 6, 4, 5);
		ctx.fillStyle = "#a8121d";
		ctx.fillRect(16, 7, 2, 3);
	}
}

const previewCache = new Map<string, string>();

export function getCapePreviewDataUrl(type: CapeType): string {
	if (typeof document === "undefined" || type === "none") return "";
	if (previewCache.has(type)) {
		return previewCache.get(type)!;
	}

	const canvas = document.createElement("canvas");
	canvas.width = 40;
	canvas.height = 64;
	const ctx = canvas.getContext("2d");
	if (!ctx) return "";

	ctx.imageSmoothingEnabled = false;

	const fullCanvas = document.createElement("canvas");
	fullCanvas.width = 64;
	fullCanvas.height = 32;
	const fullCtx = fullCanvas.getContext("2d");
	if (!fullCtx) return "";

	fullCtx.imageSmoothingEnabled = false;
	drawCapeToCanvas(fullCtx, type);

	// The back face of the cape in standard MC UV is at x=12, y=1, w=10, h=16
	ctx.drawImage(fullCanvas, 12, 1, 10, 16, 0, 0, 40, 64);

	const dataUrl = canvas.toDataURL("image/png");
	previewCache.set(type, dataUrl);
	return dataUrl;
}

const fullCache = new Map<string, string>();

export function getFullCapeDataUrl(type: CapeType): string {
	if (typeof document === "undefined" || type === "none") return "";
	if (fullCache.has(type)) {
		return fullCache.get(type)!;
	}

	const canvas = document.createElement("canvas");
	canvas.width = 64;
	canvas.height = 32;
	const ctx = canvas.getContext("2d");
	if (!ctx) return "";

	ctx.imageSmoothingEnabled = false;
	drawCapeToCanvas(ctx, type);
	const dataUrl = canvas.toDataURL("image/png");
	fullCache.set(type, dataUrl);
	return dataUrl;
}
