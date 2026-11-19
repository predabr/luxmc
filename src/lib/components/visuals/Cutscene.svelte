<script lang="ts">
	import { onMount } from "svelte";
	import { APP_VERSION } from "$lib/version";
	import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { frameLoop } from "$lib/utils/frameLoop";
	import { preloadRoute } from "$lib/utils/preloadRoute";

	let { ready = false, onComplete = () => {} }: { ready?: boolean; onComplete?: () => void } = $props();

	type Particle = {
		x: number;
		y: number;
		speed: number;
		sway: number;
		phase: number;
		size: number;
		alpha: number;
		twinkle: number;
		color: string;
	};

	let canvas = $state<HTMLCanvasElement | null>(null);
	let revealed = $state(false);
	let logoOut = $state(false);
	let leaving = $state(false);
	let minElapsed = $state(false);
	let exitStarted = false;
	let finished = false;
	let revealTimer: ReturnType<typeof setTimeout> | undefined;
	let minTimer: ReturnType<typeof setTimeout> | undefined;
	let holdTimer: ReturnType<typeof setTimeout> | undefined;
	let fadeTimer: ReturnType<typeof setTimeout> | undefined;
	let finishTimer: ReturnType<typeof setTimeout> | undefined;
	let safetyTimer: ReturnType<typeof setTimeout> | undefined;

	function beginExit() {
		if (exitStarted || finished) return;
		exitStarted = true;
		logoOut = true;
		fadeTimer = setTimeout(() => { leaving = true; }, 420);
		finishTimer = setTimeout(() => {
			if (finished) return;
			finished = true;
			onComplete();
		}, 1080);
	}

	$effect(() => {
		if (!ready || !minElapsed) return;
		holdTimer = setTimeout(beginExit, 420);
		return () => clearTimeout(holdTimer);
	});

	onMount(() => {
		preloadRoute("/", true);
		revealTimer = setTimeout(() => { revealed = true; }, 140);
		minTimer = setTimeout(() => { minElapsed = true; }, 2100);
		safetyTimer = setTimeout(() => {
			minElapsed = true;
			beginExit();
		}, 15000);
		return () => {
			clearTimeout(revealTimer);
			clearTimeout(minTimer);
			clearTimeout(holdTimer);
			clearTimeout(fadeTimer);
			clearTimeout(finishTimer);
			clearTimeout(safetyTimer);
		};
	});

	onMount(() => {
		const element = canvas;
		if (!element) return;
		const drawing = element.getContext("2d");
		if (!drawing) return;
		const target = element;
		const context = drawing;
		const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
		const styles = getComputedStyle(element);
		const tone = (name: string) => `rgb(${styles.getPropertyValue(name).trim()} / 1)`;
		const palette = [tone("--brand-100"), tone("--brand-200"), tone("--brand-300"), tone("--teal-300")];
		let width = 2;
		let height = 2;
		let particles: Particle[] = [];
		let stop: (() => void) | undefined;
		let last = performance.now();

		function rebuild() {
			const ratio = Math.min(window.devicePixelRatio || 1, 1.5);
			width = Math.max(2, target.clientWidth);
			height = Math.max(2, target.clientHeight);
			target.width = Math.round(width * ratio);
			target.height = Math.round(height * ratio);
			context.setTransform(ratio, 0, 0, ratio, 0, 0);
			const count = Math.min(120, Math.max(42, Math.round((width * height) / 15000)));
			particles = Array.from({ length: count }, (_, index) => ({
				x: Math.random() * width,
				y: Math.random() * height,
				speed: 5 + Math.random() * 16,
				sway: 3 + Math.random() * 11,
				phase: Math.random() * Math.PI * 2,
				size: 0.6 + Math.random() * 1.8,
				alpha: 0.18 + Math.random() * 0.5,
				twinkle: 0.3 + Math.random() * 1.4,
				color: palette[index % palette.length]
			}));
		}

		function draw(now: number) {
			const seconds = Math.min(0.05, Math.max(0, now - last) / 1000);
			last = now;
			context.clearRect(0, 0, width, height);
			for (const particle of particles) {
				if (!reducedMotion) {
					particle.y -= particle.speed * seconds;
					particle.x += Math.sin(now * 0.00045 + particle.phase) * particle.sway * seconds;
					if (particle.y < -12) {
						particle.y = height + 12;
						particle.x = Math.random() * width;
					}
					if (particle.x < -16) particle.x = width + 16;
					else if (particle.x > width + 16) particle.x = -16;
				}
				const twinkle = 0.62 + 0.38 * Math.sin(now * 0.0012 * particle.twinkle + particle.phase);
				context.fillStyle = particle.color;
				context.globalAlpha = particle.alpha * twinkle * 0.22;
				context.beginPath();
				context.arc(particle.x, particle.y, particle.size * 3.2, 0, Math.PI * 2);
				context.fill();
				context.globalAlpha = particle.alpha * twinkle;
				context.beginPath();
				context.arc(particle.x, particle.y, particle.size, 0, Math.PI * 2);
				context.fill();
			}
			context.globalAlpha = 1;
		}

		rebuild();
		last = performance.now();
		draw(last);
		if (!reducedMotion) stop = frameLoop(draw, 60);
		const observer = new ResizeObserver(() => {
			rebuild();
			last = performance.now();
			draw(last);
		});
		observer.observe(target);
		return () => {
			stop?.();
			observer.disconnect();
		};
	});
