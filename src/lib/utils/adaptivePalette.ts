import { Vibrant } from "node-vibrant/browser";

let lastExtractedUrl = "";

export async function applyAdaptivePalette(imageUrl: string | null | undefined) {
	if (typeof window === "undefined" || !imageUrl || imageUrl === lastExtractedUrl) {
		return;
	}

	lastExtractedUrl = imageUrl;

	try {
		const vibrant = new Vibrant(imageUrl);
		const palette = await vibrant.getPalette();
		const root = document.documentElement;

		const vibrantSwatch = palette.Vibrant || palette.LightVibrant || palette.DarkVibrant;
		if (vibrantSwatch) {
			const hex = vibrantSwatch.hex;
			const rgb = vibrantSwatch.rgb.join(", ");
			root.style.setProperty("--brand-500", hex);
			root.style.setProperty("--brand-glow", `rgba(${rgb}, 0.25)`);
			root.style.setProperty("--brand-glow-strong", `rgba(${rgb}, 0.4)`);
		}
	} catch {
	}
}
