import { Vibrant } from "./vibrant";

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
			const channels = vibrantSwatch.rgb.map(Math.round).join(" ");
			if (lastExtractedUrl === imageUrl) root.style.setProperty("--ambient-accent", channels);
		}
	} catch {
	}
}
