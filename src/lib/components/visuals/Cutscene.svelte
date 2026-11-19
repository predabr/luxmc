<script lang="ts">
	import { onMount } from "svelte";
	import { APP_VERSION } from "$lib/version";
	import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { frameLoop } from "$lib/utils/frameLoop";
	import { preloadRoute } from "$lib/utils/preloadRoute";

	let { ready = false, onComplete = () => {} }: { ready?: boolean; onComplete?: () => void } = $props();

	type Star = { x: number; y: number; depth: number; size: number; alpha: number; twinkle: number; phase: number; color: string };
	type Spark = { x: number; y: number; vx: number; vy: number; life: number; max: number; size: number; color: string };
	type Comet = { x: number; y: number; vx: number; vy: number; life: number; max: number; length: number };
	type Nebula = { x: number; y: number; vx: number; vy: number; radius: number; color: string; alpha: number };

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
		const context = element.getContext("2d");
		if (!context) return;
		const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
		const styles = getComputedStyle(element);
		const tone = (name: string) => `rgb(${styles.getPropertyValue(name).trim()})`;
		const palette = [tone("--brand-100"), tone("--brand-200"), tone("--brand-300"), tone("--teal-300")];

		let width = 2;
		let height = 2;
		let stars: Star[] = [];
		let sparks: Spark[] = [];
		let comets: Comet[] = [];
		let nebulae: Nebula[] = [];
		let stop: (() => void) | undefined;
		let last = performance.now();
		let cometTimer = 1.2;

		function rebuild() {
			const ratio = Math.min(window.devicePixelRatio || 1, 1.5);
			width = Math.max(2, element!.clientWidth);
			height = Math.max(2, element!.clientHeight);
			element!.width = Math.round(width * ratio);
			element!.height = Math.round(height * ratio);
			context!.setTransform(ratio, 0, 0, ratio, 0, 0);

			const density = (width * height) / 9000;
			const count = Math.min(260, Math.max(70, Math.round(density)));
			stars = Array.from({ length: count }, (_, index) => {
				const depth = 0.25 + Math.random() * 0.75;
				return {
					x: Math.random() * width,
					y: Math.random() * height,
					depth,
					size: 0.4 + depth * 1.7,
					alpha: 0.2 + Math.random() * 0.6,
					twinkle: 0.4 + Math.random() * 1.8,
					phase: Math.random() * Math.PI * 2,
					color: palette[index % palette.length]
				};
			});

			const sparkCount = Math.min(90, Math.max(26, Math.round(density * 0.35)));
			sparks = Array.from({ length: sparkCount }, () => spawnSpark(true));

			nebulae = [
				{ x: width * 0.22, y: height * 0.3, vx: 5, vy: 3, radius: Math.min(width, height) * 0.55, color: "56,160,255", alpha: 0.1 },
				{ x: width * 0.78, y: height * 0.72, vx: -4, vy: -3, radius: Math.min(width, height) * 0.62, color: "34,196,214", alpha: 0.075 },
				{ x: width * 0.55, y: height * 0.18, vx: 3, vy: 5, radius: Math.min(width, height) * 0.42, color: "120,110,255", alpha: 0.06 }
			];
			comets = [];
		}

		function spawnSpark(initial = false): Spark {
			const edge = Math.random();
			const startX = width * (0.12 + Math.random() * 0.76);
			const startY = initial ? Math.random() * height : height + 10 + (edge > 0.5 ? 0 : Math.random() * height * 0.2);
			const max = 3.4 + Math.random() * 4.6;
			return {
				x: startX,
				y: startY,
				vx: (Math.random() - 0.5) * 14,
				vy: -(8 + Math.random() * 26),
				life: initial ? Math.random() * max : 0,
				max,
				size: 0.5 + Math.random() * 1.5,
				color: palette[Math.floor(Math.random() * palette.length)]
			};
		}

		function spawnComet() {
			const fromLeft = Math.random() > 0.5;
			const speed = 420 + Math.random() * 380;
			comets.push({
				x: fromLeft ? -60 : width + 60,
				y: height * (0.08 + Math.random() * 0.45),
				vx: (fromLeft ? 1 : -1) * speed,
				vy: speed * (0.18 + Math.random() * 0.2),
				life: 0,
				max: 2.6,
				length: 90 + Math.random() * 120
			});
		}

		function draw(now: number) {
			const seconds = Math.min(0.05, Math.max(0, now - last) / 1000);
			last = now;
			const drift = reducedMotion ? 0 : seconds;
			context!.clearRect(0, 0, width, height);

			context!.globalCompositeOperation = "lighter";
			for (const nebula of nebulae) {
				if (!reducedMotion) {
					nebula.x += nebula.vx * drift;
					nebula.y += nebula.vy * drift;
					if (nebula.x < -nebula.radius * 0.4 || nebula.x > width + nebula.radius * 0.4) nebula.vx *= -1;
					if (nebula.y < -nebula.radius * 0.4 || nebula.y > height + nebula.radius * 0.4) nebula.vy *= -1;
				}
				const gradient = context!.createRadialGradient(nebula.x, nebula.y, 0, nebula.x, nebula.y, nebula.radius);
				gradient.addColorStop(0, `rgba(${nebula.color}, ${nebula.alpha})`);
				gradient.addColorStop(0.55, `rgba(${nebula.color}, ${nebula.alpha * 0.34})`);
				gradient.addColorStop(1, `rgba(${nebula.color}, 0)`);
				context!.fillStyle = gradient;
				context!.fillRect(0, 0, width, height);
			}

			for (const star of stars) {
				if (!reducedMotion) {
					star.y += (2 + star.depth * 7) * drift;
					star.x += Math.sin(now * 0.00016 + star.phase) * (2 + star.depth * 5) * drift;
					if (star.y > height + 6) {
						star.y = -6;
						star.x = Math.random() * width;
					}
					if (star.x < -8) star.x = width + 8;
					else if (star.x > width + 8) star.x = -8;
				}
				const twinkle = 0.55 + 0.45 * Math.sin(now * 0.0011 * star.twinkle + star.phase);
				context!.fillStyle = star.color;
				context!.globalAlpha = star.alpha * twinkle * 0.16;
				context!.beginPath();
				context!.arc(star.x, star.y, star.size * (3 + star.depth * 2.6), 0, Math.PI * 2);
				context!.fill();
				context!.globalAlpha = star.alpha * twinkle;
				context!.beginPath();
				context!.arc(star.x, star.y, star.size, 0, Math.PI * 2);
				context!.fill();
			}

			for (let index = sparks.length - 1; index >= 0; index -= 1) {
				const spark = sparks[index];
				spark.life += seconds;
				if (spark.life > spark.max) {
					sparks[index] = spawnSpark(false);
					continue;
				}
				if (!reducedMotion) {
					spark.x += spark.vx * seconds + Math.sin(now * 0.0014 + spark.life) * 6 * seconds;
					spark.y += spark.vy * seconds;
				}
				const progress = spark.life / spark.max;
				const fade = Math.sin(Math.PI * progress);
				const centerPull = 1 - Math.min(1, Math.hypot(spark.x - width / 2, spark.y - height * 0.45) / (Math.min(width, height) * 0.7));
				context!.fillStyle = spark.color;
				context!.globalAlpha = fade * (0.16 + centerPull * 0.3);
				context!.beginPath();
				context!.arc(spark.x, spark.y, spark.size * (3.4 + centerPull * 5), 0, Math.PI * 2);
				context!.fill();
				context!.globalAlpha = fade * (0.5 + centerPull * 0.5);
				context!.beginPath();
				context!.arc(spark.x, spark.y, spark.size * (0.9 + centerPull * 0.7), 0, Math.PI * 2);
				context!.fill();
			}

			cometTimer -= seconds;
			if (!reducedMotion && cometTimer <= 0 && comets.length < 2) {
				spawnComet();
				cometTimer = 4.5 + Math.random() * 5;
			}
			for (let index = comets.length - 1; index >= 0; index -= 1) {
				const comet = comets[index];
				comet.life += seconds;
				comet.x += comet.vx * seconds;
				comet.y += comet.vy * seconds;
				const fade = Math.max(0, 1 - comet.life / comet.max);
				if (fade <= 0) {
					comets.splice(index, 1);
					continue;
				}
				const magnitude = Math.hypot(comet.vx, comet.vy) || 1;
				const tailX = comet.x - (comet.vx / magnitude) * comet.length;
				const tailY = comet.y - (comet.vy / magnitude) * comet.length;
				const gradient = context!.createLinearGradient(comet.x, comet.y, tailX, tailY);
				gradient.addColorStop(0, `rgba(190, 232, 255, ${0.85 * fade})`);
				gradient.addColorStop(0.35, `rgba(96, 180, 255, ${0.35 * fade})`);
				gradient.addColorStop(1, "rgba(96, 180, 255, 0)");
				context!.strokeStyle = gradient;
				context!.lineWidth = 1.6;
				context!.lineCap = "round";
				context!.globalAlpha = 1;
				context!.beginPath();
				context!.moveTo(comet.x, comet.y);
				context!.lineTo(tailX, tailY);
				context!.stroke();
				context!.globalAlpha = fade;
				context!.fillStyle = "rgba(235, 248, 255, 0.95)";
				context!.beginPath();
				context!.arc(comet.x, comet.y, 1.9, 0, Math.PI * 2);
				context!.fill();
			}

			context!.globalAlpha = 1;
			context!.globalCompositeOperation = "source-over";
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
		observer.observe(element);
		return () => {
			stop?.();
			observer.disconnect();
		};
	});
