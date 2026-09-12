<script lang="ts">
	import { onMount } from "svelte";
	import { appInit, getSystemSpecs, instancesList } from "$lib/api";

	let { onComplete = () => {} }: { onComplete?: () => void } = $props();

	let visible = $state(true);
	let fadeOut = $state(false);
	let progress = $state(10);
	let statusText = $state("Inicializando subsistemas...");
	let completed = false;

	function playHarmonicChime() {
		try {
			const AudioContextClass = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
			if (!AudioContextClass) return;
			const ctx = new AudioContextClass();
			if (ctx.state === "suspended") {
				void ctx.resume();
			}
			const now = ctx.currentTime;

			const harmonics = [
				{ freq: 528, gain: 0.25, decay: 1.5 },
				{ freq: 660, gain: 0.18, decay: 1.3 },
				{ freq: 792, gain: 0.14, decay: 1.1 },
				{ freq: 1056, gain: 0.10, decay: 0.9 }
			];

			const masterGain = ctx.createGain();
			masterGain.gain.setValueAtTime(0.7, now);
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
		} catch {
			// Audio context not allowed or unsupported
		}
	}

	function finish() {
		if (completed) return;
		completed = true;
		progress = 100;
		statusText = "Pronto!";
		playHarmonicChime();
		fadeOut = true;
		setTimeout(() => {
			visible = false;
			onComplete();
		}, 450);
	}

	async function warmup() {
		const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

		await sleep(250);
		progress = 25;
		statusText = "Carregando configurações e perfis...";
		await appInit().catch(() => null);

		await sleep(200);
		progress = 55;
		statusText = "Identificando recursos do sistema...";
		await getSystemSpecs().catch(() => null);

		await sleep(200);
		progress = 85;
		statusText = "Sincronizando instâncias e modpacks...";
		await instancesList().catch(() => null);

		await sleep(180);
		progress = 100;
		statusText = "Pronto para jogar!";
		await sleep(250);
		finish();
	}

	onMount(() => {
		warmup();

		const keyHandler = (e: KeyboardEvent) => {
			if (e.key === " " || e.key === "Enter" || e.key === "Escape") {
				finish();
			}
		};
		window.addEventListener("keydown", keyHandler);

		return () => {
			window.removeEventListener("keydown", keyHandler);
		};
	});
</script>

{#if visible}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-[99999] bg-[#0c0d11] flex flex-col items-center justify-center select-none overflow-hidden transition-all duration-500 ease-out cursor-pointer"
		class:opacity-0={fadeOut}
		class:scale-105={fadeOut}
		class:pointer-events-none={fadeOut}
		onclick={finish}
	>
		<!-- Ambient Glows -->
		<div class="absolute w-[500px] h-[500px] rounded-full bg-[#caa97c]/10 blur-[120px] pointer-events-none"></div>
		<div class="absolute w-[350px] h-[350px] rounded-full bg-[#6c5ce7]/10 blur-[100px] pointer-events-none translate-y-16"></div>

		<!-- Center Stage -->
		<div class="relative flex flex-col items-center z-10">
			<!-- Medium Logo with subtle glow -->
			<div class="relative group">
				<div class="absolute inset-0 bg-[#caa97c]/20 rounded-3xl blur-2xl transform scale-110"></div>
				<div class="w-28 h-28 sm:w-32 sm:h-32 rounded-3xl bg-black/40 border border-white/10 p-4 flex items-center justify-center backdrop-blur-md shadow-2xl relative">
					<img
						src="/logo.png"
						alt="Luxmc Logo"
						class="w-full h-full object-contain drop-shadow-[0_0_25px_rgba(202,169,124,0.45)]"
					/>
				</div>
			</div>

			<!-- Branding Title -->
			<div class="mt-6 text-center">
				<h1 class="text-2xl sm:text-3xl font-black text-white tracking-wider">LUXMC</h1>
				<p class="text-xs font-semibold tracking-widest text-[#caa97c] uppercase mt-1">Minecraft Launcher</p>
			</div>

			<!-- Useful Warming Loading Bar -->
			<div class="w-72 sm:w-80 mt-8 space-y-2">
				<div class="h-2 w-full bg-white/5 rounded-full overflow-hidden border border-white/10 p-0.5 shadow-inner">
					<div
						class="h-full bg-gradient-to-r from-[#caa97c] via-[#e2b86b] to-[#6c5ce7] rounded-full transition-all duration-300 ease-out shadow-[0_0_12px_rgba(202,169,124,0.4)]"
						style="width: {progress}%"
					></div>
				</div>

				<div class="flex items-center justify-between text-[11px] font-medium px-0.5">
					<span class="text-white/60 truncate mr-2">{statusText}</span>
					<span class="text-[#caa97c] font-bold shrink-0">{Math.round(progress)}%</span>
				</div>
			</div>

			<p class="text-[10px] text-white/30 mt-8 hover:text-white/50 transition-colors">
				Clique ou pressione Espaço para pular
			</p>
		</div>
	</div>
{/if}
