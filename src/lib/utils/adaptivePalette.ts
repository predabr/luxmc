import { api } from "$lib/api/client";

let lastExtractedUrl = "";
const dataUrlCache = new Map<string, string>();

function runLater(fn: () => void) {
	if (typeof window === "undefined") return;
	if (typeof window.requestIdleCallback === "function") {
		window.requestIdleCallback(fn, { timeout: 2000 });
		return;
	}
	window.setTimeout(fn, 120);
}

async function imageDataUrl(url: string): Promise<string | null> {
	const cached = dataUrlCache.get(url);
	if (cached) return cached;
	try {
		const dataUrl = await api.invoke<string | null>("util_image_data_url", { url });
		if (dataUrl) {
			if (dataUrlCache.size >= 24) dataUrlCache.delete(dataUrlCache.keys().next().value!);
			dataUrlCache.set(url, dataUrl);
		}
		return dataUrl;
	} catch {
		return null;
	}
}

export async function applyAdaptivePalette(imageUrl: string | null | undefined) {
	if (typeof window === "undefined" || !imageUrl || imageUrl === lastExtractedUrl) {
		return;
	}

	lastExtractedUrl = imageUrl;
	const target = imageUrl;

	runLater(() => {
		if (lastExtractedUrl !== target) return;
		void (async () => {
			try {
				let source = target;
				if (target.startsWith("http://") || target.startsWith("https://")) {
					const dataUrl = await imageDataUrl(target);
					if (lastExtractedUrl !== target) return;
					if (!dataUrl) return;
					source = dataUrl;
				}
				const { Vibrant } = await import("./vibrant");
				const palette = await new Vibrant(source).getPalette();
				if (lastExtractedUrl !== target) return;
				const swatch = palette.Vibrant || palette.LightVibrant || palette.DarkVibrant;
				if (!swatch) return;
				document.documentElement.style.setProperty(
					"--ambient-accent",
					swatch.rgb.map(Math.round).join(" "),
				);
			} catch {
			}
		})();
	});
}
