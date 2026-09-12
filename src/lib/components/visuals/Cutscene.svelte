<script lang="ts">
	import { onMount } from "svelte";

	let { onComplete = () => {} }: { onComplete?: () => void } = $props();

	let visible = $state(true);
	let phase = $state<"enter" | "split" | "merge" | "reveal" | "fadeout">("enter");
	let completed = false;
	let timeouts: ReturnType<typeof setTimeout>[] = [];

	function schedule(fn: () => void, ms: number) {
		const id = setTimeout(fn, ms);
		timeouts.push(id);
		return id;
	}

	function playHarmonicChime() {
		try {
			const AudioContextClass = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
			if (!AudioContextClass) return;
			const ctx = new AudioContextClass();
			if (ctx.state === "suspended") {
				void ctx.resume();
			}
			const now = ctx.currentTime;

			// Harmonious crystalline chord: Solfeggio 528Hz + major 3rd (660Hz) + perfect 5th (792Hz) + octave (1056Hz) + shimmer (2112Hz)
			const harmonics = [
				{ freq: 528, gain: 0.28, decay: 1.9 },
				{ freq: 660, gain: 0.20, decay: 1.6 },
				{ freq: 792, gain: 0.16, decay: 1.4 },
				{ freq: 1056, gain: 0.12, decay: 1.2 },
				{ freq: 2112, gain: 0.06, decay: 0.9 }
			];

			const masterGain = ctx.createGain();
			masterGain.gain.setValueAtTime(0.75, now);
			masterGain.connect(ctx.destination);

			harmonics.forEach(({ freq, gain, decay }) => {
				const osc = ctx.createOscillator();
				const noteGain = ctx.createGain();

				osc.type = "sine";
				osc.frequency.setValueAtTime(freq, now);

				noteGain.gain.setValueAtTime(0.0001, now);
				noteGain.gain.exponentialRampToValueAtTime(gain, now + 0.03);
				noteGain.gain.exponentialRampToValueAtTime(0.0001, now + decay);

				osc.connect(noteGain);
				noteGain.connect(masterGain);

				osc.start(now);
				osc.stop(now + decay + 0.05);
			});
		} catch (e) {
			console.warn("Harmonic audio skipped:", e);
		}
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
		}, 600);
	}

	onMount(() => {
		schedule(() => { phase = "split"; }, 350);
		schedule(() => {
			phase = "merge";
			playHarmonicChime();
		}, 1550);
		schedule(() => { phase = "reveal"; }, 2250);
		schedule(() => { phase = "fadeout"; }, 3300);
		schedule(() => {
			visible = false;
			onComplete();
		}, 3900);

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
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="cutscene-container"
		class:fadeout={phase === "fadeout"}
		onclick={finish}
	>
		<!-- Background ambient stars & radial vignette -->
		<div class="ambient-glow"></div>
		<div class="stars-layer"></div>

		<!-- Main Logo Stage -->
		<div class="logo-stage">
			
			<!-- Reunited Shockwave Halo -->
			{#if phase === "merge" || phase === "reveal"}
				<div class="shockwave-ring"></div>
				<div class="impact-flash"></div>
			{/if}

			<!-- The 4 Quadrants Container (240px x 240px) -->
			<div class="logo-quad-wrapper {phase}">
				<!-- Quadrant 1: Top-Left -->
				<div class="quadrant quad-tl">
					<img src="/logo.png" alt="Luxmc" class="quad-img quad-img-tl" />
				</div>

				<!-- Quadrant 2: Top-Right -->
				<div class="quadrant quad-tr">
					<img src="/logo.png" alt="Luxmc" class="quad-img quad-img-tr" />
				</div>

				<!-- Quadrant 3: Bottom-Left -->
				<div class="quadrant quad-bl">
					<img src="/logo.png" alt="Luxmc" class="quad-img quad-img-bl" />
				</div>

				<!-- Quadrant 4: Bottom-Right -->
				<div class="quadrant quad-br">
					<img src="/logo.png" alt="Luxmc" class="quad-img quad-img-br" />
				</div>

				<!-- Full unified overlay logo for seamless final lock -->
				{#if phase === "merge" || phase === "reveal"}
					<img src="/logo.png" alt="Luxmc" class="unified-logo" />
				{/if}
			</div>

			<!-- Brand Glow & Text Reveal -->
			<div class="brand-reveal" class:visible={phase === "merge" || phase === "reveal"}>
				<h1 class="brand-title">LUXMC</h1>
				<p class="brand-subtitle">O LANÇADOR DEFINITIVO</p>
			</div>

		</div>

		<!-- Skip hint -->
		<div class="skip-hint">
			<span>Pressione qualquer tecla ou clique para pular</span>
		</div>
	</div>
{/if}

<style>
	.cutscene-container {
		position: fixed;
		inset: 0;
		z-index: 99999;
		background: #08090c;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		user-select: none;
		cursor: pointer;
		transition: opacity 0.6s cubic-bezier(0.16, 1, 0.3, 1), transform 0.6s ease;
	}

	.cutscene-container.fadeout {
		opacity: 0;
		transform: scale(1.05);
		pointer-events: none;
	}

	.ambient-glow {
		position: absolute;
		width: 700px;
		height: 700px;
		background: radial-gradient(circle, rgba(226, 184, 107, 0.16) 0%, rgba(108, 92, 231, 0.08) 40%, transparent 70%);
		border-radius: 50%;
		filter: blur(80px);
		pointer-events: none;
		animation: pulse-ambient 4s ease-in-out infinite alternate;
	}

	@keyframes pulse-ambient {
		0% { transform: scale(0.9); opacity: 0.7; }
		100% { transform: scale(1.2); opacity: 1; }
	}

	.stars-layer {
		position: absolute;
		inset: 0;
		background-image: 
			radial-gradient(1.5px 1.5px at 20% 30%, rgba(255, 255, 255, 0.45) 50%, transparent 100%),
			radial-gradient(1.5px 1.5px at 70% 20%, rgba(255, 255, 255, 0.35) 50%, transparent 100%),
			radial-gradient(1px 1px at 40% 70%, rgba(255, 255, 255, 0.3) 50%, transparent 100%),
			radial-gradient(2px 2px at 85% 65%, rgba(226, 184, 107, 0.5) 50%, transparent 100%),
			radial-gradient(1.5px 1.5px at 15% 80%, rgba(255, 255, 255, 0.35) 50%, transparent 100%);
		pointer-events: none;
	}

	.logo-stage {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
	}

	/* Shockwave and impact flash on merge */
	.shockwave-ring {
		position: absolute;
		top: 50%;
		left: 50%;
		width: 140px;
		height: 140px;
		margin-top: -70px;
		margin-left: -70px;
		border-radius: 50%;
		border: 2px solid rgba(226, 184, 107, 0.9);
		box-shadow: 0 0 35px rgba(226, 184, 107, 0.7), inset 0 0 25px rgba(226, 184, 107, 0.5);
		pointer-events: none;
		animation: shockwave-expand 0.85s cubic-bezier(0.1, 0.8, 0.2, 1) forwards;
	}

	@keyframes shockwave-expand {
		0% {
			transform: scale(0.3);
			opacity: 1;
		}
		100% {
			transform: scale(4.8);
			opacity: 0;
		}
	}

	.impact-flash {
		position: absolute;
		top: 50%;
		left: 50%;
		width: 240px;
		height: 240px;
		margin-top: -120px;
		margin-left: -120px;
		background: radial-gradient(circle, rgba(255, 255, 255, 0.95) 0%, rgba(226, 184, 107, 0.6) 45%, transparent 75%);
		border-radius: 50%;
		filter: blur(12px);
		pointer-events: none;
		animation: flash-fade 0.5s ease-out forwards;
	}

	@keyframes flash-fade {
		0% { transform: scale(0.6); opacity: 1; }
		100% { transform: scale(2.4); opacity: 0; }
	}

	/* Logo 4-Quadrants Container (Prominent 240px x 240px) */
	.logo-quad-wrapper {
		position: relative;
		width: 240px;
		height: 240px;
		filter: drop-shadow(0 0 25px rgba(226, 184, 107, 0.3));
		transition: filter 0.5s ease;
	}

	.logo-quad-wrapper.merge,
	.logo-quad-wrapper.reveal {
		filter: drop-shadow(0 0 45px rgba(226, 184, 107, 0.7));
	}

	/* Base quadrant structure: each occupies 50% width and 50% height */
	.quadrant {
		position: absolute;
		width: 50%;
		height: 50%;
		overflow: hidden;
		transition: transform 0.65s cubic-bezier(0.18, 0.9, 0.25, 1.25), opacity 0.4s ease;
		will-change: transform;
	}

	/* Unified seamless logo overlay */
	.unified-logo {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: contain;
		pointer-events: none;
		animation: lock-snap 0.5s ease-out forwards;
	}

	@keyframes lock-snap {
		0% { opacity: 0; transform: scale(0.96); }
		100% { opacity: 1; transform: scale(1); }
	}

	/* Internal image sized 200% x 200% positioned to form the quadrant */
	.quad-img {
		position: absolute;
		width: 200%;
		height: 200%;
		object-fit: contain;
		pointer-events: none;
	}

	/* Quadrant 1: Top-Left */
	.quad-tl {
		top: 0;
		left: 0;
	}
	.quad-img-tl {
		top: 0;
		left: 0;
	}

	/* Quadrant 2: Top-Right */
	.quad-tr {
		top: 0;
		left: 50%;
	}
	.quad-img-tr {
		top: 0;
		left: -100%;
	}

	/* Quadrant 3: Bottom-Left */
	.quad-bl {
		top: 50%;
		left: 0;
	}
	.quad-img-bl {
		top: -100%;
		left: 0;
	}

	/* Quadrant 4: Bottom-Right */
	.quad-br {
		top: 50%;
		left: 50%;
	}
	.quad-img-br {
		top: -100%;
		left: -100%;
	}

	/* Phase: enter -> normal centered */
	.logo-quad-wrapper.enter .quadrant {
		transform: translate(0px, 0px) rotate(0deg) scale(1);
	}

	/* Phase: split -> 4 quadrants separate outwards and spin 360° */
	.logo-quad-wrapper.split .quad-tl {
		transform: translate(-120px, -120px) rotate(-360deg) scale(0.9);
		transition: transform 0.85s cubic-bezier(0.2, 0.8, 0.25, 1);
	}
	.logo-quad-wrapper.split .quad-tr {
		transform: translate(120px, -120px) rotate(360deg) scale(0.9);
		transition: transform 0.85s cubic-bezier(0.2, 0.8, 0.25, 1);
	}
	.logo-quad-wrapper.split .quad-bl {
		transform: translate(-120px, 120px) rotate(-360deg) scale(0.9);
		transition: transform 0.85s cubic-bezier(0.2, 0.8, 0.25, 1);
	}
	.logo-quad-wrapper.split .quad-br {
		transform: translate(120px, 120px) rotate(360deg) scale(0.9);
		transition: transform 0.85s cubic-bezier(0.2, 0.8, 0.25, 1);
	}

	/* Phase: merge, reveal & fadeout -> Guaranteed magnetic snap back to center */
	.logo-quad-wrapper.merge .quadrant,
	.logo-quad-wrapper.reveal .quadrant,
	.logo-quad-wrapper.fadeout .quadrant,
	.logo-quad-wrapper.merge .quad-tl,
	.logo-quad-wrapper.reveal .quad-tl,
	.logo-quad-wrapper.fadeout .quad-tl,
	.logo-quad-wrapper.merge .quad-tr,
	.logo-quad-wrapper.reveal .quad-tr,
	.logo-quad-wrapper.fadeout .quad-tr,
	.logo-quad-wrapper.merge .quad-bl,
	.logo-quad-wrapper.reveal .quad-bl,
	.logo-quad-wrapper.fadeout .quad-bl,
	.logo-quad-wrapper.merge .quad-br,
	.logo-quad-wrapper.reveal .quad-br,
	.logo-quad-wrapper.fadeout .quad-br {
		transform: translate(0px, 0px) rotate(0deg) scale(1) !important;
		transition: transform 0.65s cubic-bezier(0.18, 0.9, 0.25, 1.25) !important;
	}

	/* Brand Title & Subtitle */
	.brand-reveal {
		margin-top: 32px;
		display: flex;
		flex-direction: column;
		align-items: center;
		opacity: 0;
		transform: translateY(16px);
		transition: opacity 0.6s cubic-bezier(0.16, 1, 0.3, 1), transform 0.6s cubic-bezier(0.16, 1, 0.3, 1);
	}

	.brand-reveal.visible {
		opacity: 1;
		transform: translateY(0);
	}

	.brand-title {
		font-size: 34px;
		font-weight: 900;
		letter-spacing: 0.2em;
		color: #ffffff;
		text-shadow: 0 0 25px rgba(226, 184, 107, 0.6);
		margin: 0;
		line-height: 1;
	}

	.brand-subtitle {
		font-size: 11px;
		font-weight: 800;
		letter-spacing: 0.35em;
		color: rgba(226, 184, 107, 0.9);
		text-transform: uppercase;
		margin-top: 8px;
	}

	.skip-hint {
		position: absolute;
		bottom: 28px;
		font-size: 11px;
		color: rgba(255, 255, 255, 0.25);
		letter-spacing: 0.05em;
		pointer-events: none;
	}
</style>