</script>

<div
	data-cutscene
	class="fixed inset-0 z-[99999] select-none overflow-hidden bg-brand-950 text-brand-50 transition-opacity duration-500 ease-smooth"
	class:opacity-0={leaving}
	class:pointer-events-none={leaving}
	role="status"
	aria-busy={!ready}
	aria-label={uiText("ui.6b516b49d3d1f7a3")}
>
	<div class="absolute inset-0 bg-gradient-to-br from-brand-900 via-brand-950 to-brand-950" aria-hidden="true"></div>
	<div class="pointer-events-none absolute -left-[18%] top-[12%] h-[38rem] w-[38rem] rounded-full bg-brand-500/25 blur-[140px]" aria-hidden="true"></div>
	<div class="pointer-events-none absolute -right-[16%] bottom-[-8%] h-[34rem] w-[34rem] rounded-full bg-info/20 blur-[150px]" aria-hidden="true"></div>
	<canvas bind:this={canvas} class="absolute inset-0 h-full w-full" aria-hidden="true"></canvas>
	<div class="cutscene-vignette pointer-events-none absolute inset-0" aria-hidden="true"></div>

	<div class="relative z-10 flex h-full w-full flex-col items-center justify-center px-6">
		<div class="flex flex-col items-center transition-opacity duration-700 ease-smooth" class:opacity-0={logoOut}>
			<div class="flex flex-col items-center transition-all duration-700 ease-smooth" class:opacity-0={!revealed} class:translate-y-3={!revealed}>
				<div class="relative flex items-center justify-center">
					<div class="cutscene-halo absolute h-44 w-44 rounded-full bg-brand-500/30 blur-3xl" aria-hidden="true"></div>
					<img src="/logo.png" alt="" fetchpriority="high" class="relative w-52 max-w-[46vw] shadow-glow" />
				</div>
				<p class="mt-9 text-sm font-medium tracking-wide text-brand-100/75">{uiText("introMotion.caption")}</p>
				<div class="mt-4 flex items-center gap-2.5 text-xs text-brand-200/70" aria-live="polite">
					<span class="relative flex h-1.5 w-1.5" aria-hidden="true">
						<span class="absolute inline-flex h-full w-full rounded-full bg-brand-300 opacity-60 {ready ? 'hidden' : 'animate-ping'}"></span>
						<span class="relative inline-flex h-1.5 w-1.5 rounded-full bg-brand-300"></span>
					</span>
					<span>{ready ? uiText("introMotion.ready") : uiText("introMotion.preparing")}</span>
				</div>
			</div>
		</div>
	</div>

	<p class="absolute bottom-8 left-1/2 -translate-x-1/2 text-[11px] font-semibold tracking-[0.3em] text-brand-200/40">LUXMC v{APP_VERSION}</p>
</div>

<style>
	.cutscene-vignette {
		background: radial-gradient(circle at 50% 45%, transparent 0%, rgb(var(--brand-950) / 0.5) 62%, rgb(var(--brand-950) / 0.92) 100%);
	}
	.cutscene-halo {
		animation: cutscene-halo 4.5s ease-in-out infinite;
	}
	@keyframes cutscene-halo {
		0%, 100% { opacity: 0.45; transform: scale(0.92); }
		50% { opacity: 0.8; transform: scale(1.06); }
	}
</style>
