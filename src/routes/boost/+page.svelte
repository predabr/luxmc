<script lang="ts">
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import { 
		Zap, 
		Flame, 
		Cpu, 
		Gauge, 
		HardDrive, 
		RefreshCw, 
		CheckCircle2, 
		Sliders, 
		Download, 
		Sparkles, 
		Activity, 
		Check, 
		Copy,
		Layers
	} from "lucide-svelte";
	import { 
		getSystemSpecs, 
		optimizerTrimMemory, 
		optimizerGetFlags, 
		optimizerInstallPerfPack,
		profilesUpdate,
		type SystemSpecs 
	} from "$lib/api";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";
	import Button from "$lib/components/ui/Button.svelte";

	let specs = $state<SystemSpecs | null>(null);
	let loadingSpecs = $state(true);
	let trimming = $state(false);
	let installingPack = $state(false);
	let selectedRamPreset = $state<number>(4096);
	let generatedFlags = $state<string[]>([]);
	let copiedFlags = $state(false);
	let activeProfile = $derived(profiles.active);

	const ramPresets = [
		{ mb: 2048, label: "2 GB", desc: "Vanilla Leve", icon: "🌱" },
		{ mb: 4096, label: "4 GB", desc: "Padrão / Mods Moderados", icon: "⚡" },
		{ mb: 6144, label: "6 GB", desc: "Modpacks Médios (100+ mods)", icon: "🔥" },
		{ mb: 8192, label: "8 GB", desc: "Heavy Modpacks (200+ mods / Shaders)", icon: "🚀" },
		{ mb: 12288, label: "12 GB", desc: "Extremo / Texturas 256x+", icon: "💎" }
	];

	async function loadSpecs() {
		loadingSpecs = true;
		try {
			specs = await getSystemSpecs();
		} catch (e) {
			console.warn("Falha ao carregar specs do sistema:", e);
		} finally {
			loadingSpecs = false;
		}
	}

	async function updateFlags(ramMb: number) {
		selectedRamPreset = ramMb;
		try {
			generatedFlags = await optimizerGetFlags(ramMb, true);
		} catch (e) {
			console.warn("Falha ao gerar flags:", e);
		}
	}

	onMount(() => {
		loadSpecs();
		const initialRam = activeProfile?.ramMb || 4096;
		updateFlags(initialRam);
	});

	async function handleTrimMemory() {
		trimming = true;
		playSound("click");
		try {
			const freed = await optimizerTrimMemory();
			if (freed) {
				playSound("chime");
				toast("Memória RAM da aplicação purgada e compactada com sucesso!", "success");
			} else {
				toast("Limpeza concluída no sistema.", "info");
			}
		} catch (e) {
			toast(`Falha ao purgar memória: ${String(e)}`, "error");
		} finally {
			trimming = false;
		}
	}

	async function handleInstallPerfPack() {
		if (!activeProfile) {
			toast("Selecione ou crie uma instância primeiro", "error");
			return;
		}
		installingPack = true;
		playSound("click");
		try {
			const installed = await optimizerInstallPerfPack(activeProfile.id);
			playSound("achievement");
			toast(`Pacote de Performance instalado: ${installed.join(", ")}`, "success");
		} catch (e) {
			toast(`Erro ao instalar pacote: ${String(e)}`, "error");
		} finally {
			installingPack = false;
		}
	}

	async function applyFlagsToProfile() {
		if (!activeProfile) return;
		const argsString = generatedFlags.join(" ");
		try {
			await profilesUpdate({
				id: activeProfile.id,
				name: activeProfile.name,
				ramMb: selectedRamPreset,
				jvmArgs: argsString,
				autoOptimize: true
			});
			activeProfile.ramMb = selectedRamPreset;
			activeProfile.jvmArgs = argsString;
			activeProfile.autoOptimize = true;
			playSound("chime");
			toast("Flags Aikar e RAM aplicadas à instância ativa!", "success");
		} catch (e) {
			toast(`Erro ao aplicar configurações: ${String(e)}`, "error");
		}
	}

	function copyFlags() {
		const text = generatedFlags.join(" ");
		navigator.clipboard.writeText(text).then(() => {
			copiedFlags = true;
			toast("Flags JVM copiadas para a área de transferência!", "success");
			setTimeout(() => copiedFlags = false, 2500);
		});
	}
</script>

