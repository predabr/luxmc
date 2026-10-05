import type { CapeType } from "$lib/stores/skin.svelte";
import { loadTextureImage } from "./textureImage";

export function drawCapeToCanvas(ctx: CanvasRenderingContext2D, type: CapeType) {
	ctx.imageSmoothingEnabled = false;
	ctx.clearRect(0, 0, 64, 32);

	if (type === "luxmc") {
		ctx.fillStyle = "#0a0e17";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#0e1422";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#060910";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#a17c24";
		ctx.fillRect(1, 1, 10, 1);
		ctx.fillRect(1, 16, 10, 1);
		ctx.fillRect(1, 1, 1, 16);
		ctx.fillRect(10, 1, 1, 16);

		ctx.fillStyle = "#e5b546";
		ctx.fillRect(2, 2, 8, 1);
		ctx.fillRect(2, 15, 8, 1);
		ctx.fillRect(2, 2, 1, 14);
		ctx.fillRect(9, 2, 1, 14);

		ctx.fillStyle = "#fadb70";
		ctx.fillRect(4, 5, 4, 1);
		ctx.fillRect(3, 6, 6, 2);
		ctx.fillStyle = "#e5b546";
		ctx.fillRect(4, 8, 4, 3);
		ctx.fillRect(5, 11, 2, 2);

		ctx.fillStyle = "#38bdf8";
		ctx.fillRect(5, 7, 2, 2);
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(5, 7, 1, 1);
	} else if (type === "migrator") {
		ctx.fillStyle = "#4a0612";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#5c0817";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#33030b";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#946f1d";
		ctx.fillRect(1, 15, 10, 2);
		ctx.fillStyle = "#deb038";
		ctx.fillRect(2, 15, 8, 1);
		ctx.fillStyle = "#fae58c";
		ctx.fillRect(3, 15, 6, 1);

		ctx.fillStyle = "#946f1d";
		ctx.fillRect(4, 4, 4, 1);
		ctx.fillStyle = "#deb038";
		ctx.fillRect(3, 5, 6, 2);
		ctx.fillStyle = "#fae58c";
		ctx.fillRect(4, 5, 4, 1);
		ctx.fillStyle = "#deb038";
		ctx.fillRect(5, 7, 2, 5);
		ctx.fillStyle = "#fae58c";
		ctx.fillRect(5, 8, 1, 3);
		ctx.fillStyle = "#deb038";
		ctx.fillRect(4, 12, 4, 1);
	} else if (type === "optifine") {
		ctx.fillStyle = "#991414";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#b81818";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#730c0c";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#590909";
		ctx.fillRect(1, 16, 10, 1);
		ctx.fillRect(10, 1, 1, 16);

		ctx.fillStyle = "#ffffff";
		ctx.fillRect(3, 5, 3, 6);
		ctx.fillStyle = "#b81818";
		ctx.fillRect(4, 6, 1, 4);

		ctx.fillStyle = "#ffffff";
		ctx.fillRect(7, 5, 3, 1);
		ctx.fillRect(7, 5, 1, 6);
		ctx.fillRect(7, 7, 2, 1);
	} else if (type === "minecon2011") {
		ctx.fillStyle = "#6e1010";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#8a1515";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#4a0a0a";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#1e3d1b";
		ctx.fillRect(3, 5, 2, 2);
		ctx.fillRect(7, 5, 2, 2);
		ctx.fillRect(5, 7, 2, 3);
		ctx.fillRect(4, 9, 4, 3);
		ctx.fillRect(3, 10, 1, 3);
		ctx.fillRect(8, 10, 1, 3);
		ctx.fillStyle = "#8a1515";
		ctx.fillRect(5, 11, 2, 1);
	} else if (type === "minecon2012") {
		ctx.fillStyle = "#0e1529";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#15203d";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#070b17";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#9e7519";
		ctx.fillRect(2, 6, 1, 2);
		ctx.fillRect(9, 6, 1, 2);
		ctx.fillStyle = "#e5b533";
		ctx.fillRect(3, 4, 6, 2);
		ctx.fillStyle = "#fced8a";
		ctx.fillRect(4, 4, 4, 1);

		ctx.fillStyle = "#54371b";
		ctx.fillRect(5, 6, 2, 8);
		ctx.fillStyle = "#704b26";
		ctx.fillRect(5, 6, 1, 7);
	} else if (type === "minecon2013") {
		ctx.fillStyle = "#16381e";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#1f4d2a";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#0c2112";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#784b23";
		ctx.fillRect(3, 4, 6, 2);
		ctx.fillStyle = "#996231";
		ctx.fillRect(3, 4, 6, 1);

		ctx.fillStyle = "#5c5e63";
		ctx.fillRect(4, 6, 4, 8);
		ctx.fillStyle = "#3d3e42";
		ctx.fillRect(5, 7, 2, 5);
		ctx.fillStyle = "#b81d1d";
		ctx.fillRect(5, 12, 2, 1);
	} else if (type === "minecon2015") {
		ctx.fillStyle = "#172f33";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#1f3e42";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#0c1a1c";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#d1d5db";
		ctx.fillRect(3, 4, 6, 3);
		ctx.fillStyle = "#9ca3af";
		ctx.fillRect(4, 7, 4, 4);

		ctx.fillStyle = "#dc2626";
		ctx.fillRect(7, 10, 2, 2);
		ctx.fillStyle = "#15803d";
		ctx.fillRect(7, 12, 1, 2);
	} else if (type === "minecon2016") {
		ctx.fillStyle = "#11071c";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#1a0b2b";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#07020d";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#c084fc";
		ctx.fillRect(2, 7, 3, 1);
		ctx.fillRect(7, 7, 3, 1);
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(3, 7, 1, 1);
		ctx.fillRect(8, 7, 1, 1);
		ctx.fillStyle = "#7e22ce";
		ctx.fillRect(2, 8, 3, 1);
		ctx.fillRect(7, 8, 3, 1);
	} else if (type === "cherry") {
		ctx.fillStyle = "#ec4899";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#f472b6";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#be185d";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#fdf2f8";
		ctx.fillRect(3, 5, 2, 2);
		ctx.fillRect(7, 6, 2, 2);
		ctx.fillRect(4, 11, 2, 2);
		ctx.fillStyle = "#db2777";
		ctx.fillRect(4, 6, 1, 1);
		ctx.fillRect(8, 7, 1, 1);
		ctx.fillRect(5, 12, 1, 1);
		ctx.fillStyle = "#15803d";
		ctx.fillRect(3, 13, 1, 2);
	} else if (type === "vanilla") {
		ctx.fillStyle = "#1e1b4b";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#0f0e26";
		ctx.fillRect(12, 1, 10, 16);
		ctx.fillStyle = "#2e2a72";
		ctx.fillRect(1, 1, 5, 16);
		ctx.fillStyle = "#d97706";
		ctx.fillRect(6, 1, 5, 16);

		ctx.fillStyle = "#fbbf24";
		ctx.fillRect(4, 6, 4, 5);
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(5, 7, 2, 3);
	} else if (type === "tiktok") {
		ctx.fillStyle = "#09090b";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#18181b";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#000000";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#06b6d4";
		ctx.fillRect(3, 5, 4, 7);
		ctx.fillStyle = "#f43f5e";
		ctx.fillRect(5, 6, 4, 7);
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(4, 5, 4, 7);
		ctx.fillStyle = "#18181b";
		ctx.fillRect(5, 7, 2, 3);
	} else if (type === "twitch") {
		ctx.fillStyle = "#581c87";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#7e22ce";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#3b0764";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#ffffff";
		ctx.fillRect(3, 4, 6, 7);
		ctx.fillRect(3, 11, 2, 2);
		ctx.fillStyle = "#7e22ce";
		ctx.fillRect(4, 6, 1, 2);
		ctx.fillRect(7, 6, 1, 2);
	} else {
		ctx.fillStyle = "#8b0e17";
		ctx.fillRect(0, 0, 22, 17);
		ctx.fillStyle = "#a8121d";
		ctx.fillRect(1, 1, 10, 16);
		ctx.fillStyle = "#63070e";
		ctx.fillRect(12, 1, 10, 16);

		ctx.fillStyle = "#ffffff";
		ctx.fillRect(4, 6, 4, 5);
		ctx.fillStyle = "#a8121d";
		ctx.fillRect(5, 7, 2, 3);
	}

	ctx.drawImage(ctx.canvas, 1, 1, 10, 16, 12, 1, 10, 16);
    ctx.drawImage(ctx.canvas, 1, 1, 1, 16, 0, 1, 1, 16);
    ctx.drawImage(ctx.canvas, 10, 1, 1, 16, 11, 1, 1, 16);
    ctx.drawImage(ctx.canvas, 1, 1, 10, 1, 1, 0, 10, 1);
    ctx.drawImage(ctx.canvas, 1, 16, 10, 1, 11, 0, 10, 1);
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

	ctx.save();
	ctx.drawImage(fullCanvas, 1, 1, 10, 16, 0, 0, 40, 64);
	ctx.restore();

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

export async function migrateGeneratedCape(source: string, signal: AbortSignal): Promise<string> {
    if (!source.startsWith("data:image/png;base64,")) return source;
    const image = await loadTextureImage(source, signal);
    if (image.naturalWidth !== 64 || image.naturalHeight !== 32) return source;
    const canvas = document.createElement("canvas");
    canvas.width = 64; canvas.height = 32;
    const context = canvas.getContext("2d");
    if (!context) return source;
    context.drawImage(image, 0, 0);
    const actual = context.getImageData(0, 0, 64, 32).data;
    const types: CapeType[] = ["luxmc", "migrator", "optifine", "mojang", "minecon2011", "minecon2012", "minecon2013", "minecon2015", "minecon2016", "cherry", "vanilla", "tiktok", "twitch"];
    for (const type of types) {
        drawCapeToCanvas(context, type);
        const expected = context.getImageData(0, 0, 64, 32).data;
        const legacy = expected.slice();
        for (let y = 1; y < 17; y++) for (let x = 0; x < 10; x++) for (let channel = 0; channel < 4; channel++) {
            legacy[(y * 64 + 1 + x) * 4 + channel] = expected[(y * 64 + 12 + x) * 4 + channel];
            legacy[(y * 64 + 12 + x) * 4 + channel] = expected[(y * 64 + 1 + x) * 4 + channel];
        }
        if (actual.every((value, index) => value === legacy[index])) return getFullCapeDataUrl(type);
        const mirrored = legacy.slice();
        for (let y = 1; y < 17; y++) for (let x = 12; x < 22; x++) for (let channel = 0; channel < 4; channel++) {
            mirrored[(y * 64 + x) * 4 + channel] = legacy[(y * 64 + 33 - x) * 4 + channel];
        }
        if (actual.every((value, index) => value === mirrored[index])) return getFullCapeDataUrl(type);
    }
    return source;
}
