<script lang="ts">
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
		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		const img = new Image();
		img.crossOrigin = "anonymous";
		img.onload = () => {
			const isSlim = model === "alex";
			const armWidth = isSlim ? 3 : 4;
			const isModern = img.height >= 64;

			canvas.width = 16;
			canvas.height = 32;
			ctx.clearRect(0, 0, 16, 32);
			ctx.imageSmoothingEnabled = false;

			ctx.drawImage(img, 8, 8, 8, 8, 4, 0, 8, 8);
			ctx.drawImage(img, 40, 8, 8, 8, 4, 0, 8, 8);

			ctx.drawImage(img, 20, 20, 8, 12, 4, 8, 8, 12);
			if (isModern) {
				ctx.drawImage(img, 20, 36, 8, 12, 4, 8, 8, 12);
			}

			const rightArmX = isSlim ? 1 : 0;
			ctx.drawImage(img, 44, 20, 4, 12, rightArmX, 8, armWidth, 12);
			if (isModern) {
				ctx.drawImage(img, 44, 36, 4, 12, rightArmX, 8, armWidth, 12);
			}

			const leftArmX = 12;
			if (isModern) {
				ctx.drawImage(img, 36, 52, 4, 12, leftArmX, 8, armWidth, 12);
				ctx.drawImage(img, 52, 52, 4, 12, leftArmX, 8, armWidth, 12);
			} else {
				ctx.save();
				ctx.scale(-1, 1);
				ctx.drawImage(img, 44, 20, 4, 12, -(leftArmX + armWidth), 8, armWidth, 12);
				ctx.restore();
			}

			ctx.drawImage(img, 4, 20, 4, 12, 4, 20, 4, 12);
			if (isModern) {
				ctx.drawImage(img, 4, 36, 4, 12, 4, 20, 4, 12);
			}

			if (isModern) {
				ctx.drawImage(img, 20, 52, 4, 12, 8, 20, 4, 12);
				ctx.drawImage(img, 4, 52, 4, 12, 8, 20, 4, 12);
			} else {
				ctx.save();
				ctx.scale(-1, 1);
				ctx.drawImage(img, 4, 20, 4, 12, -12, 20, 4, 12);
				ctx.restore();
			}
		};
		img.src = src;
	});
</script>

<canvas
	bind:this={canvasEl}
	aria-label={alt}
	class="object-contain [image-rendering:pixelated] drop-shadow-md transition-transform group-hover:scale-105 {className}"
	style="aspect-ratio: 1 / 2;"
></canvas>
