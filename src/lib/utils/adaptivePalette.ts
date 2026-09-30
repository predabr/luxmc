let lastExtractedUrl = "";

function runLater(fn: () => void) {
	if (typeof window === "undefined") return;
	if (typeof window.requestIdleCallback === "function") {
		window.requestIdleCallback(fn, { timeout: 2000 });
		return;
	}
	window.setTimeout(fn, 120);
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
				const { Vibrant } = await import("./vibrant");
				const palette = await new Vibrant(target).getPalette();
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
