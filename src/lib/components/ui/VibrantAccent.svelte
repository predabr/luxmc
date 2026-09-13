<script lang="ts">
	import { onDestroy } from "svelte";

	type Props = { src?: string | null };
	let { src }: Props = $props();

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
		try {
			const mod = await import("node-vibrant");
			const VibrantLib = (mod as { default?: unknown }).default ?? mod;
			const builder = (VibrantLib as { from: (src: string) => { getPalette: () => Promise<Record<string, { rgb: number[] } | null>> } }).from(url);
			const palette = await builder.getPalette();
			const swatch = palette["Vibrant"] ?? palette["Muted"] ?? palette["DarkVibrant"];
			if (swatch) {
				const [r, g, b] = swatch.rgb;
				applyAccent(Math.round(r), Math.round(g), Math.round(b));
			}
		} catch {
			resetAccent();
		}
	}

	$effect(() => {
		if (src) {
			extractColor(src);
		} else {
			resetAccent();
		}
	});

	onDestroy(() => {
		resetAccent();
	});
</script>

<span class="sr-only" aria-hidden="true"></span>
