<script lang="ts">
	import { fade } from "svelte/transition";
	import FilterableVersionSelect from "$lib/components/ui/FilterableVersionSelect.svelte";
	import {
		Plus,
		Check,
		X,
		AlertTriangle,
		Sparkles,
		Zap
	} from "lucide-svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	const { t } = useTranslation();

	interface Props {
		isOpen: boolean;
		onClose?: () => void;
		versions: Array<{ id: string; versionType: string; releaseTime: string }>;
		versionsLoading?: boolean;
		systemRamMb?: number;
		onCreate?: (input: CreateInput) => Promise<void>;
	}

	interface CreateInput {
		name: string;
		version: string;
		loader: string;
		icon: string;
		ramGb: number;
		autoOptimize: boolean;
		useVulkan: boolean;
		installPerfPack: boolean;
	}

	let {
		isOpen = $bindable(false),
		onClose,
		versions = [],
		versionsLoading = false,
		systemRamMb = 8192,
		onCreate
	}: Props = $props();

	let newName = $state("My Instance");
	let newVersion = $state("1.21.4");
	let newLoader = $state("vanilla");
	let newIcon = $state("grass_block");
	let selectedRamGb = $state(4);
	let newAutoOptimize = $state(true);
	let newUseVulkan = $state(false);
	let newInstallPerfPack = $state(true);
	let creating = $state(false);
	let lastError = $state<string | null>(null);
	let createSuccess = $state(false);

	const ramPresets = $derived.by(() => {
		const totalGb = Math.floor(systemRamMb / 1024);
		if (totalGb <= 4) return [1, 2, 3];
		if (totalGb <= 8) return [2, 4, 6];
		if (totalGb <= 16) return [2, 4, 6, 8, 12];
		if (totalGb <= 32) return [2, 4, 6, 8, 12, 16, 24];
		return [2, 4, 6, 8, 12, 16, 24, 32];
	});

	const iconPresets = [
		{ id: "grass_block", label: "Bloco de Grama", src: "/grass_block.png" },
		{ id: "modpack_fo", label: "Fabulously Optimized", src: "/modpack_fo_icon.png" },
		{ id: "modpack_better_mc", label: "Better MC", src: "/modpack_bmc_icon.webp" },
		{ id: "modpack_cobblemon", label: "Cobblemon", src: "/modpack_cobblemon_icon.png" },
		{ id: "logo", label: "Luxmc Logo", src: "/logo.png" },
		{ id: "grass_head", label: "Steve", src: "/grass_head.png" }
	];

	function getIconSrc(iconStr?: string): string {
		if (!iconStr || iconStr === "grass_block") return "/grass_block.png";
		if (iconStr === "modpack_fo") return "/modpack_fo_icon.png";
		if (iconStr === "modpack_better_mc") return "/modpack_bmc_icon.webp";
		if (iconStr === "modpack_cobblemon") return "/modpack_cobblemon_icon.png";
		if (iconStr === "logo") return "/logo.png";
		if (iconStr === "grass_head") return "/grass_head.png";
		if (iconStr.startsWith("/") || iconStr.startsWith("http") || iconStr.startsWith("data:")) return iconStr;
		return "/grass_block.png";
	}

	const loaderOptions = [
		{ id: 'vanilla', name: 'Vanilla', desc: 'Minecraft Oficial Puro', badge: 'Estável', color: 'border-emerald-500/30' },
		{ id: 'fabric', name: 'Fabric', desc: 'Mais Leve & Alto FPS', badge: 'Recomendado', color: 'border-blue-500/30' },
		{ id: 'neoforge', name: 'NeoForge', desc: 'Mods Modernos 1.20+', badge: 'Novo', color: 'border-amber-500/30' },
		{ id: 'forge', name: 'Forge', desc: 'Maior Biblioteca Clássica', badge: 'Clássico', color: 'border-orange-500/30' },
		{ id: 'quilt', name: 'Quilt', desc: 'Ecossistema Aberto', badge: 'Comunitário', color: 'border-purple-500/30' }
	];

	const presets = [
		{ title: 'Vanilla Otimizado', version: '1.21.4', loader: 'fabric', ram: 4, desc: 'Fabric + pronto para Sodium, Iris & Lithium.', tag: 'Máximo FPS', tagColor: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20' },
		{ title: 'PvP Clássico 1.8.9', version: '1.8.9', loader: 'vanilla', ram: 3, desc: 'Otimizado para Hypixel, Mush e servidores de PvP.', tag: 'Competitivo', tagColor: 'text-amber-400 bg-amber-500/10 border-amber-500/20' },
		{ title: 'Survival 1.21.4', version: '1.21.4', loader: 'vanilla', ram: 4, desc: 'Última versão vanilla pura e estável da Mojang.', tag: 'Exploração', tagColor: 'text-blue-400 bg-blue-500/10 border-blue-500/20' },
		{ title: 'Modded NeoForge', version: '1.21.1', loader: 'neoforge', ram: 6, desc: 'Perfil com 6 GB de RAM para modpacks pesados.', tag: 'Heavy Mods', tagColor: 'text-purple-400 bg-purple-500/10 border-purple-500/20' }
	];

	function applyPreset(preset: typeof presets[number]) {
		newLoader = preset.loader;
		newVersion = preset.version;
		newName = preset.title;
		selectedRamGb = preset.ram;
		toast(`Modelo "${preset.title}" aplicado com sucesso!`, "success");
	}

	function selectLoader(ldr: typeof loaderOptions[number]) {
		newLoader = ldr.id;
		newName = `Minecraft ${newVersion} (${ldr.name})`;
	}

	function handleVersionChange(ver: string) {
		const ldrName = newLoader.charAt(0).toUpperCase() + newLoader.slice(1);
		newName = `Minecraft ${ver} (${ldrName})`;
	}

	function resetForm() {
		newName = "My Instance";
		newIcon = "grass_block";
		newAutoOptimize = true;
		newUseVulkan = false;
		newInstallPerfPack = true;
		lastError = null;
		createSuccess = false;
	}

	function handleClose() {
		resetForm();
		onClose?.();
	}

	let { open } = $state({ open: async () => null }) as any;

	async function handleCreate() {
		if (!newName.trim()) return;
		creating = true;
		lastError = null;
		createSuccess = false;
		try {
			await onCreate?.({
				name: newName.trim(),
				version: newVersion,
				loader: newLoader,
				icon: newIcon,
				ramGb: selectedRamGb,
				autoOptimize: newAutoOptimize,
				useVulkan: newUseVulkan,
				installPerfPack: newInstallPerfPack
			});
			createSuccess = true;
			toast(t("instances.createdSuccess"), "success");
			setTimeout(() => handleClose(), 1500);
		} catch (e) {
			lastError = "Failed to create instance: " + String(e);
			toast(lastError, "error");
		} finally {
			creating = false;
		}
	}

	function getRamDescription(ram: number): string {
		if (ram <= 2) return "Ideal para versões clássicas e hardware básico.";
		if (ram === 4) return "Recomendado para Minecraft moderno (1.20+) Vanilla ou com poucos mods.";
		if (ram === 6) return "Excelente para modpacks médios (Fabulously Optimized, shaders moderados).";
		if (ram === 8) return "Configuração para modpacks pesados (Better MC, Cobblemon, All The Mods).";
		return "Alocação extrema para modpacks com centenas de mods e shaders ultra realistas.";
	}
</script>

{#if isOpen}
	<div class="rounded-3xl bg-[#141518] border border-white/10 p-6 shadow-2xl space-y-6 select-none" in:fade={{ duration: 200 }}>
		<div class="flex items-center justify-between border-b border-white/5 pb-4">
			<div>
				<h3 class="text-lg font-black text-white flex items-center gap-2">
					<Sparkles class="w-5 h-5 text-brand-500" /> Criar Nova Instância
				</h3>
				<p class="text-xs text-white/50 mt-0.5">Selecione o modloader e a versão desejada para configurar sua instância com alto desempenho.</p>
			</div>
			<button
				type="button"
				class="h-8 w-8 rounded-full bg-white/5 hover:bg-white/10 text-white/60 hover:text-white flex items-center justify-center transition-all cursor-pointer"
				onclick={handleClose}
				title="Fechar"
			>
				<X class="w-4 h-4" />
			</button>
		</div>

		{#if createSuccess}
			<div class="flex items-center gap-3 rounded-2xl p-4 text-sm bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 font-bold">
				<Check class="h-5 w-5" />
				Instância "{newName}" criada com sucesso! Redirecionando...
			</div>
		{:else}
			<!-- Presets -->
			<div class="space-y-2.5 bg-[#18191c]/60 p-4 rounded-3xl border border-white/5">
				<div class="flex items-center justify-between">
					<span class="text-xs font-bold text-brand-500 uppercase tracking-wider flex items-center gap-1.5">
						<Zap class="w-3.5 h-3.5 fill-current" /> Modelos Prontos (1-Clique)
					</span>
					<span class="text-[10px] text-white/40 font-medium">Configurações pré-otimizadas</span>
				</div>
				<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-2.5">
					{#each presets as preset}
						<button
							type="button"
							class="p-3.5 rounded-2xl border text-left transition-all flex flex-col justify-between h-28 cursor-pointer bg-[#141518] border-white/5 hover:border-brand-500/50 hover:bg-[#1f2026] group relative overflow-hidden active:scale-98"
							onclick={() => applyPreset(preset)}
						>
							<div class="flex items-center justify-between w-full">
								<span class="text-xs font-black text-white group-hover:text-brand-500 transition-colors truncate">{preset.title}</span>
								<span class="text-[8px] font-extrabold uppercase px-1.5 py-0.5 rounded border {preset.tagColor} shrink-0">{preset.tag}</span>
							</div>
							<p class="text-[10px] text-white/40 leading-snug">{preset.desc}</p>
							<div class="flex items-center gap-2 text-[9px] font-mono text-white/30">
								<span>{preset.version}</span>
								<span>·</span>
								<span class="capitalize">{preset.loader}</span>
								<span>·</span>
								<span>{preset.ram} GB</span>
							</div>
						</button>
					{/each}
				</div>
			</div>

			<!-- Step 1: Loader -->
			<div class="space-y-2">
				<span class="text-xs font-bold text-white/70 uppercase tracking-wider block">1. Selecione o Modloader</span>
				<div class="grid grid-cols-2 sm:grid-cols-5 gap-3">
					{#each loaderOptions as ldr}
						<button
							type="button"
							class="p-3.5 rounded-2xl border text-left transition-all flex flex-col justify-between h-24 cursor-pointer {newLoader === ldr.id ? 'bg-[#222328] border-brand-500 shadow-[0_0_15px_rgba(226,184,107,0.25)]' : 'bg-[#18191c] border-white/5 hover:border-white/15'}"
							onclick={() => selectLoader(ldr)}
						>
							<div class="flex items-center justify-between w-full">
								<span class="text-xs font-black text-white">{ldr.name}</span>
								<span class="text-[8px] font-extrabold uppercase px-1.5 py-0.5 rounded bg-white/5 text-white/60">{ldr.badge}</span>
							</div>
							<span class="text-[10px] text-white/40 leading-snug">{ldr.desc}</span>
						</button>
					{/each}
				</div>
			</div>

			<!-- Step 2: Version -->
			<div class="space-y-2">
				<span class="text-xs font-bold text-white/70 uppercase tracking-wider block">2. Versão do Minecraft</span>
				<FilterableVersionSelect
					{versions}
					bind:value={newVersion}
					loading={versionsLoading}
					onChange={handleVersionChange}
				/>
			</div>

			<!-- Step 3: Name, Icon, RAM -->
			<div class="space-y-4">
				<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
					<div class="md:col-span-2 space-y-1.5">
						<label for="instance-name" class="block text-xs font-bold text-white/70 uppercase tracking-wider">{t("instances.name")}</label>
						<div class="flex items-center gap-3">
							<div class="h-11 w-11 rounded-2xl bg-[#18191c] border border-white/10 flex items-center justify-center shrink-0 p-1">
								<img src={getIconSrc(newIcon)} alt="Ícone da Instância" class="w-8 h-8 object-contain [image-rendering:pixelated]" />
							</div>
							<input
								id="instance-name"
								type="text"
								class="h-11 flex-1 rounded-2xl px-5 text-xs font-bold text-white bg-[#18191c] border border-white/10 focus:border-brand-500 outline-none transition-all"
								placeholder={t("instances.namePlaceholder")}
								bind:value={newName}
							/>
						</div>
					</div>

					<div class="space-y-1.5">
						<span class="block text-xs font-bold text-white/70 uppercase tracking-wider">Ícone da Instância</span>
						<div class="flex items-center gap-1.5 bg-[#18191c] p-1.5 rounded-2xl border border-white/10">
							{#each iconPresets as ip}
								<button
									type="button"
									class="w-8 h-8 rounded-xl p-1 transition-all cursor-pointer flex items-center justify-center {newIcon === ip.id ? 'bg-brand-500/20 border border-brand-500 scale-105' : 'hover:bg-white/5 opacity-60 hover:opacity-100'}"
									onclick={() => newIcon = ip.id}
									title={ip.label}
								>
									<img src={ip.src} alt={ip.label} class="w-6 h-6 object-contain [image-rendering:pixelated]" />
								</button>
							{/each}
							<button
								type="button"
								class="h-8 px-2 rounded-xl text-[10px] font-bold text-brand-500 hover:bg-brand-500/10 border border-brand-500/30 transition-all cursor-pointer shrink-0 ml-auto"
								title="Carregar imagem do PC"
							>
								+ Imagem
							</button>
						</div>
					</div>
				</div>

				<!-- RAM -->
				<div class="bg-[#18191c] border border-white/5 rounded-3xl p-4 space-y-2.5">
					<div class="flex items-center justify-between text-xs">
						<div class="flex items-center gap-2">
							<span class="font-bold text-white/80">Alocação de Memória RAM</span>
							<span class="text-[10px] text-white/40 font-mono bg-white/5 px-2 py-0.5 rounded-full border border-white/5">
								Detectado no PC: {Math.round(systemRamMb / 1024)} GB
							</span>
						</div>
						<span class="font-mono font-black text-brand-500 bg-brand-500/10 px-3 py-0.5 rounded-full border border-brand-500/20">
							{selectedRamGb} GB ({selectedRamGb * 1024} MB)
						</span>
					</div>
					<div class="flex items-center gap-2">
						{#each ramPresets as ram}
							{@const isRecommended = (systemRamMb >= 12288 && ram === 6) || (systemRamMb < 12288 && ram === 4)}
							<button
								type="button"
								class="flex-1 py-2 rounded-2xl text-xs font-black transition-all cursor-pointer relative {selectedRamGb === ram ? 'bg-brand-500 text-black shadow-md scale-[1.02]' : 'bg-[#222328] text-white/60 hover:text-white border border-white/5 hover:border-white/20'}"
								onclick={() => selectedRamGb = ram}
							>
								<span>{ram} GB</span>
								{#if isRecommended}
									<span class="absolute -top-2 left-1/2 -translate-x-1/2 text-[8px] font-extrabold uppercase px-1 rounded bg-emerald-500 text-black shadow-xs">Ideal</span>
								{/if}
							</button>
						{/each}
					</div>
					<p class="text-[10px] text-white/40">{getRamDescription(selectedRamGb)}</p>
				</div>

				<!-- Optimizations -->
				<div class="bg-[#18191c] border border-white/5 rounded-3xl p-4 space-y-3">
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-2">
							<Zap class="w-4 h-4 text-brand-500" />
							<span class="text-xs font-bold text-white/90">Otimizações Nativas Luxmc</span>
						</div>
						<span class="text-[9px] font-bold uppercase tracking-wider text-brand-500 bg-brand-500/10 px-2 py-0.5 rounded-full border border-brand-500/20">Auto Tuning</span>
					</div>
					<div class="space-y-2">
						<label class="flex items-start gap-3 p-2.5 rounded-2xl bg-[#222328]/60 hover:bg-[#222328] border border-white/5 cursor-pointer transition-colors">
							<input type="checkbox" bind:checked={newAutoOptimize} class="mt-0.5 accent-brand-500 rounded" />
							<div class="text-xs space-y-0.5">
								<div class="font-bold text-white/90 flex items-center gap-1.5">
									<span>Flags de GC Inteligentes (Aikar G1GC)</span>
									<span class="text-[9px] font-bold text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded border border-emerald-500/20">Recomendado</span>
								</div>
								<p class="text-[10px] text-white/50">Ajusta dinamicamente as regiões de heap e threads do garbage collector da JVM para eliminar microtravamentos.</p>
							</div>
						</label>

						{#if newLoader !== "vanilla"}
							<label class="flex items-start gap-3 p-2.5 rounded-2xl bg-[#222328]/60 hover:bg-[#222328] border border-white/5 cursor-pointer transition-colors">
								<input type="checkbox" bind:checked={newInstallPerfPack} class="mt-0.5 accent-brand-500 rounded" />
								<div class="text-xs space-y-0.5">
									<div class="font-bold text-white/90 flex items-center gap-1.5">
										<Sparkles class="w-3.5 h-3.5 text-amber-400" />
										<span>Pacote de Otimização Essencial</span>
									</div>
									<p class="text-[10px] text-white/50">Baixa automaticamente Sodium, Lithium e FerriteCore oficiais compatíveis com a versão escolhida.</p>
								</div>
							</label>
						{/if}

						<label class="flex items-start gap-3 p-2.5 rounded-2xl bg-[#222328]/60 hover:bg-[#222328] border border-white/5 cursor-pointer transition-colors">
							<input type="checkbox" bind:checked={newUseVulkan} class="mt-0.5 accent-brand-500 rounded" />
							<div class="text-xs space-y-0.5">
								<div class="font-bold text-white/90 flex items-center gap-1.5">
									<span>Aceleração Gráfica Mesa Zink / Vulkan (Linux)</span>
									<span class="text-[9px] font-mono text-white/40 bg-white/5 px-1.5 py-0.5 rounded">Experimental</span>
								</div>
								<p class="text-[10px] text-white/50">Redireciona o pipeline OpenGL para o driver Vulkan nativo da sua GPU via Gallium Zink.</p>
							</div>
						</label>
					</div>
				</div>
			</div>

			<!-- Actions -->
			<div class="flex gap-3 pt-2">
				<button
					type="button"
					class="px-8 py-3 rounded-full bg-brand-500 hover:bg-[#ebd095] text-black font-black text-xs flex items-center gap-2 transition-all active:scale-95 shadow-lg cursor-pointer"
					onclick={handleCreate}
					disabled={creating}
				>
					{#if creating}
						<div class="w-4 h-4 rounded-full border-2 border-black border-t-transparent animate-spin"></div>
						Criando Instância...
					{:else}
						<Check class="h-4 w-4 stroke-[3]" />
						Criar Instância Agora
					{/if}
				</button>

				<button
					type="button"
					class="px-6 py-3 rounded-full bg-white/5 hover:bg-white/10 text-white/60 hover:text-white font-bold text-xs flex items-center gap-1.5 transition-all cursor-pointer"
					onclick={handleClose}
				>
					<X class="h-4 w-4" />
					Cancelar
				</button>
			</div>
		{/if}

		{#if lastError}
			<div class="flex items-center justify-between rounded-full px-5 py-3 text-xs bg-red-500/10 border border-red-500/30 text-red-400 font-bold">
				<div class="flex items-center gap-2">
					<AlertTriangle class="h-4 w-4 shrink-0" />
					{lastError}
				</div>
				<button
					type="button"
					class="p-1 rounded hover:bg-white/10 cursor-pointer"
					onclick={() => (lastError = null)}
				>
					<X class="h-3.5 w-3.5" />
				</button>
			</div>
		{/if}
	</div>
{/if}
