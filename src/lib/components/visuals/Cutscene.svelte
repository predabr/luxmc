<script lang="ts">
	import { onMount } from "svelte";
	import { APP_VERSION } from "$lib/version";
	import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { preloadRoute } from "$lib/utils/preloadRoute";

	let { ready = false, onComplete = () => {} }: { ready?: boolean; onComplete?: () => void } = $props();

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
		revealTimer = setTimeout(() => { revealed = true; }, 120);
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

	<div class="relative z-10 flex h-full w-full flex-col items-center justify-center px-6">
		<div class="flex flex-col items-center transition-opacity duration-700 ease-smooth" class:opacity-0={logoOut}>
			<div class="flex flex-col items-center transition-all duration-[900ms] ease-smooth" class:opacity-0={!revealed} class:translate-y-2={!revealed} class:scale-[.985]={!revealed}>
				<div class="cutscene-mark">
					<img src="/logo-mark.png" alt="" fetchpriority="high" class="cutscene-logo" />
					<span class="cutscene-shine" aria-hidden="true"></span>
				</div>

				<div class="cutscene-rule" aria-hidden="true"><span></span></div>

				<p class="cutscene-caption">{uiText("introMotion.caption")}</p>
				<p class="cutscene-status" aria-live="polite">{ready ? uiText("introMotion.ready") : uiText("introMotion.preparing")}</p>
			</div>
		</div>
	</div>

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

	.cutscene-rule {
		position: relative;
		margin-top: 2.25rem;
		height: 1px;
		width: 13rem;
		max-width: 60vw;
		background: rgb(150 200 255 / 0.14);
		overflow: hidden;
	}

	.cutscene-rule span {
		position: absolute;
		inset: 0 auto 0 0;
		display: block;
		background: linear-gradient(90deg, transparent, rgb(180 224 255 / 0.85) 70%, rgb(236 248 255));
		transform-origin: left center;
		transform: scaleX(0);
		animation: cutscene-rule 2.1s cubic-bezier(0.32, 0.72, 0.24, 1) forwards;
	}

	.cutscene-caption {
		margin-top: 1.35rem;
		font-size: 0.8125rem;
		letter-spacing: 0.01em;
		color: rgb(214 232 255 / 0.72);
	}

	.cutscene-status {
		margin-top: 0.6rem;
		font-size: 0.6875rem;
		letter-spacing: 0.16em;
		text-transform: uppercase;
		color: rgb(150 190 240 / 0.42);
	}

	.cutscene-version {
		position: absolute;
		bottom: 2rem;
		left: 50%;
		transform: translateX(-50%);
		font-size: 0.6875rem;
		font-weight: 600;
		letter-spacing: 0.32em;
		color: rgb(140 180 230 / 0.3);
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

	@keyframes cutscene-rule {
		0% { transform: scaleX(0); opacity: 0.2; }
		100% { transform: scaleX(1); opacity: 1; }
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