</script>

<div
	data-cutscene
	class="cutscene fixed inset-0 z-[99999] select-none overflow-hidden bg-brand-950 text-brand-50 transition-opacity duration-500 ease-smooth"
	class:opacity-0={leaving}
	class:pointer-events-none={leaving}
	role="status"
	aria-busy={!ready}
	aria-label={uiText("ui.6b516b49d3d1f7a3")}
>
	<div class="cutscene-base absolute inset-0" aria-hidden="true"></div>
	<div class="cutscene-aurora" aria-hidden="true"></div>
	<div class="cutscene-sweep" aria-hidden="true"></div>
	<canvas bind:this={canvas} class="absolute inset-0 h-full w-full" aria-hidden="true"></canvas>
	<div class="cutscene-grid" aria-hidden="true"></div>
	<div class="cutscene-vignette pointer-events-none absolute inset-0" aria-hidden="true"></div>
	<div class="cutscene-grain pointer-events-none absolute inset-0" aria-hidden="true"></div>

	<div class="relative z-10 flex h-full w-full flex-col items-center justify-center px-6">
		<div class="flex flex-col items-center transition-opacity duration-700 ease-smooth" class:opacity-0={logoOut}>
			<div class="flex flex-col items-center transition-all duration-700 ease-smooth" class:opacity-0={!revealed} class:translate-y-3={!revealed}>
				<div class="cutscene-mark relative flex items-center justify-center">
					<span class="cutscene-ring cutscene-ring--outer" aria-hidden="true"></span>
					<span class="cutscene-ring cutscene-ring--inner" aria-hidden="true"></span>
					<div class="cutscene-halo absolute rounded-full" aria-hidden="true"></div>
					<img
						src="/logo-mark.png"
						alt=""
						fetchpriority="high"
						class="cutscene-logo relative w-80 max-w-[64vw] sm:w-[22rem]"
					/>
				</div>
				<p class="mt-10 text-sm font-medium tracking-wide text-brand-100/80">{uiText("introMotion.caption")}</p>
				<div class="mt-4 flex items-center gap-2.5 text-xs text-brand-200/75" aria-live="polite">
					<span class="relative flex h-1.5 w-1.5" aria-hidden="true">
						<span class="absolute inline-flex h-full w-full rounded-full bg-brand-300 opacity-70 {ready ? 'hidden' : 'animate-ping'}"></span>
						<span class="relative inline-flex h-1.5 w-1.5 rounded-full bg-brand-300"></span>
					</span>
					<span>{ready ? uiText("introMotion.ready") : uiText("introMotion.preparing")}</span>
				</div>
			</div>
		</div>
	</div>

	<p class="absolute bottom-8 left-1/2 -translate-x-1/2 text-[11px] font-semibold tracking-[0.3em] text-brand-200/45">LUXMC v{APP_VERSION}</p>
