<script lang="ts">
	import { onMount } from "svelte";

	let { onComplete = () => {} }: { onComplete?: () => void } = $props();

	let visible = $state(true);
	let phase = $state<"dark" | "lightning" | "impact" | "reveal" | "tagline" | "fadeout">("dark");
	let completed = false;
	let timeouts: ReturnType<typeof setTimeout>[] = [];

	function schedule(fn: () => void, ms: number) {
		const id = setTimeout(fn, ms);
		timeouts.push(id);
		return id;
	}

	function finish() {
		if (completed) return;
		completed = true;
		timeouts.forEach(clearTimeout);
		timeouts = [];
		phase = "fadeout";
		schedule(() => {
			visible = false;
			onComplete();
		}, 700);
	}

	onMount(() => {
		schedule(() => { phase = "lightning"; }, 500);
		schedule(() => { phase = "impact"; }, 800);
		schedule(() => { phase = "reveal"; }, 1200);
		schedule(() => { phase = "tagline"; }, 2000);
		schedule(() => { phase = "fadeout"; }, 2500);
		schedule(() => {
			visible = false;
			onComplete();
		}, 3200);

		const keyHandler = (e: KeyboardEvent) => {
			if (e.key === " " || e.key === "Enter" || e.key === "Escape") {
				finish();
			}
		};
		window.addEventListener("keydown", keyHandler);

		return () => {
			timeouts.forEach(clearTimeout);
			window.removeEventListener("keydown", keyHandler);
		};
	});
</script>

