import type { CapeType } from "$lib/stores/skin.svelte";

export function drawCapeToCanvas(ctx: CanvasRenderingContext2D, type: CapeType) {
	ctx.imageSmoothingEnabled = false;

	if (type === "migrator") {
		ctx.fillStyle = "#5c0818";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#40040f";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#690a1c";
		ctx.fillRect(12, 1, 10, 16);

		const gold = "#d8a646";
		const lightGold = "#fae596";
		const darkGold = "#9d7426";
		ctx.fillStyle = darkGold;
		ctx.fillRect(12, 16, 10, 1);
		ctx.fillStyle = gold;
		ctx.fillRect(13, 15, 8, 1);
		ctx.fillStyle = gold;
		ctx.fillRect(14, 4, 6, 2);
		ctx.fillStyle = lightGold;
		ctx.fillRect(15, 4, 4, 1);
		ctx.fillStyle = gold;
		ctx.fillRect(16, 6, 2, 6);
		ctx.fillStyle = lightGold;
		ctx.fillRect(16, 7, 1, 4);
		ctx.fillStyle = gold;
		ctx.fillRect(14, 12, 6, 2);
		ctx.fillStyle = darkGold;
		ctx.fillRect(14, 13, 6, 1);
	} else if (type === "optifine") {
		ctx.fillStyle = "#b51a1a";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#8a1010";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#c92020";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#ffffff";
		ctx.fillRect(14, 6, 3, 6);
		ctx.fillStyle = "#c92020";
		ctx.fillRect(15, 7, 1, 4);
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(18, 6, 1, 6);
		ctx.fillRect(18, 6, 3, 1);
		ctx.fillRect(18, 8, 2, 1);
	} else if (type === "minecon2011") {
		ctx.fillStyle = "#7a1212";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#520a0a";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#931717";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#5a3a1d";
		ctx.fillRect(16, 8, 2, 7);
		ctx.fillStyle = "#f5c842";
		ctx.fillRect(14, 5, 6, 2);
		ctx.fillStyle = "#d89f25";
		ctx.fillRect(13, 6, 1, 3);
		ctx.fillRect(20, 6, 1, 3);
	} else if (type === "minecon2012") {
		ctx.fillStyle = "#111833";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#090d1c";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#19244c";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#5c3d1e";
		ctx.fillRect(16, 8, 2, 7);
		ctx.fillStyle = "#f5c842";
		ctx.fillRect(14, 5, 6, 2);
		ctx.fillStyle = "#fde076";
		ctx.fillRect(15, 5, 4, 1);
		ctx.fillStyle = "#d89f25";
		ctx.fillRect(13, 6, 1, 3);
		ctx.fillRect(20, 6, 1, 3);
	} else if (type === "minecon2013") {
		ctx.fillStyle = "#1b4725";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#0f2b16";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#225930";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#8a582c";
		ctx.fillRect(14, 4, 6, 2);
		ctx.fillStyle = "#787a7d";
		ctx.fillRect(15, 6, 4, 8);
		ctx.fillStyle = "#555759";
		ctx.fillRect(15, 8, 4, 2);
	} else if (type === "minecon2015") {
		ctx.fillStyle = "#18383c";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#0d2023";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#1f474d";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#d1d5db";
		ctx.fillRect(14, 5, 6, 3);
		ctx.fillStyle = "#9ca3af";
		ctx.fillRect(15, 7, 4, 4);
		ctx.fillStyle = "#ef4444";
		ctx.fillRect(18, 11, 2, 2);
	} else if (type === "minecon2016") {
		ctx.fillStyle = "#130a21";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#090412";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#1c0e30";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#a855f7";
		ctx.fillRect(13, 7, 3, 1);
		ctx.fillRect(18, 7, 3, 1);
		ctx.fillStyle = "#f0abfc";
		ctx.fillRect(14, 7, 1, 1);
		ctx.fillRect(19, 7, 1, 1);
	} else if (type === "cherry") {
		ctx.fillStyle = "#f472b6";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#db2777";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#fbcfe8";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#ffffff";
		ctx.fillRect(14, 5, 2, 2);
		ctx.fillRect(18, 6, 2, 2);
		ctx.fillRect(16, 12, 2, 2);
		ctx.fillStyle = "#f43f5e";
		ctx.fillRect(15, 6, 1, 1);
		ctx.fillRect(19, 7, 1, 1);
	} else if (type === "vanilla") {
		ctx.fillStyle = "#1e1b4b";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#0f0e26";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#312e81";
		ctx.fillRect(12, 1, 5, 16);
		ctx.fillStyle = "#d97706";
		ctx.fillRect(17, 1, 5, 16);

		ctx.fillStyle = "#fde68a";
		ctx.fillRect(16, 7, 2, 4);
	} else if (type === "tiktok") {
		ctx.fillStyle = "#09090b";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#000000";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#18181b";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#06b6d4";
		ctx.fillRect(14, 6, 3, 6);
		ctx.fillStyle = "#f43f5e";
		ctx.fillRect(17, 7, 3, 6);
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(15, 6, 3, 5);
	} else if (type === "twitch") {
		ctx.fillStyle = "#581c87";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#3b0764";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#7e22ce";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#ffffff";
		ctx.fillRect(14, 5, 6, 6);
		ctx.fillRect(14, 11, 2, 2);
		ctx.fillStyle = "#7e22ce";
		ctx.fillRect(15, 7, 1, 2);
		ctx.fillRect(18, 7, 1, 2);
	} else if (type === "luxmc") {
		ctx.fillStyle = "#090d16";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#04060a";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#0f172a";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#e2b86b";
		ctx.fillRect(12, 1, 10, 1);
		ctx.fillRect(12, 16, 10, 1);
		ctx.fillRect(12, 1, 1, 16);
		ctx.fillRect(21, 1, 1, 16);

		ctx.fillStyle = "#f5c842";
		ctx.fillRect(15, 5, 2, 7);
		ctx.fillRect(15, 11, 4, 2);
		ctx.fillStyle = "#38bdf8";
		ctx.fillRect(18, 5, 1, 1);
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(15, 5, 1, 1);
	} else {
		// Default mojang
		ctx.fillStyle = "#8b0e17";
		ctx.fillRect(0, 0, 64, 32);
		ctx.fillStyle = "#63070e";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#a8121d";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#ffffff";
		ctx.fillRect(15, 7, 4, 4);
		ctx.fillStyle = "#a8121d";
		ctx.fillRect(16, 8, 2, 2);
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

	drawCapeToCanvas(fullCtx, type);

	// The back face of the cape in standard MC UV is at x=12, y=1, w=10, h=16
	ctx.drawImage(fullCanvas, 12, 1, 10, 16, 0, 0, 40, 64);

	const dataUrl = canvas.toDataURL("image/png");
	previewCache.set(type, dataUrl);
	return dataUrl;
}
