<script lang="ts">
	import { onMount } from "svelte";
	import { APP_VERSION } from "$lib/version";
	import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { frameLoop } from "$lib/utils/frameLoop";
	import { preloadRoute } from "$lib/utils/preloadRoute";

	let {
		ready = false,
		greeting = "",
		avatarUrl = "",
		instanceCount = 0,
		updateVersion = "",
		onComplete = () => {}
	}: {
		ready?: boolean;
		greeting?: string;
		avatarUrl?: string;
		instanceCount?: number;
		updateVersion?: string;
		onComplete?: () => void;
	} = $props();

	const MIN_DURATION = 2500;

	type Mote = { x: number; y: number; size: number; depth: number; speed: number; alpha: number; phase: number; bokeh: boolean };

	let canvas = $state<HTMLCanvasElement | null>(null);
	let revealed = $state(false);
	let logoOut = $state(false);
	let leaving = $state(false);
	let minElapsed = $state(false);
	let canSkip = $state(false);
	let progress = $state(0);
	let exitStarted = $state(false);
	let finished = false;
	let revealTimer: ReturnType<typeof setTimeout> | undefined;
	let minTimer: ReturnType<typeof setTimeout> | undefined;
	let skipTimer: ReturnType<typeof setTimeout> | undefined;
	let holdTimer: ReturnType<typeof setTimeout> | undefined;
	let fadeTimer: ReturnType<typeof setTimeout> | undefined;
	let finishTimer: ReturnType<typeof setTimeout> | undefined;
	let safetyTimer: ReturnType<typeof setTimeout> | undefined;
	let progressTimer: ReturnType<typeof setInterval> | undefined;
	let tipTimer: ReturnType<typeof setInterval> | undefined;
	let tipIndex = $state(0);

	const tips = $derived([uiText("introMotion.tip1"), uiText("introMotion.tip2"), uiText("introMotion.tip3")]);
	const circumference = 2 * Math.PI * 118;
	const instanceLabel = $derived(
		instanceCount === 0
			? ""
			: instanceCount === 1
				? uiText("introMotion.instanceOne")
				: uiText("introMotion.instances", { count: instanceCount })
	);

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

	function skip() {
		minElapsed = true;
		beginExit();
	}

	function handleKey(event: KeyboardEvent) {
		if (["Escape", " ", "Enter"].includes(event.key) && canSkip) {
			event.preventDefault();
			skip();
		}
	}

	$effect(() => {
		if (!ready || !minElapsed) return;
		holdTimer = setTimeout(beginExit, 320);
		return () => clearTimeout(holdTimer);
	});

	onMount(() => {
		preloadRoute("/", true);
		const started = performance.now();
		revealTimer = setTimeout(() => { revealed = true; }, 120);
		skipTimer = setTimeout(() => { canSkip = true; }, 700);
		minTimer = setTimeout(() => { minElapsed = true; }, MIN_DURATION);
		safetyTimer = setTimeout(() => {
			minElapsed = true;
			beginExit();
		}, 15000);
		progressTimer = setInterval(() => {
			const elapsed = performance.now() - started;
			const timePart = Math.min(1, elapsed / MIN_DURATION);
			progress = timePart;
		}, 80);
		tipTimer = setInterval(() => { tipIndex = (tipIndex + 1) % 3; }, 2100);
		window.addEventListener("keydown", handleKey);
		return () => {
			clearTimeout(revealTimer);
			clearTimeout(minTimer);
			clearTimeout(skipTimer);
			clearTimeout(holdTimer);
			clearTimeout(fadeTimer);
			clearTimeout(finishTimer);
			clearTimeout(safetyTimer);
			clearInterval(progressTimer);
			clearInterval(tipTimer);
			window.removeEventListener("keydown", handleKey);
		};
	});

	onMount(() => {
		const element = canvas;
		if (!element) return;
		const context = element.getContext("2d");
		if (!context) return;
		const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

		let width = 2;
		let height = 2;
		let motes: Mote[] = [];
		let stop: (() => void) | undefined;
		let last = performance.now();
		let pointerX = 0;
		let pointerY = 0;
		let targetX = 0;
		let targetY = 0;

		function rebuild() {
			const ratio = Math.min(window.devicePixelRatio || 1, 1.5);
			width = Math.max(2, element!.clientWidth);
			height = Math.max(2, element!.clientHeight);
			element!.width = Math.round(width * ratio);
			element!.height = Math.round(height * ratio);
			context!.setTransform(ratio, 0, 0, ratio, 0, 0);

			const count = Math.round(Math.min(96, Math.max(34, width / 16)));
			motes = Array.from({ length: count }, () => {
				const depth = 0.25 + Math.random() * 0.75;
				return {
					x: Math.random() * width,
					y: Math.random() * height,
					size: 0.7 + depth * 2.2,
					depth,
					speed: 4 + depth * 16,
					alpha: 0.22 + Math.random() * 0.5,
					phase: Math.random() * Math.PI * 2,
					bokeh: false
				};
			});
			for (let index = 0; index < 4; index += 1) {
				motes.push({
					x: Math.random() * width,
					y: Math.random() * height,
					size: 26 + Math.random() * 40,
					depth: 1,
					speed: 5 + Math.random() * 7,
					alpha: 0.05 + Math.random() * 0.05,
					phase: Math.random() * Math.PI * 2,
					bokeh: true
				});
			}
		}

		function onPointer(event: PointerEvent) {
			targetX = (event.clientX / window.innerWidth - 0.5) * 2;
			targetY = (event.clientY / window.innerHeight - 0.5) * 2;
		}

		function draw(now: number) {
			const seconds = Math.min(0.05, Math.max(0, now - last) / 1000);
			last = now;
			const time = now / 1000;
			const drift = reducedMotion ? 0 : seconds;
			pointerX += (targetX - pointerX) * 0.045;
			pointerY += (targetY - pointerY) * 0.045;

			context!.clearRect(0, 0, width, height);
			context!.globalCompositeOperation = "lighter";

			for (const mote of motes) {
				if (!reducedMotion) {
					mote.y -= mote.speed * drift;
					mote.x += Math.sin(time * 0.28 + mote.phase) * 5 * drift;
					if (mote.y < -mote.size) {
						mote.y = height + mote.size;
						mote.x = Math.random() * width;
					}
				}
				const x = mote.x - pointerX * (mote.bokeh ? 6 : 16) * mote.depth;
				const y = mote.y - pointerY * (mote.bokeh ? 6 : 16) * mote.depth;
				const twinkle = 0.55 + 0.45 * Math.sin(time * 0.9 + mote.phase);
				const gradient = context!.createRadialGradient(x, y, 0, x, y, mote.size * 2.4);
				gradient.addColorStop(0, `rgba(206, 232, 255, ${mote.alpha * twinkle})`);
				gradient.addColorStop(0.42, `rgba(150, 200, 255, ${mote.alpha * twinkle * 0.35})`);
				gradient.addColorStop(1, "rgba(120, 180, 255, 0)");
				context!.fillStyle = gradient;
				context!.beginPath();
				context!.arc(x, y, mote.size * 2.4, 0, Math.PI * 2);
				context!.fill();
			}

			context!.globalCompositeOperation = "source-over";
		}

		rebuild();
		last = performance.now();
		draw(last);
		if (!reducedMotion) stop = frameLoop(draw, 60);
		window.addEventListener("pointermove", onPointer);
		const observer = new ResizeObserver(() => {
			rebuild();
			last = performance.now();
			draw(last);
		});
		observer.observe(element);
		return () => {
			stop?.();
			observer.disconnect();
			window.removeEventListener("pointermove", onPointer);
		};
	});