{#if visible}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="cutscene-container"
		class:fadeout={phase === "fadeout"}
		onclick={finish}
		onkeydown={(e) => { if (e.key === "Enter") finish(); }}
	>
		<div class="screen" class:shake={phase === "impact"}>
			<!-- Lightning bolt SVG -->
			{#if phase === "lightning" || phase === "impact"}
				<div class="lightning-wrapper" class:flash={phase === "lightning"}>
					<svg class="bolt" viewBox="0 0 200 600" xmlns="http://www.w3.org/2000/svg">
						<path
							class="bolt-path"
							d="M100 0 L95 80 L120 120 L85 200 L115 240 L70 340 L110 380 L60 480 L105 520 L50 600"
							fill="none"
							stroke="white"
							stroke-width="6"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
						<path
							class="bolt-glow"
							d="M100 0 L95 80 L120 120 L85 200 L115 240 L70 340 L110 380 L60 480 L105 520 L50 600"
							fill="none"
							stroke="#00e5ff"
							stroke-width="14"
							stroke-linecap="round"
							stroke-linejoin="round"
							opacity="0.6"
						/>
					</svg>
				</div>
			{/if}

			<!-- Screen flash -->
			{#if phase === "lightning" || phase === "impact"}
				<div class="screen-flash" class:strong={phase === "impact"}></div>
			{/if}

			<!-- Impact sparks -->
			{#if phase === "impact" || phase === "reveal"}
				<div class="impact-zone">
					<div class="crack crack-1"></div>
					<div class="crack crack-2"></div>
					<div class="crack crack-3"></div>
					<div class="spark spark-1"></div>
					<div class="spark spark-2"></div>
					<div class="spark spark-3"></div>
					<div class="spark spark-4"></div>
					<div class="spark spark-5"></div>
					<div class="spark spark-6"></div>
				</div>
			{/if}

			<!-- Logo reveal -->
			{#if phase === "reveal" || phase === "tagline" || phase === "fadeout"}
				<div class="logo-container" class:visible={phase === "reveal" || phase === "tagline"}>
					<div class="golden-glow"></div>
					<div class="logo-frame">
						<img src="/logo.png" alt="Luxmc" class="logo-img" />
					</div>
					<div class="golden-particles">
						{#each Array(12) as _, i}
							<div class="g-particle" style="--i:{i}; --angle:{i * 30}deg; --delay:{i * 0.08}s"></div>
						{/each}
					</div>
				</div>
			{/if}

			<!-- Tagline -->
			{#if phase === "tagline" || phase === "fadeout"}
				<div class="tagline" class:visible={phase === "tagline"}>
					<div class="title-row">
						<h1 class="title">Luxmc</h1>
						<span class="badge">v1.3.0-BETA</span>
					</div>
					<p class="subtitle">Linux-First Minecraft Launcher</p>
				</div>
			{/if}

			<!-- Skip hint -->
			<div class="skip-hint">
				<span class="key">ESPAÇO</span>
				<span>ou clique para pular</span>
			</div>
		</div>
	</div>
{/if}

<style>
	.cutscene-container {
		position: fixed;
		inset: 0;
		z-index: 99999;
		background: #05060a;
		user-select: none;
		cursor: pointer;
		overflow: hidden;
		transition: opacity 0.6s ease, filter 0.6s ease;
	}

	.cutscene-container.fadeout {
		opacity: 0;
		filter: blur(12px);
	}

	.screen {
		width: 100%;
		height: 100%;
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.screen.shake {
		animation: shake 0.35s ease-out;
	}

	@keyframes shake {
		0% { transform: translate(0, 0); }
		10% { transform: translate(-8px, 4px); }
		20% { transform: translate(7px, -6px); }
		30% { transform: translate(-6px, 3px); }
		40% { transform: translate(5px, -4px); }
		50% { transform: translate(-3px, 2px); }
		60% { transform: translate(2px, -1px); }
		70% { transform: translate(-1px, 1px); }
		80% { transform: translate(1px, 0px); }
		100% { transform: translate(0, 0); }
	}

	/* Lightning bolt */
	.lightning-wrapper {
		position: absolute;
		top: -5%;
		left: 50%;
		transform: translateX(-50%);
		width: 160px;
		height: 110%;
		z-index: 10;
		pointer-events: none;
		opacity: 0;
		animation: boltAppear 0.15s ease-out forwards;
	}

	.lightning-wrapper.flash {
		filter: drop-shadow(0 0 30px rgba(0, 229, 255, 0.9)) drop-shadow(0 0 80px rgba(0, 229, 255, 0.5));
	}

	@keyframes boltAppear {
		0% { opacity: 0; clip-path: inset(0 0 100% 0); }
		30% { opacity: 1; clip-path: inset(0 0 60% 0); }
		60% { clip-path: inset(0 0 20% 0); }
		100% { opacity: 1; clip-path: inset(0 0 0% 0); }
	}

	.bolt {
		width: 100%;
		height: 100%;
	}

	.bolt-path {
		stroke-dasharray: 1200;
		stroke-dashoffset: 1200;
		animation: drawBolt 0.4s ease-out forwards;
		filter: drop-shadow(0 0 4px white);
	}

	.bolt-glow {
		stroke-dasharray: 1200;
		stroke-dashoffset: 1200;
		animation: drawBolt 0.4s ease-out forwards;
		filter: blur(3px);
	}

	@keyframes drawBolt {
		to { stroke-dashoffset: 0; }
	}

	/* Screen flash */
	.screen-flash {
		position: absolute;
		inset: 0;
		background: white;
		z-index: 5;
		pointer-events: none;
		animation: flash 0.3s ease-out forwards;
	}

	.screen-flash.strong {
		background: rgba(0, 229, 255, 0.8);
		animation: flashStrong 0.4s ease-out forwards;
	}

	@keyframes flash {
		0% { opacity: 0.9; }
		100% { opacity: 0; }
	}

	@keyframes flashStrong {
		0% { opacity: 1; }
		30% { opacity: 0.8; }
		100% { opacity: 0; }
	}

	/* Impact zone */
	.impact-zone {
		position: absolute;
		bottom: 15%;
		left: 50%;
		transform: translateX(-50%);
		width: 300px;
		height: 200px;
		z-index: 8;
		pointer-events: none;
	}

	.crack {
		position: absolute;
		bottom: 50%;
		left: 50%;
		width: 3px;
		background: linear-gradient(to bottom, rgba(0, 229, 255, 0.9), transparent);
		transform-origin: bottom center;
		animation: crackGrow 0.5s ease-out forwards;
		opacity: 0;
	}

	.crack-1 {
		height: 80px;
		transform: translateX(-50%) rotate(-35deg);
		animation-delay: 0s;
	}
	.crack-2 {
		height: 100px;
		transform: translateX(-50%) rotate(0deg);
		animation-delay: 0.05s;
	}
	.crack-3 {
		height: 70px;
		transform: translateX(-50%) rotate(40deg);
		animation-delay: 0.1s;
	}

	@keyframes crackGrow {
		0% { opacity: 0; height: 0; }
		40% { opacity: 1; }
		100% { opacity: 0.3; }
	}

	.spark {
		position: absolute;
		bottom: 50%;
		left: 50%;
		width: 4px;
		height: 4px;
		border-radius: 50%;
		background: #00e5ff;
		box-shadow: 0 0 6px #00e5ff, 0 0 12px rgba(0, 229, 255, 0.5);
		animation: sparkFly 0.6s ease-out forwards;
		opacity: 0;
	}

	.spark-1 { --sx: -60px; --sy: -80px; animation-delay: 0s; }
	.spark-2 { --sx: 50px; --sy: -90px; animation-delay: 0.03s; }
	.spark-3 { --sx: -40px; --sy: -50px; animation-delay: 0.06s; }
	.spark-4 { --sx: 70px; --sy: -60px; animation-delay: 0.04s; }
	.spark-5 { --sx: -80px; --sy: -30px; animation-delay: 0.08s; }
	.spark-6 { --sx: 30px; --sy: -100px; animation-delay: 0.02s; }

	@keyframes sparkFly {
		0% { opacity: 1; transform: translate(0, 0) scale(1); }
		100% { opacity: 0; transform: translate(var(--sx), var(--sy)) scale(0); }
	}

	/* Logo */
	.logo-container {
		position: relative;
		z-index: 20;
		display: flex;
		flex-direction: column;
		align-items: center;
		opacity: 0;
		transform: scale(0.5);
		transition: opacity 0.7s ease-out, transform 0.7s cubic-bezier(0.16, 1, 0.3, 1);
	}

	.logo-container.visible {
		opacity: 1;
		transform: scale(1);
	}

	.golden-glow {
		position: absolute;
		width: 260px;
		height: 260px;
		border-radius: 50%;
		background: radial-gradient(circle, rgba(202, 169, 124, 0.35) 0%, rgba(202, 169, 124, 0.08) 50%, transparent 70%);
		filter: blur(30px);
		animation: glowPulse 2s ease-in-out infinite;
		pointer-events: none;
	}

	@keyframes glowPulse {
		0%, 100% { transform: scale(1); opacity: 0.8; }
		50% { transform: scale(1.1); opacity: 1; }
	}

	.logo-frame {
		width: 110px;
		height: 110px;
		border-radius: 24px;
		background: #111318;
		border: 1px solid rgba(255, 255, 255, 0.1);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 14px;
		box-shadow:
			0 0 30px rgba(202, 169, 124, 0.25),
			0 0 60px rgba(202, 169, 124, 0.1),
			0 20px 40px rgba(0, 0, 0, 0.5);
	}

	.logo-img {
		width: 100%;
		height: 100%;
		object-fit: contain;
		filter: drop-shadow(0 4px 16px rgba(202, 169, 124, 0.4));
	}

	/* Golden particles */
	.golden-particles {
		position: absolute;
		width: 200px;
		height: 200px;
		pointer-events: none;
	}

	.g-particle {
		position: absolute;
		top: 50%;
		left: 50%;
		width: 3px;
		height: 3px;
		border-radius: 50%;
		background: #caa97c;
		box-shadow: 0 0 6px rgba(202, 169, 124, 0.8);
		animation: particleOrbit 2.5s ease-in-out infinite;
		animation-delay: var(--delay);
		opacity: 0;
	}

	@keyframes particleOrbit {
		0% {
			opacity: 0;
			transform: rotate(var(--angle)) translateX(40px) scale(0.5);
		}
		20% {
			opacity: 0.8;
		}
		50% {
			opacity: 1;
			transform: rotate(calc(var(--angle) + 60deg)) translateX(80px) scale(1);
		}
		80% {
			opacity: 0.6;
		}
		100% {
			opacity: 0;
			transform: rotate(calc(var(--angle) + 120deg)) translateX(40px) scale(0.5);
		}
	}

	/* Tagline */
	.tagline {
		position: absolute;
		bottom: 22%;
		left: 50%;
		transform: translateX(-50%);
		z-index: 20;
		text-align: center;
		opacity: 0;
		transition: opacity 0.6s ease-out;
	}

	.tagline.visible {
		opacity: 1;
	}

	.title-row {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 10px;
	}

	.title {
		font-size: 2rem;
		font-weight: 900;
		color: white;
		letter-spacing: 0.25em;
		text-transform: uppercase;
		margin: 0;
		font-family: system-ui, sans-serif;
	}

	.badge {
		font-size: 0.6rem;
		font-weight: 800;
		padding: 2px 8px;
		border-radius: 9999px;
		background: rgba(202, 169, 124, 0.15);
		color: #caa97c;
		border: 1px solid rgba(202, 169, 124, 0.3);
		font-family: monospace;
	}

	.subtitle {
		font-size: 0.7rem;
		font-weight: 600;
		color: rgba(255, 255, 255, 0.35);
		letter-spacing: 0.1em;
		text-transform: uppercase;
		margin-top: 8px;
	}

	/* Skip hint */
	.skip-hint {
		position: absolute;
		bottom: 32px;
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 0.7rem;
		color: rgba(255, 255, 255, 0.25);
		font-weight: 500;
		letter-spacing: 0.05em;
		z-index: 30;
	}

	.key {
		padding: 2px 8px;
		border-radius: 6px;
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid rgba(255, 255, 255, 0.1);
		font-size: 0.6rem;
		font-family: monospace;
		color: rgba(255, 255, 255, 0.4);
	}
</style>
