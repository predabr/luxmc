<script lang="ts">
	import { onMount } from "svelte";
	import {
		appInit,
		getSystemSpecs,
		instancesList,
		discordSetActivity,
		optimizerTrimMemory
	} from "$lib/api";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";

	let { onComplete = () => {} }: { onComplete?: () => void } = $props();

	let visible = $state(true);
	let fadeOut = $state(false);
	let progress = $state(0);
	let statusText = $state("Inicializando subsistemas...");
	let completed = false;
	let skipped = false;
	let stageIndex = $state(0);

	const stages = [
		{ label: "Detectando hardware e versões do Java...", target: 25 },
		{ label: "Sincronizando instâncias e perfis locais...", target: 55 },
		{ label: "Autenticando sessão e pré-carregando skins...", target: 80 },
		{ label: "Inicializando Discord RPC e otimizando memória...", target: 95 },
		{ label: "Pronto para jogar!", target: 100 }
	];

	let particles = $state<Array<{ x: number; y: number; size: number; delay: number; duration: number; opacity: number }>>([]);

	function generateParticles() {
		const arr: typeof particles = [];
		for (let i = 0; i < 40; i++) {
			arr.push({
				x: Math.random() * 100,
				y: Math.random() * 100,
				size: Math.random() * 3 + 1,
				delay: Math.random() * 5,
				duration: Math.random() * 4 + 3,
				opacity: Math.random() * 0.4 + 0.1
			});
		}
		particles = arr;
	}

	function finish() {
		if (completed) return;
		completed = true;
		progress = 100;
		statusText = "Pronto para jogar!";
		fadeOut = true;
		setTimeout(() => {
			visible = false;
			onComplete();
		}, 450);
	}

	function updateStage(idx: number) {
		if (skipped || completed) return;
		stageIndex = idx;
		statusText = stages[idx].label;
		progress = stages[idx].target;
	}

	async function warmup() {
		try { await appInit().catch(() => {}); } catch {}

		const hardwarePromise = (async () => {
			updateStage(0);
			try {
				await getSystemSpecs().catch(() => null);
				await import("$lib/api").then(m => m.javaScan()).catch(() => null);
			} catch {}
		})();

		const instancesPromise = (async () => {
			await hardwarePromise;
			if (skipped || completed) return;
			updateStage(1);
			try {
				await instancesList().catch(() => null);
			} catch {}
		})();

		const authPromise = (async () => {
			await instancesPromise;
			if (skipped || completed) return;
			updateStage(2);
			try {
				const acc = account.value;
				if (acc?.skinUrl || acc?.username) {
					const username = acc.username || "Steve";
					const skinUrl = acc.skinUrl || `https://minotar.net/skin/${username}`;
				const avatarUrl = `https://mc-heads.net/avatar/${username}/100`;
				const bodyUrl = `https://mc-heads.net/body/${username}/300`;
				activeSkinStore.setSkin({
					id: acc.uuid || username,
					name: username,
					url: bodyUrl,
					skinUrl,
					avatarUrl,
					type: "steve"
				});
				}
			} catch {}
		})();

		const discordPromise = (async () => {
			await authPromise;
			if (skipped || completed) return;
			updateStage(3);
			try {
				await discordSetActivity({
					details: "No Launcher",
					state: "Explorando o Luxmc",
					largeImage: "luxmc_logo",
					smallText: "Luxmc v1.9.1",
					startTime: Math.floor(Date.now() / 1000)
				}).catch(() => {});
				optimizerTrimMemory().catch(() => {});
			} catch {}
		})();

		const maxWait = new Promise<void>(r => setTimeout(r, 2200));
		await Promise.race([discordPromise, maxWait]);

		if (!skipped && !completed) {
			updateStage(4);
			await new Promise(r => setTimeout(r, 350));
			finish();
		}
	}

	let keyHandler: ((e: KeyboardEvent) => void) | null = null;

	onMount(() => {
		generateParticles();
		warmup();
		const forceTimer = setTimeout(() => { if (!completed) { skipped = true; finish(); } }, 2800);

		keyHandler = (e: KeyboardEvent) => {
			if (e.key === " " || e.key === "Enter" || e.key === "Escape") {
				skipped = true;
				finish();
			}
		};

		window.addEventListener("keydown", keyHandler);
		return () => {
			if (keyHandler) window.removeEventListener("keydown", keyHandler);
		};
	});

	const circumference = 2 * Math.PI * 42;
	const strokeDashoffset = $derived(circumference - (progress / 100) * circumference);
</script>

