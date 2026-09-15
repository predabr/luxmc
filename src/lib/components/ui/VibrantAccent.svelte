<script lang="ts">
	import { onDestroy } from "svelte";

	type Props = { src?: string | null };
	let { src }: Props = $props();

	const colorCache = new Map<string, { r: number; g: number; b: number }>();
	let debounceHandle: ReturnType<typeof setTimeout> | null = null;
	let lastSrc = "";

	function applyAccent(r: number, g: number, b: number) {
		const root = document.documentElement;
		root.style.setProperty("--vibrant-r", String(r));
		root.style.setProperty("--vibrant-g", String(g));
		root.style.setProperty("--vibrant-b", String(b));
		root.style.setProperty("--vibrant-glow", `rgba(${r},${g},${b},0.35)`);
		root.style.setProperty("--vibrant-border", `rgba(${r},${g},${b},0.25)`);
		root.style.setProperty("--vibrant-bg", `rgba(${r},${g},${b},0.07)`);
	}

	function resetAccent() {
		const root = document.documentElement;
		["--vibrant-r", "--vibrant-g", "--vibrant-b", "--vibrant-glow", "--vibrant-border", "--vibrant-bg"]
			.forEach((v) => root.style.removeProperty(v));
	}

	async function extractColor(url: string) {
		if (!url.trim()) { resetAccent(); return; }

		const cached = colorCache.get(url);
		if (cached) {
			applyAccent(cached.r, cached.g, cached.b);
			return;
		}

		try {
			const mod = await import("node-vibrant");
			const VibrantLib = (mod as { default?: unknown }).default ?? mod;
			const builder = (VibrantLib as { from: (src: string) => { getPalette: () => Promise<Record<string, { rgb: number[] } | null>> } }).from(url);
			const palette = await builder.getPalette();
			const swatch = palette["Vibrant"] ?? palette["Muted"] ?? palette["DarkVibrant"];
			if (swatch) {
				const [r, g, b] = swatch.rgb;
				const rounded = { r: Math.round(r), g: Math.round(g), b: Math.round(b) };
				if (colorCache.size > 20) {
					const firstKey = colorCache.keys().next().value;
					if (firstKey) colorCache.delete(firstKey);
				}
				colorCache.set(url, rounded);
				applyAccent(rounded.r, rounded.g, rounded.b);
			}
		} catch {
			resetAccent();
		}
	}

	$effect(() => {
		const currentSrc = src ?? "";
		if (currentSrc === lastSrc) return;
		lastSrc = currentSrc;

		if (debounceHandle) clearTimeout(debounceHandle);

		if (currentSrc) {
			debounceHandle = setTimeout(() => {
				extractColor(currentSrc);
			}, 300);
		} else {
			resetAccent();
		}

		return () => {
			if (debounceHandle) clearTimeout(debounceHandle);
		};
	});

	onDestroy(() => {
		if (debounceHandle) clearTimeout(debounceHandle);
		resetAccent();
	});
</script>

<span class="sr-only" aria-hidden="true"></span>