<div class="space-y-6 pb-12">
	<!-- Hero Header -->
	<div class="relative overflow-hidden rounded-3xl bg-gradient-to-br from-bg-elevated via-bg-subtle to-bg border border-white/10 p-7 shadow-2xl">
		<div class="absolute -right-10 -top-10 w-80 h-80 bg-brand-500/10 rounded-full blur-3xl pointer-events-none"></div>
		<div class="flex flex-col md:flex-row md:items-center justify-between gap-5 relative z-10">
			<div class="flex items-center gap-4">
				<div class="h-14 w-14 rounded-2xl bg-brand-500/20 text-brand-500 border border-brand-500/30 flex items-center justify-center shadow-lg shadow-brand-500/10">
					<Zap class="w-7 h-7" />
				</div>
				<div>
					<div class="flex items-center gap-2">
						<h1 class="text-2xl font-black text-white tracking-tight">Luxmc Turbo Booster</h1>
						<span class="px-2 py-0.5 rounded-full text-[10px] font-black uppercase tracking-wider bg-brand-500 text-black shadow-md">
							Engine
						</span>
					</div>
					<p class="text-xs text-white/50 mt-1">Otimizador de Baixo Nível, Purga de Memória & Afinação JVM</p>
				</div>
			</div>

			<div class="flex items-center gap-3">
				<button
					type="button"
					class="px-5 py-2.5 rounded-full text-xs font-bold bg-bg-overlay hover:bg-white/10 text-white/80 hover:text-white border border-white/10 transition-all flex items-center gap-2 cursor-pointer active:scale-95"
					onclick={loadSpecs}
					disabled={loadingSpecs}
				>
					<RefreshCw class="w-3.5 h-3.5 {loadingSpecs ? 'animate-spin' : ''}" />
					<span>Atualizar Dados</span>
				</button>
				<button
					type="button"
					class="px-6 py-2.5 rounded-full text-xs font-black bg-brand-500 hover:brightness-110 text-black transition-all flex items-center gap-2 shadow-lg shadow-brand-500/20 cursor-pointer active:scale-95 disabled:opacity-50"
					onclick={handleTrimMemory}
					disabled={trimming}
				>
					<Flame class="w-4 h-4 {trimming ? 'animate-bounce' : ''}" />
					<span>{trimming ? 'Purgando...' : 'Purgar RAM do Launcher'}</span>
				</button>
			</div>
		</div>
	</div>

	<!-- Hardware & Diagnostic Cards -->
	<div class="grid grid-cols-1 md:grid-cols-4 gap-4">
		<div class="bg-bg-elevated border border-white/5 rounded-2xl p-4 flex items-center gap-3.5 shadow-sm">
			<div class="w-10 h-10 rounded-xl bg-blue-500/10 text-blue-400 border border-blue-500/20 flex items-center justify-center shrink-0">
				<Cpu class="w-5 h-5" />
			</div>
			<div class="min-w-0">
				<span class="text-[10px] font-bold uppercase tracking-wider text-white/40 block">Arquitetura / CPU</span>
				<span class="text-xs font-bold text-white truncate block" title={specs?.arch || "Linux x86_64"}>
					{specs?.arch ? `${specs.arch} (${specs.osDistro})` : (loadingSpecs ? "Analisando..." : "x86_64")}
				</span>
			</div>
		</div>

		<div class="bg-bg-elevated border border-white/5 rounded-2xl p-4 flex items-center gap-3.5 shadow-sm">
			<div class="w-10 h-10 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center justify-center shrink-0">
				<Gauge class="w-5 h-5" />
			</div>
			<div class="min-w-0">
				<span class="text-[10px] font-bold uppercase tracking-wider text-white/40 block">Memória RAM</span>
				<span class="text-xs font-bold text-white block">
					{specs ? `${Math.round(specs.totalRamMb / 1024)} GB Alocável` : "8 GB Total"}
				</span>
			</div>
		</div>

		<div class="bg-bg-elevated border border-white/5 rounded-2xl p-4 flex items-center gap-3.5 shadow-sm">
			<div class="w-10 h-10 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/20 flex items-center justify-center shrink-0">
				<Activity class="w-5 h-5" />
			</div>
			<div class="min-w-0">
				<span class="text-[10px] font-bold uppercase tracking-wider text-white/40 block">Placa de Vídeo (GPU)</span>
				<span class="text-xs font-bold text-white truncate block" title={specs ? `${specs.gpuVendor} - ${specs.gpuRenderer}` : "GPU Acelerada"}>
					{specs ? `${specs.gpuVendor} · ${specs.gpuRenderer}` : "GPU Acelerada"}
				</span>
			</div>
		</div>

		<div class="bg-bg-elevated border border-white/5 rounded-2xl p-4 flex items-center gap-3.5 shadow-sm">
			<div class="w-10 h-10 rounded-xl bg-amber-500/10 text-amber-400 border border-amber-500/20 flex items-center justify-center shrink-0">
				<HardDrive class="w-5 h-5" />
			</div>
			<div class="min-w-0">
				<span class="text-[10px] font-bold uppercase tracking-wider text-white/40 block">Kernel & Versão</span>
				<span class="text-xs font-bold text-white truncate block" title={specs ? `${specs.kernelVersion}` : "Linux Nativo"}>
					{specs ? `${specs.kernelVersion} · v${specs.launcherVersion}` : "Linux Nativo"}
				</span>
			</div>
		</div>
	</div>

	<!-- 1-Click Performance Pack Injection -->
	<div class="bg-bg-elevated border border-white/10 rounded-3xl p-6 shadow-md relative overflow-hidden">
		<div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
			<div class="flex items-start gap-4">
				<div class="w-12 h-12 rounded-2xl bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 flex items-center justify-center shrink-0 shadow-inner">
					<Sparkles class="w-6 h-6" />
				</div>
				<div>
					<h3 class="text-sm font-bold text-white">Pacote de Otimização Extrema (1-Clique)</h3>
					<p class="text-xs text-white/50 mt-1 max-w-xl">
						Instala automaticamente mods de ponta na instância ativa: <strong class="text-white">FerriteCore</strong> (reduz uso de RAM pela metade), <strong class="text-white">ModernFix</strong> (carregamento ultra-rápido), <strong class="text-white">ImmediatelyFast</strong> e <strong class="text-white">Lithium</strong>.
					</p>
					{#if activeProfile}
						<div class="flex items-center gap-2 mt-2 text-[11px] text-white/60">
							<span>Instância Selecionada:</span>
							<span class="font-bold text-brand-500">{activeProfile.name}</span>
							<span class="px-1.5 py-0.5 rounded bg-white/5 text-white/40 font-mono">{activeProfile.loader} · {activeProfile.mcVersion}</span>
						</div>
					{/if}
				</div>
			</div>

			<Button
				variant="primary"
				onclick={handleInstallPerfPack}
				loading={installingPack}
				class="shrink-0"
			>
				<Download class="w-4 h-4 mr-1.5" />
				Instalar Pacote de Performance
			</Button>
		</div>
	</div>

	<!-- Aikar's Flag Generator & RAM Tuner -->
	<div class="bg-bg-elevated border border-white/10 rounded-3xl p-6 shadow-md space-y-6">
		<div class="flex items-center justify-between">
			<div>
				<h3 class="text-sm font-bold text-white flex items-center gap-2">
					<Sliders class="w-4 h-4 text-brand-500" />
					Gerador Inteligente de Flags JVM (Aikar's Flags)
				</h3>
				<p class="text-xs text-white/50 mt-0.5">Calcula as flags ideais para Garbage Collection (G1GC) e estabilização de framerate</p>
			</div>

			<div class="flex items-center gap-2">
				<button
					type="button"
					class="px-4 py-2 rounded-full text-xs font-bold bg-white/5 hover:bg-white/10 text-white/80 hover:text-white border border-white/5 transition-all flex items-center gap-1.5 cursor-pointer"
					onclick={copyFlags}
				>
					{#if copiedFlags}
						<Check class="w-3.5 h-3.5 text-emerald-400" />
						<span class="text-emerald-400">Copiado!</span>
					{:else}
						<Copy class="w-3.5 h-3.5" />
						<span>Copiar Flags</span>
					{/if}
				</button>
				<Button
					variant="primary"
					onclick={applyFlagsToProfile}
					disabled={!activeProfile}
				>
					<Check class="w-4 h-4 mr-1.5" />
					Aplicar à Instância
				</Button>
			</div>
		</div>

		<!-- RAM Presets -->
		<div class="space-y-2">
			<span class="text-xs font-bold text-white/70 block">Selecione a Alocação de Memória RAM:</span>
			<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-3">
				{#each ramPresets as preset}
					<button
						type="button"
						class="p-3.5 rounded-2xl border text-left transition-all cursor-pointer {selectedRamPreset === preset.mb ? 'border-brand-500 bg-brand-500/10 shadow-md ring-1 ring-brand-500/30' : 'border-white/5 bg-bg-subtle hover:border-white/15'}"
						onclick={() => updateFlags(preset.mb)}
					>
						<div class="text-lg">{preset.icon}</div>
						<div class="text-xs font-black text-white mt-1">{preset.label}</div>
						<div class="text-[10px] text-white/40 mt-0.5 truncate">{preset.desc}</div>
					</button>
				{/each}
			</div>
		</div>

		<!-- Generated Flags Output -->
		<div class="space-y-2">
			<span class="text-xs font-bold text-white/70 block">Flags Geradas Automaticamente:</span>
			<div class="bg-bg-subtle border border-white/10 rounded-2xl p-4 font-mono text-[11px] text-emerald-400/90 leading-relaxed break-all select-all shadow-inner">
				{#if generatedFlags.length > 0}
					{generatedFlags.join(" ")}
				{:else}
					<span class="text-white/30 font-sans">Gerando flags de otimização...</span>
				{/if}
			</div>
		</div>
	</div>
</div>