{#if visible}
	<div
		class="fixed inset-0 z-[99999] bg-bg flex flex-col items-center justify-center select-none overflow-hidden transition-all duration-500 ease-out cursor-pointer"
		class:opacity-0={fadeOut}
		class:scale-[1.03]={fadeOut}
		class:pointer-events-none={fadeOut}
		onclick={() => { skipped = true; finish(); }}
		onkeydown={(e) => { if (e.key === " " || e.key === "Enter" || e.key === "Escape") { skipped = true; finish(); } }}
		role="presentation"
		tabindex="-1"
	>
		{#each particles as p (p.x + p.y)}
			<div
				class="absolute rounded-full bg-brand-400 pointer-events-none animate-[particleFloat_var(--dur)_ease-in-out_infinite]"
				style="left:{p.x}%;top:{p.y}%;width:{p.size}px;height:{p.size}px;opacity:{p.opacity};--dur:{p.duration}s;animation-delay:{p.delay}s;"
			></div>
		{/each}

		<div class="absolute w-[600px] h-[600px] rounded-full bg-[radial-gradient(circle,rgb(var(--brand-400)_/_0.12)_0%,transparent_70%)] pointer-events-none animate-[orbPulse_6s_ease-in-out_infinite]"></div>
		<div class="absolute w-[400px] h-[400px] rounded-full bg-[radial-gradient(circle,rgb(var(--brand-600)/0.10)_0%,transparent_70%)] pointer-events-none translate-y-12 animate-[orbPulse_8s_ease-in-out_infinite_reverse]"></div>
		<div class="absolute w-[300px] h-[300px] rounded-full bg-[radial-gradient(circle,rgb(var(--brand-400)_/_0.06)_0%,transparent_60%)] pointer-events-none -translate-y-20 animate-[orbPulse_5s_ease-in-out_infinite]"></div>

		<div class="relative flex flex-col items-center z-10">
			<div class="relative group">
				<div class="absolute inset-0 bg-brand-400/15 rounded-[2rem] blur-3xl transform scale-125 animate-[logoGlow_4s_ease-in-out_infinite]"></div>
				<div class="w-32 h-32 sm:w-36 sm:h-36 rounded-[2rem] bg-gradient-to-br from-bg to-bg border border-fg/[0.08] p-5 flex items-center justify-center backdrop-blur-xl shadow-elevated relative overflow-hidden group-hover:scale-105 transition-transform duration-700">
					<div class="absolute inset-0 bg-gradient-to-br from-brand-400/10 via-transparent to-brand-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-700"></div>
					<img
						src="/logo.png"
						alt="Luxmc Logo"
						class="w-full h-full object-contain drop-shadow-[0_0_32px_rgb(var(--brand-400)_/_0.4)] relative z-10"
					/>
				</div>
			</div>

			<div class="mt-7 text-center">
				<h1 class="text-3xl sm:text-4xl font-black text-fg tracking-[0.2em] bg-gradient-to-r from-fg via-fg/90 to-fg/70 bg-clip-text text-transparent">LUXMC</h1>
				<p class="text-[11px] font-bold tracking-[0.3em] text-brand-400/80 uppercase mt-1.5">Minecraft Launcher</p>
			</div>

			<div class="relative mt-10">
				<svg class="w-28 h-28 sm:w-32 sm:h-32 -rotate-90" viewBox="0 0 100 100">
					<circle
						cx="50" cy="50" r="42"
						fill="none"
						stroke="rgb(var(--fg) / 0.04)"
						stroke-width="3"
					/>
					<circle
						cx="50" cy="50" r="42"
						fill="none"
						stroke="url(#progressGradient)"
						stroke-width="3"
						stroke-linecap="round"
						stroke-dasharray={circumference}
						stroke-dashoffset={strokeDashoffset}
						class="transition-all duration-700 ease-out"
						style="filter: drop-shadow(0 0 8px rgb(var(--brand-400) / 0.4));"
					/>
					<defs>
						<linearGradient id="progressGradient" x1="0%" y1="0%" x2="100%" y2="0%">
							<stop offset="0%" stop-color="#caa97c" />
							<stop offset="50%" stop-color="#e2b86b" />
							<stop offset="100%" stop-color="#6c5ce7" />
						</linearGradient>
					</defs>
				</svg>
				<div class="absolute inset-0 flex items-center justify-center">
					<span class="text-lg font-black text-fg/90">{Math.round(progress)}%</span>
				</div>
			</div>

			<div class="w-72 sm:w-80 mt-6 space-y-2.5">
				<div class="h-1.5 w-full bg-fg/[0.04] rounded-full overflow-hidden border border-fg/[0.06] p-px shadow-inner">
					<div
						class="h-full bg-gradient-to-r from-brand-400 via-brand-400 to-brand-500 rounded-full transition-all duration-700 ease-out relative"
						style="width: {progress}%"
					>
						<div class="absolute inset-0 bg-gradient-to-r from-transparent via-fg/20 to-transparent animate-[shimmer_2s_ease-in-out_infinite]"></div>
					</div>
				</div>

				<div class="flex items-center justify-between text-[11px] font-medium px-0.5">
					<span class="text-fg/50 truncate mr-2 transition-all duration-300">{statusText}</span>
					<span class="text-brand-400/80 font-bold shrink-0 tabular-nums">{Math.round(progress)}%</span>
				</div>

				<div class="flex items-center gap-1.5 pt-1">
					{#each stages as _, i}
						<div class="h-0.5 flex-1 rounded-full transition-all duration-500 {i < stageIndex ? 'bg-brand-400/60' : i === stageIndex ? 'bg-brand-400/30' : 'bg-fg/[0.04]'}"></div>
					{/each}
				</div>
			</div>

			<p class="text-[10px] text-fg/20 mt-8 hover:text-fg/40 transition-colors duration-300 tracking-wider">
				Clique ou pressione Espaço para pular
			</p>
		</div>
	</div>
{/if}

<style>
	@keyframes particleFloat {
		0%, 100% { transform: translateY(0) translateX(0); opacity: var(--tw-opacity, 0.2); }
		25% { transform: translateY(-20px) translateX(10px); opacity: 0.4; }
		50% { transform: translateY(-40px) translateX(-5px); opacity: 0.15; }
		75% { transform: translateY(-20px) translateX(-10px); opacity: 0.35; }
	}
	@keyframes orbPulse {
		0%, 100% { transform: scale(1); opacity: 1; }
		50% { transform: scale(1.15); opacity: 0.7; }
	}
	@keyframes logoGlow {
		0%, 100% { opacity: 0.15; transform: scale(1.25); }
		50% { opacity: 0.25; transform: scale(1.35); }
	}
	@keyframes shimmer {
		0% { transform: translateX(-100%); }
		100% { transform: translateX(200%); }
	}
</style>
