<script lang="ts">
	import { authResolveTexture } from "$lib/api/auth";
	type Props = {
		src: string;
		alt?: string;
		model?: "steve" | "alex";
		className?: string;
	};

	let {
		src,
		alt = "Skin Preview",
		model = "steve",
		className = "h-28"
	}: Props = $props();

	let canvasEl = $state<HTMLCanvasElement | null>(null);

	$effect(() => {
		if (!canvasEl || !src) return;
		const canvas = canvasEl;
		const selectedModel = model;
		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		let disposed = false;
		const img = new Image();
		img.crossOrigin = "anonymous";
		img.onload = () => {
			if (disposed) return;
			const isSlim = selectedModel === "alex";
			const armWidth = isSlim ? 3 : 4;
			const isModern = img.naturalHeight === img.naturalWidth;
			const scale = img.naturalWidth / 64;
			const draw = (sx: number, sy: number, sw: number, sh: number, dx: number, dy: number, dw: number, dh: number) =>
				ctx.drawImage(img, sx * scale, sy * scale, sw * scale, sh * scale, dx, dy, dw, dh);

			canvas.width = 16;
			canvas.height = 32;
			ctx.clearRect(0, 0, 16, 32);
			ctx.imageSmoothingEnabled = false;

			draw(8, 8, 8, 8, 4, 0, 8, 8);
			draw(40, 8, 8, 8, 4, 0, 8, 8);

			draw(20, 20, 8, 12, 4, 8, 8, 12);
			if (isModern) {
				draw(20, 36, 8, 12, 4, 8, 8, 12);
			}

			const rightArmX = isSlim ? 1 : 0;
			draw(44, 20, 4, 12, rightArmX, 8, armWidth, 12);
			if (isModern) {
				draw(44, 36, 4, 12, rightArmX, 8, armWidth, 12);
			}

			const leftArmX = 12;
			if (isModern) {
				draw(36, 52, 4, 12, leftArmX, 8, armWidth, 12);
				draw(52, 52, 4, 12, leftArmX, 8, armWidth, 12);
			} else {
				ctx.save();
				ctx.scale(-1, 1);
				draw(44, 20, 4, 12, -(leftArmX + armWidth), 8, armWidth, 12);
				ctx.restore();
			}

			draw(4, 20, 4, 12, 4, 20, 4, 12);
			if (isModern) {
				draw(4, 36, 4, 12, 4, 20, 4, 12);
			}

			if (isModern) {
				draw(20, 52, 4, 12, 8, 20, 4, 12);
				draw(4, 52, 4, 12, 8, 20, 4, 12);
			} else {
				ctx.save();
				ctx.scale(-1, 1);
				draw(4, 20, 4, 12, -12, 20, 4, 12);
				ctx.restore();
			}
		};
		img.onerror = () => {
			if (disposed) return;
			img.onerror = null;
			img.src = selectedModel === "alex" ? "/alex.png" : "/steve.png";
		};
		if (src.startsWith("https://")) {
			void authResolveTexture(src).then(resolved => {
				if (!disposed) img.src = resolved;
			}).catch(() => {
				if (!disposed) img.src = src;
			});
		} else img.src = src;
		return () => { disposed = true; img.onload = null; img.onerror = null; img.src = ""; };
	});
</script>

<canvas
	bind:this={canvasEl}
	aria-label={alt}
	class="object-contain [image-rendering:pixelated] drop-shadow-md transition-transform group-hover:scale-105 {className}"
	style="aspect-ratio: 1 / 2;"
></canvas>
