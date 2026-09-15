<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { appState } from "$lib/stores/app.svelte";

	let canvasEl: HTMLCanvasElement | null = $state(null);
	let animationFrameId: number | null = null;
	let isActive = $state(true);

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

	function shouldRun() {
		return !appState.performanceMode && settings.value.liveWallpaper !== false && !document.hidden;
	}

	function stopLoop() {
		if (animationFrameId !== null) {
			cancelAnimationFrame(animationFrameId);
			animationFrameId = null;
		}
	}

	function startLoop() {
		if (animationFrameId !== null) return;
		const canvas = canvasEl;
		if (!canvas) return;
		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		let width = canvas.width;
		let height = canvas.height;
		let time = 0;

		const render = () => {
			if (!shouldRun()) {
				stopLoop();
				isActive = false;
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
				ctx.fillStyle = `rgba(215, 175, 120, ${pulsingAlpha * 0.25})`;
				ctx.fill();
				ctx.beginPath();
				ctx.arc(p.x, p.y, (p.size * 0.5) * p.z, 0, Math.PI * 2);
				ctx.fillStyle = `rgba(240, 205, 150, ${pulsingAlpha * 0.6})`;
				ctx.fill();
				ctx.restore();
			}

			animationFrameId = requestAnimationFrame(render);
		};

		isActive = true;
		render();
	}

	function checkAndToggle() {
		if (shouldRun()) {
			if (!isActive) startLoop();
		} else {
			if (isActive) stopLoop();
			isActive = false;
		}
	}

	function handleVisibilityChange() {
		if (document.hidden) {
			stopLoop();
			isActive = false;
		} else {
			checkAndToggle();
		}
	}

	onMount(() => {
		if (!canvasEl) return;
		const canvas = canvasEl;
		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		canvas.width = window.innerWidth;
		canvas.height = window.innerHeight;

		const handleResize = () => {
			if (!canvasEl) return;
			canvasEl.width = window.innerWidth;
			canvasEl.height = window.innerHeight;
		};
		window.addEventListener("resize", handleResize);
		window.addEventListener("mousemove", handleMouseMove);
		document.addEventListener("visibilitychange", handleVisibilityChange);

		const count = 45;
		particles = Array.from({ length: count }, () => ({
			x: Math.random() * canvas.width,
			y: Math.random() * canvas.height,
			z: Math.random() * 0.8 + 0.2,
			vx: (Math.random() - 0.5) * 0.35,
			vy: -Math.random() * 0.45 - 0.1,
			size: Math.random() * 2.5 + 1.0,
			alpha: Math.random() * 0.5 + 0.2,
			pulseSpeed: Math.random() * 0.02 + 0.01,
		}));

		if (shouldRun()) {
			startLoop();
		}

		return () => {
			stopLoop();
			window.removeEventListener("resize", handleResize);
			window.removeEventListener("mousemove", handleMouseMove);
			document.removeEventListener("visibilitychange", handleVisibilityChange);
		};
	});

	onDestroy(() => {
		stopLoop();
	});
</script>

<canvas
	bind:this={canvasEl}
	class="fixed inset-0 pointer-events-none z-[-1] transition-opacity duration-700 {!isActive ? 'opacity-0' : 'opacity-100'}"
></canvas>
