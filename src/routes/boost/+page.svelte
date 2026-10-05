<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
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

	const ramPresets = $derived([
		{ mb: 2048, label: uiText("ui.b2dd6247c7a0cd98"), desc: uiText("ui.8b37ff10118189bf"), icon: "🌱" },
		{ mb: 4096, label: uiText("ui.64a214401e793659"), desc: uiText("ui.8d2833400aa6d9ca"), icon: "⚡" },
		{ mb: 6144, label: uiText("ui.ba344e24e41f5b5c"), desc: uiText("ui.950761d9e7b41dae"), icon: "🔥" },
		{ mb: 8192, label: uiText("ui.64b29b62d14dd300"), desc: uiText("ui.d48b5926e7871f36"), icon: "🚀" },
		{ mb: 12288, label: uiText("ui.f2d32ef1e01403f8"), desc: uiText("ui.f0848910981ca1ea"), icon: "💎" }
	]);

	async function loadSpecs() {
		loadingSpecs = true;
		try {
			specs = await getSystemSpecs();
		} catch (e) {
			console.warn(uiText("ui.26b05fbe7c7a5150"), e);
		} finally {
			loadingSpecs = false;
		}
	}

	async function updateFlags(ramMb: number) {
		selectedRamPreset = ramMb;
		try {
			generatedFlags = await optimizerGetFlags(ramMb, true);
		} catch (e) {
			console.warn(uiText("ui.5b5479b36ac80588"), e);
		}
	}

	import { goto } from "$app/navigation";

	onMount(() => {
		goto("/instances");
	});

	async function handleTrimMemory() {
		trimming = true;
		playSound("click");
		try {
			const freed = await optimizerTrimMemory();
			if (freed) {
				playSound("chime");
				toast(uiText("ui.960cf645c805c439"), "success");
			} else {
				toast(uiText("ui.8306b22090417500"), "info");
			}
		} catch (e) {
			toast(uiText("ui.d3956d124f4a851d", {arg0: (String(e))}), "error");
		} finally {
			trimming = false;
		}
	}

	async function handleInstallPerfPack() {
		if (!activeProfile) {
			toast(uiText("ui.94ed7f3735d7c0af"), "error");
			return;
		}
		installingPack = true;
		playSound("click");
		try {
			const installed = await optimizerInstallPerfPack(activeProfile.id);
			playSound("achievement");
			toast(`Pacote de Performance instalado: ${installed.join(", ")}`, "success");
		} catch (e) {
			toast(uiText("ui.13955fe0e626bc38", {arg0: (String(e))}), "error");
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
			toast(uiText("ui.56a49ac0592f7861"), "success");
		} catch (e) {
			toast(uiText("ui.7a916bb8e0104909", {arg0: (String(e))}), "error");
		}
	}

	function copyFlags() {
		const text = generatedFlags.join(" ");
		navigator.clipboard.writeText(text).then(() => {
			copiedFlags = true;
			toast(uiText("ui.b1bd70e209fd2186"), "success");
			setTimeout(() => copiedFlags = false, 2500);
		}).catch((e) => toast(uiText("ui.25be28c16b3339a8") + String(e), "error"));
	}
</script>

