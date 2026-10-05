import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
import { authResolveTexture } from "$lib/api/auth";

export function loadTextureImage(src: string, signal: AbortSignal, timeoutMs = 15000): Promise<HTMLImageElement> {
	return new Promise((resolve, reject) => {
		const image = new Image();
		if (src.startsWith("http://") || src.startsWith("https://")) {
			image.crossOrigin = "anonymous";
		}
		const timeout = setTimeout(() => finish(new Error(uiText("ui.a5ba2552601d7b97"))), timeoutMs);
		const cancel = () => finish(new DOMException("Cancelado", "AbortError"));
		function finish(error?: Error) {
			clearTimeout(timeout);
			signal.removeEventListener("abort", cancel);
			image.onload = null;
			image.onerror = null;
			if (error) { image.src = ""; reject(error); }
			else resolve(image);
		}
		image.onload = () => finish();
		image.onerror = () => finish(new Error(uiText("ui.9779190602bb4ea9")));
		signal.addEventListener("abort", cancel, { once: true });
		if (signal.aborted) cancel();
		else image.src = src;
	});
}

export function isCapeTexture(width: number, height: number): boolean {
    return Number.isInteger(width) && Number.isInteger(height) && width > 0 && height > 0 && width <= 2048 && height <= 2048 &&
        ((width % 22 === 0 && height === (width / 22) * 17) || (width === height * 2) || (width === height) || (Math.abs(width / height - 10 / 16) < 0.2));
}

export function isSkinTexture(width: number, height: number): boolean {
    return Number.isInteger(width) && width >= 64 && width <= 2048 && width % 64 === 0 && (height === width || height === width / 2);
}

export function isCapeUVPattern(ctx: CanvasRenderingContext2D): boolean {
    const { width, height } = ctx.canvas;
    if (!isCapeTexture(width, height)) return false;
    if (width / height === 22 / 17) return true;
    const scale = width / 64;
    const pixels = ctx.getImageData(0, 0, width, height).data;
    let facePixels = 0;
    for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
            if (pixels[(y * width + x) * 4 + 3] === 0) continue;
            if (x >= 22 * scale || y >= 17 * scale) return false;
            if (x >= scale && x < 11 * scale && y >= scale && y < 17 * scale) facePixels++;
        }
    }
    return facePixels >= 80 * scale * scale;
}

export function textureCanvas(image: HTMLImageElement): HTMLCanvasElement {
    if (!image.naturalWidth || !image.naturalHeight || image.naturalWidth > 2048 || image.naturalHeight > 2048) throw new Error(uiText("ui.685f5a70b09fa2ad"));
    const canvas = document.createElement("canvas");
    canvas.width = image.naturalWidth;
    canvas.height = image.naturalHeight;
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    if (!ctx) throw new Error(uiText("ui.facb67b092a4f99b"));
    ctx.drawImage(image, 0, 0);
    return canvas;
}

export function classifyTexture(canvas: HTMLCanvasElement, name = ""): "skin" | "cape" {
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error(uiText("ui.facb67b092a4f99b"));
    if (isCapeTexture(canvas.width, canvas.height) && (/cape|capa/i.test(name) || !isSkinTexture(canvas.width, canvas.height) || isCapeUVPattern(ctx))) return "cape";
    if (isSkinTexture(canvas.width, canvas.height)) return "skin";
    if (/cape|capa/i.test(name)) return "cape";
    throw new Error(uiText("ui.d1b100a4bbe3a3e7"));
}

export function normalizeCape(image: HTMLImageElement): HTMLCanvasElement {
	const w = image.naturalWidth;
	const h = image.naturalHeight;
	if (!w || !h || w > 2048 || h > 2048) throw new Error(uiText("ui.8899004fc5527d76"));

	if (w === h * 2) {
		return textureCanvas(image);
	}

	const optifineScale = w / 22;
	const isOptifine = Number.isInteger(optifineScale) && h === 17 * optifineScale;
	if (isOptifine) {
		const s = Math.max(1, Math.round(optifineScale));
		const canvas = document.createElement("canvas");
		canvas.width = 64 * s;
		canvas.height = 32 * s;
		const ctx = canvas.getContext("2d");
		if (!ctx) throw new Error(uiText("ui.facb67b092a4f99b"));
		ctx.imageSmoothingEnabled = false;
		ctx.drawImage(image, 0, 0);
		return canvas;
	}

	const srcCanvas = textureCanvas(image);
	const srcCtx = srcCanvas.getContext("2d", { willReadFrequently: true });
	let minX = w, minY = h, maxX = 0, maxY = 0;
	if (srcCtx) {
		const imgData = srcCtx.getImageData(0, 0, w, h).data;
		for (let y = 0; y < h; y++) {
			for (let x = 0; x < w; x++) {
				const alpha = imgData[(y * w + x) * 4 + 3];
				if (alpha > 10) {
					if (x < minX) minX = x;
					if (x > maxX) maxX = x;
					if (y < minY) minY = y;
					if (y > maxY) maxY = y;
				}
			}
		}
	}
	const hasBBox = minX <= maxX && minY <= maxY;
	const cropX = hasBBox ? minX : 0;
	const cropY = hasBBox ? minY : 0;
	const cropW = hasBBox ? Math.max(1, maxX - minX + 1) : w;
	const cropH = hasBBox ? Math.max(1, maxY - minY + 1) : h;

	const targetW = w >= 128 || h >= 128 ? 128 : 64;
	const targetH = targetW / 2;
	const s = targetW / 64;

	const canvas = document.createElement("canvas");
	canvas.width = targetW;
	canvas.height = targetH;
	const ctx = canvas.getContext("2d");
	if (!ctx) throw new Error(uiText("ui.facb67b092a4f99b"));
	ctx.imageSmoothingEnabled = false;

	const artW = 10 * s;
	const artH = 16 * s;

	const scale = Math.min(artW / cropW, artH / cropH);
	const drawW = Math.max(1, Math.round(cropW * scale));
	const drawH = Math.max(1, Math.round(cropH * scale));
	const offsetX = Math.round((artW - drawW) / 2);
	const offsetY = Math.round((artH - drawH) / 2);

	let bgColor = "rgba(0,0,0,0)";
	if (srcCtx) {
		const p = srcCtx.getImageData(cropX, cropY, 1, 1).data;
		if (p[3] > 20) {
			bgColor = `rgba(${p[0]},${p[1]},${p[2]},${p[3] / 255})`;
		}
	}

	if (bgColor !== "rgba(0,0,0,0)") {
		ctx.fillStyle = bgColor;
		ctx.fillRect(1 * s, 1 * s, artW, artH);
		ctx.fillRect(12 * s, 1 * s, artW, artH);
		ctx.fillRect(0, 1 * s, 1 * s, artH);
		ctx.fillRect(11 * s, 1 * s, 1 * s, artH);
		ctx.fillRect(1 * s, 0, artW, 1 * s);
		ctx.fillRect(11 * s, 0, artW, 1 * s);
	}

	ctx.drawImage(srcCanvas, cropX, cropY, cropW, cropH, 1 * s + offsetX, 1 * s + offsetY, drawW, drawH);
	ctx.drawImage(srcCanvas, cropX, cropY, cropW, cropH, 12 * s + offsetX, 1 * s + offsetY, drawW, drawH);

	return canvas;
}

