<script lang="ts">
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import { cubicOut, cubicInOut } from "svelte/easing";

	let { onComplete = () => {} }: { onComplete?: () => void } = $props();

	let visible = $state(true);
	let logoVisible = $state(false);
	let completed = false;
	let timeoutId: ReturnType<typeof setTimeout> | null = null;

	function finish() {
		if (completed) return;
		completed = true;
		if (timeoutId) clearTimeout(timeoutId);
		logoVisible = false;
		visible = false;
		setTimeout(() => {
			onComplete();
		}, 450);
	}

	function playChime() {
		try {
			const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
			if (!AudioCtx) return;
			const ctx = new AudioCtx();
			const now = ctx.currentTime;

			const bell = ctx.createOscillator();
			const bellGain = ctx.createGain();
			bell.type = "sine";
			bell.frequency.setValueAtTime(587.33, now); // D5
			bell.frequency.setValueAtTime(880.00, now + 0.1); // A5
			bell.frequency.setValueAtTime(1174.66, now + 0.2); // D6

			bellGain.gain.setValueAtTime(0.001, now);
			bellGain.gain.exponentialRampToValueAtTime(0.12, now + 0.15);
			bellGain.gain.exponentialRampToValueAtTime(0.001, now + 0.8);

			bell.connect(bellGain);
			bellGain.connect(ctx.destination);
			bell.start(now);
			bell.stop(now + 0.8);
		} catch {}
	}

	onMount(() => {
		setTimeout(() => {
			logoVisible = true;
			playChime();
		}, 80);

		timeoutId = setTimeout(() => {
			finish();
		}, 1600);

		const keyHandler = (e: KeyboardEvent) => {
			if (e.key === " " || e.key === "Enter" || e.key === "Escape") {
				finish();
			}
		};
		window.addEventListener("keydown", keyHandler);

		return () => {
			if (timeoutId) clearTimeout(timeoutId);
			window.removeEventListener("keydown", keyHandler);
		};
	});
</script>

{#if visible}
	<div 
		class="fixed inset-0 z-[99999] flex flex-col items-center justify-center bg-[#0d0e12] select-none cursor-pointer overflow-hidden"
		onclick={finish}
		role="button"
		tabindex="0"
		onkeydown={(e) => { if (e.key === 'Enter') finish(); }}
		transition:fade={{ duration: 400, easing: cubicInOut }}
	>
		<!-- Ambient Radial Glow -->
		<div class="absolute w-[420px] h-[420px] rounded-full bg-gradient-to-br from-[#caa97c]/15 via-[#caa97c]/5 to-transparent blur-3xl pointer-events-none"></div>

		<!-- Centered Logo Content -->
		{#if logoVisible}
			<div 
				class="relative flex flex-col items-center justify-center z-10"
				in:scale={{ start: 0.93, duration: 600, easing: cubicOut }}
			>
				<!-- 3D Logo Frame -->
				<div class="relative h-28 w-28 flex items-center justify-center mb-5">
					<div class="absolute inset-0 bg-[#caa97c]/20 rounded-full blur-2xl animate-pulse"></div>
					<img 
						src="/logo.png" 
						alt="Luxmc" 
						class="w-full h-full object-contain relative z-10 drop-shadow-[0_8px_32px_rgba(202,169,124,0.35)]" 
					/>
				</div>

				<!-- App Title & Tagline -->
				<div class="text-center space-y-1.5" in:fade={{ duration: 500, delay: 150 }}>
					<h1 class="text-2xl font-black text-white tracking-[0.25em] uppercase font-sans">
						Luxmc
					</h1>
					<p class="text-[11px] font-semibold text-white/40 tracking-wider uppercase">
						Linux-First Minecraft Launcher
					</p>
				</div>

				<!-- Subtle Indeterminate Loading Line -->
				<div class="w-28 h-0.5 bg-white/10 rounded-full overflow-hidden mt-6" in:fade={{ duration: 400, delay: 250 }}>
					<div class="h-full w-12 bg-gradient-to-r from-transparent via-[#caa97c] to-transparent rounded-full animate-[shimmer_1.2s_infinite]"></div>
				</div>
			</div>
		{/if}

		<!-- Bottom Skip Hint -->
		<div class="absolute bottom-6 text-[10px] text-white/20 font-medium tracking-wide">
			Pressione qualquer tecla ou clique para continuar
		</div>
	</div>
{/if}

<style>
	@keyframes shimmer {
		0% {
			transform: translateX(-100%);
		}
		100% {
			transform: translateX(250%);
		}
	}
</style>
