<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { appState } from "$lib/stores/app.svelte";

	let canvasEl: HTMLCanvasElement | null = $state(null);
	let animationFrameId: number | null = null;

	type Particle = {
		x: number;
		y: number;
		z: number;
		vx: number;
		vy: number;
		size: number;
		alpha: number;
		pulseSpeed: number;
	};

	let particles: Particle[] = [];
	let mouseX = 0;
	let mouseY = 0;

	function handleMouseMove(e: MouseEvent) {
		mouseX = (e.clientX / (window.innerWidth || 1) - 0.5) * 40;
		mouseY = (e.clientY / (window.innerHeight || 1) - 0.5) * 40;
	}

	onMount(() => {
		if (!canvasEl) return;
		const canvas = canvasEl;
		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		let width = (canvas.width = window.innerWidth);
		let height = (canvas.height = window.innerHeight);

		const handleResize = () => {
			if (!canvasEl) return;
			width = canvasEl.width = window.innerWidth;
			height = canvasEl.height = window.innerHeight;
		};
		window.addEventListener("resize", handleResize);
		window.addEventListener("mousemove", handleMouseMove);

		// Generate ambient particle dust
		const count = 45;
		particles = Array.from({ length: count }, () => ({
			x: Math.random() * width,
			y: Math.random() * height,
			z: Math.random() * 0.8 + 0.2,
			vx: (Math.random() - 0.5) * 0.35,
			vy: -Math.random() * 0.45 - 0.1,
			size: Math.random() * 2.5 + 1.0,
			alpha: Math.random() * 0.5 + 0.2,
			pulseSpeed: Math.random() * 0.02 + 0.01,
		}));

		let time = 0;
		const render = () => {
			if (appState.performanceMode || settings.value.liveWallpaper === false || document.hidden) {
				animationFrameId = requestAnimationFrame(render);
				return;
			}

			time += 0.02;
			ctx.clearRect(0, 0, width, height);

			for (const p of particles) {
				p.x += p.vx + mouseX * 0.01 * p.z;
				p.y += p.vy + mouseY * 0.01 * p.z;

				if (p.x < -20) p.x = width + 20;
				if (p.x > width + 20) p.x = -20;
				if (p.y < -20) p.y = height + 20;
				if (p.y > height + 20) p.y = -20;

				const pulsingAlpha = p.alpha * (0.6 + 0.4 * Math.sin(time * 2 + p.pulseSpeed * 100));
				ctx.save();
				ctx.beginPath();
				ctx.arc(p.x, p.y, p.size * p.z, 0, Math.PI * 2);

				// Amber / gold / brand glow
				ctx.fillStyle = `rgba(215, 175, 120, ${pulsingAlpha * 0.5})`;
				ctx.shadowColor = "rgba(226, 184, 107, 0.6)";
				ctx.shadowBlur = 8 * p.z;
				ctx.fill();
				ctx.restore();
			}

			animationFrameId = requestAnimationFrame(render);
		};

		render();

		return () => {
			window.removeEventListener("resize", handleResize);
			window.removeEventListener("mousemove", handleMouseMove);
		};
	});

	onDestroy(() => {
		if (animationFrameId) {
			cancelAnimationFrame(animationFrameId);
			animationFrameId = null;
		}
	});
</script>

<canvas
	bind:this={canvasEl}
	class="fixed inset-0 pointer-events-none z-[-1] transition-opacity duration-700 {appState.performanceMode || settings.value.liveWallpaper === false ? 'opacity-0' : 'opacity-100'}"
></canvas>