<div class="space-y-6 pb-12">
	<!-- Hero Header -->
	<div class="relative overflow-hidden rounded-3xl bg-gradient-to-br from-bg-elevated via-bg-subtle to-bg border border-fg/10 p-7 shadow-2xl">
		<div class="absolute -right-10 -top-10 w-80 h-80 bg-[radial-gradient(circle_at_center,rgb(var(--brand-500)/0.15),transparent_70%)] rounded-full pointer-events-none"></div>
		<div class="flex flex-col md:flex-row md:items-center justify-between gap-5 relative z-10">
			<div class="flex items-center gap-4">
				<div class="h-14 w-14 rounded-2xl bg-brand-500/20 text-brand-500 border border-brand-500/30 flex items-center justify-center shadow-lg shadow-brand-500/10">
					<Zap class="w-7 h-7" />
				</div>
				<div>
					<div class="flex items-center gap-2">
						<h1 class="text-2xl font-black text-fg tracking-tight">{uiText("ui.8ef1131ad4861779")}</h1>
						<span class="px-2 py-0.5 rounded-full text-[10px] font-black uppercase tracking-wider bg-brand-500 text-brand-foreground shadow-md">
							{uiText("ui.8e75ebbdb21505d2")}
						</span>
					</div>
					<p class="text-xs text-fg/50 mt-1">{uiText("ui.38c251943d6c688f")}</p>
				</div>
			</div>

			<div class="flex items-center gap-3">
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
					onclick={loadSpecs}
					disabled={loadingSpecs}
				>
					<RefreshCw class="w-3.5 h-3.5 {loadingSpecs ? 'animate-spin' : ''}" />
					<span>{uiText("ui.b7ac1eb6ee6512e5")}</span>
				</button>
				<button
					type="button"
					class={launcherButton({ variant: "primary", size: "sm", class: "flex items-center gap-2 disabled:opacity-50" })}
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
		<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 flex items-center gap-3.5 shadow-sm">
			<div class="w-10 h-10 rounded-xl bg-blue-500/10 text-blue-400 border border-blue-500/20 flex items-center justify-center shrink-0">
				<Cpu class="w-5 h-5" />
			</div>
			<div class="min-w-0">
				<span class="text-[10px] font-bold uppercase tracking-wider text-fg/40 block">{uiText("ui.73da38361010f47b")}</span>
				<span class="text-xs font-bold text-fg truncate block" title={specs?.arch || "Linux x86_64"}>
					{specs?.arch ? `${specs.arch} (${specs.osDistro})` : (loadingSpecs ? "Analisando..." : "x86_64")}
				</span>
			</div>
		</div>

		<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 flex items-center gap-3.5 shadow-sm">
			<div class="w-10 h-10 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center justify-center shrink-0">
				<Gauge class="w-5 h-5" />
			</div>
			<div class="min-w-0">
				<span class="text-[10px] font-bold uppercase tracking-wider text-fg/40 block">{uiText("ui.99fdbbe208962a9d")}</span>
				<span class="text-xs font-bold text-fg block">
					{specs ? uiText("ui.59b8dcf609e819df", {arg0: (Math.round(specs.totalRamMb / 1024))}) : uiText("ui.06984d799243cd84")}
				</span>
			</div>
		</div>

		<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 flex items-center gap-3.5 shadow-sm">
			<div class="w-10 h-10 rounded-xl bg-purple-500/10 text-purple-400 border border-purple-500/20 flex items-center justify-center shrink-0">
				<Activity class="w-5 h-5" />
			</div>
			<div class="min-w-0">
				<span class="text-[10px] font-bold uppercase tracking-wider text-fg/40 block">{uiText("ui.a7d1cd383f8baa2b")}</span>
				<span class="text-xs font-bold text-fg truncate block" title={specs ? `${specs.gpuVendor} - ${specs.gpuRenderer}` : "GPU Acelerada"}>
					{specs ? `${specs.gpuVendor} · ${specs.gpuRenderer}` : "GPU Acelerada"}
				</span>
			</div>
		</div>

		<div class="bg-bg-elevated border border-fg/5 rounded-2xl p-4 flex items-center gap-3.5 shadow-sm">
			<div class="w-10 h-10 rounded-xl bg-amber-500/10 text-amber-400 border border-amber-500/20 flex items-center justify-center shrink-0">
				<HardDrive class="w-5 h-5" />
			</div>
			<div class="min-w-0">
				<span class="text-[10px] font-bold uppercase tracking-wider text-fg/40 block">{uiText("ui.34d2c5f186d6bcf1")}</span>
				<span class="text-xs font-bold text-fg truncate block" title={specs ? `${specs.kernelVersion}` : "Linux Nativo"}>
					{specs ? `${specs.kernelVersion} · v${specs.launcherVersion}` : "Linux Nativo"}
				</span>
			</div>
		</div>
	</div>

	<!-- 1-Click Performance Pack Injection -->
	<div class="bg-bg-elevated border border-fg/10 rounded-3xl p-6 shadow-md relative overflow-hidden">
		<div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
			<div class="flex items-start gap-4">
				<div class="w-12 h-12 rounded-2xl bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 flex items-center justify-center shrink-0 shadow-inner">
					<Sparkles class="w-6 h-6" />
				</div>
				<div>
					<h3 class="text-sm font-bold text-fg">{uiText("ui.f034c8c78e3c85e4")}</h3>
					<p class="text-xs text-fg/50 mt-1 max-w-xl">
						{uiText("ui.368bf33325fe2891")} <strong class="text-fg">FerriteCore</strong> {uiText("ui.35d30ad990c06194")} <strong class="text-fg">ModernFix</strong> {uiText("ui.ffaa72d8a10f8564")} <strong class="text-fg">ImmediatelyFast</strong> {uiText("ui.3f79bb7b435b0532")} <strong class="text-fg">Lithium</strong>.
					</p>
					{#if activeProfile}
						<div class="flex items-center gap-2 mt-2 text-[11px] text-fg/60">
							<span>{uiText("ui.49b1aa18edb7f21a")}</span>
							<span class="font-bold text-brand-500">{activeProfile.name}</span>
							<span class="px-1.5 py-0.5 rounded bg-fg/5 text-fg/40 font-mono">{activeProfile.loader} · {activeProfile.mcVersion}</span>
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
				{uiText("ui.d75f243a714bed40")}
			</Button>
		</div>
	</div>

	<!-- Aikar's Flag Generator & RAM Tuner -->
	<div class="bg-bg-elevated border border-fg/10 rounded-3xl p-6 shadow-md space-y-6">
		<div class="flex items-center justify-between">
			<div>
				<h3 class="text-sm font-bold text-fg flex items-center gap-2">
					<Sliders class="w-4 h-4 text-brand-500" />
					{uiText("ui.6bbbf8e999fc75be")}
				</h3>
				<p class="text-xs text-fg/50 mt-0.5">{uiText("ui.246c4acad7cf557b")}</p>
			</div>

			<div class="flex items-center gap-2">
				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
					onclick={copyFlags}
				>
					{#if copiedFlags}
						<Check class="w-3.5 h-3.5 text-emerald-400" />
						<span class="text-emerald-400">{uiText("ui.a8fe0fc805d5fd50")}</span>
					{:else}
						<Copy class="w-3.5 h-3.5" />
						<span>{uiText("ui.33d4b5ed611992a1")}</span>
					{/if}
				</button>
				<Button
					variant="primary"
					onclick={applyFlagsToProfile}
					disabled={!activeProfile}
				>
					<Check class="w-4 h-4 mr-1.5" />
					{uiText("ui.86522217c086587f")}
				</Button>
			</div>
		</div>

		<!-- RAM Presets -->
		<div class="space-y-2">
			<span class="text-xs font-bold text-fg/70 block">{uiText("ui.4c5586a758b8178e")}</span>
			<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-3">
				{#each ramPresets as preset}
					<button
						type="button"
						class="p-3.5 rounded-2xl border text-left transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {selectedRamPreset === preset.mb ? 'border-brand-500 bg-brand-500/10 shadow-md ring-1 ring-brand-500/30' : 'border-fg/5 bg-bg-subtle hover:border-fg/15'}"
						onclick={() => updateFlags(preset.mb)}
					>
						<div class="text-lg">{preset.icon}</div>
						<div class="text-xs font-black text-fg mt-1">{preset.label}</div>
						<div class="text-[10px] text-fg/40 mt-0.5 truncate">{preset.desc}</div>
					</button>
				{/each}
			</div>
		</div>

		<!-- Generated Flags Output -->
		<div class="space-y-2">
			<span class="text-xs font-bold text-fg/70 block">{uiText("ui.039b25151841684b")}</span>
			<div class="bg-bg-subtle border border-fg/10 rounded-2xl p-4 font-mono text-[11px] text-emerald-400/90 leading-relaxed break-all select-all shadow-inner">
				{#if generatedFlags.length > 0}
					{generatedFlags.join(" ")}
				{:else}
					<span class="text-fg/30 font-sans">{uiText("ui.9148680d26a1705a")}</span>
				{/if}
			</div>
		</div>
	</div>
</div>