</div>

<style>
	.cutscene-base {
		background:
			radial-gradient(circle at 50% 42%, rgb(var(--brand-800) / 0.95) 0%, rgb(var(--brand-950) / 1) 52%, #01040a 100%);
	}

	.cutscene-aurora {
		position: absolute;
		inset: -30%;
		background:
			conic-gradient(from 210deg at 30% 35%, rgb(56 160 255 / 0.34), transparent 32%, rgb(34 196 214 / 0.26) 58%, transparent 78%),
			conic-gradient(from 40deg at 72% 68%, rgb(120 110 255 / 0.22), transparent 42%, rgb(56 160 255 / 0.2) 70%, transparent 88%);
		filter: blur(64px) saturate(125%);
		animation: cutscene-aurora 22s ease-in-out infinite alternate;
		opacity: 0.85;
	}

	.cutscene-sweep {
		position: absolute;
		inset: 0;
		background: linear-gradient(102deg, transparent 38%, rgb(150 210 255 / 0.12) 47%, rgb(210 240 255 / 0.24) 50%, rgb(150 210 255 / 0.12) 53%, transparent 62%);
		mix-blend-mode: screen;
		animation: cutscene-sweep 7.5s cubic-bezier(0.4, 0, 0.2, 1) infinite;
		opacity: 0;
	}

	.cutscene-grid {
		position: absolute;
		inset: 0;
		background-image:
			linear-gradient(rgb(120 190 255 / 0.055) 1px, transparent 1px),
			linear-gradient(90deg, rgb(120 190 255 / 0.055) 1px, transparent 1px);
		background-size: 72px 72px;
		mask-image: radial-gradient(circle at 50% 45%, rgb(0 0 0 / 0.9), transparent 72%);
		opacity: 0.5;
	}

	.cutscene-vignette {
		background: radial-gradient(circle at 50% 44%, transparent 0%, rgb(1 4 10 / 0.55) 58%, rgb(1 4 10 / 0.95) 100%);
	}

	.cutscene-grain {
		opacity: 0.05;
		mix-blend-mode: overlay;
		background-image: repeating-conic-gradient(rgb(255 255 255 / 0.7) 0% 0.0009%, transparent 0.0009% 0.0022%);
		background-size: 160px 160px;
		animation: cutscene-grain 1.6s steps(4) infinite;
	}

	.cutscene-mark { isolation: isolate; }

	.cutscene-mark::before {
		content: "";
		position: absolute;
		width: 34rem;
		height: 34rem;
		border-radius: 9999px;
		background: radial-gradient(circle at 50% 50%, rgb(2 7 16) 0%, rgb(2 7 16 / 0.94) 38%, rgb(3 10 22 / 0.42) 58%, transparent 90%);
		z-index: -1;
	}

	.cutscene-logo {
		filter: drop-shadow(0 0 46px rgb(66 168 255 / 0.55)) saturate(110%);
		animation: cutscene-logo 5.2s ease-in-out infinite;
	}

	.cutscene-halo {
		height: 26rem;
		width: 26rem;
		background: radial-gradient(circle, rgb(64 170 255 / 0.3) 0%, rgb(56 160 255 / 0.12) 46%, transparent 72%);
		filter: blur(22px);
		animation: cutscene-halo 4.5s ease-in-out infinite;
	}

	.cutscene-ring {
		position: absolute;
		border-radius: 9999px;
		border: 1px solid rgb(150 210 255 / 0.5);
		opacity: 0;
	}

	.cutscene-ring--inner {
		height: 15rem;
		width: 15rem;
		animation: cutscene-ring 4.5s ease-out infinite;
	}

	.cutscene-ring--outer {
		height: 21rem;
		width: 21rem;
		animation: cutscene-ring 4.5s ease-out 1.4s infinite;
	}

	@keyframes cutscene-logo {
		0%, 100% { transform: translateY(0) scale(1); }
		50% { transform: translateY(-6px) scale(1.015); }
	}

	@keyframes cutscene-halo {
		0%, 100% { opacity: 0.55; transform: scale(0.94); }
		50% { opacity: 0.95; transform: scale(1.08); }
	}

	@keyframes cutscene-ring {
		0% { opacity: 0.75; transform: scale(0.72); }
		70% { opacity: 0; transform: scale(1.25); }
		100% { opacity: 0; transform: scale(1.25); }
	}

	@keyframes cutscene-aurora {
		0% { transform: translate3d(-4%, -2%, 0) rotate(0deg) scale(1); }
		50% { transform: translate3d(3%, 2%, 0) rotate(8deg) scale(1.08); }
		100% { transform: translate3d(-2%, 3%, 0) rotate(-6deg) scale(1.04); }
	}

	@keyframes cutscene-sweep {
		0%, 62% { opacity: 0; transform: translateX(-28%); }
		70% { opacity: 1; }
		100% { opacity: 0; transform: translateX(28%); }
	}

	@keyframes cutscene-grain {
		0% { transform: translate3d(0, 0, 0); }
		50% { transform: translate3d(-1.5%, 1%, 0); }
		100% { transform: translate3d(1%, -1.5%, 0); }
	}

	@media (prefers-reduced-motion: reduce) {
		.cutscene-aurora,
		.cutscene-sweep,
		.cutscene-grain,
		.cutscene-logo,
		.cutscene-halo,
		.cutscene-ring {
			animation: none;
		}
		.cutscene-sweep { opacity: 0; }
		.cutscene-ring { opacity: 0.2; }
	}
</style>
