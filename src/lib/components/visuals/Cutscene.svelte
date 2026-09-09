<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { fade } from "svelte/transition";
	import gsap from "gsap";

	let { onComplete = () => {} }: { onComplete?: () => void } = $props();

	let container: HTMLElement;
	let canvas: HTMLCanvasElement;
	let animId: number;
	let tl: gsap.core.Timeline;

	onMount(() => {
		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		const img = new Image();
		img.src = "/logo.png";
		img.onload = () => {
			runCutscene(img, ctx);
		};
	});

	function runCutscene(img: HTMLImageElement, ctx: CanvasRenderingContext2D) {
		const dpr = window.devicePixelRatio || 1;
		const width = window.innerWidth;
		const height = window.innerHeight;

		canvas.width = width * dpr;
		canvas.height = height * dpr;
		ctx.scale(dpr, dpr);

		const logoSize = Math.min(width, height) * 0.32;
		const half = logoSize / 2;
		const centerX = width / 2;
		const centerY = height / 2;

		// 4 Quadrants: TL (top-left), TR (top-right), BL (bottom-left), BR (bottom-right)
		const quadrants = [
			{ sx: 0, sy: 0, dx: -half, dy: -half, x: -half, y: -half, rot: 0, targetX: -half - 80, targetY: -half - 80, targetRot: -0.4 },
			{ sx: img.width / 2, sy: 0, dx: 0, dy: -half, x: 0, y: -half, rot: 0, targetX: 80, targetY: -half - 80, targetRot: 0.4 },
			{ sx: 0, sy: img.height / 2, dx: -half, dy: 0, x: -half, y: 0, rot: 0, targetX: -half - 80, targetY: 80, targetRot: 0.4 },
			{ sx: img.width / 2, sy: img.height / 2, dx: 0, dy: 0, x: 0, y: 0, rot: 0, targetX: 80, targetY: 80, targetRot: -0.4 }
		];

		let globalScale = 1.0;
		let globalAlpha = 1.0;

		function render() {
			ctx.clearRect(0, 0, width, height);

			ctx.save();
			ctx.translate(centerX, centerY);
			ctx.scale(globalScale, globalScale);
			ctx.globalAlpha = globalAlpha;

			quadrants.forEach((q) => {
				ctx.save();
				const originX = q.dx < 0 ? q.x + half / 2 : q.x + half / 2;
				const originY = q.dy < 0 ? q.y + half / 2 : q.y + half / 2;

				ctx.translate(originX, originY);
				ctx.rotate(q.rot);
				ctx.translate(-originX, -originY);

				ctx.drawImage(
					img,
					q.sx, q.sy, img.width / 2, img.height / 2,
					q.x, q.y, half, half
				);
				ctx.restore();
			});

			ctx.restore();
		}

		tl = gsap.timeline({
			onUpdate: render,
			onComplete: () => {
				gsap.to(container, {
					opacity: 0,
					duration: 0.5,
					ease: "power2.inOut",
					onComplete: () => {
						onComplete();
					}
				});
			}
		});

		// 1. Initial State: Centered image
		render();

		// 2. Separate into 4 pieces and rotate smoothly (1.5 seconds)
		quadrants.forEach((q) => {
			tl.to(q, {
				x: q.targetX,
				y: q.targetY,
				rot: q.targetRot,
				duration: 1.4,
				ease: "power2.out"
			}, 0.3);
		});

		// 3. Pause for a moment (0.3s)
		tl.to({}, { duration: 0.3 });

		// 4. Smoothly reassemble back together (1.2 seconds)
		quadrants.forEach((q) => {
			tl.to(q, {
				x: q.dx,
				y: q.dy,
				rot: 0,
				duration: 1.2,
				ease: "power3.inOut"
			}, 2.0);
		});

		// 5. Final transition effect: Smooth scale up and reveal launcher
		tl.to({ scale: 1 }, {
			scale: 1.15,
			duration: 0.5,
			ease: "power2.out",
			onUpdate: function() {
				globalScale = this.targets()[0].scale;
			}
		}, 3.3);

		tl.to({ alpha: 1 }, {
			alpha: 0,
			duration: 0.5,
			ease: "power2.inOut",
			onUpdate: function() {
				globalAlpha = this.targets()[0].alpha;
			}
		}, 3.4);
	}

	function skip() {
		if (tl) tl.kill();
		onComplete();
	}

	onDestroy(() => {
		if (animId) cancelAnimationFrame(animId);
		if (tl) tl.kill();
	});
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
	bind:this={container} 
	class="fixed inset-0 z-[9999] bg-[#07080a] flex flex-col items-center justify-center overflow-hidden select-none cursor-pointer"
	onclick={skip}
>
	<canvas bind:this={canvas} class="absolute inset-0 w-full h-full pointer-events-none"></canvas>

	<!-- Subtle Title Text -->
	<div class="absolute bottom-16 z-20 text-center space-y-1 pointer-events-none">
		<h1 class="text-3xl font-black text-white tracking-[0.3em] uppercase">
			LUXMC
		</h1>
		<p class="text-xs font-mono tracking-[0.4em] text-brand-500 uppercase font-bold">
			MINECRAFT LAUNCHER
		</p>
	</div>
</div>
