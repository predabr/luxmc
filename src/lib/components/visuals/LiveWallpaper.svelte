<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { appState } from "$lib/stores/app.svelte";

	let canvasEl: HTMLCanvasElement | null = $state(null);
	let animationFrameId: number | null = null;
	let isActive = $state(true);
	let focused = $state(true);
	const wallpaperFps = $derived(Math.max(5, Math.min(60, settings.value.wallpaperFps || 60)));

	const MAX_PIXELS = 200000;

	function renderScale() {
		const width = Math.max(1, window.innerWidth || 1);
		const height = Math.max(1, window.innerHeight || 1);
		const budget = Math.sqrt(MAX_PIXELS / (width * height));
		const resolutionCap = (settings.value.wallpaperWidth ?? 1280) / width;
		return Math.min(0.4, budget, resolutionCap);
	}

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

	let mouseMoveFrame: number | null = null;
	function handleMouseMove(e: MouseEvent) {
		if (mouseMoveFrame !== null) return;
		mouseMoveFrame = requestAnimationFrame(() => {
			mouseX = (e.clientX / (window.innerWidth || 1) - 0.5) * 40;
			mouseY = (e.clientY / (window.innerHeight || 1) - 0.5) * 40;
			mouseMoveFrame = null;
		});
	}

	function shouldRun() {
		return (
			!appState.performanceMode &&
			!appState.isGameRunning &&
			settings.value.liveWallpaper !== false &&
			!document.hidden
		);
	}

	function applyCanvasSize() {
		const canvas = canvasEl;
		if (!canvas) return;
		const scale = renderScale();
		const width = Math.max(2, Math.round((window.innerWidth || 1) * scale));
		const height = Math.max(2, Math.round((window.innerHeight || 1) * scale));
		if (canvas.width !== width) canvas.width = width;
		if (canvas.height !== height) canvas.height = height;
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

		let time = 0;
		let lastFrame = 0;

		const render = (now = performance.now()) => {
			if (!shouldRun()) {
				stopLoop();
				isActive = false;
				return;
			}

			const frameInterval = 1000 / wallpaperFps - 1;
			if (now - lastFrame < frameInterval) {
				animationFrameId = requestAnimationFrame(render);
				return;
			}
			const speed = Math.min((now - lastFrame) / 1000, 0.1) * 60;
			lastFrame = now;
			const width = canvas.width;
			const height = canvas.height;

			time += 0.02 * speed;
			ctx.clearRect(0, 0, width, height);

			for (const p of particles) {
				p.x += (p.vx + mouseX * 0.01 * p.z) * speed;
				p.y += (p.vy + mouseY * 0.01 * p.z) * speed;

				if (p.x < -20) p.x = width + 20;
				if (p.x > width + 20) p.x = -20;
				if (p.y < -20) p.y = height + 20;
				if (p.y > height + 20) p.y = -20;
			}

			ctx.fillStyle = "rgb(215, 175, 120)";
			for (const p of particles) {
				ctx.globalAlpha = p.alpha * (0.6 + 0.4 * Math.sin(time * 2 + p.pulseSpeed * 100)) * 0.25;
				ctx.beginPath();
				ctx.arc(p.x, p.y, p.size * p.z, 0, Math.PI * 2);
				ctx.fill();
			}
			ctx.fillStyle = "rgb(240, 205, 150)";
			for (const p of particles) {
				ctx.globalAlpha = p.alpha * (0.6 + 0.4 * Math.sin(time * 2 + p.pulseSpeed * 100)) * 0.6;
				ctx.beginPath();
				ctx.arc(p.x, p.y, p.size * 0.5 * p.z, 0, Math.PI * 2);
				ctx.fill();
			}
			ctx.globalAlpha = 1;

			animationFrameId = requestAnimationFrame(render);
		};

		isActive = true;
		render();
	}

	function checkAndToggle() {
		if (appState.isScrolling) {
			stopLoop();
			return;
		}
		applyCanvasSize();
		if (shouldRun()) {
			startLoop();
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
		applyCanvasSize();

		let blurTimer: ReturnType<typeof setTimeout> | null = null;
		const handleResize = () => checkAndToggle();
		const handleFocus = () => {
			if (blurTimer) clearTimeout(blurTimer);
			blurTimer = null;
			focused = true;
		};
		const handleBlur = () => {
			if (blurTimer) clearTimeout(blurTimer);
			blurTimer = setTimeout(() => {
				focused = document.hasFocus();
			}, 250);
		};
		focused = document.hasFocus();
		window.addEventListener("resize", handleResize);
		window.addEventListener("mousemove", handleMouseMove);
		window.addEventListener("focus", handleFocus);
		window.addEventListener("blur", handleBlur);
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
			if (blurTimer) clearTimeout(blurTimer);
			stopLoop();
			window.removeEventListener("resize", handleResize);
			window.removeEventListener("mousemove", handleMouseMove);
			window.removeEventListener("focus", handleFocus);
			window.removeEventListener("blur", handleBlur);
			document.removeEventListener("visibilitychange", handleVisibilityChange);
		};
	});

	$effect(() => {
		void appState.isScrolling;
		void appState.performanceMode;
		void appState.isGameRunning;
		void settings.value.liveWallpaper;
		void settings.value.wallpaperFps;
		void settings.value.wallpaperWidth;
		void settings.value.pauseWallpaperOnBlur;
		void focused;
		if (canvasEl) checkAndToggle();
	});

	onDestroy(() => {
		if (mouseMoveFrame !== null) cancelAnimationFrame(mouseMoveFrame);
		stopLoop();
	});
</script>

<canvas
	bind:this={canvasEl}
	class="fixed inset-0 pointer-events-none z-[-1] transform-gpu transition-opacity duration-700 {!isActive ? 'opacity-0' : 'opacity-100'}"
></canvas>