export function skinAvatar(src: string): string {
    if (typeof document === "undefined") return "";
    const cached = avatarCache.get(`steve:${src}`) || avatarCache.get(`alex:${src}`);
    return cached || "";
}

const avatarCache = new Map<string, string>();

export async function createSkinAvatar(src: string, signal: AbortSignal, model: "steve" | "alex" = "steve"): Promise<string> {
    const key = `${model}:${src}`;
    const cached = avatarCache.get(key);
    if (cached) return cached;
    let resolved = src;
    if (src.startsWith("http://") || src.startsWith("https://")) {
        try {
            resolved = await authResolveTexture(src);
        } catch {
            resolved = src;
        }
    }
    let image = await loadTextureImage(resolved, signal);
    if (!isSkinTexture(image.naturalWidth, image.naturalHeight) || isCapeUVPattern(textureCanvas(image).getContext("2d")!)) {
        image = await loadTextureImage(model === "alex" ? "/alex.png" : "/steve.png", signal);
    }
    const scale = image.naturalWidth / 64;
    const canvas = document.createElement("canvas");
    canvas.width = 64; canvas.height = 64;
    const context = canvas.getContext("2d");
    if (!context) throw new Error(uiText("ui.facb67b092a4f99b"));
    context.imageSmoothingEnabled = false;
    for (const x of [8, 40]) context.drawImage(image, x * scale, 8 * scale, 8 * scale, 8 * scale, 0, 0, 64, 64);
    const result = canvas.toDataURL("image/png");
    if (avatarCache.size >= 16) avatarCache.delete(avatarCache.keys().next().value!);
    avatarCache.set(key, result);
    return result;
}

function hasTransparency(ctx: CanvasRenderingContext2D, x0: number, y0: number, w: number, h: number): boolean {
	const imgData = ctx.getImageData(x0, y0, w, h);
	for (let i = 3; i < imgData.data.length; i += 4) {
		if (imgData.data[i] !== 0xff) return true;
	}
	return false;
}

function isAreaSolidColor(ctx: CanvasRenderingContext2D, x0: number, y0: number, w: number, h: number, r: number, g: number, b: number): boolean {
	const imgData = ctx.getImageData(x0, y0, w, h);
	for (let i = 0; i < imgData.data.length; i += 4) {
		if (imgData.data[i] !== r || imgData.data[i + 1] !== g || imgData.data[i + 2] !== b || imgData.data[i + 3] !== 0xff) {
			return false;
		}
	}
	return true;
}

export function inferSkinModelType(canvas: HTMLCanvasElement): "steve" | "alex" {
    if (!isSkinTexture(canvas.width, canvas.height) || canvas.height !== canvas.width) return "steve";
	const ctx = canvas.getContext("2d", { willReadFrequently: true });
	if (!ctx) return "steve";
	const scale = canvas.width / 64.0;
	const checkTrans = (x: number, y: number, w: number, h: number) => hasTransparency(ctx, x * scale, y * scale, w * scale, h * scale);
	const checkBlack = (x: number, y: number, w: number, h: number) => isAreaSolidColor(ctx, x * scale, y * scale, w * scale, h * scale, 0, 0, 0);
	const checkWhite = (x: number, y: number, w: number, h: number) => isAreaSolidColor(ctx, x * scale, y * scale, w * scale, h * scale, 255, 255, 255);

	const isSlim =
		(checkTrans(50, 16, 2, 4) || checkTrans(54, 20, 2, 12) || checkTrans(42, 48, 2, 4) || checkTrans(46, 52, 2, 12)) ||
		(checkBlack(50, 16, 2, 4) && checkBlack(54, 20, 2, 12) && checkBlack(42, 48, 2, 4) && checkBlack(46, 52, 2, 12)) ||
		(checkWhite(50, 16, 2, 4) && checkWhite(54, 20, 2, 12) && checkWhite(42, 48, 2, 4) && checkWhite(46, 52, 2, 12));

	return isSlim ? "alex" : "steve";
}
