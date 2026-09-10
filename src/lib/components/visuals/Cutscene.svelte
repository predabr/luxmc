<script lang="ts">
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import { cubicOut, cubicInOut } from "svelte/easing";

	let { onComplete = () => {} }: { onComplete?: () => void } = $props();

	let visible = $state(true);
	let logoVisible = $state(false);
	let progressValue = $state(0);
	let statusText = $state("Inicializando sistema...");
	let completed = false;
	let intervalId: ReturnType<typeof setInterval> | null = null;
	let finishTimeoutId: ReturnType<typeof setTimeout> | null = null;

	function finish() {
		if (completed) return;
		completed = true;
		if (intervalId) clearInterval(intervalId);
		if (finishTimeoutId) clearTimeout(finishTimeoutId);
		logoVisible = false;
		visible = false;
		setTimeout(() => {
			onComplete();
		}, 500);
	}

	onMount(() => {
		setTimeout(() => {
			logoVisible = true;
		}, 60);

		const startTime = Date.now();
		const totalDuration = 3200; // 3.2s cinematic pacing

		intervalId = setInterval(() => {
			const elapsed = Date.now() - startTime;
			const p = Math.min(1, elapsed / totalDuration);
			
			// Smooth easeOut curve for progress
			const eased = 1 - Math.pow(1 - p, 2.5);
			progressValue = Math.round(eased * 100);

			if (p < 0.3) {
				statusText = "Carregando módulos do sistema...";
			} else if (p < 0.65) {
				statusText = "Sincronizando bibliotecas e instâncias...";
			} else if (p < 0.92) {
				statusText = "Configurando aceleração gráfica...";
			} else {
				statusText = "Tudo pronto!";
			}

			if (p >= 1) {
				if (intervalId) clearInterval(intervalId);
				finishTimeoutId = setTimeout(() => {
					finish();
				}, 200);
			}
		}, 30);

		const keyHandler = (e: KeyboardEvent) => {
			if (e.key === " " || e.key === "Enter" || e.key === "Escape") {
				finish();
			}
		};
		window.addEventListener("keydown", keyHandler);

		return () => {
			if (intervalId) clearInterval(intervalId);
			if (finishTimeoutId) clearTimeout(finishTimeoutId);
			window.removeEventListener("keydown", keyHandler);
		};
	});
</script>

{#if visible}
	<div 
		class="fixed inset-0 z-[99999] flex flex-col items-center justify-center bg-[#090a0d] select-none cursor-pointer overflow-hidden"
		onclick={finish}
		role="button"
		tabindex="0"
		onkeydown={(e) => { if (e.key === 'Enter') finish(); }}
		transition:fade={{ duration: 500, easing: cubicInOut }}
	>
		<!-- Ambient Cinematic Radial Glows -->
		<div class="absolute w-[500px] h-[500px] rounded-full bg-gradient-to-br from-[#caa97c]/15 via-[#caa97c]/5 to-transparent blur-3xl pointer-events-none"></div>
		<div class="absolute w-[300px] h-[300px] rounded-full bg-gradient-to-tr from-[#14b8a6]/10 via-transparent to-transparent blur-3xl pointer-events-none"></div>

		<!-- Centered Logo Content -->
		{#if logoVisible}
			<div 
				class="relative flex flex-col items-center justify-center z-10 max-w-sm w-full px-6"
				in:scale={{ start: 0.92, duration: 700, easing: cubicOut }}
			>
				<!-- Glowing 3D Logo Frame -->
				<div class="relative h-28 w-28 flex items-center justify-center mb-6">
					<div class="absolute inset-0 bg-[#caa97c]/25 rounded-full blur-2xl animate-pulse"></div>
					<div class="relative w-24 h-24 rounded-3xl bg-[#131418] border border-white/10 p-2 shadow-2xl flex items-center justify-center">
						<img 
							src="/logo.png" 
							alt="Luxmc" 
							class="w-full h-full object-contain drop-shadow-[0_8px_24px_rgba(202,169,124,0.4)]" 
						/>
					</div>
				</div>

				<!-- App Title, Tagline & Version Badge -->
				<div class="text-center space-y-2 mb-8" in:fade={{ duration: 600, delay: 150 }}>
					<div class="flex items-center justify-center gap-2">
						<h1 class="text-3xl font-black text-white tracking-[0.25em] uppercase font-sans">
							Luxmc
						</h1>
						<span class="text-[10px] font-extrabold px-2 py-0.5 rounded-full bg-[#caa97c]/15 text-[#caa97c] border border-[#caa97c]/30 font-mono">
							v1.2.0-ALPHA
						</span>
					</div>
					<p class="text-[11px] font-semibold text-white/40 tracking-wider uppercase">
						Linux-First Minecraft Launcher
					</p>
				</div>

				<!-- Smooth Progress Bar Container -->
				<div class="w-full space-y-2" in:fade={{ duration: 500, delay: 250 }}>
					<div class="flex items-center justify-between text-[11px] text-white/50 font-medium px-0.5">
						<span class="truncate text-white/60">{statusText}</span>
						<span class="font-mono text-[#caa97c] font-bold">{progressValue}%</span>
					</div>
					<div class="w-full h-1.5 bg-white/10 rounded-full overflow-hidden relative">
						<div 
							class="h-full bg-gradient-to-r from-[#caa97c] via-[#ebd095] to-[#caa97c] rounded-full transition-all duration-150 ease-out"
							style="width: {progressValue}%"
						></div>
					</div>
				</div>
			</div>
		{/if}

		<!-- Bottom Skip Hint -->
		<div class="absolute bottom-8 flex items-center gap-2 text-[11px] text-white/30 font-medium tracking-wide">
			<span class="px-2 py-0.5 rounded-md bg-white/5 border border-white/10 text-[10px] font-mono text-white/50">ESPAÇO</span>
			<span>ou clique para iniciar</span>
		</div>
	</div>
{/if}