</script>

<div
	data-cutscene
	class="cutscene fixed inset-0 z-[99999] select-none overflow-hidden text-brand-50 transition-opacity duration-500 ease-smooth"
	class:opacity-0={leaving}
	class:pointer-events-none={leaving}
	role="status"
	aria-busy={!ready}
	aria-label={uiText("ui.6b516b49d3d1f7a3")}
>
	<div class="cutscene-glow absolute inset-0" aria-hidden="true"></div>
	<canvas bind:this={canvas} class="absolute inset-0 h-full w-full" aria-hidden="true"></canvas>

	<div class="relative z-10 flex h-full w-full flex-col items-center justify-center px-6">
		<div class="flex flex-col items-center transition-opacity duration-700 ease-smooth" class:opacity-0={logoOut}>
			<div class="flex flex-col items-center transition-all duration-[900ms] ease-smooth" class:opacity-0={!revealed} class:translate-y-2={!revealed} class:scale-[.985]={!revealed}>
				<div class="cutscene-mark">
					<svg class="cutscene-arc" viewBox="0 0 240 240" aria-hidden="true">
						<circle cx="120" cy="120" r="118" class="cutscene-arc-track" />
						<circle
							cx="120"
							cy="120"
							r="118"
							class="cutscene-arc-fill"
							class:cutscene-arc-fill--done={ready && minElapsed}
							stroke-dasharray={circumference}
							stroke-dashoffset={circumference * (1 - progress)}
						/>
					</svg>
					<img src="/logo-mark.png" alt="" fetchpriority="high" class="cutscene-logo" />
					<span class="cutscene-shine" aria-hidden="true"></span>
				</div>

				<p class="cutscene-caption">{uiText("introMotion.caption")}</p>

				{#if greeting}
					<div class="cutscene-account">
						{#if avatarUrl}
							<img src={avatarUrl} alt="" class="cutscene-avatar" />
						{/if}
						<span>{uiText("introMotion.welcome", { name: greeting })}</span>
					</div>
				{/if}

				<div class="cutscene-facts">
					{#if instanceLabel}
						<span class="cutscene-fact">{instanceLabel}</span>
					{/if}
					{#if updateVersion}
						<span class="cutscene-fact cutscene-fact--update">{uiText("introMotion.updateReady", { version: updateVersion })}</span>
					{/if}
				</div>

				<p class="cutscene-status" aria-live="polite">{ready ? uiText("introMotion.ready") : uiText("introMotion.preparing")}</p>
				<p class="cutscene-tip">{tips[tipIndex]}</p>
			</div>
		</div>
	</div>

	{#if canSkip && !exitStarted}
		<button type="button" class="cutscene-skip" onclick={skip}>{uiText("introMotion.skip")}</button>
	{/if}

	<p class="cutscene-version">LUXMC v{APP_VERSION}</p>
</div>

<style>
	.cutscene {
		background:
			radial-gradient(ellipse 70% 50% at 50% 38%, #0a1b33 0%, #050c18 45%, #02060d 100%);
	}

	.cutscene-glow {
		background: radial-gradient(circle at 50% 41%, rgb(74 158 255 / 0.16) 0%, rgb(58 128 255 / 0.06) 34%, transparent 62%);
		animation: cutscene-glow 6.5s ease-in-out infinite;
	}

	.cutscene-mark {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		isolation: isolate;
	}

	.cutscene-arc {
		position: absolute;
		inset: -10%;
		width: 120%;
		height: 120%;
		transform: rotate(-90deg);
		overflow: visible;
	}

	.cutscene-arc-track {
		fill: none;
		stroke: rgb(150 200 255 / 0.1);
		stroke-width: 1;
	}

	.cutscene-arc-fill {
		fill: none;
		stroke: rgb(172 216 255 / 0.55);
		stroke-width: 1.2;
		stroke-linecap: round;
		transition: stroke-dashoffset 0.35s linear, stroke 0.5s ease;
	}

	.cutscene-arc-fill--done {
		stroke: rgb(110 231 183 / 0.9);
	}

	.cutscene-logo {
		width: 20rem;
		max-width: 64vw;
		filter: drop-shadow(0 26px 54px rgb(2 8 20 / 0.7)) drop-shadow(0 0 34px rgb(64 158 255 / 0.28));
		animation: cutscene-logo 7s ease-in-out infinite;
	}

	.cutscene-shine {
		position: absolute;
		inset: 0;
		background: linear-gradient(104deg, transparent 42%, rgb(226 244 255 / 0.9) 50%, transparent 58%);
		background-size: 320% 100%;
		background-repeat: no-repeat;
		mask-image: url("/logo-mark.png");
		mask-size: contain;
		mask-repeat: no-repeat;
		mask-position: center;
		-webkit-mask-image: url("/logo-mark.png");
		-webkit-mask-size: contain;
		-webkit-mask-repeat: no-repeat;
		-webkit-mask-position: center;
		mix-blend-mode: screen;
		opacity: 0.75;
		animation: cutscene-shine 5.5s cubic-bezier(0.45, 0, 0.2, 1) 1.1s infinite;
	}

	.cutscene-caption {
		margin-top: 2rem;
		font-size: 0.8125rem;
		letter-spacing: 0.01em;
		color: rgb(214 232 255 / 0.72);
	}

	.cutscene-account {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-top: 0.85rem;
		font-size: 0.75rem;
		color: rgb(198 222 255 / 0.6);
	}

	.cutscene-avatar {
		width: 1.35rem;
		height: 1.35rem;
		border-radius: 0.4rem;
		border: 1px solid rgb(150 200 255 / 0.22);
		object-fit: cover;
		image-rendering: pixelated;
	}

	.cutscene-facts {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 0.4rem;
		margin-top: 0.85rem;
	}

	.cutscene-fact {
		font-size: 0.625rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		padding: 0.2rem 0.55rem;
		border-radius: 9999px;
		border: 1px solid rgb(150 200 255 / 0.16);
		background: rgb(120 180 255 / 0.06);
		color: rgb(186 214 255 / 0.62);
	}

	.cutscene-fact--update {
		border-color: rgb(110 231 183 / 0.3);
		background: rgb(110 231 183 / 0.08);
		color: rgb(167 243 208 / 0.85);
	}

	.cutscene-status {
		margin-top: 1.1rem;
		font-size: 0.6875rem;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		color: rgb(150 190 240 / 0.42);
	}

	.cutscene-tip {
		margin-top: 0.55rem;
		max-width: 26rem;
		font-size: 0.6875rem;
		line-height: 1.5;
		text-align: center;
		color: rgb(150 186 230 / 0.34);
		transition: opacity 0.6s ease;
	}

	.cutscene-skip {
		position: absolute;
		bottom: 3.6rem;
		left: 50%;
		transform: translateX(-50%);
		font-size: 0.6875rem;
		letter-spacing: 0.14em;
		text-transform: uppercase;
		padding: 0.4rem 0.9rem;
		border-radius: 9999px;
		border: 1px solid rgb(150 200 255 / 0.18);
		color: rgb(190 216 255 / 0.55);
		background: rgb(120 180 255 / 0.05);
		cursor: pointer;
		transition: border-color 0.2s ease, color 0.2s ease, background-color 0.2s ease;
	}

	.cutscene-skip:hover {
		border-color: rgb(150 200 255 / 0.4);
		color: rgb(224 240 255 / 0.9);
		background: rgb(120 180 255 / 0.12);
	}

	.cutscene-version {
		position: absolute;
		bottom: 1.7rem;
		left: 50%;
		transform: translateX(-50%);
		font-size: 0.6875rem;
		font-weight: 600;
		letter-spacing: 0.32em;
		color: rgb(140 180 230 / 0.26);
	}

	@keyframes cutscene-logo {
		0%, 100% { transform: translateY(0) scale(1); }
		50% { transform: translateY(-5px) scale(1.012); }
	}

	@keyframes cutscene-glow {
		0%, 100% { opacity: 0.75; }
		50% { opacity: 1; }
	}

	@keyframes cutscene-shine {
		0% { background-position: 180% 0; }
		70%, 100% { background-position: -80% 0; }
	}

	@media (prefers-reduced-motion: reduce) {
		.cutscene-glow,
		.cutscene-logo,
		.cutscene-shine {
			animation: none;
		}
		.cutscene-shine { opacity: 0; }
	}
</style>
